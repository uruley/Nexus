"""Utility to download the MoveNet ONNX model.

Usage:
    # PowerShell
    python perception/scripts/download_movenet.py

Environment variables:
    MOVENET_URL   (optional) - override remote URL of the ONNX file
    MOVENET_ONNX  (optional) - destination path (default: perception/models/movenet_thunder.onnx)
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import requests


CHUNK_SIZE = 1 << 15  # 32 KiB
# Resolve the repository root regardless of the working directory used to invoke the script.
REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_DEST = REPO_ROOT / "perception" / "models" / "movenet_thunder.onnx"
DEFAULT_MOVENET_URL = "https://huggingface.co/Xenova/movenet-singlepose-thunder/resolve/main/onnx/model.onnx"


def download(url: str, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = destination.with_suffix(destination.suffix + ".tmp")

    with requests.get(url, stream=True, timeout=30) as response:
        response.raise_for_status()
        total = int(response.headers.get("content-length", 0))
        downloaded = 0

        with tmp_path.open("wb") as handle:
            for chunk in response.iter_content(chunk_size=CHUNK_SIZE):
                if not chunk:
                    continue
                handle.write(chunk)
                downloaded += len(chunk)
                if total:
                    progress = downloaded / total * 100
                    print(f"\rDownloading... {progress:5.1f}%", end="", flush=True)

    if total and downloaded != total:
        tmp_path.unlink(missing_ok=True)
        raise RuntimeError(
            f"Incomplete download: expected {total} bytes, got {downloaded} bytes"
        )

    if not downloaded:
        tmp_path.unlink(missing_ok=True)
        raise RuntimeError("Downloaded file is empty")

    tmp_path.replace(destination)
    print(f"\nSaved model to {destination}")


def main() -> int:
    url = os.getenv("MOVENET_URL", DEFAULT_MOVENET_URL)
    destination_raw = os.getenv("MOVENET_ONNX")
    destination = (
        Path(destination_raw).expanduser()
        if destination_raw
        else DEFAULT_DEST
    )
    try:
        download(url, destination)
    except Exception as exc:  # pragma: no cover - network dependent
        print(f"Download failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

