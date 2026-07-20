# Codebase Overview

**Project**: `shellous` (aka "Shellous")  
**Language**: Rust (edition 2024)  
**Purpose**: A custom Wayland status bar rendered via `wlr-layer-shell` protocol with OpenGL ES 3.0 shaders, targeting the Hyprland compositor. Surface protocol support is polymorphic (via a `Surface` trait over `Layer`/`Toplevel`) so future windows, popups, and notifications can plug into the same render loop.

## What It Does

Shellous draws a custom top bar on Hyprland that shows workspace indicators (left panel) and a centered clock (middle panel). Data flows unidirectionally: background services push `ShellEvent` variants through registered `ShellModule` callbacks into the calloop event loop, which dispatches them to `Controller::update()` on each surface. Components never pull from services. It connects to Hyprland's Unix sockets for workspace state and event listening, uses calloop for event-driven I/O, and uses OpenGL ES 3.0 fragment shaders with signed distance fields (SDFs) to render complex UI elements including rounded rectangles, concave corners, borders, gradients, drop shadows, and text via fontdue rasterization.

## High-Level Structure

| Directory / File | Purpose |
|---|---|---|
| `src/shell/` | Wayland connection, EGL/OpenGL setup, surface management, input dispatch, event loop |
| `src/shell/compositor.rs` | `Compositor` trait + `CompositorEvent` enum + `StateCallback` |
| `src/shell/event.rs` | `ShellEvent` enum, `ShellModule` trait for pluggable background services |
| `src/services/workspace.rs` | `WorkspaceService`, `WorkspaceHandle`, `WorkspaceState`, `WorkspaceSnapshot` |
| `src/services/clock.rs` | `ClockService`, `ClockSnapshot` — sends minute-boundary timer events |
| `src/services/hyprland.rs` | `HyprlandCompositor` — Hyprland IPC via Unix sockets |
| `src/renderer/` | Per-surface EGL rendering, `DrawBatch`, `Shape` enum (Rect, Text) |
| `src/renderer/batch.rs` | `DrawBatch`, `DrawCommand`, `DrawParams`, `Shape`, `ShapeGroups` |
| `src/renderer/renderer.rs` | `Renderer` — per-surface EGL surface + `render_frame()` + `render_batch()` |
| `src/renderer/types.rs` | `Color` struct (rgba/f32) |
| `src/renderer/animation/` | `AnimationCache`, `AnimSlot`, `AnimSpec`, `Easing` (Linear, EaseOutCubic, EaseOutQuad) |
| `src/renderer/programs/rect/` | `RectProgram` — instanced SDF quad rendering, `RectStyle` builder |
| `src/renderer/programs/text/` | `TextProgram` — fontdue glyph atlas + per-glyph quads, `TextStyle` builder |
| `src/components/geom.rs` | `Rect`, `Size` — geometry primitives + placement methods (`place_center`, `inset`, etc.) |
| `src/components/arena.rs` | `Arena<T>` + `Slot` — generational slot map for the Node tree |
| `src/components/ui.rs` | `Node` enum (7 base primitives), `Controller` trait, `RenderContext`, `ElementArena` |
| `src/components/base/` | Layout primitives: `RectNode`, `TextNode`, `RowNode`, `ColumnNode`, `Group`, `Align`, `Padding` |
| `src/components/base/alignment.rs` | `Alignment` enum (`Center`, `TopCenter`, `Start`, `End`, `Fill`) |
| `src/components/base/stacks.rs` | `stack_horizontal`, `stack_vertical` — pure-function layout algorithms |
| `src/components/widgets/bar/` | `mount()` + `LeftPanelController` + `MiddlePanelController` |
| `src/components/keyed_list.rs` | `KeyedList<K,V>` — reconciler for keyed child lists |
| `src/main.rs` | Entry point: wires compositor, shell, bar, and service modules together |

## Key Dependencies

- **calloop** — event loop (poll + timer + channel sources)
- **wayland-client** — core Wayland protocol bindings and event queue
- **wayland-egl** — bridges EGL window surfaces to `wl_surface` handles
- **wayland-protocols-wlr** — `zwlr-layer-shell-v1` (the bar's layer-shell protocol)
- **wayland-protocols** — `xdg-shell@6` (toplevel scaffolding, pass-2)
- **khronos-egl**, **gl** — EGL/OpenGL ES 3.0 rendering
- **fontdue** — glyph rasterization for text rendering
- **chrono** — local time formatting for the clock widget
- **serde** / **serde_json** — Hyprland JSON IPC deserialization
- **libloading** — Dynamic EGL library loading

## Running

Requires Hyprland (sets `HYPRLAND_INSTANCE_SIGNATURE` and `XDG_RUNTIME_DIR`).  
Build: `cargo build`  
Run: `cargo run`
