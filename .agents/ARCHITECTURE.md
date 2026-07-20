# Architecture

## Initialization Sequence

1. `main()` creates an `Arc<dyn Compositor>` (`HyprlandCompositor::new()`)
2. `Shell::new()` initializes Wayland globals, EGL context, calloop channel
3. `bar::mount()` composes `SurfaceSpec::Layer`, builds the Node tree (`ElementArena`), wires `Controller` instances, and calls `shell.mount()`
4. A `Vec<Box<dyn ShellModule>>` is created with `WorkspaceService::new(compositor)` + `ClockService::new()` (future services added here)
5. `shell.run(modules)` enters the event loop (never returns) — calloop-based, not a raw loop

## Module Dependency Graph

```
main.rs
  ├── shell/ (core infrastructure)
  │     ├── compositor.rs       — Compositor trait, CompositorEvent, SubscriptionId, StateCallback
  │     ├── egl.rs              — EglState (shared EGL context + program registry)
  │     ├── event.rs            — ShellEvent enum, ShellModule trait (pluggable services)
  │     ├── layer_surface.rs    — LayerSurface (impls Surface), ShellAnchor, ShellLayer
  │     ├── surface.rs          — SurfaceState, Surface trait, SurfaceKind enum
  │     ├── xdg_surface.rs      — XdgToplevelSurface (impls Surface; pass-2 scaffold)
  │     ├── runtime.rs          — Shell + SurfaceSpec/LayerSpec/ToplevelSpec, mount(), run()
  │     ├── state.rs            — ShellState (shared mutable state across dispatch/render)
  │     ├── managed_surface.rs  — ManagedSurface (per-surface state + Node tree + controllers + renderer)
  │     ├── wayland.rs          — WaylandState (conn + all globals), Dispatch impls
  │     └── surface_id.rs       — SurfaceId = usize
  ├── services/
  │     ├── workspace.rs        — WorkspaceService, WorkspaceHandle, WorkspaceState, WorkspaceSnapshot
  │     ├── clock.rs            — ClockService, ClockSnapshot (implements ShellModule)
  │     └── hyprland.rs         — HyprlandCompositor (Unix socket IPC, impls Compositor)
  ├── renderer/
  │     ├── mod.rs              — re-exports Renderer
  │     ├── types.rs            — Color struct (rgba/f32)
  │     ├── renderer.rs         — Renderer (per-surface EGL surface + render_frame + render_batch)
  │     ├── batch.rs            — DrawBatch, DrawCommand, Shape enum {Rect, Text}, ShapeGroups
  │     ├── animation/
  │     │     ├── mod.rs
  │     │     ├── easing.rs     — Easing enum (Linear, EaseOutCubic, EaseOutQuad)
  │     │     └── cache.rs      — AnimationCache, AnimSlot, AnimSpec
  │     └── programs/
  │           ├── mod.rs        — ProgramRegistry, RenderProgram trait
  │           ├── rect/
  │           │     ├── mod.rs  — RectProgram (instanced quad rendering via SDF shader)
  │           │     ├── style.rs — RectStyle, FillMode, CornerShape, Corners, GradientStop, LogicalInset
  │           │     ├── rect.vert — instanced quad vertex shader
  │           │     └── rect.frag — SDF fragment shader (corners, border, gradient, shadow)
  │           └── text/
  │                 ├── mod.rs  — TextProgram (fontdue glyph atlas + per-glyph quads)
  │                 ├── style.rs — TextStyle builder
  │                 ├── text.vert — text vertex shader
  │                 └── text.frag — text fragment shader
  ├── components/
  │     ├── geom.rs             — Rect, Size (geometry primitives + placement methods)
  │     ├── arena.rs            — Arena<T> (generational slot map for Node tree)
  │     ├── keyed_list.rs       — KeyedList<K,V> reconciler
  │     ├── layout_tree.rs      — LayoutNode (flat rect tree after layout pass)
  │     ├── ui.rs               — Node enum (7 base variants), Controller trait, RenderContext
  │     ├── base/               — Layout primitives (building blocks)
  │     │     ├── mod.rs        — re-exports
  │     │     ├── rect.rs       — RectNode (styled round-rect leaf, optional click callback)
  │     │     ├── text.rs       — TextNode (text leaf, self-measuring)
  │     │     ├── row.rs        — RowNode (horizontal stack + spacing)
  │     │     ├── column.rs     — ColumnNode (vertical stack + spacing)
  │     │     ├── group.rs      — Group (pass-through, all children share same rect)
  │     │     ├── align.rs      — Align (wraps child with Alignment)
  │     │     ├── alignment.rs  — Alignment enum (Fill/Center/TopCenter/Start/End)
  │     │     ├── stacks.rs     — stack_horizontal, stack_vertical layout algorithms
  │     │     └── padding.rs    — Padding (insets a child)
  │     └── widgets/            — Widget controllers (process ShellEvents, drive Node updates)
  │           ├── mod.rs
  │           └── bar/
  │                 ├── mod.rs        — mount() — composes SurfaceSpec::Layer, instantiates controllers
  │                 ├── left_panel.rs — LeftPanelController (workspace dots, animated panel width)
  │                 └── middle_panel.rs — MiddlePanelController (clock display)
  └── main.rs                   — Entry point: wires compositor, shell, bar and modules together
```

