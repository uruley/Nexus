# Perception Demos

This directory houses the perception microservice and Hugging Face (HF) demo scripts that surface the Nexus world in the browser.

## Suggested Structure
- `spaces/` – lightweight Gradio or Streamlit apps deployed to HF Spaces.
- `notebooks/` – exploratory Colab-compatible notebooks for perception research.
- `snapshots/` – exported assets (images, audio) used by demos.

Each demo should reference `assets/world.json` for ground truth data and rely on the router API for incremental updates. Use environment variables for HF tokens instead of committing credentials.

## Schemas

- `PerceptionFrame.schema.json` – JSON Schema describing the depth + pose payload expected from perception demos.

## Quickstart – Perception Service

1. **Create a virtual environment and install requirements**

   ```bash
   python -m venv .venv
   source .venv/bin/activate  # Windows PowerShell: .\.venv\Scripts\Activate.ps1
   pip install -r perception/requirements.txt
   ```

2. **Obtain the MoveNet ONNX model**

   - **Option A:** Place the file at `perception/models/movenet_thunder.onnx`.
   - **Option B:** Set `MOVENET_URL` then run the downloader:

     ```powershell
     $env:MOVENET_URL="https://example.com/movenet_thunder.onnx"
     py perception/scripts/download_movenet.py
     ```

   - **Option C:** Point to an existing file by setting `MOVENET_ONNX` in the environment or in `.env`.

3. **Run the service**

   ```bash
   # macOS/Linux
   HOST=127.0.0.1 PORT=5055 ./perception/run_perception.sh

   # Windows PowerShell
   ./perception/run_perception.ps1 -Host 127.0.0.1 -Port 5055
   ```

4. **Verify endpoints**

   - `http://127.0.0.1:5055/health` → returns `{ ok, midas, movenet, camera_index, model_path }`.
   - `http://127.0.0.1:5055/frame` → returns JSON (depth may be `null`; `persons` may be empty when the camera or model is unavailable).
