# Router

The router mediates information flow between agents by diffing workspace artifacts.

## Components
- `config.toml` – mapping between agent roles and the files they monitor.
- `hooks/` – scripts that preprocess command outputs before they are shared.
- `archive/` – compressed snapshots for long-running tasks.

The router listens for new log files in `runtime/logs/`, computes semantic diffs against prior snapshots, and forwards the summaries to interested agents. Extend `hooks/` to normalize noisy output (e.g., timestamps) before diffing.
