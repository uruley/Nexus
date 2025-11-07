# Perception Demos

This directory houses Hugging Face (HF) demo scripts that surface the Nexus world in the browser.

## Suggested Structure
- `spaces/` – lightweight Gradio or Streamlit apps deployed to HF Spaces.
- `notebooks/` – exploratory Colab-compatible notebooks for perception research.
- `snapshots/` – exported assets (images, audio) used by demos.

Each demo should reference `assets/world.json` for ground truth data and rely on the router API for incremental updates. Use environment variables for HF tokens instead of committing credentials.

## Schemas

- `PerceptionFrame.schema.json` – JSON Schema describing the depth + pose payload expected from perception demos.
