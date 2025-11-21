# Nexus Quickstart (M1–M3 Demo)

## Overview
This demo wires the full Nexus loop together: camera input feeds perception, runtime applies patches into `world.json`, and the Bevy app renders a cube you can manipulate live from the browser-based UI.

## Prerequisites
- Python installed with the project dependencies available.
- Rust and `cargo` installed.
- Repository cloned locally with dependencies already fetched.

## Step 1: Start perception (optional for now)
From the repo root, start your perception server to emit `perception/state.json` (use the same command you normally use, e.g., `python perception/server.py`). The `/frame` endpoint is optional for the command router demo; you can proceed without live camera input.

## Step 2: Start the Command Router + UI
From the repo root:

```bash
uvicorn router.server:app --host 127.0.0.1 --port 5056 --reload
```

Then open the browser at `http://127.0.0.1:5056/ui`. The UI provides a textbox, Send button, command history, and a live JSON patch display showing what will be written to `router/generated/command.json`.

## Step 3: Start runtime in watch mode
Use the runtime watcher to apply patches as they land in `router/generated/command.json`:

```bash
python runtime/main.py --watch router/generated/command.json
```

The runtime will detect new commands, apply them to `apps/nexus_desktop/assets/world.json`, and log each patch as it runs. Leave this process running while you test.

## Step 4: Start the Bevy app
In a separate shell from the repo root:

```bash
cargo run -p app
```

A window titled “Nexus” should appear, showing a cube that briefly falls under gravity before resting on the floor. HUD text in the corner reports frame timing and diagnostics.

## Step 5: Drive the cube from the UI
In the `/ui` page, try commands such as:

- `move cube up`
- `move cube down`
- `make cube red`
- `spawn cube`
- `delete cube`

Patches for each command will display in the UI. The runtime logs every application, and the cube(s) in the Bevy window should move, change color, appear, or disappear without restarting the app.

## Troubleshooting
- If `cargo` hits a temporary 403 from crates.io but you have built before, the cached artifacts may still let the app run.
- If the cube does not react, confirm that runtime watch mode is active and pointing at `router/generated/command.json`.
- If `/ui` does not load, ensure `uvicorn` is running on port 5056.
