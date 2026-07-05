use crate::components::canvas::{DrawingSurface, Rect};
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::Animated;
use crate::renderer::animation::easing::Easing;
use crate::renderer::programs::rect::{
    Color, Mat3, RectStyle,
};
use crate::services::workspace::WorkspaceSnapshot;

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
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        self.is_active = snapshot.active_id == self.workspace_id;
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let target = if self.is_active { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
        if target != self.width.target() {
            self.width.set_target(target, absolute_time);
        }

        self.width.tick(absolute_time)
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let w = self.width.value();
        let fill = if self.is_active {
            Color { r: 1.0, g: 0.12, b: 0.14, a: 1.0 }
        } else {
            Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
        };
        surface.draw_rect(
            w,
            WORKSPACE_R * 2.0,
            &RectStyle::solid(fill, WORKSPACE_R),
            Mat3::identity(),
        );
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        Rect::from_size(self.width.value(), WORKSPACE_R * 2.0).contains(x, y) && {
            ctx.state.compositor.activate_workspace(self.workspace_id);
            true
        }
    }

    fn id(&self) -> Option<i32> {
        Some(self.workspace_id)
    }

    fn size(&self) -> (f32, f32) {
        (self.width.value(), WORKSPACE_R * 2.0)
    }
}
