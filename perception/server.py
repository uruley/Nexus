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

app = FastAPI()


# ---------------------------------------------------------------------------
# Environment configuration
# ---------------------------------------------------------------------------
DEFAULT_MODEL_PATH = "models/movenet_thunder.onnx"
MODEL_PATH = os.getenv("MOVENET_ONNX", DEFAULT_MODEL_PATH)
CAMERA_INDEX = int(os.getenv("CAMERA_INDEX", "0"))


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
    model_path = Path(MODEL_PATH)
    if not model_path.exists():
        print(f"[MoveNet] model file not found at {model_path}")
        return None, None

    try:
        session = ort.InferenceSession(str(model_path), providers=["CPUExecutionProvider"])
        input_name = session.get_inputs()[0].name
        print(f"[MoveNet] session initialised with input '{input_name}'")
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
        rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
        transformed = midas_transforms(Image.fromarray(rgb))

        if isinstance(transformed, dict):
            transformed = transformed.get("image", None)

        if transformed is None:
            raise RuntimeError("MiDaS transform returned no image tensor")

        if isinstance(transformed, Image.Image):
            transformed = np.array(transformed)

        if isinstance(transformed, np.ndarray):
            transformed = torch.from_numpy(transformed)

        if not torch.is_tensor(transformed):
            raise TypeError(f"Unexpected MiDaS transform output type: {type(transformed)!r}")

        if transformed.ndim == 3:
            transformed = transformed.unsqueeze(0)

        inp = transformed.to(dtype=torch.float32)
        with torch.no_grad():
            depth = midas(inp).squeeze().cpu().numpy()
        depth -= depth.min()
        depth /= depth.max() + 1e-6
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


def run_movenet(bgr: np.ndarray) -> List[dict]:
    if movenet_sess is None or movenet_in_name is None:
        return []

    try:
        height, width, _ = bgr.shape
        rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
        img = cv2.resize(rgb, (IN_W, IN_H)).astype(np.float32)[None, ...]
        img /= 255.0

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


@app.get("/frame")
def frame() -> JSONResponse:
    try:
        bgr = grab_frame()
        depth16 = estimate_depth_bgr(bgr)
        persons = run_movenet(bgr)

        height, width = bgr.shape[:2]
        payload = {
            "ts": int(time.time() * 1000),
            "size": [width, height],
            "depth": None if depth16 is None else {"format": "png16", "uri": encode_png16(depth16)},
            "persons": persons,
        }
        return JSONResponse(payload)
    except Exception as exc:  # pragma: no cover - protect runtime
        print("[/frame] fatal error:", exc)
        traceback.print_exc()
        return JSONResponse({"error": "internal", "detail": str(exc)}, status_code=500)
