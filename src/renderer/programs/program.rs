use std::collections::HashMap;

use crate::renderer::batch::{DrawCommand, Shape};

// ==================== SHAPE PROGRAM TRAIT ====================

/// Each shape variant registers a program that knows how to render
/// a batch of commands of that shape.  Programs own their own VAO,
/// VBO, and shader state.  The default `draw_batch` implementation
/// iterates individual commands; programs override it for instancing.
pub trait ShapeProgram {
    /// Render a slice of commands — all guaranteed to have the same
    /// `Shape` variant that this program registered for.
    fn draw_batch(&self, commands: &[DrawCommand], surface_w: f32, surface_h: f32);
}

// ==================== PROGRAM REGISTRY ====================

/// Owned collection of shape → program mappings.
///
/// The registry is built once at startup and shared across all
/// surface renderers via `Arc<EglState>`.  Registering a new shape
/// is as simple as implementing `ShapeProgram` and calling
/// `register` — no match arms, no batch changes.
pub struct ProgramRegistry {
    programs: HashMap<Shape, Box<dyn ShapeProgram>>,
}

impl ProgramRegistry {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
        }
    }

    pub fn register<S: ShapeProgram + 'static>(&mut self, shape: Shape, program: S) {
        self.programs.insert(shape, Box::new(program));
    }

    pub fn get(&self, shape: &Shape) -> Option<&dyn ShapeProgram> {
        self.programs.get(shape).map(|p| p.as_ref())
    }
}
