# Codebase Overview

**Project**: `test-rs` (aka "Shellous")  
**Language**: Rust (edition 2024)  
**Purpose**: A custom Wayland status bar rendered via `wlr-layer-shell` protocol with OpenGL ES 3.0 shaders, targeting the Hyprland compositor. Surface protocol support is polymorphic (via a `Surface` trait over `Layer`/`Toplevel`) so future windows, popups, and notifications can plug into the same render loop.

## What It Does

Shellous draws a custom top bar on Hyprland that shows workspace indicators (left panel) and a centered panel (middle). It connects to Hyprland's Unix sockets for workspace state and event listening, and uses OpenGL ES 3.0 fragment shaders with signed distance fields (SDFs) to render complex UI elements including rounded rectangles, concave corners, borders, gradients, and drop shadows.

## High-Level Structure

| Directory | Purpose |
|-----------|---------|
| `src/shell/` | Wayland connection, EGL/OpenGL setup, surface management, input dispatch |
| `src/shell/surface.rs` | `Surface` trait + `SurfaceKind` enum + `SurfaceState` — the polymorphic surface facade |
| `src/shell/layer_surface.rs` | `LayerSurface` — wraps `zwlr-layer-shell-v1` and implements `Surface` |
| `src/shell/xdg_surface.rs` | `XdgToplevelSurface` — wraps `xdg-shell@6` (XdgWmBase + XdgSurface + XdgToplevel) and implements `Surface`. Pass-2 scaffold awaiting the first toplevel consumer. |
| `src/shell/managed_surface.rs` | `ManagedSurface` — per-surface state, `kind: SurfaceKind`, element list, frame/dirty flags |
| `src/shell/state.rs` | `ShellState` — shared mutable state across the dispatch thread (pointer focus, surface registry, compositor Arc) |
| `src/shell/runtime.rs` | `Shell` — owns `WaylandState` + `ShellState` + `EglState` + `WorkspaceService`; provides `mount(SurfaceSpec)` and `run()` |
| `src/shell/wayland.rs` | `WaylandState` (connection, queue, globals: `wl_compositor`, `zwlr_layer_shell_v1`, `xdg_wm_base@6`, `wl_seat`) + Dispatch impls for everything the shell listens to |
| `src/shell/compositor.rs` | `Compositor` trait + `CompositorEvent` enum + `SubscriptionId` + `StateCallback` type alias |
| `src/shell/egl.rs` | `EglState` — shared EGL context + `RectProgram` |
| `src/services/workspace.rs` | `WorkspaceService` (owns `Arc<Mutex<WorkspaceState>>` and the compositor subscription), `WorkspaceHandle` (clones the inner Arc for components — exposes `snapshot()` + `read(\|s\|)`), `SubscriptionCleanup` RAII guard |
| `src/renderer/` | Per-surface EGL rendering (mod.rs re-exports), `RectProgram` (shaders + VBO + uniform upload), GLSL shaders under `shaders/` |
| `src/ui.rs` | `Element` trait + `RenderContext` — UI widget abstraction layer |
| `src/canvas.rs` | `Canvas` — drawing surface that wraps shader programs for UI elements |
| `src/components/bar/` | Concrete UI widgets: `LeftPanel` (workspace indicators), `MiddlePanel` (centered). Constructs `SurfaceSpec::Layer`. |
| `src/hyprland.rs` | `HyprlandCompositor` — Hyprland IPC via Unix sockets; implements `Compositor` trait with multi-subscriber support, typed event emission, lazy-spawned listener thread with resurrect-on-panic |
| `src/workspace.rs` | `Workspace` + `WorkspaceState` — compositor-agnostic types shared between bar and compositor backend |
| `src/main.rs` | Entry point: wires compositor, shell, and bar together |

## Key Dependencies

- **wayland-client** — core Wayland protocol bindings and event queue
- **wayland-egl** — bridges EGL window surfaces to `wl_surface` handles
- **wayland-protocols-wlr** — `zwlr-layer-shell-v1` (the bar's layer-shell protocol)
- **wayland-protocols** — `xdg-shell@6` (toplevel scaffolding added in pass 2)
- **khronos-egl**, **gl** — EGL/OpenGL ES 3.0 rendering
- **serde** / **serde_json** — Hyprland JSON IPC deserialization
- **libloading** — Dynamic EGL library loading

## Running

Requires Hyprland (sets `HYPRLAND_INSTANCE_SIGNATURE` and `XDG_RUNTIME_DIR`).  
Build: `cargo build`  
Run: `cargo run`
