# Project Brain

## 0. What Nexus Is
Nexus is a layered engine that turns perception data and natural language commands into live world updates rendered in Bevy. The loop flows from perception → routing → runtime → renderer, with motion compilation and research prototypes feeding future animation fidelity. World edits are expressed as JSON patches against `apps/nexus_desktop/assets/world.json`, letting each subsystem stay decoupled while sharing a common state contract.

## 1. Current Architecture
### 1. Runtime — JSON patch-based engine loop controlling world state
- Location: `runtime/main.py` watches or applies JSON patches and persists them into `apps/nexus_desktop/assets/world.json`.
- Responsibilities: poll `router/generated/command.json`, apply patch types like `spawn_entity`, `move_entity`, `set_color`, `move_camera`, `set_light`, and keep the world file consistent for the renderer.
- Interfaces: consumes router-generated patches; produces serialized world snapshots for downstream Bevy/renderer code.

### 2. Perception Kit — depth, pose, and flow via ONNX models (MoveNet + MiDaS)
- Location: expected under `perception/server.py`, emitting `perception/state.json` for runtime ingestion (see `docs/NexusQuickstart.md`).
- Responsibilities: run ONNX-based pose (MoveNet), depth (MiDaS), and flow inference; expose `/frame` for visualization; keep `perception/state.json` up to date.
- Status: wiring described in Quickstart; server implementation is external/pending in this repo.

### 3. Neural Motion Compiler — video → skeleton → animation (planned motion_compiler/)
- Location: `crates/motion_compiler/` Rust crate defines backends, configs, and mock compiler + library types.
- Responsibilities: ingest motion sources (video, keypoints, pose streams) and compile them into rig-ready animation blobs; provide storage via a motion library.
- Status: mock backend and data types landed; planning integration into the engine loop (M5).

### 4. Neural Renderer — neural/splat/NeRF experiments
- Location: `crates/neural_renderer/` Rust crate converts `world_state` snapshots into render requests and currently ships a mock backend. Output targets Bevy via higher-level app integration.
- Responsibilities: select renderer backend (mock today, future neural/splat/NeRF), transform `WorldSnapshot` into renderable entities, and surface rendered frame summaries.
- Status: mock renderer and request builder implemented; integration with the desktop app is part of the current milestone (M4).

### 5. Command Router — NL → schema → JSON patches
- Location: `router/server.py` (FastAPI) and `router/schema.json` define the command schema and patch generation contract.
- Responsibilities: accept natural language via `/command`, convert to structured patches through `router.commands`, persist patches to `router/generated/command.json`, and serve the `/ui` page for manual testing.
- Status: M3 router + UI completed; continues to feed runtime watch mode.

### 6. Docs & Tools — ProjectBrain, STATUS, FileIndex, ArchitectRules, prompt packs
- Location: docs live under `docs/` (this file, `STATUS.md`, `NexusQuickstart.md`). FileIndex/ArchitectRules/prompt packs are currently absent and should be added when available.
- Responsibilities: capture architectural intent, milestone status, repository standards, and prompt collections for operators.
- Status: Project Brain recreated; STATUS maintained alongside Quickstart for operator flow.

## 2. Active Milestone
- **M4: Neural Renderer Stub + Integration** — wire the `crates/neural_renderer` mock backend into the Bevy desktop app, ensuring it consumes `apps/nexus_desktop/assets/world.json` updates and can be swapped for future neural/splat/NeRF experiments.
- **M5: Motion Compiler Planning** — design the motion ingestion path (video → skeleton → animation) and prepare runtime/renderer hooks that will consume compiled motions from `crates/motion_compiler`.

## 3. Next Milestones
- Flesh out renderer backends beyond mock (splat/NeRF prototypes) and validate performance inside the Bevy loop.
- Implement motion compiler service surface (API + storage) and connect compiled motions to runtime entities.
- Expand command router grammar and schema to cover multi-entity selection, camera controls, and light manipulation with richer intents.
- Restore/author FileIndex and ArchitectRules docs to keep navigation and architectural constraints explicit.

## 4. Repository Standards
- JSON patch contract: runtime expects patches in `router/generated/command.json` as a list of objects with `type`, `id`, and `data` fields; keep compatibility with `runtime/main.py` handlers.
- World state location: `apps/nexus_desktop/assets/world.json` is the shared source of truth for renderer/Bevy.
- Router schema: align new commands with `router/schema.json` and the `router.commands` helpers before exposing via `/command` or `/ui`.
- Crate hygiene: new Rust crates should join the workspace with `Cargo.toml` entries and follow existing module patterns (as seen in `crates/neural_renderer` and `crates/motion_compiler`).
- Docs: update `docs/STATUS.md` for milestone changes and keep Project Brain in sync with real file paths; add FileIndex/ArchitectRules when available.

## 5. Decisions Log
- **M1:** Perception → Runtime integration established; runtime consumes `perception/state.json` and writes `world.json`.
- **M2:** Entity system upgrade delivered colliders/gravity and floor clamp in Bevy.
- **M3:** Command router + runtime watch + renderer reload + `/ui` completed; patches flow from `router/server.py` to `world.json` in real time.
- **M4 (in progress):** Integrate `crates/neural_renderer` mock backend with the desktop app while keeping hot-reload of `world.json`.

## 6. How to Work With Project Brain
- Treat this document as the architectural source of truth; update sections rather than rewriting wholesale to preserve historical context.
- When adding features, note the affected layer(s) and update the relevant section plus the Decisions Log.
- Keep file paths exact and verify against the repository to avoid drift (especially for router schema, runtime entrypoints, and crate locations).
- Cross-link with `docs/STATUS.md` for daily focus and `docs/NexusQuickstart.md` for operator runbooks.
