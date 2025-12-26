# Tasks: Fix Pedantic Clippy Lints

**Input**: Design documents from `/specs/006-fix-pedantic-lint/`
**Prerequisites**: plan.md, spec.md, research.md

**Tests**: No new tests required - existing `cargo test` validates functionality.

**Organization**: Tasks are organized by file (highest warning count first) to enable parallel execution across files.

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

---

## Phase 1: Preparation

**Purpose**: Establish baseline and prepare for lint fixes

- [ ] T001 Run `cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"` to confirm baseline (expect 95)
- [ ] T002 Run `cargo test` to confirm all tests pass before changes
- [ ] T003 Review fix patterns in plan.md for reference

**Checkpoint**: Baseline established - proceed with lint fixes

---

## Phase 2: High-Impact Files (60+ warnings)

**Purpose**: Fix the 4 highest-warning files first for maximum early progress

**Goal**: Eliminate ~63 warnings (66% of total)

### src/pages/markdown_renderer.rs (21 warnings)

- [ ] T004 [P] Fix all pedantic lints in src/pages/markdown_renderer.rs:
  - Convert `needless_pass_by_value` to references
  - Add `#[expect(clippy::cast_possible_truncation)]` for UI casts
  - Convert `needless_for_each` to for loops
  - Add `#[expect(clippy::too_many_lines)]` where needed
  - Fix `match_same_arms` by combining patterns

### src/nodes/textcomponent.rs (18 warnings)

- [ ] T005 [P] Fix all pedantic lints in src/nodes/textcomponent.rs:
  - Add `#[expect(clippy::cast_possible_truncation)]` for height/width casts
  - Add `#[expect(clippy::cast_sign_loss)]` for float-to-int casts
  - Convert `needless_for_each` to for loops
  - Fix `match_same_arms` by combining patterns
  - Add `#[expect(clippy::too_many_lines)]` for transform_list

### src/util/colors.rs (15 warnings)

- [ ] T006 [P] Fix all pedantic lints in src/util/colors.rs:
  - Migrate `lazy_static!` to `std::sync::LazyLock`
  - Add `# Panics` doc sections or `#[expect(clippy::missing_panics_doc)]`
  - Add `#[expect(clippy::too_many_lines)]` for read_color_config_from_file

### src/search.rs (9 warnings)

- [ ] T007 [P] Fix all pedantic lints in src/search.rs:
  - Convert `needless_pass_by_value` (Vec<&str>) to &[&str]
  - Convert `manual_let_else` to let-else syntax
  - Convert `needless_for_each` to for loops
  - Add `# Panics` doc section or `#[expect(clippy::missing_panics_doc)]`

**Checkpoint**: Run `cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"` - expect ~32 remaining

---

## Phase 3: Medium-Impact Files (25+ warnings)

**Purpose**: Fix the next 4 files with 6-9 warnings each

**Goal**: Eliminate ~31 more warnings

### src/pages/file_explorer.rs (9 warnings)

- [ ] T008 [P] Fix all pedantic lints in src/pages/file_explorer.rs:
  - Add `#[expect(clippy::cast_possible_truncation)]` for UI casts
  - Convert `needless_pass_by_value` to references
  - Convert `needless_for_each` to for loops

### src/event_handler.rs (9 warnings)

- [ ] T009 [P] Fix all pedantic lints in src/event_handler.rs:
  - Add `#[expect(clippy::cast_possible_truncation)]` for index casts
  - Add `#[expect(clippy::too_many_lines)]` for keyboard_mode_* functions
  - Fix case-sensitive file extension using Path::extension()

### src/nodes/root.rs (7 warnings)

- [ ] T010 [P] Fix all pedantic lints in src/nodes/root.rs:
  - Add `#[expect(clippy::cast_possible_truncation)]` for index casts
  - Convert `needless_for_each` to for loops
  - Add `# Errors` doc section or `#[expect(clippy::missing_errors_doc)]`
  - Add `# Panics` doc section or `#[expect(clippy::missing_panics_doc)]`

### src/parser.rs (6 warnings)

- [ ] T011 [P] Fix all pedantic lints in src/parser.rs:
  - Add `#[expect(clippy::too_many_lines)]` for parse_component
  - Add `#[expect(clippy::cast_possible_truncation)]` for heading level cast
  - Fix `match_same_arms` by combining Rule patterns
  - Convert `needless_for_each` to for loops
  - Add `# Panics` doc section or `#[expect(clippy::missing_panics_doc)]`

**Checkpoint**: Run `cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"` - expect ~12 remaining

---

