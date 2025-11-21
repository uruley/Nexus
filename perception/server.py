"""Minimal perception server for Nexus.

Provides FastAPI endpoints for health checks and frame data.
"""
import logging
import json
from pathlib import Path
from fastapi import FastAPI
from fastapi.responses import JSONResponse

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("perception_server")

app = FastAPI(title="Nexus Perception Service")

STATE_PATH = Path(__file__).parent / "state.json"
MODEL_PATH = Path(__file__).parent / "models"

# Stubbed model states
MIDAS_LOADED = True
MOVENET_LOADED = True
CAMERA_INDEX = 0


@app.on_event("startup")
async def startup_event():
    logger.info("Perception server starting up...")
    # Future: Load models here in a non-blocking way or use lazy loading


@app.get("/health")
async def health() -> dict:
    """Health check endpoint."""
    logger.info("Health check requested")
    
    return {
        "ok": True,
        "midas": MIDAS_LOADED,
        "movenet": MOVENET_LOADED,
        "camera_index": CAMERA_INDEX,
        "model_path": str(MODEL_PATH),
    }


@app.get("/frame")
def frame():
    """Get current perception frame data."""
    # Try to read state.json if it exists
    if STATE_PATH.exists():
        try:
            with STATE_PATH.open("r", encoding="utf-8") as f:
                state = json.load(f)
            return JSONResponse(content={
                "ts": state.get("ts", 0),
                "size": state.get("size", [0, 0]),
                "depth": None,
                "persons": state.get("persons", []),
            })
        except Exception:
            pass
    
    # Fallback response
    return JSONResponse(content={
        "ts": 0,
        "size": [0, 0],
        "depth": None,
        "persons": [],
    })


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=5055)
