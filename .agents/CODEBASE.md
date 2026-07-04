# Codebase Overview

**Project**: `test-rs` (aka "Shellous")  
**Language**: Rust (edition 2024)  
**Purpose**: A custom Wayland status bar rendered via `wlr-layer-shell` protocol with OpenGL ES 3.0 shaders, targeting the Hyprland compositor.

## What It Does

Shellous draws a custom top bar on Hyprland that shows workspace indicators (left panel) and a centered panel (middle). It connects to Hyprland's Unix sockets for workspace state and event listening, and uses OpenGL ES 3.0 fragment shaders with signed distance fields (SDFs) to render complex UI elements including rounded rectangles, concave corners, borders, gradients, and drop shadows.

## High-Level Structure

| Directory | Purpose |
|-----------|---------|
| `src/shell/` | Wayland connection, EGL/OpenGL setup, layer surface management, input dispatch |
| `src/renderer/` | Per-surface EGL rendering, GLSL shaders (`rect.frag`, `circle.frag`), `RectProgram` |
| `src/ui.rs` | `Element` trait and `RenderContext` — the widget abstraction layer |
| `src/components/bar/` | Concrete UI widgets: workspace indicators (left), center panel (middle) |
| `src/hyprland.rs` | Hyprland IPC via Unix sockets, implements `Compositor` trait |
| `src/main.rs` | Entry point: wires compositor, shell, and bar together |

## Key Dependencies

- **wayland-client**, **wayland-egl**, **wayland-protocols-wlr** — Wayland protocol bindings
- **khronos-egl**, **gl** — EGL/OpenGL ES 3.0 rendering
- **serde** / **serde_json** — Hyprland JSON IPC deserialization
- **libloading** — Dynamic EGL library loading

## Running

Requires Hyprland (sets `HYPRLAND_INSTANCE_SIGNATURE` and `XDG_RUNTIME_DIR`).  
Build: `cargo build`  
Run: `cargo run`
