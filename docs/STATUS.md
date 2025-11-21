# STATUS

## Recently Completed
- **M1: Perception → Runtime integration**
  - Processes `perception/state.json` into runtime `world.json`.
  - Establishes end-to-end data flow for camera-driven state.
- **M2: Entity system upgrade**
  - Added colliders and gravity to the Bevy entities.
  - Floor clamp keeps spawned cubes from falling indefinitely.
- **M3: Command router + runtime watch + renderer reload + /ui page**
  - FastAPI command router now produces JSON patches and serves a browser UI at `/ui`.
  - Runtime watch mode applies patches live from `router/generated/command.json`.
  - Renderer reloads `world.json` so Bevy updates without restarts.

## Today's Focus
- Follow `docs/NexusQuickstart.md` to run the full M1–M3 pipeline.
- Verify the loop end-to-end:
  - `/ui` sends commands and shows the generated patches.
  - Runtime watch mode applies patches without crashing.
  - The Bevy window updates live in response to commands.
- Potential next steps:
  - Expand natural language variations and synonyms.
  - Handle multiple entities cleanly.
  - Improve camera controls from commands or UI.
