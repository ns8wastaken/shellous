# Coding Conventions

## Module Structure

- One Rust file per logical unit (e.g., `runtime.rs`, `state.rs`, `layer_surface.rs`)
- `mod.rs` files declare submodules and re-export public types
- Shaders live in `renderer/shaders/` and are embedded via `include_str!()`
- UI components go under `components/` with their own subdirectories
- Cross-cutting shell-level services (shared data + subscription/teardown logic) go under `services/`. Each service exposes
  - a process-singleton owning the actual state
  - `pub fn handle(&self) -> ...Handle` returning a cheap-to-clone handle for component use
  - RAII cleanup tied to its lifetime

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
- `Compositor: Send + Sync` — allows compositor backend swapping
- `Element` — dynamic dispatch for UI widgets (`Vec<Box<dyn Element>>`). Trait methods: `update()` (receive data), `sync_children()` (reconcile child tree by ID), `tick_animations()` (interpolate animated values), `draw()`, `on_click()`, `id()`, `size()`.
- `Surface: Send` — implemented by `LayerSurface` and `XdgToplevelSurface`; the polymorphic `SurfaceKind` enum impls `Surface` by pass-through (`match` over variants).

### Shared State via Arc<Mutex<>> with RAII Handles
- `WorkspaceState` is owned by `WorkspaceService` (`src/services/workspace.rs`), wrapped in `Arc<Mutex<WorkspaceState>>`
- Components never see the `Mutex`. They receive a `WorkspaceHandle` clone from `WorkspaceService::handle()` and read via:
  - `handle.snapshot()` — one brief lock, returns `Clone` `WorkspaceSnapshot`
  - `handle.read(|s| ...)` — runs a closure under the held lock, returns whatever the closure returns
- `SurfaceState` is shared between Wayland protocol dispatcher and the render loop as `Arc<Mutex<SurfaceState>>` per surface (carried as `user_data` in the Dispatch impls).

### Multi-Subscriber Compositor Events
The `Compositor` trait exposes `subscribe_workspace_change(callback) -> SubscriptionId` and `unsubscribe(id) -> bool`. Callbacks are typed `CompositorEvent` variants (`WorkspaceChanged { workspaces, active_id }`) rather than raw closures. The listener thread snapshots callback Arcs and iterates IDs outside the `subs` mutex lock — no callback-induced mutex poisoning.

### `Arc<dyn Fn ... + Send + Sync>` for Cross-Thread Callbacks
Use `Arc`, not `Box`, for callback fields exposed cross-thread (`StateCallback`). `Arc` lets the dispatcher clone the callback reference and invoke it after dropping its own locks.

### Interior Mutability
- `Cell<bool>` for `dirty` / `frame_pending` flags on `ManagedSurface`
- `Mutex` for shared cross-thread state (always wrapped behind a Handle for component use)
- `AtomicUsize` / `AtomicU64` where the access pattern is hot and lock-free is desirable (e.g., `listener_count`, `next_sub_id`)

### RAII Drop Guards for Cleanup
- `SubscriptionCleanup` (`src/services/workspace.rs`) — holds `(compositor: Arc<dyn Compositor>, id: SubscriptionId)`, calls `unsubscribe` on Drop. Declared LAST in `WorkspaceService` so the field drops FIRST (Rust's reverse-declaration drop order).
- `ListenerIncarnation` (`src/hyprland.rs`) — holds `Arc<AtomicUsize>`, decrements on Drop. Guarantees the next subscriber can detect a dead listener and respawn.
- RAII guards with side effects in `Drop` carry `#[allow(dead_code)]` (the Rust dead-code lint doesn't count `Drop` as a field read). Comment the attribute with the upcoming consumer / capability so the intent is explicit.

### Trait-Based Polymorphism with Enum Variants
- Protocol-agnostic `Surface` trait gives the render loop a uniform view: `dimensions()`, `wl_surface()`, `surface_state()`.
- The enum `SurfaceKind { Layer, Toplevel }` (and future variants like `Popup`, `Subsurface`) implements `Surface` by pass-through `match`.
- The mount-time mirror is `SurfaceSpec { Layer(LayerSpec) | Toplevel(ToplevelSpec) | ... }`. `Shell::mount` matches on the spec, builds the protocol-specific object, registers the matching `SurfaceKind`.

### `#[allow(dead_code)]` for API Surface
Used for fields / variants / imports that are part of an upcoming feature but not currently read. Always accompanied by an explanatory comment naming the upcoming consumer or capability — never used for genuine dead code.

### Default Trait
- Used consistently for leaf-style structs (`RectStyle`, `Corners`, `Color`, `MiddlePanel`)
- Fallible construction uses `new()`, widgets use `Default` (exception: `LeftPanel::new(bottom_offset)` because of the compositor offset dependency)
- Service constructors always use `new` (services may require external dependencies)

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
- Per-element `draw()` implementations inline all rendering; shared draw helpers live as free functions in the same module
- `pub` visibility only where needed; module-private helpers remain private
- Type-level section banners (`// ==================== TYPE NAME ====================`) before each Rust struct/enum and before each trait impl block; the same convention is used for non-Rust file sections (e.g., `// ==================== INTERNAL JSON TYPES ====================` in `hyprland.rs`).

## Commit Convention

Commit messages are lowercase, informal, describing changes in past/present tense:
- "bar ui slight changes"
- "some cleanups? (i think)"
- "on_click simplification"
- "more abstraction"
- "more refactoring"
