# Architecture

## Initialization Sequence

1. `main()` creates an `Arc<dyn Compositor>` (HyprlandCompositor)
2. `Shell::new()` initializes Wayland globals, EGL context, AND the `WorkspaceService` (which seeds workspace state and installs a compositor subscription)
3. `bar::mount()` obtains a `WorkspaceHandle` from `Shell::workspace().handle()` and mounts the bar via `SurfaceSpec::Layer(LayerSpec)`
4. `Shell::run()` enters the render loop (never returns)

## Module Dependency Graph

```
main.rs
  ├── shell/ (core infrastructure)
  │     ├── compositor.rs       — Compositor trait, CompositorEvent, SubscriptionId
  │     ├── layer_surface.rs    — LayerSurface (impls Surface), ShellAnchor, ShellLayer
  │     ├── surface.rs          — SurfaceState, Surface trait, SurfaceKind enum
  │     ├── xdg_surface.rs      — XdgToplevelSurface (impls Surface; pass-2 scaffold)
  │     ├── runtime.rs          — Shell + SurfaceSpec/LayerSpec/ToplevelSpec (mount)
  │     ├── state.rs            — ShellState (shared mutable state across dispatch/render)
  │     ├── managed_surface.rs  — ManagedSurface (per-surface state + Element list + kind)
  │     ├── wayland.rs          — WaylandState (conn + all globals), Dispatch impls
  │     ├── egl.rs              — EglState (shared GL context + RectProgram)
  │     └── surface_id.rs       — SurfaceId = usize
  ├── services/
  │     └── workspace.rs        — WorkspaceService, WorkspaceHandle, SubscriptionCleanup (RAII)
  ├── renderer/
  │     ├── mod.rs              — re-exports Renderer
  │     ├── renderer.rs         — Renderer (per-surface EGL surface + draw loop)
  │     ├── programs/rect.rs    — RectProgram (GLSL shader + VBO + uniform upload)
  │     └── shaders/            — .vert / .frag GLSL sources
  ├── ui.rs                     — Element trait, RenderContext, draw/click dispatch helpers
  ├── canvas.rs                 — Canvas drawing surface abstraction
  ├── hyprland.rs               — HyprlandCompositor (Unix socket IPC, impls Compositor)
  └── components/bar/
        ├── mod.rs              — mount() — composes SurfaceSpec::Layer
        ├── left.rs             — LeftPanel (workspace indicators)
        └── middle.rs           — MiddlePanel (centered clock/widget)
```

## Data Flow

### Surface Creation (polymorphic)
1. `Shell::mount(SurfaceSpec)` allocates a `SurfaceId` from `state.next_id`
2. Matches on `SurfaceSpec`:
   - `Layer(LayerSpec)` → `LayerSurface::new(...)` — sets anchor + size + exclusive_zone, commits
   - `Toplevel(ToplevelSpec)` → `XdgToplevelSurface::new(...)` — sets title + app_id + min/max_size, commits
3. Registers a `ManagedSurface { id, elements, kind: SurfaceKind, renderer: None, ... }`
4. `wayland.wait_for_configure(state, kind.surface_state())` blocks until the protocol-specific `Dispatch` (`ZwlrLayerSurfaceV1::Configure` or `XdgSurface::Configure`) flips `configured = true`
5. The renderer is built using `kind.wl_surface()` + the assigned `kind.dimensions()`

### Render Loop
```
Shell::run()
  ├── wayland.dispatch_pending()  — drain any events already buffered
  └── for each dirty surface:
        ├── request_frame()    — kind.wl_surface().frame(qh, id)
        ├── renderer.make_current()
        ├── renderer.render_frame(ctx, || draw_elements())
        │     ├── glClear + glViewport
        │     ├── draw closure → Element::draw() for each widget
        │     │     └── RectProgram::draw() → uniform upload + glDrawArrays
        │     └── eglSwapBuffers()
        └── dirty = false
  └── wayland.blocking_dispatch()  — wait for the next event (typically a frame Done)
```

### Input Flow
```
wl_pointer events → ShellState Dispatch impls
  ├── Enter { surface, surface_x, surface_y } → set_focus_by_surface:
  │       match focused WlSurface against surfaces[*].kind.wl_surface() → focused_surface
  │     then pointer_pos = Some((x, y))
  ├── Motion → update pointer_pos
  ├── Leave → focused_surface = None, pointer_pos = None
  └── Button (BTN_LEFT press) → state.handle_click()
        └── find focused surface → click_elements() in reverse z-order
              └── Element::on_click() → compositor.activate_workspace() etc.
```

