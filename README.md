# Nexus Workspace

This repository contains the **Nexus** Cargo workspace. It hosts shared libraries alongside a Bevy desktop application that can be launched locally.

## Workspace layout

- `crates/anchor` – placeholder logic for configuring and initializing anchor services.
- `crates/http_api` – Axum-based HTTP API that mirrors the Bevy world's state and accepts intents.
- `crates/world_state` – serializable world snapshots, diffs, and checksum utilities.
- `apps/app` – Bevy application that opens a demo-friendly window with HUD and HTTP hooks.
- `docs/ProjectBrain.md` – overview of the multi-agent workflow, prompts, and runtime loop supporting Nexus.
- `codex/` – prompts and task definitions that guide architect, builder, critic, and runner personas.
- `runtime/`, `router/`, `perception/`, `assets/` – shared infrastructure for the multi-agent loop.

## Prerequisites

- Rust toolchain (Rust 1.75+ is recommended). Install via [`rustup`](https://rustup.rs/).

## Building

```sh
cargo build
```

This command compiles all workspace members, including the Bevy application and libraries.

## Running the app

```sh
cargo run -p app [-- [--record <FILE> | --replay <FILE>] [--duration <seconds>] [--bind <ADDR:PORT>]]
```

Key runtime options:

- `--record <FILE>` – write a simulation recording.
- `--replay <FILE>` – replay a simulation from disk.
- `--duration <seconds>` – automatically exit after the requested number of seconds (fractions are supported).
- `--bind <ADDR:PORT>` – HTTP server bind address (defaults to `127.0.0.1:8787`).

While running you can press `Esc` to close the window at any time. The application centers a 1280×720 window titled **Nexus**, uses `PresentMode::AutoVsync`, and sets a tinted clear color to keep the scene presentation-ready.

## Interacting with the HTTP API

With the app running, world state is exposed over HTTP:

- `GET /world/snapshot` – fetch the latest world snapshot, including a checksum.
- `GET /world/diff?since=<checksum>` – fetch a diff from a previously observed checksum to the latest state.
- `POST /intent` – submit intents such as spawn, move, apply_force, and despawn.

The checksum returned by `/world/snapshot` can be reused in `/world/diff` to incrementally stay in sync. Update the bind address via `--bind` when launching the app to expose the API on a different interface.
