# Implementation Plan: Fix Pedantic Clippy Lints

**Branch**: `006-fix-pedantic-lint` | **Date**: 2025-12-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-fix-pedantic-lint/spec.md`

## Summary

Fix all 95 pedantic clippy warnings to achieve zero warnings with `cargo clippy -- -W clippy::pedantic`. Use idiomatic Rust fixes where possible; use `#[expect(...)]` annotations for intentional patterns (casts in UI code, long functions).

## Technical Context

**Language/Version**: Rust 1.92.0 (2024 edition)
**Primary Dependencies**: ratatui 0.29.0, crossterm 0.29.0, pest (parser), tree-sitter (syntax highlighting)
**Storage**: N/A (file-based markdown viewer)
**Testing**: cargo test
**Target Platform**: Cross-platform terminal (macOS, Linux, Windows)
**Project Type**: Single Rust crate (lib + bin)
**Performance Goals**: N/A (code quality task, no runtime changes)
**Constraints**: All existing tests must pass; no behavioral changes
**Scale/Scope**: 95 warnings across 15 files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: PASS - Constitution is a placeholder template with no specific gates defined.

This is a code quality refactoring task with no architectural changes. No constitution violations possible.

## Project Structure

### Documentation (this feature)

```text
specs/006-fix-pedantic-lint/
├── plan.md              # This file
├── research.md          # Phase 0: Lint category analysis
├── checklists/
│   └── requirements.md  # Validation checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs                      # 5 warnings
├── lib.rs                       # Entry point
├── event_handler.rs             # 9 warnings
├── parser.rs                    # 6 warnings
├── search.rs                    # 9 warnings
├── util.rs                      # 2 warnings
├── boxes/                       # Input boxes
├── highlight/
│   └── mod.rs                   # 2 warnings
├── nodes/
│   ├── image.rs                 # 2 warnings
│   ├── root.rs                  # 7 warnings
│   ├── textcomponent.rs         # 18 warnings
│   └── word.rs                  # 2 warnings
├── pages/
│   ├── file_explorer.rs         # 9 warnings
│   └── markdown_renderer.rs     # 21 warnings (highest)
└── util/
    ├── colors.rs                # 15 warnings
    ├── general.rs               # 1 warning
    └── keys.rs                  # 3 warnings

tests/
└── (existing tests - must continue passing)
```

**Structure Decision**: Existing single-crate structure. No structural changes needed.

## Lint Analysis by Category

| Category | Count | Fix Strategy |
|----------|-------|--------------|
| `cast_possible_truncation` (usize→u16, u32→u16) | 28 | `#[expect(...)]` - intentional UI casts |
| `needless_pass_by_value` | 14 | Change to `&T` or `&[T]` references |
| `missing_panics_doc` | 12 | Add `# Panics` doc section or `#[expect(...)]` |
| `match_same_arms` | 6 | Combine duplicate arms with `\|` pattern |
| `manual_let_else` | 5 | Convert to `let ... else { return }` syntax |
| `needless_for_each` | 5 | Convert `.for_each(\|x\| ...)` to `for x in ...` |
| `non_std_lazy_statics` | 4 | Migrate `lazy_static!` to `std::sync::LazyLock` |
| `missing_errors_doc` | 4 | Add `# Errors` doc section or `#[expect(...)]` |
| `too_many_lines` | 7 | `#[expect(...)]` - refactoring out of scope |
| `unnecessary_wraps` | 1 | Remove unnecessary `Result` wrapper |
| `case_sensitive_file_extension` | 1 | Use `Path::extension().is_some_and()` |
| `struct_field_names` | 1 | `#[expect(...)]` - field name is appropriate |
| `must_use` | 1 | Add `#[must_use]` attribute |
| `unused_self` | 1 | Remove self or use `_self` |

## Implementation Order

Files ordered by warning count (highest first) for maximum early progress:

1. **src/pages/markdown_renderer.rs** (21 warnings)
2. **src/nodes/textcomponent.rs** (18 warnings)
3. **src/util/colors.rs** (15 warnings)
4. **src/search.rs** (9 warnings)
5. **src/pages/file_explorer.rs** (9 warnings)
6. **src/event_handler.rs** (9 warnings)
7. **src/nodes/root.rs** (7 warnings)
8. **src/parser.rs** (6 warnings)
9. **src/main.rs** (5 warnings)
10. **src/util/keys.rs** (3 warnings)
11. **src/util.rs** (2 warnings)
12. **src/nodes/word.rs** (2 warnings)
13. **src/nodes/image.rs** (2 warnings)
14. **src/highlight/mod.rs** (2 warnings)
15. **src/util/general.rs** (1 warning)

## Fix Patterns Reference

### Cast Truncation (`cast_possible_truncation`)
```rust
// Use #[expect(...)] for intentional UI coordinate casts
#[expect(clippy::cast_possible_truncation, reason = "terminal size bounded")]
let height = content.len() as u16;
```

### Manual Let-Else (`manual_let_else`)
```rust
// Before
let url = if let Some(url) = markdown.file_name() {
    url
} else {
    app.mode = Mode::FileTree;
    continue;
};

// After
let Some(url) = markdown.file_name() else {
    app.mode = Mode::FileTree;
    continue;
};
```

### Needless For-Each (`needless_for_each`)
```rust
// Before
heights.iter_mut().for_each(|h| *h += offset);

// After
for h in &mut heights {
    *h += offset;
}
```

### Match Same Arms (`match_same_arms`)
```rust
// Before
match key {
    KeyCode::PageUp => return Action::PageUp,
    KeyCode::Left => return Action::PageUp,  // duplicate
    ...
}

// After
match key {
    KeyCode::PageUp | KeyCode::Left => return Action::PageUp,
    ...
}
```

### LazyLock Migration (`non_std_lazy_statics`)
```rust
// Before
lazy_static! {
    static ref CONFIG: Arc<RwLock<Config>> = RwLock::new(load_config()).into();
}

// After
static CONFIG: LazyLock<Arc<RwLock<Config>>> =
    LazyLock::new(|| Arc::new(RwLock::new(load_config())));
```

### Pass by Reference (`needless_pass_by_value`)
```rust
// Before
pub fn find(query: &str, text: Vec<&str>) -> Vec<usize>

// After
pub fn find(query: &str, text: &[&str]) -> Vec<usize>
```

## Verification Commands

```bash
# Check for remaining warnings (target: 0)
cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"

# Ensure tests still pass
cargo test

# Ensure release build works
cargo build --release
```

## Complexity Tracking

No constitution violations - this is a code quality task with no architectural changes.
