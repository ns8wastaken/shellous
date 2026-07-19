use crate::components::layout_tree::LayoutNode;
use crate::components::rect::Size;
use crate::components::ui::RenderContext;
use crate::renderer::animation::cache::{AnimationCache, AnimSlot, AnimSpec};
use crate::renderer::animation::easing::Easing;
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::rect::RectStyle;
use crate::renderer::types::Color;
use crate::shell::event::ShellEvent;

pub(super) const WORKSPACE_R: f32 = 5.5;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;

pub(super) struct WorkspaceDot {
    workspace_id: i32,
    is_active: bool,
    pub(super) width: AnimSlot,
}

impl WorkspaceDot {
    pub(super) fn new(workspace_id: i32, cache: &mut AnimationCache) -> Self {
        Self {
            workspace_id,
            is_active: false,
            width: cache.insert(
                AnimSpec::new(WORKSPACE_INACTIVE_W)
                    .with_duration(0.26)
                    .with_easing(Easing::EaseOutCubic),
            ),
        }
    }

    pub(super) fn update(
        &mut self,
        event: &ShellEvent,
        now: f32,
        cache: &mut AnimationCache,
    ) -> bool {
        if let ShellEvent::WorkspaceUpdated(snapshot) = event {
            let was_active = self.is_active;
            self.is_active = snapshot.active_id == self.workspace_id;
            if was_active != self.is_active {
                let target = if self.is_active {
                    WORKSPACE_ACTIVE_W
                } else {
                    WORKSPACE_INACTIVE_W
                };
                cache.set_target(self.width, target, now);
                return true;
            }
        }
        false
    }

    pub(super) fn layout(&self, _available: Size, cache: &AnimationCache) -> Size {
        Size {
            w: cache.value(self.width),
            h: WORKSPACE_R * 2.0,
        }
    }

    pub(super) fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, _ctx: &RenderContext) {
        let fill = if self.is_active {
            Color::rgb(1.0, 0.12, 0.14)
        } else {
            Color::rgb(0.25, 0.28, 0.35)
        };
        batch.push(
            node.rect,
            DrawParams::Rect(RectStyle::solid(fill, WORKSPACE_R)),
        );
    }

    pub(super) fn on_click(
        &self,
        node: &LayoutNode,
        x: f32,
        y: f32,
        ctx: &RenderContext,
    ) -> bool {
        node.rect.contains(x, y)
            && {
                ctx.state.compositor.activate_workspace(self.workspace_id);
                true
            }
    }
}
