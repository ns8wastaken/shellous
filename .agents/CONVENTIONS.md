# Coding Conventions

## Module Structure

- One Rust file per logical unit (e.g., `runtime.rs`, `state.rs`, `layer_surface.rs`)
- `mod.rs` files declare submodules and re-export public types
- Shaders live in `renderer/programs/<name>/` and are embedded via `include_str!()`
- UI components go under `components/` with subdirectories for related groups
- Services implement `ShellModule` for pluggable background event sources. Each service owns its state and sends typed `ShellEvent` variants via a calloop channel sender

## Naming

- **Types**: PascalCase (`ManagedSurface`, `WorkspaceService`, `RectProgram`)
- **Functions**: snake_case (`make_current`, `render_frame`, `handle_click`)
- **Enums**: PascalCase variants (`Solid`, `LinearGradient`; `Top`, `Bottom`; `Layer`, `Toplevel`)
- **Constants**: SCREAMING_SNAKE_CASE (`QUAD_VERTS`, `WORKSPACE_SPACING`, `BTN_LEFT`)
- **Module files**: lowercase (`runtime.rs`, `egl.rs`)
- **Comment headers**: `// ==================== SECTION ====================`

## Error Handling

- Uses `unwrap()` / `expect()` liberally for initialization paths where failure means the app cannot function (missing Wayland globals, EGL failure, missing env vars)
- `eprintln!()` for non-fatal runtime errors (failed socket reads, JSON parse failures returned as empty/default)
- No custom error types — failures that can be recovered from return empty/default values (e.g., `Vec::new()`, `-1`)

## Patterns

### Trait Objects for Extensibility
- `Compositor: Send + Sync` — allows compositor backend swapping. Methods: `workspaces()`, `active_workspace()`, `activate_workspace()`, `subscribe_workspace_change(StateCallback) -> SubscriptionId`, `unsubscribe(id) -> bool`.
- `Controller` trait — per-frame data flow. Methods: `update(&mut self, event, now, cache, arena) -> bool`, `sync(&self, now, cache, arena, w, h)`, `on_click(&self, x, y, arena, layout, ctx) -> bool`.
- `ShellModule: Send + 'static` — pluggable background service. Methods: `register(handle, tx)`, `initial_event() -> Option<ShellEvent>`.
- `RenderProgram` — per-shape GPU rendering. Method: `draw_batch(commands, surface_w, surface_h)`.
- `Surface: Send` — implemented by `LayerSurface` and `XdgToplevelSurface`; the polymorphic `SurfaceKind` enum impls `Surface` by pass-through (`match` over variants).

### Three-Pass Render Pipeline
Rendering is split into three decoupled phases:
1. **Animation Tick (`tick_animations()`)** — CPU only. `AnimationCache::tick(now)` interpolates animated values; `Controller::sync()` writes results into Node fields (sizes, colors, text).
2. **Layout (`compute_layouts()`)** — CPU only. `Node::layout(available, cache, arena)` computes desired sizes; `Node::layout_tree(rect, cache, arena)` produces a `LayoutNode` tree (flat rect hierarchy, no side effects).
3. **Render (`render()`)** — GPU. `Node::draw(layout, &mut batch, ctx)` collects `DrawCommand` structs into `DrawBatch` via `batch.push(rect, params)`. `batch.sort_by_shape()` groups by shape. `renderer.render_batch(&batch, w, h)` dispatches shape groups to registered `RenderProgram`s.

### Declarative Layout Without Coordinates
Node `draw()` methods never see raw `x`/`y` coordinates. Position is given by the `rect` parameter from the `LayoutNode`. Elements draw at the given rect using `batch.push(rect, &style)`. Positioning is handled by:
- `Align` container + `Alignment` enum
- `Rect::place_center(child)`, `Rect::inset(l,t,r,b)` — return `Rect` directly
- `stack_horizontal(bounds, sizes, spacing)` / `stack_vertical(bounds, sizes, spacing)` — pure functions, return `Vec<Rect>`

### Node Enum — Closed Set of Primitives
The `Node` enum has exactly 7 variants: `Rect`, `Text`, `Row`, `Column`, `Group`, `Align`, `Padding`. No widget-specific variants. Widgets (controllers) build their UI trees from these primitives using the `ElementArena` (`Arena<Node>`). The `Node` enum implements `layout()`, `layout_tree()`, `draw()`, and `on_click()` as monolithic match arms — no per-variant traits.

### ShellEvent-Driven Data Flow
Services never directly mutate the UI. They send `ShellEvent` variants (currently `ClockUpdated(ClockSnapshot)`, `WorkspaceUpdated(WorkspaceSnapshot)`) through a `calloop::channel::Sender`. The main loop receives these in a channel callback, calls `ShellState::update_surfaces(&event, now)`, which iterates `Controller::update()` on each surface. Components filter events by `match` and return `true` if they changed.

