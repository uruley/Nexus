# Nexus Rust Workspace

This repository hosts the `nexus` Cargo workspace that aggregates core crates and the desktop Bevy application used by the project.

## Workspace Layout

- `crates/anchor_core` – shared runtime primitives.
- `crates/intent_api` – HTTP interface and request/response types.
- `crates/world_introspection` – world state diagnostics utilities.
- `crates/perf_hud` – performance heads-up display components.
- `crates/unified_field` – shared data models.
- `apps/nexus_desktop` – Bevy-based desktop frontend.

## Building

```powershell
cd C:\Users\ruley\Nexus
cargo build --workspace
```

## Running the Desktop App

```powershell
cd C:\Users\ruley\Nexus
cargo run -p nexus_desktop
```

The desktop application launches a Bevy window powered by `MinimalPlugins` and `WinitPlugin`, initializing a simple scene with a 2D camera and a cube-like sprite.

