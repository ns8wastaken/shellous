# Coding Conventions

## Module Structure

- One Rust file per logical unit (e.g., `runtime.rs`, `state.rs`, `layer_surface.rs`)
- `mod.rs` files declare submodules and re-export public types
- Shaders live in `renderer/shaders/` and are embedded via `include_str!()`
- UI components go under `components/` with their own subdirectories

## Naming

- **Types**: PascalCase (`ManagedSurface`, `ShellState`, `RectProgram`)
- **Functions**: snake_case (`make_current`, `render_frame`, `handle_click`)
- **Enums**: PascalCase variants (`Solid`, `LinearGradient`; `Top`, `Bottom`)
- **Constants**: SCREAMING_SNAKE_CASE (`QUAD_VERTS`, `WORKSPACE_SPACING`, `BTN_LEFT`)
- **Module files**: lowercase (`runtime.rs`, `egl.rs`)
- **Comment headers**: `// ==================== SECTION ====================`

## Error Handling

- Uses `unwrap()` / `expect()` liberally for initialization paths where failure means the app cannot function (missing Wayland globals, EGL failure, missing env vars)
- `eprintln!()` for non-fatal runtime errors (failed socket reads, JSON parse failures returned as empty/default)
- No custom error types — failures that can be recovered from return empty/default values (e.g., `Vec::new()`, `-1`)

## Patterns

### Trait Objects for Extensibility
- `Compositor: Send + Sync` — allows compositor backend swapping
- `Element` — dynamic dispatch for UI widgets (`Vec<Box<dyn Element>>`)

### Shared State via Arc<Mutex<>>
- `BarState` shared between Hyprland listener thread and render thread
- `SurfaceState` shared between Wayland dispatch and surface lifecycle

### Interior Mutability
- `Cell<bool>` for `dirty` / `frame_pending` flags on `ManagedSurface`
- `Mutex` for shared cross-thread state

### Default Trait
- Used consistently for style structs (`RectStyle`, `Corners`, `Color`, `LeftPanel`, `MiddlePanel`)
- Fallible construction uses `new()`, widgets use `Default`

### Bitflags
- `ShellAnchor` wraps a `u32` with const bit values and `BitOr`
- Manual implementation rather than the `bitflags` crate

### Shader Uniform Naming
- Prefix `u_` for uniforms (`u_color`, `u_radii`), `a_` for attributes (`a_position`), `v_` for varyings (`v_pixel`)

## Code Style

- Blank line between function definitions (usually 1, sometimes 2 for major sections)
- `// ----` comment lines for logical grouping within functions
- Trailing commas in struct/macro invocations
- `use` imports at top of file, grouped by crate then module
- Explicit `use` imports rather than wildcards (except for protocol enums like `wl_pointer::Event`)
- Helper functions prefixed with `draw_` for rendering (`draw_background`, `draw_workspace_indicators`)
- `pub` visibility only where needed; module-private helpers remain private

## Commit Convention

Commit messages are lowercase, informal, describing changes in past/present tense:
- "bar ui slight changes"
- "some cleanups? (i think)"
- "on_click simplification"
