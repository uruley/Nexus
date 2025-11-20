"""Perception microservice providing depth + pose estimates."""

from __future__ import annotations

import base64
import io
import os
import time
import traceback
from pathlib import Path
from typing import List, Optional

import cv2
import numpy as np
import onnxruntime as ort
import torch
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from PIL import Image
import json

app = FastAPI()


# ---------------------------------------------------------------------------
# Environment configuration
# ---------------------------------------------------------------------------
BASE_DIR = Path(__file__).resolve().parent


def _resolve_model_path(raw: str) -> Path:
    candidate = Path(raw).expanduser()
    if not candidate.is_absolute():
        candidate = BASE_DIR / candidate
    return candidate


movenet_input_type: Optional[str] = None

DEFAULT_MODEL_PATH = BASE_DIR / "models" / "movenet_thunder.onnx"
MODEL_PATH = _resolve_model_path(os.getenv("MOVENET_ONNX", str(DEFAULT_MODEL_PATH)))
CAMERA_INDEX = int(os.getenv("CAMERA_INDEX", "0"))
STATE_PATH = BASE_DIR / "state.json"


# ---------------------------------------------------------------------------
# MiDaS depth estimation (optional)
# ---------------------------------------------------------------------------
def _load_midas() -> tuple[Optional[torch.nn.Module], Optional[torch.nn.Module]]:
    try:
        midas_model = torch.hub.load("intel-isl/MiDaS", "MiDaS_small")
        midas_model.eval()
        midas_transforms = torch.hub.load("intel-isl/MiDaS", "transforms").small_transform
        print("[MiDaS] model loaded successfully")
        return midas_model, midas_transforms
    except Exception as exc:  # pragma: no cover - depends on external download
        print("[MiDaS] init error:", exc)
        traceback.print_exc()
        return None, None


midas, midas_transforms = _load_midas()


# ---------------------------------------------------------------------------
# MoveNet pose estimation (optional)
# ---------------------------------------------------------------------------
def _load_movenet() -> tuple[Optional[ort.InferenceSession], Optional[str]]:
    model_path = MODEL_PATH
    print(f"[MoveNet] DEBUG: MODEL_PATH = {MODEL_PATH!r}, type = {type(MODEL_PATH)}")
    print(f"[MoveNet] DEBUG: BASE_DIR = {BASE_DIR!r}")
    print(f"[MoveNet] DEBUG: model_path.exists() = {model_path.exists()}")
    if not model_path.exists():
        print(f"[MoveNet] model file not found at {model_path}")
        return None, None

    try:
        session = ort.InferenceSession(str(model_path), providers=["CPUExecutionProvider"])
        input_meta = session.get_inputs()[0]
        input_name = input_meta.name
        global movenet_input_type
        movenet_input_type = input_meta.type
        print(f"[MoveNet] session initialised with input '{input_name}' (type={movenet_input_type})")
        return session, input_name
    except Exception as exc:  # pragma: no cover - depends on external file
        print("[MoveNet] init error:", exc)
        traceback.print_exc()
        return None, None


movenet_sess, movenet_in_name = _load_movenet()
IN_H, IN_W = 256, 256
KEYPOINT_NAMES: List[str] = [
    "nose",
    "left_eye",
    "right_eye",
    "left_ear",
    "right_ear",
    "left_shoulder",
    "right_shoulder",
    "left_elbow",
    "right_elbow",
    "left_wrist",
    "right_wrist",
    "left_hip",
    "right_hip",
    "left_knee",
    "right_knee",
    "left_ankle",
    "right_ankle",
]


# ---------------------------------------------------------------------------
# Video capture (optional)
# ---------------------------------------------------------------------------
cap = cv2.VideoCapture(CAMERA_INDEX)


def grab_frame() -> np.ndarray:
    ok, frame = cap.read()
    if not ok or frame is None:
        frame = np.zeros((480, 640, 3), dtype=np.uint8)
    return frame


# ---------------------------------------------------------------------------
# Utility helpers
# ---------------------------------------------------------------------------
def estimate_depth_bgr(bgr: np.ndarray) -> Optional[np.ndarray]:
    if midas is None or midas_transforms is None:
        return None
    try:
        rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB).astype(np.float32)
        rgb /= 255.0

        sample = midas_transforms(rgb)
        if isinstance(sample, dict):
            tensor = sample.get("image")
        else:
            tensor = sample

        if tensor is None:
            raise RuntimeError("MiDaS transform returned no tensor")

        if isinstance(tensor, np.ndarray):
            tensor = torch.from_numpy(tensor)

        if not torch.is_tensor(tensor):
            raise TypeError(f"Unexpected MiDaS transform output type: {type(tensor)!r}")

        if tensor.ndim == 3:
            tensor = tensor.unsqueeze(0)

        inp = tensor.to(dtype=torch.float32)
        with torch.no_grad():
            depth = midas(inp).squeeze().cpu().numpy()
        depth -= float(depth.min())
        depth /= float(depth.max()) + 1e-6
        depth16 = (depth * 65535.0).astype(np.uint16)
        return depth16
    except Exception as exc:  # pragma: no cover - protect runtime
        print("[MiDaS] run error:", exc)
        traceback.print_exc()
        return None


