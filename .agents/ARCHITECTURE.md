# Architecture

## Initialization Sequence

1. `main()` creates an `Arc<dyn Compositor>` (HyprlandCompositor)
2. `Shell::new()` initializes Wayland globals, EGL context, AND the `WorkspaceService` (which seeds workspace state and installs a compositor subscription)
3. `bar::mount()` mounts the bar via `SurfaceSpec::Layer(LayerSpec)` (the bar receives data later through the `update()` phase)
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
  │     ├── managed_surface.rs  — ManagedSurface (per-surface state + Element tree + renderer)
  │     ├── wayland.rs          — WaylandState (conn + all globals), Dispatch impls
  │     ├── egl.rs              — EglState (shared EGL context + RectProgram)
  │     └── surface_id.rs       — SurfaceId = usize
  ├── services/
  │     └── workspace.rs        — WorkspaceService, WorkspaceHandle, SubscriptionCleanup (RAII)
  ├── renderer/
  │     ├── mod.rs              — re-exports Renderer
  │     ├── renderer.rs         — Renderer (per-surface EGL surface + render_batch)
  │     ├── batch.rs            — DrawBatch, DrawCommand, Shape enum
  │     ├── programs/rect.rs    — RectProgram (GLSL shader + VBO + uniform upload)
  │     └── shaders/            — .vert / .frag GLSL sources
  ├── components/
  │     ├── rect.rs             — Rect, Size (geometry primitives + placement methods)
  │     ├── ui.rs               — Element trait, RenderContext
  │     ├── keyed_list.rs       — KeyedList<K,V> reconciler
  │     ├── layout/
  │     │     ├── mod.rs        — re-exports Alignment, stack_horizontal
  │     │     ├── align.rs      — Align container (wraps child with alignment)
  │     │     ├── alignment.rs  — Alignment enum + align_* helper functions
  │     │     ├── stacks.rs     — stack_horizontal, stack_vertical layout algorithms
  │     │     ├── group.rs      — Group container (pass-through, children overlap)
  │     │     ├── padding.rs    — Padding container (insets child)
  │     │     └── row.rs        — Row container (horizontal layout via stack_horizontal)
  │     └── bar/
  │           ├── mod.rs        — mount() — composes SurfaceSpec::Layer, layout constants
  │           ├── left.rs       — LeftPanel (workspace indicators row)
  │           ├── workspace_dot.rs — WorkspaceDot (single workspace pill indicator)
  │           └── middle.rs     — MiddlePanel (centered clock/widget, wrapped in Align)
  ├── services/ (deprecated)
  │     └── hyprland.rs         — HyprlandCompositor (Unix socket IPC, impls Compositor)
  └── main.rs                   — Entry point: wires compositor, shell, and bar together
```

## Data Flow

### Surface Creation (polymorphic)
1. `Shell::mount(SurfaceSpec)` allocates a `SurfaceId` from `state.next_id`
2. Matches on `SurfaceSpec`:
   - `Layer(LayerSpec)` → `LayerSurface::new(...)` — sets anchor + size + exclusive_zone, commits
   - `Toplevel(ToplevelSpec)` → `XdgToplevelSurface::new(...)` — sets title + app_id + min/max_size, commits
3. Registers a `ManagedSurface { id, root, kind: SurfaceKind, renderer: None, ... }`
4. `wayland.wait_for_configure(state, kind.surface_state())` blocks until the protocol-specific `Dispatch` (`ZwlrLayerSurfaceV1::Configure` or `XdgSurface::Configure`) flips `configured = true`
5. The renderer is built using `kind.wl_surface()` + the assigned `kind.dimensions()`

### Three-Pass Render Pipeline
```
Shell::run()

  Pass 1 — Update + Layout (CPU only):
    state.update_surfaces(&snapshot)    — push data through element tree
    state.tick_animations(now)          — interpolate animated values
    root.layout(available_size)         — compute desired sizes, no side effects

  Pass 2 — Geometry Batching (CPU memory):
    root.draw(root_rect, &mut batch, ctx) — collect DrawCommands into DrawBatch
    Elements push rect + style via batch.push(rect, &style)
    No GPU calls — purely building a command list

  Pass 3 — GPU Render (GPU):
    renderer.render_frame(ctx, || renderer.render_batch(&batch, w, h))
      ├── glClear + glViewport
      ├── match cmd.shape:
      │     Shape::Rect → RectProgram::draw() → uniform upload + glDrawArrays
      │     Shape::Circle → placeholder (shaders exist, not wired)
      └── eglSwapBuffers()
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
        └── find focused surface → root.on_click(root_rect, x, y, ctx)
              └── Element::on_click() → compositor.activate_workspace() etc.
```

### Compositor Subscription → WorkspaceService → Bar (via update())

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
                          → shell loop wakes via eventfd → Shell::run() takes one snapshot

Data is pushed, not pulled:
  Shell::run() takes ONE snapshot, calls ShellState::update_surfaces(&snapshot)
    → Group::update() → LeftPanel::update() + Align→MiddlePanel::update()
      → LeftPanel::update() calls dots.reconcile() to add/remove/reorder dots
      → each WorkspaceDot::update(snapshot) → sets is_active flag

Components never pull from services. Data flows down; clicks flow up.
```

## Key Design Decisions

### Three-Pass Rendering (Layout → Batch → GPU)
Rendering is split into three decoupled phases. **Pass 1** (`update/layout`) runs on the CPU and computes all element sizes via `layout()`. **Pass 2** (`geometry batching`) collects `DrawCommand` structs into a `DrawBatch` — still CPU memory, no GPU calls. **Pass 3** (`GPU render`) submits the entire batch: the renderer loops over commands, matches on `Shape` to select the shader program, and issues `glDrawArrays`. This separation makes it trivial to add instancing later (swap the loop for a single instanced call) and keeps all coordinate math (alignment, stacking) in CPU-only code.

### Declarative Layout Without Coordinates
Elements never see raw `x`/`y` coordinates in `draw()`. Positioning is handled by:
- **`Alignment` enum** (`Center`, `TopCenter`, `Start`, `End`, `Fill`) — applied by the `Align` container wrapper
- **`Rect::place_center(child)`, `Rect::inset(l,t,r,b)`** — return positioned `Rect` values, never `(f32, f32)` tuples
- **`stack_horizontal(bounds, sizes, spacing)`** — arranges children left-to-right
All layout computations are pure functions over `Rect` and `Size`. The `Align` container (`layout/align.rs`) wraps any child and resolves its position based on alignment. Components like `MiddlePanel` just draw at the `rect` they receive.

### Shape Extensibility
`DrawCommand` carries a `Shape` enum (`Rect`, `Circle`) rather than hardcoding a single shape type. The renderer dispatches on shape in a single match arm. Adding a new shape means: add a variant, add a `CircleProgram` (or similar), add an arm in `render_batch`. The batch and pipeline schema stay unchanged. Existing `batch.push(rect, &style)` still works — it defaults to `Shape::Rect`.

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

### Workspace Handle — Shell-internal Snapshot Source
`WorkspaceHandle` is used only by the shell runtime (`Shell::run()`) to produce `WorkspaceSnapshot` values. Components receive data through `Element::update()` — they never call `snapshot()` or hold a `WorkspaceHandle`. The brief `Mutex::lock` → snapshot → drop cycle happens once per workspace change, not once per component per frame.

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
- **Main render thread** (reads via `WorkspaceHandle::snapshot()` / `read(|s|)` in `LeftPanel::update()`)

Components never see the `Mutex` directly — locks are acquired and released briefly through the handle. No manual lock-and-drop dance at the UI layer.
