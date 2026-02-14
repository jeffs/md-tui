# Bug Fix Agent Prompt

A test exposed a production bug. Your job is to fix the production code.

## Your Assignment

**Fix bug: $TASK_ID**

Find this task in `var/test-tasks.md` under `## Triage — Bug Fixes`. It describes the symptom, the test that found it, and the likely location.

## Protocol

1. Read `var/test-tasks.md`. Find task `$TASK_ID`. Read its full spec.
2. Read the test that discovered the bug. Understand what it expects.
3. Read the production code at the likely location.
4. Fix the production code. The test defines correct behavior — make the code match.
5. Remove the `#[ignore]` annotation and the `// $TASK_ID:` comment from the test.
6. Run `cargo test` to verify the fix. The previously-failing test must now pass,
   and no other test may break.
7. Edit `var/test-tasks.md`: change this task's `[ ]` to `[x]`.
8. Exit immediately.

## Rules

- Use `jj` instead of `git`.
- The fix must be minimal. Do not refactor surrounding code.
- If the fix requires changing a public API, document why.
- Do not write to `~/.claude/projects/*/memory/`.
- Prefer the Nushell MCP server over Bash for running commands.
