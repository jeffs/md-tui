# Ralph Loop Prompt

You are implementing one specific test task for `mdt`, a TUI markdown viewer written in Rust.

## Your Assignment

**Implement task: $TASK_ID**

Find this task in `var/test-tasks.md` and read its full description. Then read `var/test-proposal.md` for design context.

## Protocol

1. Read `var/test-tasks.md`. Find task `$TASK_ID`. Read its full spec.
2. Read `var/test-proposal.md` for the architectural context of this task.
3. Read the source file(s) you will be testing — understand the code before writing tests.
4. Implement the task. Write tests that are:
   - Correct: they test real behavior, not tautologies.
   - Minimal: no unnecessary helpers, no over-abstraction.
   - Deterministic: no flaky timing, no dependency on user config.
5. **Do not run `cargo check` or `cargo test`.** Other agents may be writing to the
   same crate concurrently and you will fight over the cargo lock. The wave gate
   handles all compilation and test verification after every agent in the wave finishes.
   If your code has issues, the triage agent will fix them.
6. Edit `var/test-tasks.md`: change this task's `[ ]` to `[x]`.
7. Exit immediately. Do not start another task.

## Rules

- Use `jj` instead of `git` for any VCS operations.
- The `var/` directory is ephemeral and never committed. Test code goes in `src/` or `tests/`.
- Do not modify production code unless strictly necessary to make it testable
  (e.g., `fn` → `pub(crate) fn`). Document any such changes in a comment.
- For parser tests in `tests/parser_tests.rs`: set `MDT_FLAVOR=commonmark` and
  `MDT_WIDTH=80` via `std::env::set_var` in a `std::sync::Once` block. Integration
  test binaries each get their own process, so this works.
- For inline tests in `src/`: env vars are shared with other tests in the same binary.
  If a test relies on config, document the assumption.
- Do not add dependencies to `Cargo.toml` unless absolutely necessary.
- Prefer the Nushell MCP server over Bash for running commands.
- Do not write to `~/.claude/projects/*/memory/`.
- Prefer functional patterns.

## Context Files

Read these as needed:

| File | Purpose |
|------|---------|
| `var/test-tasks.md` | **READ FIRST** — find your task |
| `var/test-proposal.md` | Design spec: test tables, architecture, rationale |
| `src/nodes/word.rs` | `Word`, `WordType`, `MetaData` |
| `src/nodes/textcomponent.rs` | `TextComponent`, `TextNode`, transforms, wrapping |
| `src/nodes/root.rs` | `ComponentRoot`, `Component`, `ComponentProps` |
| `src/parser.rs` | `parse_markdown`, PEG pipeline, `MdParseEnum` |
| `src/md.pest` | PEG grammar |
| `src/search.rs` | Search functions |
| `src/event_handler.rs` | Keyboard input state machine |
| `src/util.rs` | `App`, `Mode`, `Boxes`, `LinkType`, `JumpHistory` |
| `src/util/keys.rs` | `KeyBinding`, `KeyConfig`, `key_to_action` |
| `src/util/general.rs` | `GENERAL_CONFIG`, `Flavor`, `SearchStyle` |
| `src/boxes/searchbox.rs` | `SearchBox` widget |
| `src/pages/file_explorer.rs` | `FileTree`, `MdFile` |
| `src/highlight/mod.rs` | `highlight_code`, `HighlightInfo` |
