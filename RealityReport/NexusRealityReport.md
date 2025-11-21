NEXUS REALITY REPORT

- **Existing files:**
  - From `docs/FileIndex.md`, all listed paths exist:
    - `Cargo.toml`
    - `apps/app/`
    - `crates/anchor/`, `crates/app/`, `crates/http_api/`, `crates/world_state/`
    - `runtime/`
    - `router/`
    - `perception/`
    - `docs/`
    - `codex/`
    - `tests/`

- **Missing files:**
  - None from the current `FileIndex.md` table.
  - From `docs/ProjectBrain.md` “Current Architecture” references:
    - `runtime/systems/*.py` – not present (only `runtime/main.py`, `__init__.py`, `README.md`, `logs/`, `world.json`).
    - `schema.json` at repo root – not present; there is `router/schema.json`.
    - `perception/models/movenet_thunder.onnx` – directory exists but model file is missing (only `perception/models/README.md`).

- **Unexpected files:**
  - Top-level items not enumerated in `docs/FileIndex.md` but present in the repo:
    - `assets/` (contains `assets/world.json`)
    - `README.md` (root)
    - Within indexed directories, additional subpaths not individually listed, e.g.:
      - `apps/app/src/main.rs`
      - `crates/app/hud.rs`, `crates/app/perception.rs`
      - `runtime/README.md`, `runtime/logs/`
      - `router/examples/`, `router/archive/`, `router/hooks/`
      - `perception/demo_depth_pose.py`, `perception/PerceptionFrame.schema.json`, `perception/run_perception.(ps1|sh)`, `perception/scripts/download_movenet.py`

- **runtime/main.py summary:**
  - Implements the **Runtime** layer’s patch engine.
  - Loads or creates `runtime/world.json` with a default structure (entities, camera, lighting).
  - Defines a `Patch` dataclass with fields `id`, `type`, `data` and a `from_raw` validator enforcing shape.
  - Provides `load_patches` reading one or more JSON patch files; supports single object or list of patches.
  - `apply_patch` supports three operations:
    - `spawn_entity` – appends entity data into `world["entities"]`.
    - `move_camera` – updates `world["camera"]`.
    - `set_light` – merges fields into `world["lighting"][light_name]`.
  - `discover_patch_files` resolves patch inputs either from CLI `--patch` arguments or by scanning `router/examples/*.json`.
  - `main()` parses CLI (`--world`, `--patch`, `--patch-dir`), loads world + patches, applies each patch while timing, writes the new world back, and prints per-patch and total frame timings.

- **runtime/main.py run result:**
  - Command: `python runtime/main.py`
  - Exit code: `0` (successful).
  - Output:
    - `Applied camera:update:orbit (move_camera) in 0.002 ms`
    - `Applied light:key (set_light) in 0.003 ms`
    - `Applied entity:cube:001 (spawn_entity) in 0.001 ms`
    - `Frame processed in 0.622 ms for 3 patch(es)`
  - Interpretation: it successfully loaded patches from `router/examples/`, mutated `runtime/world.json`, and reported timings; no errors.

- **perception/server.py summary:**
  - Implements the **Perception Kit** layer as a FastAPI app.
  - Environment configuration:
    - `MOVENET_ONNX` (optional) controls model path; default: `models/movenet_thunder.onnx`.
    - `CAMERA_INDEX` (optional) selects the video capture device (default `0`).
  - MiDaS depth:
    - `_load_midas()` loads `MiDaS_small` via `torch.hub` and associated transforms; logs success or error, sets `midas`, `midas_transforms`.
    - `estimate_depth_bgr()`:
      - Converts BGR frame to RGB and applies `midas_transforms`.
      - Handles multiple possible transform return types (dict, PIL image, NumPy array, tensor).
      - Normalizes depth to 16-bit and returns a `uint16` depth map; any failures log an error and return `None`.
  - MoveNet pose:
    - `_load_movenet()` attempts to load the ONNX model at `MODEL_PATH` using `onnxruntime.InferenceSession`.
    - Discovers the dynamic input name from `get_inputs()[0].name`; on missing file or load failure, logs and sets `movenet_sess=None`.
    - `run_movenet()`:
      - If session unavailable, returns an empty list.
      - Otherwise, resizes and normalizes input to `[1, 256, 256, 3]`, runs inference, and accepts output shapes `[1,1,17,3]` or `[1,17,3]`.
      - Maps each keypoint to image coordinates with names from `KEYPOINT_NAMES`, computes a loose bounding box and average confidence, returns a list with a single “person” object (`id`, `score`, `bbox`, `keypoints`).
  - Video capture:
    - Opens `cv2.VideoCapture(CAMERA_INDEX)`; `grab_frame()` falls back to a black 480×640 frame if capture fails.
  - API:
    - `GET /health` → `{"ok": true, "midas": bool, "movenet": bool, "camera_index": int, "model_path": str}`.
    - `GET /frame`:
      - Always attempts to grab a frame, estimate depth, and run MoveNet.
      - Returns: `{ ts, size: [w,h], depth: null|{format,uri}, persons: [...] }`.
      - Any internal errors are logged with tracebacks; handler returns a 500 JSON error only in truly fatal cases (currently guarded so typical missing-model/depth failures degrade gracefully to `depth=null` / `persons=[]`).

- **perception/server.py run result:**
  - Command: `python perception/server.py`
  - Exit code: `0` (no exception).
  - Output (warnings + logs):
    - Uses cached MiDaS repo and emits Triton/nvdisasm warnings (from PyTorch/Triton).
    - `Loading weights:  None`
    - `[MiDaS] model loaded successfully`
    - `[MoveNet] model file not found at models\\movenet_thunder.onnx`
  - Interpretation:
    - As a module, it imports successfully, loads MiDaS, and fails to find the MoveNet ONNX file (expected given M1 status). Since there is no `if __name__ == "__main__":` block, running `python perception/server.py` just initializes globals and exits. Uvicorn should still be used to serve HTTP (`uvicorn server:app ...`); the missing ONNX model is logged but no exception is raised.

- **Additional issues:**
  - Architectural vs. filesystem gaps:
    - `perception/models/movenet_thunder.onnx` is referenced in `docs/ProjectBrain.md` and `perception/server.py` but does not exist yet. This aligns with STATUS “Add MoveNet ONNX to perception/models/” and `movenet: false` in `/health`.
    - `runtime/systems/*.py` and root-level `schema.json` are mentioned in `docs/ProjectBrain.md` but not present; the effective schema lives in `router/schema.json`, and runtime logic is consolidated in `runtime/main.py`.
  - Behavioural:
    - `/health` and `/frame` behavior via uvicorn were previously validated: `/health` reports `movenet: false` while MiDaS is true; `/frame` returns 200 with `depth: null` and `persons: []` when the model is absent. This satisfies the “no 500s when models are missing” hardening.
  - Alignment with Milestone M1:
    - Runtime can already read and apply JSON patches.
    - Perception service runs and can produce depth (when MiDaS works) and pose (once MoveNet model is provided).
    - `perception/state.json` does not yet exist, and runtime is not yet wired to perception output—those are upcoming steps for Milestone M1.