## Data Flow

### Surface Creation (polymorphic)
1. `Shell::mount(config, animations, arena, controllers)` allocates a `SurfaceId`, matches on `SurfaceSpec`:
   - `Layer(LayerSpec)` → `LayerSurface::new(...)` — sets anchor + size + exclusive_zone, commits
   - `Toplevel(ToplevelSpec)` → `XdgToplevelSurface::new(...)` — sets title + app_id + min/max_size, commits
2. Registers a `ManagedSurface { id, root, arena, kind, controllers, renderer: None, animations, ... }`
3. `wayland.wait_for_configure(state, kind.surface_state())` blocks until the protocol-specific `Dispatch` flips `configured = true`
4. Renderer is built using `kind.wl_surface()` + the assigned `kind.dimensions()`

### Event-Driven Pipeline

```
Shell::run() loop via calloop:
  │
  ├── Channel event received → ShellEvent dispatched to controllers:
  │     LoopData::handle_event()
  │       → state.update_surfaces(&event, now)   — pushes data through Controller::update()
  │         → if changed: Controller::sync() writes animated values into Node fields
  │       → LoopData::render_frame()
  │
  ├── Wayland fd readable → process_wayland():
  │     dispatch_pending events (input, configure, frame callbacks)
  │     → LoopData::render_frame()
  │
  └── Pluggable modules register their own calloop sources:
        WorkspaceService: subscribes to compositor events, sends ShellEvent::WorkspaceUpdated via channel
        ClockService: sends ShellEvent::ClockUpdated via timer
```

### Render Pipeline (inside LoopData::render_frame)

```
render_frame():
  if !state.any_dirty(): return

  Pass 1 — Animation Tick (CPU):
    state.tick_animations(now)          — interpolate animated values via AnimationCache
      → still_moving → request_frame() for animated surfaces

  Pass 2 — Layout (CPU):
    state.compute_layouts()             — Node::layout() + Node::layout_tree() per surface
      → produces LayoutNode tree (flat rects, no side effects)

  Pass 3 — Geometry Batching + GPU (CPU memory + GPU):
    state.render()
      → for each dirty surface:
          renderer.make_current()       — bind EGL context
          Node::draw(layout, &mut batch, ctx) — collect DrawCommands via batch.push(rect, params)
          batch.sort_by_shape()          — group Rect then Text commands
          renderer.render_frame(|| renderer.render_batch(&batch, w, h))
            ├── glClear + glViewport
            ├── iterates shape_groups()
            ├── match shape:
            │     Rect → RectProgram::draw_batch() — upload instances array, glDrawArraysInstanced
            │     Text → TextProgram::draw_batch() — rasterize glyphs, upload per-char quads, glDrawArrays
            └── eglSwapBuffers()

  Clear dirty flags on all surfaces
```

### Input Flow

```
wl_pointer events → ShellState (Dispatch impl in wayland.rs)
  ├── Enter { surface, surface_x, surface_y } → set_focus_by_surface:
  │       match focused WlSurface against surfaces[].kind.wl_surface() → focused_surface
  │     then pointer_pos = Some((x, y))
  ├── Motion → update pointer_pos
  ├── Leave → focused_surface = None, pointer_pos = None
  └── Button (BTN_LEFT press) → state.handle_click()
        └── find focused surface → surface.on_click(x, y, ctx)
              ├── controllers[].on_click() first (hit-tested against LayoutNode)
              └── then Node::on_click() for base-node RectNode callbacks
```

