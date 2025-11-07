# Active Tasks

## TASK-001: Sync HTTP API with world state checksum
- **Context:** The `/world/diff` endpoint occasionally returns stale data because the checksum is not updated after entity despawns.
- **Goal:** Ensure checksums are recalculated whenever the `world_state::diff::DiffBuilder` processes entity removals.
- **Acceptance Criteria:**
  - Unit test covering checksum updates passes.
  - Manual verification via `cargo run -p app -- --duration 1 --record /tmp/out` confirms consistent diffs.
- **Guardrails:** Avoid breaking existing snapshot serialization.

## TASK-002: Document runtime loop expectations
- **Context:** New contributors struggle to understand how the multi-agent loop orchestrates commands.
- **Goal:** Write developer documentation that references the prompts, runtime, router, and perception subsystems.
- **Acceptance Criteria:**
  - Documentation is linked from `docs/ProjectBrain.md`.
  - Includes references to log locations and key directories.
- **Guardrails:** Keep under 500 words.
