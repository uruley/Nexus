# Nexus Quickstart (Local Dev)

This guide covers the standard bring-up sequence for the Nexus Engine.

## 1. Start Perception (optional for first tests)

```bash
cd perception
uvicorn server:app --host 127.0.0.1 --port 5055 --reload
```

*   **Verify**: Open `http://127.0.0.1:5055/health` and check for `{"ok": true, ...}`.

## 2. Start Command Router + UI

Open a new terminal:

```bash
cd router
uvicorn server:app --host 127.0.0.1 --port 5056 --reload
```

*   **Verify**: Open `http://127.0.0.1:5056/ui` in your browser. You should see the command interface.

## 3. Start Runtime (watch mode)

Open a new terminal. Run from the **repo root**:

```bash
python runtime/main.py --watch router/generated/command.json
```

*   **Note**: The runtime automatically finds `apps/nexus_desktop/assets/world.json` relative to the repository root.
*   **Verify**: You should see `[runtime] Using world file: ...\apps\nexus_desktop\assets\world.json` followed by `watching ...`.

## 4. Start Bevy Desktop App

Open a new terminal:

```bash
cargo run -p nexus_desktop
```

*   **Verify**: A window titled "Nexus Engine" should open.
*   **Note**: On first run you'll see a default debug cube (magenta) if your world is empty. Once you issue commands via /ui, the live world will override this.
*   **Troubleshooting**: If the window doesn't appear, check the terminal for logs starting with `Nexus desktop app startup`.

## 5. Use the UI

1.  Go to `http://127.0.0.1:5056/ui`
2.  Type `spawn cube` and hit Enter.
3.  **Observe**:
    *   The UI shows a new patch in the JSON view.
    *   The runtime terminal logs `applied spawn_entity`.
    *   The Bevy window shows a white square (cube) appearing and falling to the floor.

### Common Commands

*   `move cube up` / `down` / `left` / `right`
*   `make cube red` / `blue` / `green`
*   `delete cube`
