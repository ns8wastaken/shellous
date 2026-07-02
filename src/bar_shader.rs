/// Fragment shader for the bar panel component.
///
/// Panel (top-left bar component):
///   ┌──────────────────────────────────┐
///   │  ●  ●  ●  ═══  ●  ●  ●          │
///   │                          ╱       │
///   └──────────────╲  ────────╯        │
///                   ╲  ╱  ← convex     │
///                    ╲╱      corner     │
///   ← concave curve on right edge
///
///  - Bottom-left: convex rounded corner (bulging outward)
///  - Right edge: concave (inverted) curve transitioning upward
///  - Border: thin light-blue inner stroke along bottom edge & bottom-left curve
///  - Workspace indicators: inactive are circles (dots), active is an elongated capsule
pub const BAR_FRAG_SRC: &str = include_str!("bar_shader.glsl");
