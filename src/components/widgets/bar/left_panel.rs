use std::sync::Arc;

use crate::components::arena::{Arena, Slot};
use crate::components::base::group::GroupNode;
use crate::components::base::padding::PaddingNode;
use crate::components::base::rect::RectNode;
use crate::components::base::row::RowNode;
use crate::components::geom::Size;
use crate::components::keyed_list::KeyedList;
use crate::components::ui::{Controller, ElementArena, Node};
use crate::renderer::animation::cache::{AnimationCache, AnimSlot, AnimSpec};
use crate::renderer::animation::easing::Easing;
use crate::renderer::programs::rect::{CornerShape, RectStyle};
use crate::renderer::types::Color;
use crate::shell::compositor::Compositor;
use crate::shell::event::ShellEvent;

// ==================== CONFIGURATION CONSTANTS ====================
const WORKSPACE_SPACING: f32 = 8.0;
const WORKSPACE_R: f32 = 5.5;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;

const LEFT_PAD: f32 = 15.0;
const RIGHT_PAD: f32 = 30.0;
const TOP_PAD: f32 = 15.0 - WORKSPACE_R;
const DEFAULT_CORNER_R: f32 = 15.0;
const ANIM_DURATION: f32 = 0.26;

const COLOR_BG: Color = Color::rgb(0.085, 0.095, 0.110);
const COLOR_DOT_ACTIVE: Color = Color::rgb(1.0, 0.12, 0.14);
const COLOR_DOT_INACTIVE: Color = Color::rgb(0.25, 0.28, 0.35);

// ==================== DATA STRUCTURES ====================
struct DotState {
    workspace_id: i32,
    is_active: bool,
    width_slot: AnimSlot,
    rect_slot: Slot,
}

pub struct LeftPanelController {
    compositor: Arc<dyn Compositor>,
    panel_width_slot: AnimSlot,
    dots: KeyedList<i32, DotState>,
    bg_slot: Slot,
    row_slot: Slot,
}

// ==================== CONTROLLER IMPLEMENTATION ====================
impl LeftPanelController {
    pub fn mount(
        compositor: Arc<dyn Compositor>,
        bottom_offset: f32,
        arena: &mut Arena<Node>,
        cache: &mut AnimationCache,
    ) -> (Slot, Self) {
        let bg_style = RectStyle::new()
            .corners(
                CornerShape::Convex, CornerShape::Concave,
                CornerShape::Convex, CornerShape::Concave,
            )
            .radius(0.0, DEFAULT_CORNER_R, DEFAULT_CORNER_R, bottom_offset)
            .inset_right(DEFAULT_CORNER_R)
            .inset_bottom(bottom_offset)
            .fill(COLOR_BG.r, COLOR_BG.g, COLOR_BG.b, 1.0); // Bind color once at init

        let bg = arena.insert(Node::Rect(RectNode::new(Size::new(0.0, 0.0), bg_style)));
        let row = arena.insert(Node::Row(RowNode::new(vec![], WORKSPACE_SPACING)));
        let content = arena.insert(Node::Padding(PaddingNode::new(row, (LEFT_PAD, TOP_PAD, 0.0, 0.0))));
        let root = arena.insert(Node::Group(GroupNode::new(vec![bg, content])));

        let panel_width_slot = cache.insert(
            AnimSpec::new(LEFT_PAD + RIGHT_PAD)
                .with_duration(ANIM_DURATION)
                .with_easing(Easing::EaseOutCubic),
        );

        let ctrl = LeftPanelController {
            compositor,
            panel_width_slot,
            dots: KeyedList::new(),
            bg_slot: bg,
            row_slot: row,
        };

        (root, ctrl)
    }

    fn calculate_content_width(&self, cache: &AnimationCache) -> f32 {
        let n = self.dots.len();
        if n == 0 {
            return 0.0;
        }
        let dots_w: f32 = self.dots.iter().map(|d| cache.value(d.width_slot)).sum();
        let gaps_w = (n - 1) as f32 * WORKSPACE_SPACING;
        dots_w + gaps_w
    }
}

impl Controller for LeftPanelController {
    fn update(
        &mut self,
        event: &ShellEvent,
        now: f32,
        cache: &mut AnimationCache,
        arena: &mut ElementArena,
    ) -> bool {
        // We only respond to external workspace events here
        let ShellEvent::WorkspaceUpdated(snapshot) = event else {
            return false;
        };

        let mut cur_ids: Vec<i32> = snapshot.workspaces.iter().map(|w| w.id).collect();
        cur_ids.sort_unstable();

        // 1. Reconcile workspace dots
        self.dots.reconcile(&cur_ids, |id| {
            let comp = self.compositor.clone();
            let rect = arena.insert(Node::Rect(
                RectNode::new(
                    Size::new(WORKSPACE_INACTIVE_W, WORKSPACE_R * 2.0),
                    RectStyle::solid(COLOR_DOT_INACTIVE, WORKSPACE_R),
                )
                .with_click(Box::new(move || comp.activate_workspace(id))),
            ));
            DotState {
                workspace_id: id,
                is_active: false,
                width_slot: cache.insert(
                    AnimSpec::new(WORKSPACE_INACTIVE_W)
                        .with_duration(ANIM_DURATION)
                        .with_easing(Easing::EaseOutCubic),
                ),
                rect_slot: rect,
            }
        });

        // 2. Direct enum downcast matching
        if let Some(Node::Row(row_node)) = arena.get_mut(self.row_slot) {
            row_node.children = self.dots.items.iter().map(|(_, d)| d.rect_slot).collect();
        }

        // 3. Set the target widths for the individual workspace dots
        for dot in self.dots.iter_mut() {
            let active = snapshot.active_id == dot.workspace_id;
            if dot.is_active != active {
                dot.is_active = active;
                let target = if active { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
                cache.set_target(dot.width_slot, target, now);
            }
        }

        // 4. Update the panel width target immediately on structural changes
        let next_w = LEFT_PAD + self.calculate_content_width(cache) + RIGHT_PAD;
        cache.set_target(self.panel_width_slot, next_w, now);

        true
    }

    fn sync(
        &self,
        _now: f32,
        cache: &AnimationCache,
        arena: &mut ElementArena,
        _surface_w: f32,
        surface_h: f32,
    ) {
        if let Some(Node::Rect(bg_rect)) = arena.get_mut(self.bg_slot) {
            let pw = cache.value(self.panel_width_slot);
            bg_rect.size = Size::new(pw, surface_h);
        }

        // Synchronize active workspace dot animations
        for dot in self.dots.iter() {
            if let Some(Node::Rect(dot_rect)) = arena.get_mut(dot.rect_slot) {
                dot_rect.size.w = cache.value(dot.width_slot);
                dot_rect.style.fill = if dot.is_active { COLOR_DOT_ACTIVE } else { COLOR_DOT_INACTIVE };
            }
        }
    }
}
