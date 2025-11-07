# Runtime Loop

The runtime orchestrates command execution for Project Brain. Runner personas interact with this directory only.

## Layout
- `logs/` – timestamped command transcripts emitted by the Runner.
- `queue.json` – optional FIFO of pending shell actions.
- `state.json` – scratch space for storing incremental progress or environment variables.

## Conventions
1. Logs are written in UTF-8 with a one-line header describing the command.
2. Each log file name follows `<ISO8601>-<sanitized-command>.log`.
3. The router consumes these logs to build diffs for other agents.

To bootstrap the loop, create `queue.json` with an array of commands, then have the Runner pop and execute them sequentially.