def encode_png16(data16: np.ndarray) -> str:
    pil_image = Image.fromarray(data16, mode="I;16")
    buffer = io.BytesIO()
    pil_image.save(buffer, format="PNG")
    encoded = base64.b64encode(buffer.getvalue()).decode("ascii")
    return "data:image/png;base64," + encoded


def write_state_json(payload: dict) -> None:
    """Write a simplified perception state file.

    Errors are logged but never raised to the HTTP layer.
    """
    try:
        summary = {
            "ts": payload.get("ts"),
            "size": payload.get("size"),
            "persons": payload.get("persons", []),
            "has_depth": payload.get("depth") is not None,
        }
        STATE_PATH.parent.mkdir(parents=True, exist_ok=True)
        STATE_PATH.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    except Exception as exc:  # pragma: no cover - protect runtime
        print("[state.json] write error:", exc)
        traceback.print_exc()


def run_movenet(bgr: np.ndarray) -> List[dict]:
    if movenet_sess is None or movenet_in_name is None:
        return []

    try:
        height, width, _ = bgr.shape
        rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
        input_dtype = np.float32
        if movenet_input_type == "tensor(int32)":
            input_dtype = np.int32
        elif movenet_input_type == "tensor(int64)":
            input_dtype = np.int64

        img = cv2.resize(rgb, (IN_W, IN_H)).astype(input_dtype)
        if np.issubdtype(input_dtype, np.floating):
            img /= 255.0
        img = img[None, ...]

        outputs = movenet_sess.run(None, {movenet_in_name: img})
        if not outputs:
            return []

        arr = outputs[0]
        if arr.ndim == 4:
            keypoints_raw = arr[0, 0]
        elif arr.ndim == 3:
            keypoints_raw = arr[0]
        else:
            raise RuntimeError(f"Unexpected MoveNet output shape: {arr.shape}")

        keypoints = []
        xs: List[float] = []
        ys: List[float] = []
        for name, (yy, xx, cc) in zip(KEYPOINT_NAMES, keypoints_raw):
            x = float(xx * width)
            y = float(yy * height)
            xs.append(x)
            ys.append(y)
            keypoints.append({"name": name, "x": x, "y": y, "c": float(cc)})

        if keypoints:
            x0 = float(max(0.0, min(xs)))
            y0 = float(max(0.0, min(ys)))
            x1 = float(min(width, max(xs)))
            y1 = float(min(height, max(ys)))
            score = float(np.mean([kp["c"] for kp in keypoints]))
        else:  # pragma: no cover - defensive
            x0 = y0 = 0.0
            x1 = float(width)
            y1 = float(height)
            score = 0.0

        return [
            {
                "id": "p0",
                "score": score,
                "bbox": [x0, y0, x1, y1],
                "keypoints": keypoints,
            }
        ]
    except Exception as exc:  # pragma: no cover - protect runtime
        print("[MoveNet] run error:", exc)
        traceback.print_exc()
        return []


# ---------------------------------------------------------------------------
# API endpoints
# ---------------------------------------------------------------------------
@app.get("/health")
def health() -> dict:
    return {
        "ok": True,
        "midas": midas is not None,
        "movenet": movenet_sess is not None,
        "camera_index": CAMERA_INDEX,
        "model_path": str(MODEL_PATH),
    }


def _fallback_payload(ts: int) -> dict:
    return {"ts": ts, "size": [0, 0], "depth": None, "persons": []}


def _safe_write_state(payload: dict) -> None:
    try:
        write_state_json(payload)
    except Exception:
        # Errors already logged inside write_state_json; nothing else to do.
        pass


def _safe_response(payload: dict, ts: int) -> JSONResponse:
    try:
        return JSONResponse(payload)
    except Exception as exc:  # pragma: no cover - final guard
        print("[/frame] serialization error:", exc)
        traceback.print_exc()
        return JSONResponse(_fallback_payload(ts))


@app.get("/frame")
def frame() -> JSONResponse:
    ts = int(time.time() * 1000)
    payload = _fallback_payload(ts)

    try:
        bgr = grab_frame()
        height, width = bgr.shape[:2]
        payload["size"] = [int(width), int(height)]

        depth16 = estimate_depth_bgr(bgr)
        if depth16 is not None:
            try:
                payload["depth"] = {"format": "png16", "uri": encode_png16(depth16)}
            except Exception as exc:
                print("[/frame] depth encoding error:", exc)
                traceback.print_exc()

        persons = run_movenet(bgr)
        if persons:
            payload["persons"] = persons
    except Exception as exc:  # pragma: no cover - protect runtime
        print("[/frame] fatal error:", exc)
        traceback.print_exc()
    finally:
        _safe_write_state(payload)

    return _safe_response(payload, ts)
