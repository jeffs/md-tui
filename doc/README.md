# Documentation

## `adr/` — Architectural Decision Records

| ADR | Title |
|-----|-------|
| [001](adr/001-test-architecture.md) | Test architecture: 8-layer strategy, 175 tests |

## `guides/` — Development Guides

| Guide | When to use |
|-------|-------------|
| [feature-protocol.md](guides/feature-protocol.md) | Include in agent prompts when implementing features. Ensures tests are written and regressions caught. |
| [adding-languages.md](guides/adding-languages.md) | Adding tree-sitter syntax highlighting for a new language. |

## `agents/` — Parallel Agent Orchestration

Reusable prompts and scripts for running parallel Claude agents in a
Ralph loop (clean context windows, file-based state, wave gates with
automated triage). These were used to build the test suite and can be
adapted for future bulk tasks.

| File | Role |
|------|------|
| [run-tests.sh](agents/run-tests.sh) | Orchestrator: wave-parallel execution, cargo test gates, triage loop |
| [ralph.md](agents/ralph.md) | Per-task agent prompt (substitutes `$TASK_ID`) |
| [triage.md](agents/triage.md) | Triage agent: classifies failures, fixes tests, files production bugs |
| [bugfix.md](agents/bugfix.md) | Bug-fix agent: fixes production code flagged by triage |

Runtime state (task queues, logs) lives in `var/` which is gitignored.
