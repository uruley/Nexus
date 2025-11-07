"""Demonstration pipeline combining MiDaS depth estimation and MoveNet pose detection.

The heavy dependencies (PyTorch, TensorFlow Lite, etc.) are optional. When they
are unavailable, lightweight stubs are used so the script can still execute and
illustrate the data flow.
"""
from __future__ import annotations

import math
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Sequence, Tuple

import numpy as np

try:  # pragma: no cover - optional dependency branch
    import torch  # type: ignore
    _TORCH_AVAILABLE = True
except Exception:  # pragma: no cover - optional dependency branch
    torch = None  # type: ignore
    _TORCH_AVAILABLE = False

try:  # pragma: no cover - optional dependency branch
    from tflite_runtime.interpreter import Interpreter  # type: ignore
    _TFLITE_AVAILABLE = True
except Exception:  # pragma: no cover - optional dependency branch
    Interpreter = None  # type: ignore
    _TFLITE_AVAILABLE = False


@dataclass
class PoseLandmark:
    """A single landmark returned by the pose detector."""

    name: str
    x: float
    y: float
    confidence: float


class DepthEstimator:
    """Adapter for MiDaS-style models."""

    def __init__(self) -> None:
        self._available = _TORCH_AVAILABLE

    def infer(self, image: np.ndarray) -> np.ndarray:
        height, width = image.shape[:2]
        if not self._available:
            # Deterministic synthetic depth map for demonstration.
            xv, yv = np.meshgrid(np.linspace(-1.0, 1.0, width), np.linspace(-1.0, 1.0, height))
            depth = 1.0 / (np.sqrt(xv**2 + yv**2) + 1.0)
            return depth.astype(np.float32)

        # Placeholder for actual MiDaS inference call.
        # In a real implementation this would involve running the model via torch.hub or
        # a pretrained checkpoint.
        return np.full((height, width), 0.5, dtype=np.float32)


class PoseEstimator:
    """Adapter for MoveNet-style pose detectors."""

    def __init__(self) -> None:
        self._available = _TFLITE_AVAILABLE

    def infer(self, image: np.ndarray) -> List[PoseLandmark]:
        if not self._available:
            # Produce a simple stick figure pose.
            height, width = image.shape[:2]
            mid_x = width / 2.0
            mid_y = height / 2.0
            return [
                PoseLandmark("nose", mid_x, mid_y - height * 0.2, 0.5),
                PoseLandmark("left_wrist", mid_x - width * 0.2, mid_y, 0.4),
                PoseLandmark("right_wrist", mid_x + width * 0.2, mid_y, 0.4),
                PoseLandmark("left_ankle", mid_x - width * 0.1, mid_y + height * 0.3, 0.4),
                PoseLandmark("right_ankle", mid_x + width * 0.1, mid_y + height * 0.3, 0.4),
            ]

        # Placeholder path for MoveNet inference.
        return []


@dataclass
class DemoResult:
    depth_map: np.ndarray
    landmarks: List[PoseLandmark]


class DepthPoseDemo:
    """High level orchestrator combining depth and pose estimators."""

    def __init__(self) -> None:
        self.depth_estimator = DepthEstimator()
        self.pose_estimator = PoseEstimator()

    def run(self, image: np.ndarray) -> DemoResult:
        depth_map = self.depth_estimator.infer(image)
        landmarks = self.pose_estimator.infer(image)
        return DemoResult(depth_map=depth_map, landmarks=landmarks)


def load_demo_image(path: Path | None) -> np.ndarray:
    if path and path.exists():  # pragma: no cover - optional branch
        from imageio.v3 import imread  # type: ignore

        data = imread(path)
        if data.ndim == 2:
            data = np.stack([data] * 3, axis=-1)
        return data.astype(np.float32) / 255.0

    # Fallback synthetic gradient image.
    width, height = 640, 480
    xv, yv = np.meshgrid(np.linspace(0.0, 1.0, width), np.linspace(0.0, 1.0, height))
    image = np.stack([xv, yv, np.flipud(xv)], axis=-1)
    return image.astype(np.float32)


def summarize_landmarks(landmarks: Iterable[PoseLandmark]) -> str:
    formatted = ", ".join(f"{lm.name}@({lm.x:.1f},{lm.y:.1f})" for lm in landmarks)
    return formatted or "<no landmarks>"


def main(argv: Sequence[str] | None = None) -> DemoResult:
    import argparse

    parser = argparse.ArgumentParser(description="Demo for MiDaS depth + MoveNet pose pipeline.")
    parser.add_argument("--image", type=Path, help="Optional path to an input image.", default=None)
    args = parser.parse_args(argv)

    image = load_demo_image(args.image)
    demo = DepthPoseDemo()
    result = demo.run(image)

    print(f"Depth map shape: {result.depth_map.shape}")
    print(f"Pose landmarks: {summarize_landmarks(result.landmarks)}")
    return result


if __name__ == "__main__":  # pragma: no cover - manual execution entry point
    main()
