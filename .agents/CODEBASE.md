# Codebase Overview

**Project**: `shellous` (aka "Shellous")  
**Language**: Rust (edition 2024)  
**Purpose**: A custom Wayland status bar rendered via `wlr-layer-shell` protocol with OpenGL ES 3.0 shaders, targeting the Hyprland compositor. Surface protocol support is polymorphic (via a `Surface` trait over `Layer`/`Toplevel`) so future windows, popups, and notifications can plug into the same render loop.

## What It Does

Shellous draws a custom top bar on Hyprland that shows workspace indicators (left panel) and a centered panel (middle). Data flows unidirectionally: the shell runtime pushes `WorkspaceSnapshot` through the element tree via `Element::update()` — components never pull from services. It connects to Hyprland's Unix sockets for workspace state and event listening, and uses OpenGL ES 3.0 fragment shaders with signed distance fields (SDFs) to render complex UI elements including rounded rectangles, concave corners, borders, gradients, and drop shadows.

## High-Level Structure

| Directory / File | Purpose |
|---|---|
| `src/shell/` | Wayland connection, EGL/OpenGL setup, surface management, input dispatch |
| `src/shell/compositor.rs` | `Compositor` trait + `CompositorEvent` enum |
| `src/services/workspace.rs` | `WorkspaceService`, `WorkspaceHandle`, `SubscriptionCleanup` RAII guard |
| `src/renderer/` | Per-surface EGL rendering, `DrawBatch`, `RectProgram`, GLSL shaders |
| `src/renderer/batch.rs` | `DrawBatch`, `DrawCommand`, `Shape` enum (Rect, Circle) |
| `src/renderer/renderer.rs` | `Renderer` — per-surface EGL surface + `render_frame()` + `render_batch()` |
| `src/renderer/programs/rect.rs` | `RectProgram` — GLSL shader + VBO + uniform upload |
| `src/components/rect.rs` | `Rect`, `Size` — geometry primitives + placement methods (`place_center`, `inset`, etc.) |
| `src/components/ui.rs` | `Element` trait + `RenderContext` — UI widget abstraction layer |
| `src/components/keyed_list.rs` | `KeyedList<K,V>` — reconciler for keyed child lists |
| `src/components/layout/` | Container elements + layout algorithms |
| `src/components/layout/alignment.rs` | `Alignment` enum (`Center`, `TopCenter`, `Start`, `End`, `Fill`) + `align_*` helpers |
| `src/components/layout/stacks.rs` | `stack_horizontal`, `stack_vertical` — pure-function layout algorithms |
| `src/components/layout/align.rs` | `Align` container — wraps child with `Alignment` |
| `src/components/layout/group.rs` | `Group` — pass-through container (all children get the same rect) |
| `src/components/layout/padding.rs` | `Padding` — insets a child element |
| `src/components/layout/row.rs` | `Row` — horizontal arrangement via `stack_horizontal` |
| `src/components/bar/` | Concrete UI widgets: `LeftPanel`, `WorkspaceDot`, `MiddlePanel` |
| `src/services/hyprland.rs` | `HyprlandCompositor` — Hyprland IPC via Unix sockets |
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