## Phase 4: Low-Impact Files (12 warnings)

**Purpose**: Fix remaining 7 files with 1-5 warnings each

**Goal**: Eliminate all remaining warnings

### src/main.rs (5 warnings)

- [ ] T012 [P] Fix all pedantic lints in src/main.rs:
  - Convert `manual_let_else` patterns to let-else syntax (3 occurrences)
  - Fix `unnecessary_wraps` by removing Result from main()
  - Add `#[expect(clippy::too_many_lines)]` for run_app

### src/util/keys.rs (3 warnings)

- [ ] T013 [P] Fix all pedantic lints in src/util/keys.rs:
  - Fix `match_same_arms` by combining KeyCode patterns
  - Migrate `lazy_static!` to `std::sync::LazyLock`

### src/util.rs (2 warnings)

- [ ] T014 [P] Fix all pedantic lints in src/util.rs:
  - Add `# Panics` doc section or `#[expect(clippy::missing_panics_doc)]`

### src/nodes/word.rs (2 warnings)

- [ ] T015 [P] Fix all pedantic lints in src/nodes/word.rs:
  - Add `#[expect(clippy::struct_field_names)]` for word_type field
  - Add `#[must_use]` attribute if needed

### src/nodes/image.rs (2 warnings)

- [ ] T016 [P] Fix all pedantic lints in src/nodes/image.rs:
  - Convert `needless_pass_by_value` to reference
  - Add `#[expect(clippy::cast_possible_truncation)]` for height cast

### src/highlight/mod.rs (2 warnings)

- [ ] T017 [P] Fix all pedantic lints in src/highlight/mod.rs:
  - Add `# Panics` doc section or `#[expect(clippy::missing_panics_doc)]`
  - Fix `unused_self` if present

### src/util/general.rs (1 warning)

- [ ] T018 [P] Fix all pedantic lints in src/util/general.rs:
  - Migrate `lazy_static!` to `std::sync::LazyLock`

**Checkpoint**: Run `cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"` - expect 0

---

## Phase 5: Validation & Polish

**Purpose**: Verify all success criteria are met

- [ ] T019 Run `cargo clippy -- -W clippy::pedantic` and confirm zero warnings
- [ ] T020 Run `cargo test` and confirm all tests pass
- [ ] T021 Run `cargo build --release` and confirm successful build
- [ ] T022 Review `#[expect(...)]` annotations to ensure they include reason strings
- [ ] T023 Verify `lazy_static` dependency can be removed from Cargo.toml if fully migrated

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Preparation)**: No dependencies - must complete first
- **Phase 2-4 (Lint Fixes)**: All tasks marked [P] can run in parallel across phases
- **Phase 5 (Validation)**: Depends on all lint fix phases completing

### Parallel Opportunities

All file-specific tasks (T004-T018) are independent and can run in parallel:

```bash
# Example: Launch all Phase 2 tasks in parallel
Task: "Fix lints in src/pages/markdown_renderer.rs"
Task: "Fix lints in src/nodes/textcomponent.rs"
Task: "Fix lints in src/util/colors.rs"
Task: "Fix lints in src/search.rs"
```

### Recommended Execution

**Single Developer (Sequential)**:
1. Complete Phase 1
2. Work through files in order (highest count first for early progress)
3. Run clippy after each file to track progress
4. Complete Phase 5 validation

**Parallel Execution**:
1. Complete Phase 1
2. Launch all [P] tasks simultaneously
3. Complete Phase 5 validation

---

## Implementation Strategy

### MVP (Phases 1-2 Only)

1. Complete Phase 1: Preparation
2. Complete Phase 2: High-Impact Files (63 warnings)
3. Validate: 66% of warnings eliminated
4. Stop here for a quick win, continue later

### Full Implementation

1. Complete all phases sequentially or in parallel
2. Target: Zero warnings

---

## Summary

| Phase | Files | Warnings | Cumulative |
|-------|-------|----------|------------|
| Phase 2 | 4 | 63 | 63 (66%) |
| Phase 3 | 4 | 31 | 94 (99%) |
| Phase 4 | 7 | 12 | 95 (100%) |

**Total Tasks**: 23
**Parallel Tasks**: 15 (T004-T018)
**Sequential Tasks**: 8 (T001-T003, T019-T023)

---

## Notes

- All [P] tasks operate on different files - safe to run in parallel
- Run `cargo test` after completing each phase as a sanity check
- Commit after each file or logical group of files
- If a task reveals additional warnings, address them in that task
- Use `#[expect(..., reason = "...")]` format for all suppression annotations
