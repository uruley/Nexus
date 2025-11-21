# Router Test Recipe

## Quickstart

Run the runtime from the repository root so it writes to the world that Bevy reads:

```bash
python runtime/main.py --watch router/generated/command.json
```

## Notes

- The runtime now resolves `world.json` relative to the repo root, ensuring it always updates `apps/nexus_desktop/assets/world.json` regardless of your working directory.
- After startup you will see a log like `[runtime] Using world file: <path>` confirming exactly which `world.json` is active.
