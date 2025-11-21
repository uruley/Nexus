from __future__ import annotations

import json
from pathlib import Path
from typing import List

from fastapi import FastAPI, Query
from fastapi.responses import JSONResponse

app = FastAPI(title="Nexus Command Router")

GENERATED_DIR = Path(__file__).resolve().parent / "generated"
COMMAND_PATH = GENERATED_DIR / "command.json"
DEFAULT_ENTITY_ID = "entity:cube:001"


def ensure_generated_dir() -> None:
    GENERATED_DIR.mkdir(parents=True, exist_ok=True)


def text_to_patches(text: str) -> List[dict]:
    normalized = " ".join(text.lower().split())

    move_vectors = {
        "move cube up": (0.0, 1.0, 0.0),
        "move cube down": (0.0, -1.0, 0.0),
        "move cube left": (-1.0, 0.0, 0.0),
        "move cube right": (1.0, 0.0, 0.0),
        "move cube forward": (0.0, 0.0, 1.0),
        "move cube back": (0.0, 0.0, -1.0),
    }

    if normalized in move_vectors:
        dx, dy, dz = move_vectors[normalized]
        return [
            {
                "id": DEFAULT_ENTITY_ID,
                "type": "move_entity",
                "data": {"dx": dx, "dy": dy, "dz": dz},
            }
        ]

    if normalized == "spawn cube":
        return [
            {
                "id": "entity:cube:new",
                "type": "spawn_entity",
                "data": {"kind": "cube"},
            }
        ]

    if normalized == "delete cube":
        return [
            {
                "id": DEFAULT_ENTITY_ID,
                "type": "delete_entity",
                "data": {},
            }
        ]

    color_map = {
        "make cube red": [1.0, 0.0, 0.0],
        "make cube blue": [0.0, 0.0, 1.0],
        "make cube green": [0.0, 1.0, 0.0],
    }

    if normalized in color_map:
        return [
            {
                "id": DEFAULT_ENTITY_ID,
                "type": "set_color",
                "data": {"color": color_map[normalized]},
            }
        ]

    return []


def write_patches(patches: List[dict]) -> dict:
    ensure_generated_dir()
    payload: dict | List[dict] = patches[0] if len(patches) == 1 else patches
    COMMAND_PATH.write_text(json.dumps(payload, indent=2))
    return payload


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
