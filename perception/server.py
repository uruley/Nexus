import io
import time
import base64

import numpy as np
from PIL import Image
from fastapi import FastAPI
from fastapi.responses import JSONResponse
import cv2
import torch
import onnxruntime as ort

app = FastAPI()

# ---------- MiDaS ----------
midas = torch.hub.load("intel-isl/MiDaS", "MiDaS_small")  # small = CPU friendly
midas_transforms = torch.hub.load("intel-isl/MiDaS", "transforms").small_transform
midas.eval()

# ---------- MoveNet (ONNX) ----------
# Use a local file: ./models/movenet_thunder.onnx (put model in repo)
try:
    movenet_sess = ort.InferenceSession(
        "models/movenet_thunder.onnx", providers=["CPUExecutionProvider"]
    )
except Exception:  # pragma: no cover - fall back if model file unavailable
    movenet_sess = None
IN_H, IN_W = 256, 256
KEYPOINT_NAMES = [
    "nose", "left_eye", "right_eye", "left_ear", "right_ear",
    "left_shoulder", "right_shoulder", "left_elbow", "right_elbow",
    "left_wrist", "right_wrist", "left_hip", "right_hip", "left_knee",
    "right_knee", "left_ankle", "right_ankle"
]

cap = cv2.VideoCapture(0)

def grab_frame():
    ok, frame = cap.read()
    if not ok:
        # fallback blank
        frame = np.zeros((480, 640, 3), dtype=np.uint8)
    return frame

def estimate_depth_bgr(bgr):
    rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
    inp = midas_transforms(Image.fromarray(rgb)).unsqueeze(0)
    with torch.no_grad():
        depth = midas(inp).squeeze().cpu().numpy()
    # normalize to 16-bit for PNG
    d = depth - depth.min()
    d = (d / (d.max() + 1e-6) * 65535.0).astype(np.uint16)
    return d

def encode_png16(data16):
    pil = Image.fromarray(data16, mode="I;16")
    buf = io.BytesIO()
    pil.save(buf, format="PNG")
    b64 = base64.b64encode(buf.getvalue()).decode("ascii")
    return "data:image/png;base64," + b64

def run_movenet(bgr):
    if movenet_sess is None:
        h, w, _ = bgr.shape
        return [{
            "id": "p0",
            "score": 0.0,
            "bbox": [0.0, 0.0, float(w), float(h)],
            "keypoints": []
        }]

    h, w, _ = bgr.shape
    rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
    img = cv2.resize(rgb, (IN_W, IN_H)).astype(np.float32)[None, ...]
    # movenet expects [1,256,256,3] normalized 0..1
    img /= 255.0
    outputs = movenet_sess.run(None, {"input": img})
    # expected shape [1,1,17,3]: (y,x,score)
    kps = outputs[0][0][0]  # [17,3]
    keypoints = []
    for i, (yy, xx, cc) in enumerate(kps):
        keypoints.append({
            "name": KEYPOINT_NAMES[i],
            "x": float(xx * w),
            "y": float(yy * h),
            "c": float(cc)
        })
    # loose bbox
    xs = [kp["x"] for kp in keypoints]
    ys = [kp["y"] for kp in keypoints]
    x0, y0 = float(max(0, min(xs))), float(max(0, min(ys)))
    x1, y1 = float(min(w, max(xs))), float(min(h, max(ys)))
    return [{
        "id": "p0",
        "score": float(np.mean([k["c"] for k in keypoints])),
        "bbox": [x0, y0, x1, y1],
        "keypoints": keypoints
    }]


@app.get("/health")
def health():
    return {"ok": True}


@app.get("/frame")
def frame():
    bgr = grab_frame()
    depth16 = estimate_depth_bgr(bgr)
    persons = run_movenet(bgr)

    h, w = bgr.shape[:2]
    payload = {
        "ts": int(time.time() * 1000),
        "size": [w, h],
        "depth": {
            "format": "png16",
            "uri": encode_png16(depth16)
        },
        "persons": persons
    }
    return JSONResponse(payload)
