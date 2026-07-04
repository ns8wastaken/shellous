# Architecture

## Initialization Sequence

1. `main()` creates an `Arc<dyn Compositor>` (HyprlandCompositor)
2. `Shell::new()` initializes Wayland globals, EGL context, and an empty surface list
3. `bar::mount()` refreshes bar state from Hyprland, spawns the event listener thread, then mounts the bar layer surface
4. `Shell::run()` enters the render loop (never returns)

## Module Dependency Graph

```
main.rs
  ├── shell/ (core infrastructure)
  │     ├── compositor.rs       — Compositor trait
  │     ├── layer_surface.rs    — WaylandState, LayerSurface, ShellAnchor, ShellLayer
  │     ├── runtime.rs          — Shell (owner of all state), SurfaceSpec
  │     ├── state.rs            — ShellState (shared mutable state)
  │     ├── managed_surface.rs  — ManagedSurface (per-surface state + Element list)
  │     ├── wayland.rs          — Dispatch impls for Wayland protocol objects
  │     ├── egl.rs              — EglState (shared GL context + RectProgram)
  │     └── surface_id.rs       — SurfaceId = usize
  ├── renderer/
  │     ├── mod.rs              — re-exports Renderer
  │     ├── renderer.rs         — Renderer (per-surface EGL surface + draw loop)
  │     ├── programs/rect.rs    — RectProgram (GLSL shader + VBO + uniform upload)
  │     └── shaders/            — .vert / .frag GLSL sources
  ├── ui.rs                     — Element trait, RenderContext, draw/click dispatch helpers
  ├── canvas.rs                 — Canvas drawing surface abstraction
  ├── hyprland.rs               — HyprlandCompositor (Unix socket IPC)
  ├── workspace.rs              — Workspace / WorkspaceState types
  └── components/bar/
        ├── mod.rs              — mount(), wiring, surface spec
        ├── left.rs             — LeftPanel (workspace indicators)
        └── middle.rs           — MiddlePanel (centered clock/widget)
```

## Data Flow

### Surface Creation
1. `Shell::mount(SurfaceSpec)` allocates an ID, creates a `zwlr_layer_surface_v1`, sets anchor/size/exclusive zone
2. Blocks on `wait_for_configure()` until the compositor assigns dimensions
3. Creates a `Renderer` (per-surface EGL window) and stores the `ManagedSurface`

### Render Loop
```
Shell::run()
  ├── wayland.dispatch()  — process Wayland events (configure, pointer, frame callbacks)
  └── for each dirty surface:
        ├── request_frame()  — requests a wl_surface.frame callback for vsync
        ├── renderer.make_current()
        ├── renderer.render_frame(ctx, || draw_elements())
        │     ├── glClear + glViewport
        │     ├── draw closure → Element::draw() for each widget
        │     │     └── RectProgram::draw() → uniform upload + glDrawArrays
        │     └── eglSwapBuffers()
        └── dirty = false
```

### Input Flow
```
wl_pointer events → ShellState Dispatch impls
  ├── Enter/Motion → update focused_surface + pointer_pos
  ├── Leave → clear focus
  └── Button (left click) → state.handle_click()
        └── find focused surface → click_elements() in reverse z-order
              └── Element::on_click() → compositor.activate_workspace() etc.
```

### Hyprland Event → Bar Update
```
bar::mount() registers callback via compositor.on_workspace_change()
  └── HyprlandCompositor spawns a separate thread, blocking read on .socket2.sock
        └── workspace events detected → runs the callback
              └── compositor.refresh_state(Arc<Mutex<WorkspaceState>>)
                    └── lock mutex, update WorkspaceState.workspaces + active_id
                          → next render frame picks up new state in LeftPanel::draw()
```

## Key Design Decisions

### Trait-based Compositor Abstraction
The `Compositor` trait (`src/shell/compositor.rs`) decouples workspace queries from any specific backend. Currently only `HyprlandCompositor` exists, but other compositors (sway, niri) could be added by implementing the trait.

### Shared GL State via `Arc<EglState>`
All surfaces share one EGL context (`EglState` owns it). Each `Renderer` holds an `Arc<EglState>` and creates its own `egl::Surface` (window surface). Before drawing, `make_current()` binds that surface.

### Cell<bool> for Per-Surface Flags
`ManagedSurface` uses `Cell<bool>` for `dirty` and `frame_pending` to allow mutation through shared references during the Wayland dispatch loop, avoiding borrow-checker issues with `&mut self`.

### SDF-based Fragment Shader
Instead of traditional geometry-based rendering, `rect.frag` uses signed distance fields to compute pixel coverage for corners, borders, insets, shadows, and gradient fills — all in a single draw call per element. This allows concave corners, variable radii per corner, and soft anti-aliasing.

### Thread Safety for Workspace State
`WorkspaceState` is wrapped in `Arc<Mutex<WorkspaceState>>` shared between:
- **Hyprland event listener thread** (writes workspace data via callback)
- **Main render thread** (reads workspace data during `LeftPanel::draw()`)
- Locks are held briefly in both cases, avoiding contention.
