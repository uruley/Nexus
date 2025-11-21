from __future__ import annotations

import json
from pathlib import Path
from typing import List

from fastapi import FastAPI, Query
from fastapi.responses import HTMLResponse, JSONResponse, RedirectResponse

from router.commands import DEFAULT_ENTITY_ID, text_to_patches

app = FastAPI(title="Nexus Command Router")

BASE_DIR = Path(__file__).resolve().parent
GENERATED_DIR = BASE_DIR / "generated"
UI_PATH = BASE_DIR / "ui" / "index.html"
UI_HTML = UI_PATH.read_text(encoding="utf-8") if UI_PATH.exists() else None
COMMAND_PATH = GENERATED_DIR / "command.json"
# Command UI manual flow (M3 Phase 3)
# 1) Start router:
#    uvicorn router.server:app --host 127.0.0.1 --port 5056 --reload
# 2) Open in browser:
#    http://127.0.0.1:5056/ui
# 3) Type commands like: move cube up, make cube red, spawn cube, delete cube
# 4) Observe patches JSON in the UI; runtime/main.py (watch mode) applies patches;
#    Bevy app updates the scene from those patches.


def ensure_generated_dir() -> None:
    """Ensure the router/generated directory exists."""

    GENERATED_DIR.mkdir(parents=True, exist_ok=True)


def write_patches(patches: List[dict]) -> List[dict]:
    """Persist the patches to router/generated/command.json.

    The payload is always written as a JSON list to keep the runtime contract
    consistent regardless of how many patches were generated.
    """

    ensure_generated_dir()
    COMMAND_PATH.write_text(json.dumps(patches, indent=2))
    return patches


@app.get("/", include_in_schema=False)
def root() -> RedirectResponse:
    """Redirect the bare root to the UI for convenience."""

    return RedirectResponse(url="/ui")


@app.get("/ui", response_class=HTMLResponse, include_in_schema=False)
def ui() -> HTMLResponse:
    """Serve the command UI HTML file from router/ui/index.html."""

    if UI_HTML is None:
        return HTMLResponse(
            status_code=500,
            content="<h1>UI missing</h1><p>router/ui/index.html not found.</p>",
        )

    return HTMLResponse(content=UI_HTML)


@app.get("/health")
def health() -> dict:
    return {"ok": True}


@app.get("/command")
def command(text: str = Query(..., description="Natural language command")):
    patches = text_to_patches(text)

    if not patches:
        return JSONResponse(
            status_code=400, content={"error": "Unknown command", "patches": []}
        )

    payload = write_patches(patches)
    return JSONResponse(content=payload)


# Manual test flow (M3 Command Router MVP)
# 1. Start command router:
#    uvicorn router.server:app --host 127.0.0.1 --port 5056 --reload
# 2. Send command from browser:
#    http://127.0.0.1:5056/command?text=move%20cube%20up
# 3. Check router/generated/command.json
# 4. Apply patch:
#    python runtime/main.py --patch router/generated/command.json
# 5. Run renderer:
#    cargo run -p app
#    -> Cube should move upward.
#    Commands to try: make cube red, spawn cube, delete cube.
