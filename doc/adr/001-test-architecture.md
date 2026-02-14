# ADR-001: Test Architecture

**Status:** Accepted and implemented (175 tests)
**Date:** 2026-02-14

## Context

The project had 15 tests across 3 files, with no coverage of the parser,
data model, layout transforms, event handler, key bindings, or rendering.

## Prior State

**15 tests total** across 3 files:

| Module | Tests | Coverage |
|--------|-------|----------|
| `search.rs` | 13 | Substring/fuzzy search, line matching, word-ref search, parser integration |
| `util.rs` | 1 | `JumpHistory` push/pop |
| `highlight/mod.rs` | 1 | `HIGHLIGHT_NAMES` / `COLOR_MAP` length parity |

**Completely untested:**

- PEG grammar and parse pipeline (`parser.rs`, `md.pest`)
- Data model (`Word`, `TextComponent`, `ComponentRoot`)
- Layout transforms (`word_wrapping`, `transform_paragraph`, `transform_list`, `transform_table`)
- Event handler state machine (`event_handler.rs`)
- `FileTree` navigation logic
- Key binding parsing and matching (`keys.rs`)
- `LinkType` classification (`util.rs`)
- Config loading (`general.rs`, `colors.rs`)
- Box widgets (`SearchBox`, `ErrorBox`, `HelpBox`, `LinkBox`)
- Rendering to terminal buffer (all ratatui `Widget` impls)

---

## Test Architecture

Seven layers, ordered from fastest/most isolated to slowest/most integrated. Each layer catches different classes of bugs.

### Layer 1: Data Model Unit Tests

**Goal:** Verify the fundamental types behave as specified.