### calloop Event Loop
The main loop is driven by `calloop::EventLoop`. Core sources: the internal `channel` (receiving `ShellEvent`) and the Wayland connection fd. `ShellModule::register()` installs additional sources (`Timer`, `Generic`, `channel`, etc.). The loop runs `run(None, &mut data, |_| {})` — never returns.

### Shared State via Arc with Clone Handles
- `WorkspaceState` is wrapped in `Arc<Mutex<WorkspaceState>>` inside `WorkspaceService`. Components receive `WorkspaceSnapshot` (clone) through `ShellEvent` — they never hold locks.
- `EglState` is shared across all surface renderers via `Arc<EglState>`. Only the main thread touches it, so no mutex needed.

### Multi-Subscriber Compositor Events
The `Compositor` trait exposes `subscribe_workspace_change(StateCallback) -> SubscriptionId` and `unsubscribe(id) -> bool`. Callbacks receive typed `CompositorEvent` variants (`WorkspaceChanged { workspaces, active_id }`). The listener thread snapshots callback Arcs and iterates IDs outside the `subs` mutex lock.

### `Arc<dyn Fn ... + Send + Sync>` for Cross-Thread Callbacks
Use `Arc`, not `Box`, for callback fields exposed cross-thread (`StateCallback`). `Arc` lets the dispatcher clone the callback reference and invoke it after dropping its own locks.

### Interior Mutability
- `Cell<bool>` for `dirty`, `frame_pending`, `animating`, `layout_dirty` flags on `ManagedSurface`
- `Mutex` for shared cross-thread state (subscriber map in `HyprlandCompositor`)
- `AtomicU64` for `next_sub_id`
- `RefCell` for mutable per-program state within `RectProgram` and `TextProgram` (instance buffer, glyph atlas)

### Trait-Based Polymorphism with Enum Variants
- Protocol-agnostic `Surface` trait gives the render loop a uniform view: `dimensions()`, `wl_surface()`, `surface_state()`.
- The enum `SurfaceKind { Layer, Toplevel }` implements `Surface` by pass-through `match`.
- The mount-time mirror is `SurfaceSpec { Layer(LayerSpec) | Toplevel(ToplevelSpec) }`. `Shell::mount` matches on the spec, builds the protocol-specific object, registers the matching `SurfaceKind`.

### `#[allow(dead_code)]` for API Surface
Used for fields / variants / imports that are part of an upcoming feature but not currently read. Always accompanied by an explanatory comment naming the upcoming consumer or capability — never used for genuine dead code.

### Default Trait
- Used for leaf-style structs (`RectStyle`, `Corners`, `Color`, `MiddlePanel`, `DrawBatch`)
- Fallible construction uses `new()`, widgets use `Default`
- Service constructors always use `new` (may require external dependencies)

### Bitflags
- `ShellAnchor` wraps a `u32` with const bit values and `BitOr`
- Manual implementation rather than the `bitflags` crate

### Shader Uniform Naming
- Prefix `u_` for uniforms (`u_color`, `u_radii`, `u_surface_size`), `a_` for attributes (`a_position`, `a_inst0`..`a_inst14`), `v_` for varyings (`v_pixel`)

### KeyedList for Keyed Child Reconciliation
`KeyedList<K,V>` (`components/keyed_list.rs`) drives dot add/remove/reorder in `LeftPanelController`. `reconcile(cur_ids, factory)` computes minimal insert/remove/keep operations against a `Vec<Option<V>>` backing store.

### Instanced Rect Rendering
`RectProgram` uses `glDrawArraysInstanced` with 15 vec4 instance attributes. Each `RectInstance` encodes position, quad size, fill color/mode, border, corner shapes, radii, gradient stops, shadow parameters, and inset. All rects on a surface render in a single draw call.

### Inline Tests in animation/cache.rs
`AnimationCache` has inline `#[cfg(test)] mod tests` with basic unit tests for insert/read, animation tick settling, no-op target setting, slot reuse, and chasing behavior. No external test framework.

## Code Style

- Blank line between function definitions (usually 1, sometimes 2 for major sections)
- `// ----` comment lines for logical grouping within functions
- Trailing commas in struct/macro invocations
- `use` imports at top of file, grouped by crate then module
- Explicit `use` imports rather than wildcards (except for protocol enums like `wl_pointer::Event`)
- Per-element `draw()` implementations push to `batch` via `batch.push(rect, DrawParams::Rect(style))` or `batch.push(rect, DrawParams::Text(style))`
- `pub` visibility only where needed; module-private helpers remain private
- Type-level section banners (`// ==================== TYPE NAME ====================`) before each Rust struct/enum and before each trait impl block; the same convention is used for non-Rust file sections (e.g., `// ==================== INTERNAL JSON TYPES ====================` in `hyprland.rs`).
- Builder pattern for `RectStyle`, `TextStyle`, `AnimSpec` — methods like `.with_duration()`, `.with_easing()`, `.corners()`, `.fill()` return `Self`.

## Commit Convention

Commit messages are lowercase, informal, describing changes in past/present tense:
- "bar ui slight changes"
- "some cleanups? (i think)"
- "on_click simplification"
- "more abstraction"
- "more refactoring"
