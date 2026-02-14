# Feature Implementation Protocol

You are adding a feature or making a change to `mdt`, a TUI markdown viewer in Rust.

## Test Discipline

This project has 175+ tests. You must leave it with at least as many, all passing.

### Before writing any code

1. Run `cargo test`. Record the count. Every test must pass.
   If any test is already failing, stop and report it — do not proceed on a broken baseline.

### After every meaningful change

2. Run `cargo test` again. If you broke something, fix it before moving on.
   A "meaningful change" is any edit to a `.rs` or `.pest` file. Do not batch up
   changes and test at the end — test incrementally so you know what broke what.

### Before finishing

3. Write tests for every new behavior you introduced (see table below).
4. Run `cargo test` one final time. Report the before/after count.
   The count must increase. Zero regressions.

## Where to add tests

| What you changed | Test location | Style |
|---|---|---|
| `Word`, `WordType`, `MetaData` | `src/nodes/word.rs` | Inline `#[cfg(test)] mod tests` |
| `TextComponent`, wrapping, transforms | `src/nodes/textcomponent.rs` | Inline |
| `ComponentRoot`, component tree | `src/nodes/root.rs` | Inline |
| PEG grammar (`md.pest`) or parser pipeline | `tests/parser_tests.rs` | Integration test |
| Key bindings, `KeyConfig` | `src/util/keys.rs` | Inline |
| `App`, `Mode`, `LinkType`, `JumpHistory` | `src/util.rs` | Inline |
| `SearchBox` | `src/boxes/searchbox.rs` | Inline |
| Event handler, mode transitions | `tests/event_handler_tests.rs` | Integration test |
| Widget rendering, visual output | `tests/render_tests.rs` | Integration test |
| Full UI flows | `tests/e2e/run.sh` | Shell + tmux snapshots |

## Test conventions

### Parser tests (`tests/parser_tests.rs`)

Environment is set once per binary for determinism:

```rust
std::sync::Once::call_once(|| {
    std::env::set_var("MDT_FLAVOR", "commonmark");
    std::env::set_var("MDT_WIDTH", "80");
});
```

Use the existing `parse()` helper:

```rust
fn parse(input: &str) -> ComponentRoot {
    parse_markdown(None, input, 78)
}
```

Assert on structure, not string equality — check component count, word types,
`TextNode` variants, and content. This makes tests resilient to formatting changes.

### Render tests (`tests/render_tests.rs`)

Use the existing `render_to_buffer()` and `buffer_to_string()` helpers:

```rust
let buffer = render_to_buffer("# Hello", 80, 24);
let content = buffer_to_string(&buffer);
assert!(content.contains("Hello"));
```

### Event handler tests (`tests/event_handler_tests.rs`)

Use the existing `key()` and `key_code()` helpers to build `KeyEvent` values.
Test state transitions: given a mode + key, assert the resulting mode and side effects.

### Inline tests (in `src/`)

Env vars are shared across all tests in the same binary. If your test depends on
config values, document the assumption in a comment. Prefer testing pure functions
that don't depend on `GENERAL_CONFIG`.

### E2E snapshots

After visual changes, update snapshots:

```
UPDATE_SNAPSHOTS=1 tests/e2e/run.sh
```

## What makes a good test

- **Test behavior, not implementation.** Assert on what the user sees or what the
  API returns, not internal data structure details that might change.
- **One assertion cluster per test.** A test named `parse_bold` should test bold
  parsing, not bold + italic + links.
- **Deterministic.** No dependence on wall clock, file system ordering, or user config.
  Set env vars explicitly if needed.
- **Minimal setup.** Use the smallest possible input that exercises the behavior.
  `"**bold**"` not a 50-line markdown document.
- **Name says what's tested.** `word_wrapping_breaks_at_boundary` not `test_wrapping_2`.

## What to test for a new feature

At minimum:

1. **Happy path**: the feature works as intended with typical input.
2. **Edge cases**: empty input, single character, maximum width, Unicode/CJK.
3. **Integration**: if you added a grammar rule, test it through the parser.
   If you added a key binding, test it through the event handler.
4. **Regression guard**: if your feature interacts with existing features
   (e.g., bold inside a list), add a test for the combination.

## Rules

- Do not delete or `#[ignore]` existing tests to make the suite pass.
- Do not modify existing test assertions unless the behavioral change is intentional
  and documented.
- If you discover a pre-existing bug, file it as a comment in your test:
  `// BUG: <description>` and add a test that demonstrates the correct behavior
  (mark it `#[ignore]` with `// BUG: <reason>` if it fails due to the bug).
- Do not add dependencies to `Cargo.toml` for testing unless absolutely necessary.
- Prefer `assert_eq!` with descriptive messages over bare `assert!`.
