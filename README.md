# Nexus Workspace

This repository contains the **nexus** Cargo workspace. It hosts shared libraries alongside a Bevy desktop application that can be launched locally. The application drives a small intent-based simulation, exposes an HTTP API for remote control, and renders Heads-Up-Display (HUD) timing metrics so you can monitor how the simulation performs.

## Workspace layout

- `crates/anchor` – intent-driven simulation core with record/replay support and frame timing metrics.
- `crates/http_api` – Axum router that mirrors the simulation state and exposes `/entities`, `/diff`, and `/intent` endpoints.
- `crates/world_state` – world entity tracking primitives built with `glam` vectors.
- `apps/app` – Desktop Bevy app that runs the simulation, renders the HUD, and hosts the HTTP API.

## Prerequisites

- Rust toolchain (Rust 1.75+ is recommended). Install via [`rustup`](https://rustup.rs/).

## Building

```sh
cargo build
```

This command compiles all workspace members, including the Bevy application and libraries.

## Simulation overview

The `app` binary hosts the `AnchorPlugin`, which advances the world by consuming *intents*. Intents describe actions such as spawning, moving, or despawning entities. Each frame collects timing metrics (total frame time, time spent in Anchor systems, and render time) that are exposed to the HUD overlay rendered in the top-left corner of the window.

Simulation updates can be captured to disk or replayed via the command line:

- `--record <FILE>` stores every frame's intents and input events as newline-delimited JSON (`RecordedFrame`) objects.
- `--replay <FILE>` loads the same newline-delimited recording and feeds intents/input events back into the simulation.

Only one mode can be active at a time—`--record` conflicts with `--replay`, and omitting both runs the simulation in real time.

## Running the app

From the repository root:

```sh
cargo run -p app
```

Close the Bevy window or press <kbd>Esc</kbd> to stop the program. To record or replay a session, pass the appropriate flag:

```sh
# Record the current run to recordings/session.jsonl
cargo run -p app -- --record recordings/session.jsonl

# Replay a previously captured session
cargo run -p app -- --replay recordings/session.jsonl
```

The recording path is created on demand. Replay files must already exist and contain one JSON object per line.

## HTTP API

When the application starts it also launches an HTTP server bound to `127.0.0.1:8787`. The server keeps a mirror of the Bevy world state and streams intents into the simulation through the following endpoints:

- `GET /entities` – returns the latest tick number and an array of entity snapshots (`id`, position, velocity, size).
- `GET /diff?since=<tick>` – returns the changes (added/removed/changed entities) that occurred since a given tick.
- `POST /intent` – accepts an intent payload (`{"verb": "Spawn", "args": { ... }}`) and injects it into the simulation.

### Sample usage

With the app running in a separate terminal, interact with the API using `curl`:

```sh
# List all entities that currently exist
curl http://127.0.0.1:8787/entities

# Fetch only the changes after tick 120
curl "http://127.0.0.1:8787/diff?since=120"

# Spawn a cube via an intent
curl \
  -X POST http://127.0.0.1:8787/intent \
  -H 'Content-Type: application/json' \
  -d '{
        "verb": "Spawn",
        "args": {
          "pos": [0.0, 0.5, 0.0],
          "vel": [0.0, 0.0, 0.0],
          "size": [0.5, 0.5, 0.5]
        }
      }'
```

Successful `POST /intent` calls respond with `{"status": "accepted"}`. Validation errors return an explanatory `{"error": ...}` message with HTTP 400 status.
