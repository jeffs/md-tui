# Triage Agent Prompt

A wave gate (`cargo test`) just failed. Your job is to diagnose the failure, fix what you can, and escalate what you can't.

## Context

The failing gate log is at: `$GATE_LOG`
The tasks that ran in this wave: $WAVE_TASKS
The task queue is at: `var/test-tasks.md`
This is triage attempt **$ATTEMPT** of $MAX_TRIAGE.

## Prior Attempts

$PRIOR_JOURNAL

If prior attempts are listed above, **read them carefully**. Do not repeat
fixes that have already been tried. If a prior attempt said "changed X to Y"
and the test still fails, the problem is elsewhere — dig deeper.

## Protocol

1. Read `$GATE_LOG`. Parse the test failures.
2. If there are prior attempts above, read `var/ralph-logs/triage-journal-$GATE_NAME.md`
   for full history of what was tried.
3. For each failure, classify it:

   **Category A — Test bug:** The test itself is wrong (bad assertion, wrong
   expected value, misunderstood API). These are the common case.
   → **Fix it in place.** Read the source file, fix the test, run `cargo check`.

   **Category B — Production bug:** The test is correct but reveals a real bug
   in production code (parser produces wrong output, transform miscalculates, etc.).
   → **Do not fix production code.** Instead, append a new task to `var/test-tasks.md`
   under a `## Triage — Bug Fixes` section at the bottom. Format:
   ```
   - [ ] **BXX: Fix <description>**
     - Discovered by: <test name> in <task ID>
     - Symptom: <what the test expected vs what it got>
     - Likely location: <file:line or function name>
     - File: <file to fix>
   ```
   Then mark the failing test `#[ignore]` with a comment `// BXX: <reason>` so
   the gate can pass and the wave can proceed. The bug-fix task will be picked up
   in a later pass.

   **Category C — Build/compilation error:** A task produced code that doesn't
   compile, or conflicts with another task's changes.
   → **Fix the compilation error.** Read both files involved if it's a conflict.
   Resolve it. Run `cargo check`.

   **Category D — Flaky / environment:** Failure due to config, timing, or
   file-system assumptions (e.g., `GENERAL_CONFIG` initialized with wrong flavor).
   → **Fix the test** to be deterministic. Add env var setup, use `Once` blocks,
   or restructure to avoid the dependency.

4. After fixing all Category A/C/D issues and `#[ignore]`-ing Category B:
   - Run `cargo test`. If it passes, exit with success.
   - If it still fails, that's OK — the orchestrator will re-run the gate and
     launch another triage attempt with your journal entry as context.

5. **Before exiting**, append to `var/ralph-logs/triage-journal-$GATE_NAME.md`:
   ```
   ## Attempt $ATTEMPT

   ### Failures observed
   - <test name>: <one-line description of failure>

   ### Classification
   - <test name>: Category <A/B/C/D>

   ### Actions taken
   - <what you changed, in which file, at which line>
   - <why you believe this fixes the issue>

   ### Outcome
   - cargo test: <PASS/FAIL — list remaining failures if any>

   ### What to try next (if still failing)
   - <specific suggestions for the next attempt>
   ```

## Rules

- Use `jj` instead of `git`.
- Do not modify production code to make a test pass. If production code is wrong,
  that's Category B — file a bug task and `#[ignore]` the test.
- Do not delete tests. Fix them or `#[ignore]` them.
- Do not write to `~/.claude/projects/*/memory/`.
- Prefer the Nushell MCP server over Bash for running commands.
- The `var/` directory is ephemeral and never committed.

## Bug Task Numbering

Read `var/test-tasks.md` and find the highest existing B-number (e.g., B01, B02).
Increment from there. If no B-tasks exist yet, start at B01.
