# File Index

| Path | Summary |
| --- | --- |
| `Cargo.toml` | Workspace manifest defining crates, apps, and shared dependencies. |
| `apps/app/` | Bevy desktop application entry point (`main.rs`) and related configuration. |
| `crates/anchor/` | Simulation systems, intent handling, and metrics used by the runtime. |
| `crates/app/` | Heads-up display UI logic that renders frame timing information. |
| `crates/http_api/` | Axum HTTP bridge exposing world snapshots and intent ingestion endpoints. |
| `crates/world_state/` | Shared world data structures and serialization helpers. |
| `runtime/` | Python runtime that loads `world.json`, applies patches, and logs execution metrics. |
| `router/` | JSON schemas and example patches that drive automated planner outputs. |
| `perception/` | Perception microservice (FastAPI) plus demos, models, and support tooling. |
| `docs/` | Project documentation (`ProjectBrain.md`, `STATUS.md`, `ArchitectRules.md`, this index). |
| `codex/` | Prompt packs and automation artifacts for the multi-agent workflow. |
| `tests/` | Python test suite verifying runtime behaviour (e.g., `test_runtime_main.py`). |