### ShellModule Subscription → ShellEvent → Controller::update()

```
Shell::run() calls module.register(handle, event_tx) for each module:

ClockService (calloop Timer):
  Timer fires at minute boundary → ShellEvent::ClockUpdated(ClockSnapshot) via event_tx

WorkspaceService (Compositor subscription):
  WorkspaceService::register() subscribes to compositor events via
    compositor.clone().subscribe_workspace_change(callback)
  callback receives CompositorEvent, updates Arc<Mutex<WorkspaceState>>, sends
    ShellEvent::WorkspaceUpdated(WorkspaceSnapshot) via event_tx

LoopData::handle_event():
  → ShellState::update_surfaces(&ShellEvent, now)
    → for each ManagedSurface, for each Controller:
        Controller::update(event, now, animations, arena) → bool (did change)
          → if changed: Controller::sync() writes current anim values into Node fields
            → dirty.set(true), layout_dirty.set(true)

Components never pull from services. Data flows down; clicks flow up.
```

## Key Design Decisions

### calloop Event Loop
The main loop is driven by calloop, not a busy-loop. Two core event sources: the internal channel (receiving `ShellEvent` from background modules) and the Wayland connection fd (dispatching protocol events). Pluggable `ShellModule` implementations register their own calloop sources (`Timer`, `Generic`, etc.) during `Shell::run()`. This replaces the old monolithic render loop with composable event sources.

### ShellModule Trait for Pluggable Services
Services implement `ShellModule` with `register(handle, tx)` to install their event sources and `initial_event()` → `Option<ShellEvent>` to seed the first frame. Currently `ClockService` and `WorkspaceService` use this; future services (tray, MPRIS, notifications) plug in the same way without touching the shell runtime.

### Controller Trait — Data Flow
The `Controller` trait (`components/ui.rs`) drives per-frame updates:
- **`update()`** — receives `ShellEvent`, reconciles child lists, sets animation targets. Returns `bool` (did anything change).
- **`sync()`** — called after `AnimationCache::tick`, writes current animated values into base `Node` fields (rect sizes, text content, styles). Also called after a change from `update()`.
- **`on_click()`** — optional click handling (default returns `false`, letting the base node tree handle `RectNode` callbacks).

`ManagedSurface` owns a `Vec<Box<dyn Controller>>`. `ShellState::update_surfaces` iterates controllers. Widgets (LeftPanel, MiddlePanel) are written as controllers and build their UI trees entirely from base components (no custom `Node` variants).

### Node Enum — Closed Set of Base Components
The `Node` enum has exactly 7 variants: `Rect`, `Text`, `Row`, `Column`, `Group`, `Align`, `Padding`. All are layout primitives — no widget-specific variants exist. Widgets (controllers) build their UI trees exclusively from these primitives via the `ElementArena` (`Arena<Node>`). The `Node` enum implements `layout()`, `layout_tree()`, `draw()`, and `on_click()` as match arms over all variants.

### Three-Pass Pipeline (Tick → Layout → Render)
Rendering is split into three decoupled phases:
1. **Animation Tick** — `AnimationCache::tick(now)` interpolates active animations; `Controller::sync()` writes results into Node fields.
2. **Layout** — `compute_layouts()` calls `Node::layout()` for size computation, then `Node::layout_tree()` to produce a `LayoutNode` tree (flat rect hierarchy). CPU only, pure functions.
3. **Render** — `render()` calls `Node::draw()` to collect `DrawCommand` structs into a `DrawBatch` (CPU memory, no GPU calls), sorts by shape, then submits to GPU via `render_batch()`.

### Instanced Rendering for Rect Shapes
`RectProgram` uses `glDrawArraysInstanced` with 15 vec4 instance attributes per rect, encoding position, size, fill, border, corner radii, gradient stops, shadow parameters, and logical insets into a single 60-float struct. This means all rects on a surface render in a single draw call.

Separate `Batch` iteration for `TextProgram` which uses per-character quads with fontdue rasterization and a texture atlas.

