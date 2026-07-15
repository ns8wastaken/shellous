use crate::components::layout_tree::LayoutNode;
use crate::components::rect::Size;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::Animated;
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
    width: Animated<f32>,
}

impl WorkspaceDot {
    pub(super) fn new(workspace_id: i32) -> Self {
        Self {
            workspace_id,
            is_active: false,
            width: Animated::new(WORKSPACE_INACTIVE_W)
                .with_duration(0.26)
                .with_easing(Easing::EaseOutCubic),
        }
    }
}

impl Element for WorkspaceDot {
    fn update(&mut self, event: &ShellEvent) -> bool {
        if let ShellEvent::WorkspaceUpdated(snapshot) = event {
            let was_active = self.is_active;
            self.is_active = snapshot.active_id == self.workspace_id;
            was_active != self.is_active
        } else {
            false
        }
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let target = if self.is_active { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
        if target != self.width.target() {
            self.width.set_target(target, absolute_time);
        }

        self.width.tick(absolute_time)
    }

    fn layout(&self, _available: Size) -> Size {
        Size { w: self.width.value(), h: WORKSPACE_R * 2.0 }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, _ctx: &RenderContext) {
        let fill = if self.is_active {
            Color { r: 1.0, g: 0.12, b: 0.14, a: 1.0 }
        } else {
            Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
        };
        batch.push(
            node.rect,
            DrawParams::Rect(RectStyle::solid(fill, WORKSPACE_R))
        );
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        node.rect.contains(x, y) && {
            ctx.state.compositor.activate_workspace(self.workspace_id);
            true
        }
    }
}