**File:** `src/nodes/word.rs` (inline `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `word_new_sets_type` | `Word::new` stores content and type correctly |
| `word_set_kind_preserves_previous` | `set_kind` saves `previous_type`, `clear_kind` restores it |
| `word_clear_kind_without_previous` | `clear_kind` is a no-op when `previous_type` is `None` |
| `word_is_renderable` | `MetaInfo`, `LinkData`, `FootnoteData` → false; all others → true |
| `word_split_off` | Splits content at byte boundary, preserves type on both halves |
| `wordtype_from_mdparseenum` | Exhaustive test of every non-container `MdParseEnum` → `WordType` mapping |
| `wordtype_from_container_panics` | Container variants like `Heading`, `BoldStr` trigger `unreachable!` (use `#[should_panic]`) |

**File:** `src/nodes/textcomponent.rs` (inline `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `new_filters_meta_info` | Constructor separates renderable words from meta-info |
| `new_formatted_sets_height` | `new_formatted` sets height = number of lines |
| `content_as_lines_paragraph` | Joins words per line for non-table components |
| `content_as_lines_table` | Chunks by column count, joins correctly |
| `num_links_counts_link_and_footnote` | Only counts `LinkData` and `FootnoteInline` in meta_info |
| `visually_select_and_deselect` | Selecting marks words as `Selected`; deselecting restores original type |
| `visually_select_out_of_bounds` | Returns `Err` for invalid index |
| `is_indented_list_true` | Returns true for list with whitespace-only meta indent |
| `is_indented_list_false_for_paragraph` | Returns false for non-list component |

**File:** `src/nodes/root.rs` (inline `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `component_root_height_sums_children` | Total height = sum of component heights |
| `component_root_num_links` | Counts links across all text components |
| `component_root_select_deselect` | `select(n)` focuses nth link; `deselect()` clears all |
| `component_root_select_out_of_bounds` | Returns `Err` for index beyond total links |
| `component_root_words` | Flattens all text components' words |
| `component_root_find_footnote_found` | Returns footnote content when ref matches |
| `component_root_find_footnote_not_found` | Returns "Footnote not found" |
| `component_root_heading_offset` | Returns y-offset of matching heading |
| `component_root_heading_offset_not_found` | Returns `Err` for nonexistent heading |
| `component_root_add_missing_components` | Inserts `LineBreak` between adjacent non-break components |
| `component_root_add_missing_components_task_sublist` | Does NOT insert `LineBreak` between `Task` and indented sublist |
| `set_scroll_propagates` | `set_scroll` correctly sets y_offset and scroll_offset on each child |
| `content_returns_lines` | `content()` returns one string per rendered line |

### Layer 2: Layout Transform Tests

**Goal:** The word-wrapping and layout functions are the most complex pure logic in the codebase. Bugs here manifest as garbled rendering.

**File:** `src/nodes/textcomponent.rs` (add to existing `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `word_wrapping_fits_on_one_line` | Words fitting within width produce 1 line |
| `word_wrapping_breaks_at_boundary` | Line break occurs between words at width boundary |
| `word_wrapping_long_word_hyphenation` | Word longer than width is split with `-` |
| `word_wrapping_long_word_no_hyphen_at_boundary` | Hyphenation disabled when width ≤ 4 |
| `word_wrapping_preserves_word_types` | Wrapped words retain their `WordType` |
| `word_wrapping_unicode_cjk` | CJK characters (width 2) are split correctly |
| `word_wrapping_empty_input` | Empty iterator → empty result |
| `split_by_width_ascii` | Splits ASCII at correct byte index |
| `split_by_width_unicode` | Splits multi-byte chars without truncating mid-character |
| `split_by_width_zero` | Width 0 → empty first half, full second half |
| `transform_paragraph_sets_height` | After transform, height = number of wrapped lines |
| `transform_paragraph_quote_prefix` | Quote transform prepends space to continuation lines |
| `transform_paragraph_task_width` | Task transform subtracts 4 from available width |
| `transform_list_ordered_numbering` | Ordered list items get sequential numbers |
| `transform_list_nested_indent` | Nested lists indent correctly |
| `transform_list_mixed_ordered_unordered` | Mixed list types coexist |
| `transform_table_unbalanced_fits` | Table where all columns fit uses natural widths |
| `transform_table_overflow_balanced` | Overflowing columns get proportionally reduced widths |
| `transform_table_wraps_cell_content` | Long cell content word-wraps within balanced width |
| `transform_table_zero_columns` | Degenerate table (0 columns) → height 1, empty widths |
| `transform_codeblock_preserves_lines` | Code block lines are split on `\n`, height matches |
| `transform_codeblock_with_highlighting` | Highlighted code splits into colored `Word`s per line |

### Layer 3: Parser Tests

**Goal:** The PEG grammar and parse pipeline are the foundation. These tests verify that markdown input produces the expected component tree.

**Strategy:** Create a `tests/parser_tests.rs` integration test file. Use a helper that calls `parse_markdown(None, input, WIDTH)` and inspects the resulting `ComponentRoot`. Use a standard width (e.g., 80) to keep assertions stable.

**Important:** These tests must handle the `GENERAL_CONFIG` lazy static. Since it reads `~/.config/mdt/config.toml`, tests should either:
- Set `MDT_FLAVOR=commonmark` env var before first access, or
- Accept the default behavior and document it.

The env-var approach is strongly preferred: tests should set `MDT_FLAVOR`, `MDT_WIDTH`, etc. via a shared test helper or `ctor` crate to guarantee determinism.

| Test | Input | Expected |
|------|-------|----------|
| `parse_empty` | `""` | 0 components |
| `parse_paragraph` | `"Hello world"` | 1 Paragraph with words ["Hello", " ", "world"] |
| `parse_heading_h1` | `"# Title"` | 1 Heading, meta HeadingLevel(1) |
| `parse_heading_h2` | `"## Subtitle"` | 1 Heading, meta HeadingLevel(2), content starts with "## " |
| `parse_heading_h3_through_h6` | `"### H3"` ... `"###### H6"` | Heading levels 3–6 |
| `parse_bold` | `"**bold text**"` | Paragraph containing Bold words |
| `parse_italic_star` | `"*italic*"` | Paragraph containing Italic words |
| `parse_italic_underscore` | `" _italic_"` | Paragraph containing Italic words |
| `parse_bold_italic` | `"***both***"` | Paragraph containing BoldItalic words |
| `parse_strikethrough` | `"~~struck~~"` | Paragraph containing Strikethrough words |
| `parse_inline_code` | `` "`code`" `` | Paragraph containing Code words |
| `parse_code_block_fenced` | `` "```rust\nlet x = 1;\n```" `` | 1 CodeBlock, meta PLanguage="rust" |
| `parse_code_block_tilde` | `"~~~\ncode\n~~~"` | 1 CodeBlock |
| `parse_code_block_indented` | `"    line1\n    line2"` | 1 CodeBlock with 2 lines |
| `parse_unordered_list` | `"- item1\n- item2"` | 1 List with 2 entries, UList markers |
| `parse_ordered_list` | `"1. first\n2. second"` | 1 List with 2 entries, OList markers |
| `parse_nested_list` | `"- outer\n  - inner"` | 1 List with indented subitems |
| `parse_task_open` | `"- [ ] Todo"` | 1 Task with TaskOpen meta |
| `parse_task_closed` | `"- [x] Done"` | 1 Task with TaskClosed meta |
| `parse_link_standard` | `"[text](url)"` | Paragraph with Link word + LinkData meta |
| `parse_link_wiki` | `"[[page]]"` | Paragraph with WikiLink-derived Link |
| `parse_link_wiki_with_alias` | `"[[page\|display]]"` | Link text = "display", data = "page" |
| `parse_link_inline` | `"<https://example.com>"` | Paragraph with InlineLink-derived Link |
| `parse_table` | `"\| a \| b \|\n\|---\|---\|\n\| 1 \| 2 \|"` | 1 Table with ColumnsCount=2 |
| `parse_quote` | `"> quoted text"` | 1 Quote |
| `parse_quote_admonition_note` | `"> [!note]\n> text"` | Quote with Note meta |
| `parse_quote_admonition_warning` | `"> [!warning]\n> text"` | Quote with Warning meta |
| `parse_horizontal_separator` | `"---"` | 1 HorizontalSeparator |
| `parse_footnote` | `"text[^1]\n\n[^1]: footnote"` | Contains FootnoteInline + Footnote component |
| `parse_image_missing` | `"![alt](nonexistent.png)"` | Paragraph with "Image not found/fetched" |
| `parse_latex` | `"$x^2$"` | Paragraph with Normal words (latex rendered as text) |
| `parse_comment_stripped` | `"before <!-- comment --> after"` | Paragraph with "before" and "after", no comment |
| `parse_multiple_blocks` | heading + paragraph + list | Components in correct order with LineBreaks between |
| `parse_block_separators` | `"para1\n\npara2"` | Two paragraphs separated by LineBreak |
| `parse_escaped_bold` | `"\\**not bold**"` | Rendered as literal `**not bold**` |

### Layer 4: Key Binding Tests

**File:** `src/util/keys.rs` (inline `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `parse_key_string_char` | `"k"` → `KeyCode::Char('k')`, no modifiers |
| `parse_key_string_space` | `"space"` → `KeyCode::Char(' ')` |
| `parse_key_string_ctrl` | `"C-e"` → `KeyCode::Char('e')` with `CONTROL` |
| `parse_key_string_tab` | `"tab"` → `KeyCode::Tab` |
| `parse_key_string_enter` | `"enter"` → `KeyCode::Enter` |
| `parse_key_string_esc` | `"esc"` → `KeyCode::Esc` |
| `parse_key_string_empty` | `""` → `None` |
| `parse_key_string_invalid` | `"xyz"` → `None` |
| `keybinding_matches_exact` | Binding for `'k'` matches `KeyEvent` with `Char('k')` |
| `keybinding_matches_ignores_shift` | Ctrl binding matches even when terminal reports Shift |
| `keybinding_no_false_positive` | Binding for `'k'` does NOT match `'j'` |
| `keybinding_display` | Formatting: `"C-e"`, `"space"`, `"k"` |
| `key_to_action_arrow_keys` | Arrow keys map to correct actions regardless of config |
| `key_to_action_configurable` | Default vim keys map correctly (`j`→Down, `k`→Up, etc.) |

### Layer 5: Utility and State Tests

**File:** `src/util.rs` (add to existing `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `link_type_internal` | `"#heading"` → `Internal` |
| `link_type_external_http` | `"https://example.com/page.html"` → `External` |
| `link_type_markdown_file_md` | `"other.md"` → `MarkdownFile` |
| `link_type_markdown_file_no_ext` | `"other"` → `MarkdownFile` |
| `link_type_external_dot_rs` | `"file.rs"` → `External` (has non-md extension with dot) |
| `jump_history_empty_pops_filetree` | Pop on empty history → `Jump::FileTree` |
| `app_reset_clears_state` | `reset()` zeroes scroll, deselects, closes boxes |
| `app_set_width_clamps_to_config` | Width never exceeds `GENERAL_CONFIG.width` |
| `app_set_width_returns_changed` | Returns `true` only when width actually changed |

**File:** `src/boxes/searchbox.rs` (inline `#[cfg(test)]` module)

| Test | What it validates |
|------|-------------------|
| `searchbox_insert_delete` | Insert chars, delete last, verify cursor and content |
| `searchbox_clear` | Clear resets text and cursor to empty/0 |
| `searchbox_consume` | Consume returns content and clears box |
| `searchbox_content_empty_is_none` | `content()` returns `None` when empty |
| `searchbox_content_nonempty_is_some` | `content()` returns `Some(&str)` when populated |

### Layer 6: Event Handler State Machine Tests

**Goal:** The event handler is where most user-visible bugs live. It's a state machine operating on `(Mode, Boxes)` pairs. Testing it requires constructing `App`, `ComponentRoot`, and `FileTree` states and feeding synthetic `KeyEvent`s.

**Strategy:** Create the `PollWatcher` dependency using a temp directory. Build markdown fixtures with `parse_markdown`. Verify state transitions and side effects.

**File:** `tests/event_handler_tests.rs`

**State matrix to cover:**

| Mode | Boxes | Key | Expected effect |
|------|-------|-----|-----------------|
| FileTree | None | `j` (Down) | `file_tree.next()` called |
| FileTree | None | `k` (Up) | `file_tree.previous()` called |
| FileTree | None | Enter | Mode → View, markdown loaded |
| FileTree | None | `f` | Boxes → Search |
| FileTree | None | `q` | Returns Exit |
| FileTree | Search | Char | Inserted into search_box, file_tree filtered |
| FileTree | Search | Backspace (empty) | Boxes → None |
| FileTree | Search | Esc | Search cleared, Boxes → None |
| FileTree | Search | Enter | Search consumed, Boxes → None |
| FileTree | Error | Enter/Esc | Boxes → None |
| View | None | `j` | vertical_scroll += 1 |
| View | None | `k` | vertical_scroll -= 1 |
| View | None | `g` | vertical_scroll = 0 |
| View | None | `G` | vertical_scroll = max |
| View | None | `s` | selected = true, first visible link focused |
| View | None | `q` (direct_file) | Returns Exit |
| View | None | `q` (not direct) | Mode → FileTree |
| View | None | `f` | Boxes → Search |
| View | None | `e` | Returns Edit |
| View | None | `t` | Mode → FileTree |
| View | Search | Char | Inserted into search_box |
| View | Search | Enter | find_and_mark called, scrolls to result |
| View | Search | Enter (no results) | Boxes → Error with "No results" |
| View | Search | Esc | Boxes → None |
| View | None (selected) | `j` | select_index += 1 |
| View | None (selected) | `k` | select_index -= 1 |
| View | None (selected) | Enter (internal link) | Scrolls to heading |
| View | None (selected) | Esc | selected = false, deselected |
| View | None (selected) | `K` (Hover) | Boxes → LinkPreview |
| View | LinkPreview | Esc | Boxes → None |

### Layer 7: Integration Tests (Parse → Render Pipeline)

**Goal:** Verify that the full pipeline from markdown string to rendered terminal buffer produces correct output. Uses `ratatui::backend::TestBackend`.

**Strategy:** Render components into a `TestBackend` buffer and inspect cell contents. This catches rendering bugs that unit tests on data structures would miss.

**File:** `tests/render_tests.rs`

| Test | What it validates |
|------|-------------------|
| `render_heading_h1` | H1 text appears in buffer (verify text content in cells) |
| `render_paragraph_wraps` | Long paragraph wraps at correct column |
| `render_code_block` | Code block content appears, syntax highlighting produces colored cells |
| `render_table_columns` | Table columns are visually separated, content in correct cells |
| `render_list_bullets` | Bullet character `•` appears at correct positions |
| `render_ordered_list_numbers` | Numbers `1.`, `2.`, etc. appear |
| `render_quote_indented` | Quote content is indented |
| `render_task_checkbox` | Task prefix (open/closed marker) renders |
| `render_horizontal_separator` | Separator renders as line across width |
| `render_search_highlight` | After `find_and_mark`, matched words render with `Selected` style |
| `render_link_selection` | Selected link renders with highlight style |
| `render_file_tree_widget` | FileTree widget renders file names and paths |

**Implementation pattern:**

```rust
use ratatui::{backend::TestBackend, Terminal};

fn render_markdown_to_buffer(input: &str, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut root = parse_markdown(None, input, width - 2);
    root.set_scroll(0);
    terminal.draw(|f| {
        // render each component into f
    }).unwrap();
    terminal.backend().buffer().clone()
}
```

### Layer 8: E2E Visual Tests (tmux-based)

**Goal:** Catch visual regressions that buffer inspection cannot — scroll behavior, modal overlays, focus transitions, real terminal rendering. This is the only layer that tests the *actual binary* end-to-end.

**Tooling:**

- `tmux` for headless terminal session control
- Shell script harness (`tests/e2e/run.sh`) that:
  1. Starts a tmux session with fixed geometry (e.g., 80x24)
  2. Launches `mdt` inside it with a fixture file
  3. Sends keystrokes via `tmux send-keys`
  4. Captures pane content via `tmux capture-pane -p`
  5. Compares captured output against expected snapshots
- Snapshot files stored in `tests/e2e/snapshots/` as plain text
- `UPDATE_SNAPSHOTS=1` env var to regenerate expected output

**Test scenarios:**

| Scenario | Keys | Validates |
|----------|------|-----------|
| `basic_render` | (none — just open) | File renders without crash, heading visible |
| `scroll_down_up` | `j j j k` | Content scrolls, then scrolls back |
| `page_navigation` | `d u` | Half-page scroll works |
| `search_flow` | `f`, type "hello", Enter | Search box appears, results highlighted, box closes |
| `search_no_results` | `f`, type "zzzzz", Enter | Error modal appears |
| `link_select_navigate` | `s`, `j`, `j`, Enter | Link selected, navigated, target visible |
| `link_hover` | `s`, `K` | Link preview modal appears |
| `escape_deselects` | `s`, Esc | Selection cleared |
| `file_tree_navigation` | (open dir) `j`, `k`, Enter | File tree navigates, file opens |
| `file_tree_search` | (open dir) `f`, type name, Enter | File list filtered |
| `quit_from_view` | (open file) `q` | Returns to file tree or exits |
| `resize_handling` | (resize tmux pane) | Content re-wraps without crash |
| `stdin_pipe` | `echo "# Hello" \| mdt` | Piped content renders |

**Snapshot format:** Plain text captured from `tmux capture-pane -p`. Exact comparison (after stripping trailing whitespace) catches:
- Text positioning changes
- Missing/extra characters
- Color is not captured by `-p` alone; use `-e` flag for ANSI escape sequences if color regression testing is desired (at the cost of more brittle snapshots)

**Recommended approach:** Start with `-p` (text only). Add `-e` (with escapes) for a few critical color tests only.

---

## Addressing the `GENERAL_CONFIG` Problem

The `GENERAL_CONFIG` static (`LazyLock`) reads `~/.config/mdt/config.toml` on first access. This is a major obstacle to deterministic tests. Two solutions:

### Option A: Environment Variable Override (Recommended — Least Invasive)

`GENERAL_CONFIG` already supports `MDT_*` env vars via the `config` crate's `Environment` source. Tests should:

1. Set `MDT_WIDTH=80`, `MDT_FLAVOR=commonmark`, `MDT_GITIGNORE=false`, etc.
2. Use a test harness or `ctor` to set these before any test touches `GENERAL_CONFIG`.
3. Document required env vars in a `tests/README.md`.

Limitation: Since `LazyLock` initializes once per process, all tests in the same binary share the same config. This is acceptable if tests standardize on one config. For tests needing different configs, use separate test binaries (`[[test]]` entries in `Cargo.toml`).

### Option B: Refactor to Injectable Config (Higher Investment, Better Long-Term)

Replace `LazyLock<GeneralConfig>` with a config parameter threaded through the call chain. This is a larger refactor but enables per-test config variation without process isolation.

**Recommendation:** Start with Option A. Migrate to Option B only if test variation demands it.

---

## File Organization

```
src/
  nodes/
    word.rs          ← add #[cfg(test)] mod tests { ... }
    textcomponent.rs ← add #[cfg(test)] mod tests { ... }
    root.rs          ← add #[cfg(test)] mod tests { ... }
  util.rs            ← extend existing test module
  util/
    keys.rs          ← add #[cfg(test)] mod tests { ... }
  boxes/
    searchbox.rs     ← add #[cfg(test)] mod tests { ... }

tests/
  parser_tests.rs        ← Layer 3
  event_handler_tests.rs ← Layer 6
  render_tests.rs        ← Layer 7
  fixtures/
    simple.md
    formatting.md
    links.md
    table.md
    code_block.md
    nested_list.md
    admonitions.md
    footnotes.md
    kitchen_sink.md      ← all element types combined
  e2e/
    run.sh               ← tmux harness
    snapshots/
      basic_render.txt
      scroll_down_up.txt
      search_flow.txt
      ...
```

---

## Test Fixtures

Create canonical markdown files under `tests/fixtures/` for reuse across parser, render, and E2E tests.

**`kitchen_sink.md`** — every supported element at least once:

```markdown
# Heading 1

## Heading 2

Regular paragraph with **bold**, *italic*, ***bold italic***, ~~strikethrough~~, and `inline code`.

> A blockquote
> spanning lines

> [!note]
> An admonition

- Unordered item 1
- Unordered item 2
  - Nested item

1. Ordered item
2. Another item

- [ ] Open task
- [x] Completed task

[Link text](https://example.com)
[[WikiLink]]
<https://inline.link>

| Col A | Col B |
|-------|-------|
| 1     | 2     |

```rust
fn main() {
    println!("Hello");
}
```

Text with footnote[^1].

[^1]: Footnote content here.

---

![Alt text](nonexistent.png)

$x^2 + y^2 = z^2$
```

---

## Implementation Priority

**Phase 1 — Foundation (highest value per effort):**
1. Layer 3: Parser tests. The grammar is the riskiest part of the codebase — it's complex, non-obvious, and regressions silently produce wrong output. Every new markdown feature should require a parser test.
2. Layer 2: Layout transform tests. `word_wrapping` and `transform_table` contain subtle logic around Unicode widths, hyphenation, and proportional column sizing.

**Phase 2 — Model and Input:**
3. Layer 1: Data model tests. Quick to write, catch type-conversion bugs.
4. Layer 4: Key binding tests. Ensures configurable bindings work correctly.
5. Layer 5: Utility tests. `LinkType`, `SearchBox`, etc.

**Phase 3 — Integration:**
6. Layer 6: Event handler tests. High value but requires more setup (constructing watcher, file tree, etc.).
7. Layer 7: Render tests via `TestBackend`. Catches rendering bugs.

**Phase 4 — Visual Regression:**
8. Layer 8: E2E tmux tests. Write the harness once, add scenarios incrementally. Most valuable for catching visual regressions after refactors.

---

## CI Integration

```yaml
# Suggested CI steps
test-unit:
  env:
    MDT_WIDTH: "80"
    MDT_FLAVOR: "commonmark"
    MDT_GITIGNORE: "false"
    MDT_HELP_MENU: "false"
  run: cargo test --lib --tests

test-e2e:
  needs: [test-unit]
  run: |
    cargo build
    tests/e2e/run.sh
```

---

## Growth Strategy

For each new feature added to `mdt`:

1. **Add a parser test** — verify the grammar parses the new syntax.
2. **Add a transform test** if it involves layout (new component type, width calculation).
3. **Add an event handler test** if it involves a new key binding or state transition.
4. **Add an E2E snapshot** for non-trivial visual features.

This workflow ensures the test suite grows proportionally with the codebase, and regressions are caught at the most specific layer possible.

---

## Estimated Test Count

| Layer | Tests | Effort |
|-------|-------|--------|
| 1. Data Model | ~25 | Low |
| 2. Layout Transforms | ~22 | Medium |
| 3. Parser | ~30 | Medium |
| 4. Key Bindings | ~14 | Low |
| 5. Utility/State | ~14 | Low |
| 6. Event Handler | ~30 | High |
| 7. Render Pipeline | ~12 | Medium |
| 8. E2E Visual | ~13 | High (one-time harness) |
| **Total** | **~160** | |

This brings coverage from 15 tests (search-focused) to ~175 tests across all subsystems.