### Shape Extensibility
`Shape` enum (`Rect`, `Text`) drives dispatch in `render_batch()`. The `RenderProgram` trait abstracts per-shape rendering via `draw_batch(commands, surface_w, surface_h)`. `ProgramRegistry` (owned by `EglState`) maps `Shape` → `Box<dyn RenderProgram>`. Adding a new shape means: add a variant, register a `RenderProgram`, done.

### SDF-based Fragment Shader
`rect.frag` uses signed distance fields to compute pixel coverage for corners, borders, insets, shadows, and gradient fills — all in a single instanced draw call per element. Supports concave/convex corners, variable radii per corner, and soft anti-aliasing via a `softness` parameter.

### Text Rendering via fontdue + Texture Atlas
`TextProgram` uses `fontdue` for rasterization. Glyphs are cached in a dynamic texture atlas (on-demand upload via `glTexSubImage2D`). Each text command generates per-glyph quads with UV coordinates. The atlas grows top-left to bottom-right, wrapping to the next row when full.

### Trait-based Compositor Abstraction
The `Compositor` trait (`src/shell/compositor.rs`) decouples workspace queries from any specific backend. Currently only `HyprlandCompositor` exists. Methods: `workspaces()`, `active_workspace()`, `activate_workspace()`, `subscribe_workspace_change(callback) -> SubscriptionId`, `unsubscribe(id) -> bool`. The helper `refresh_state()` seeds an `Arc<Mutex<WorkspaceState>>`.

### Typed Events Instead of Bare Closures
Subscriber callbacks take `CompositorEvent` (e.g. `WorkspaceChanged { workspaces, active_id }`) — a `Clone` enum. The listener thread synthesises the event once per tick and clones it for each subscriber; subscribers see identical snapshots.

### `Arc<dyn Fn + Send + Sync>` For Cross-Thread Callbacks
`StateCallback = Arc<dyn Fn(CompositorEvent) + Send + Sync>`. The listener thread snapshots each callback's `Arc`, drops the `subs` mutex guard, then invokes the callback outside the lock. No callback-induced mutex poisoning.

### Simple Background Thread for Hyprland Listener
`HyprlandCompositor::subscribe_workspace_change` spawns a single `thread::spawn` that connects to Hyprland's `.socket2.sock`, reads lines in a loop, and dispatches `CompositorEvent` to all subscribers on relevant events (`workspace`, `createworkspace`, `destroyworkspace`, `moveworkspace`, `focusedmon`). On connection failure it sleeps and retries. No incarnation guards, no lazy-spawn tracking — one thread per subscriber.

### Shared GL State via `Arc<EglState>`
All surfaces share one EGL context (`EglState` owns it). Each `Renderer` holds an `Arc<EglState>` and creates its own `egl::Surface` (window surface). Before drawing, `make_current()` binds that surface. `EglState` also owns the `ProgramRegistry`.

### `Cell<bool>` for Per-Surface Flags
`ManagedSurface` uses `Cell<bool>` for `dirty`, `frame_pending`, `animating`, and `layout_dirty` to allow mutation through shared references during the dispatch loop, avoiding borrow-checker issues with `&mut self`.

### Surface Polymorphism
A `Surface` trait abstracts the operations the render loop + dispatch need: `dimensions()`, `wl_surface()`, `surface_state()`. `SurfaceKind { Layer, Toplevel }` enum implements `Surface` by pass-through. `SurfaceSpec { Layer(LayerSpec) | Toplevel(ToplevelSpec) }` is the mount-time mirror. Future kinds (Popups, Subsurfaces) slot in without touching the render loop.

### KeyedList for Reconciled Children
`KeyedList<K,V>` (`components/keyed_list.rs`) drives dot add/remove/reorder in `LeftPanelController`. `reconcile(cur_ids, factory)` computes the minimal insert/remove/keep operations to match the current ID set, using `Vec`-based iteration.

### Thread Safety for Workspace State
`WorkspaceState` lives inside `WorkspaceService`, wrapped in `Arc<Mutex<WorkspaceState>>` shared between:
- **Hyprland listener thread** (writes via the compositor subscription callback)
- **Main event loop** (reads via `WorkspaceHandle::snapshot()`)

Components never see the `Mutex` directly — they receive a `WorkspaceSnapshot` (clone) through `ShellEvent`.