### Compositor Subscription → WorkspaceService → Bar
```
WorkspaceService::new(compositor) called once by Shell::new
  ├── refresh_state(&state)        — synchronous seed: writes workspaces + active_id
  └── subscribe_workspace_change(Arc<Fn(CompositorEvent)>)  → SubscriptionId
        └── HyprlandCompositor stores it; spawns listener thread ONLY IF listener_count == 0

Listener thread (lazily spawned, respawned after panic via ListenerIncarnation Drop guard):
  └── read .socket2.sock line by line
        └── target event line (workspace / createworkspace / destroyworkspace / ...)?
              ├── build CompositorEvent::WorkspaceChanged { workspaces, active_id }  ONCE
              ├── snapshot subscriber IDs under brief lock (release lock before invoking)
              └── for each id: take the callback's Arc clone under brief lock, dispatch event.clone()
                    └── WorkspaceService callback mutates its Arc<Mutex<WorkspaceState>>
                          → next render frame: LeftPanel::draw() calls handle.snapshot()

bar::mount() obtains a WorkspaceHandle from shell.workspace().handle()
LeftPanel::new(handle) stores it; every draw() / on_click uses handle.snapshot()
or handle.read(|s| ...) — no manual mutex juggling in the UI layer.
```

## Key Design Decisions

### Trait-based Compositor Abstraction
The `Compositor` trait (`src/shell/compositor.rs`) decouples workspace queries from any specific backend. Currently only `HyprlandCompositor` exists, but other compositors (sway, niri) could be added by implementing the trait. Multi-subscriber; `subscribe_workspace_change(callback) -> SubscriptionId` and `unsubscribe(id) -> bool` provide explicit lifecycle.

### Typed Events Instead of Bare Closures
Subscriber callbacks take `CompositorEvent` (e.g. `WorkspaceChanged { workspaces, active_id }`) — a `Clone` enum — rather than `Box<dyn Fn() + Send>`. The listener synthesises the event ONCE per tick and clones it for each subscriber; subscribers see identical snapshots without re-querying the compositor (read once, broadcast many).

### `Arc<dyn Fn + Send + Sync>` Over `Box<dyn Fn>` For Callbacks
`StateCallback = Arc<dyn Fn(CompositorEvent) + Send + Sync>`. The listener thread snapshots each callback's `Arc`, drops the `subs` mutex guard, then invokes the callback outside the lock. Panics inside a callback (or unsubscribe-invoked-from-callback) leave the `subs` mutex unpoisoned — the loop can resume cleanly on the next tick.

### Lazy-Spawn + Resurrection for the Listener Thread
`HyprlandCompositor` holds `Arc<AtomicUsize> listener_count` + a `ListenerIncarnation` Drop guard on each thread. `subscribe_workspace_change` does `fetch_add(1)` and only spawns when the previous value was `0`. If the thread panics or exits, the Drop guard decrements; the next subscriber observes `0` and respawns. No permanently-stuck listener.

### RAII Subscription Cleanup
`SubscriptionCleanup` (in `src/services/workspace.rs`) wraps `(compositor: Arc<dyn Compositor>, id: SubscriptionId)` and calls `unsubscribe` in its `Drop`. The field is declared LAST in `WorkspaceService` so it drops FIRST (Rust's reverse-declaration drop order) — releasing the callback Arc on shell shutdown rather than leaking it indefinitely.

### Workspace Handle — Snapshot or Read Over Mutex-and-Drop
`WorkspaceHandle::snapshot()` returns a `Clone` `WorkspaceSnapshot` after one brief lock; `WorkspaceHandle::read(|s| ...)` lets callers compute under the held lock without exposing the `Mutex`. UI components no longer juggle explicit `Mutex::lock` → use → drop.

### Shared GL State via `Arc<EglState>`
All surfaces share one EGL context (`EglState` owns it). Each `Renderer` holds an `Arc<EglState>` and creates its own `egl::Surface` (window surface). Before drawing, `make_current()` binds that surface.

### Cell<bool> for Per-Surface Flags
`ManagedSurface` uses `Cell<bool>` for `dirty` and `frame_pending` to allow mutation through shared references during the Wayland dispatch loop, avoiding borrow-checker issues with `&mut self`.

### SDF-based Fragment Shader
`rect.frag` uses signed distance fields to compute pixel coverage for corners, borders, insets, shadows, and gradient fills — all in a single draw call per element. Allows concave corners, variable radii per corner, and soft anti-aliasing.

### Surface Polymorphism (Pass 2)
A `Surface` trait abstracts the operations the render loop + dispatch need: `dimensions()`, `wl_surface()`, `surface_state()`. `SurfaceKind { Layer, Toplevel }` enum implements `Surface` by pass-through. Replaces `ManagedSurface.layer: LayerSurface` with `ManagedSurface.kind: SurfaceKind`. Future kinds (Popups, Subsurfaces) slot in without touching the render loop.

### Polymorphic SurfaceSpec (Pass 2)
`SurfaceSpec { Layer(LayerSpec) | Toplevel(ToplevelSpec) | ... }` is the mount-time mirror of `SurfaceKind`. `Shell::mount` matches on the kind, creates the matching Wayland object, and registers the resulting `SurfaceKind`. UI components don't see protocol-level details.

### Thread Safety for Workspace State
`WorkspaceState` lives inside `WorkspaceService` (`src/services/workspace.rs`), wrapped in `Arc<Mutex<WorkspaceState>>` shared between:
- **Hyprland event listener thread** (writes via the singleton `WorkspaceService` callback)
- **Main render thread** (reads via `WorkspaceHandle::snapshot()` / `read(|s|)` in `LeftPanel::draw()`)

Components never see the `Mutex` directly — locks are acquired and released briefly through the handle. No manual lock-and-drop dance at the UI layer.
