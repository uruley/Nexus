# Project Brain

Project Brain describes the multi-agent workflow that orchestrates how Nexus explores and expands new ideas. The framework is split into four deliberate phases—**Architect**, **Builder**, **Critic**, and **Runner**—that operate on a shared world model and task queue. Each phase is implemented as an LLM persona with bespoke prompts stored under [`codex/prompts/`](../codex/prompts/).

## High-level loop

1. **Architect** reads the active task from [`codex/tasks/TASKS.md`](../codex/tasks/TASKS.md), inspects the current world state in [`assets/world.json`](../assets/world.json), and produces a high-level plan.
2. **Builder** converts the architect's proposal into concrete implementation steps, grounding them in the repository layout. Builder output feeds directly into the runtime loop.
3. **Critic** evaluates the builder output, comparing it to the plan and existing code. It highlights mismatches, missing tests, or risky edits, emitting actionable feedback.
4. **Runner** executes approved steps inside the [`runtime/`](../runtime/) loop, calling cargo commands, tests, or custom scripts. Runner logs are diffed via the [`router/`](../router/) helpers so the agents can observe state transitions.

The loop repeats until the task is complete or the critic vetoes further progress. This design keeps creativity (Architect), precision (Builder), safety (Critic), and verification (Runner) distinct but cooperative.

## Shared data contracts

- **Task format** – Each task entry includes a short title, context, acceptance criteria, and optional guardrails. Agents must echo the task identifier in their messages so downstream steps can correlate progress.
- **World snapshot** – The world JSON exposes the latest Bevy ECS entities that matter to agents (camera pose, tracked assets, simulation flags). The checksum key lets the router detect divergence between runs.
- **Runtime transcripts** – Runner stores command transcripts in timestamped files inside `runtime/logs/`. Critic uses them to assess whether reality matched intent.

## Extending the brain

Add new personas by writing prompts in `codex/prompts/` and extending the loop in `runtime/`. The modular layout makes it easy to swap models or supplement the pipeline with tooling such as static analyzers, profiling hooks, or QA bots.
