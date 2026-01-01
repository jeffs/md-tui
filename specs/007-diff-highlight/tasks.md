# Tasks: Diff Syntax Highlighting

**Input**: Design documents from `/specs/007-diff-highlight/`
**Prerequisites**: plan.md, spec.md, research.md, quickstart.md

**Tests**: Not requested in feature specification - implementation tasks only.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `Cargo.toml` at repository root
- Reference implementation: `src/highlight/rust.rs`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add tree-sitter-diff dependency and feature flag

- [x] T001 Add `tree-sitter-diff = { version = "0.1.0", optional = true }` dependency in Cargo.toml (after tree-sitter-yaml line 78)
- [x] T002 Add `"tree-sitter-diff"` to tree-sitter feature group in Cargo.toml (after line 39)

**Checkpoint**: `cargo check` should pass (dependency available but not yet used)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None required - this feature adds to existing infrastructure without needing new foundational work.

**⚠️ SKIP**: No foundational tasks needed. Existing highlight infrastructure is already in place.

---

## Phase 3: User Story 1 - View Highlighted Diff Blocks (Priority: P1)

**Goal**: Enable syntax highlighting for basic diff content (added/removed/context lines)

**Independent Test**: Open a markdown file with a `diff` code block containing +/- lines and verify visual distinction

### Implementation for User Story 1

- [x] T003 [US1] Create src/highlight/diff.rs with `highlight_diff()` function following rust.rs pattern
- [x] T004 [US1] Add conditional module declaration `#[cfg(feature = "tree-sitter-diff")] mod diff;` in src/highlight/mod.rs (after yaml line 38)
- [x] T005 [US1] Add match arm for "diff" language in `highlight_code()` function in src/highlight/mod.rs (after yaml match arm ~line 158)
- [x] T006 [US1] Verify build succeeds with `cargo build`
- [x] T007 [US1] Manual test: create test.md with diff block, run `cargo run -- test.md`, verify highlighting appears

**Checkpoint**: Basic diff highlighting works - added lines (+) and removed lines (-) are visually distinct

---

## Phase 4: User Story 2 - View Patch Format Diffs (Priority: P2)

**Goal**: Ensure unified diff structural elements (hunk headers, file markers) are highlighted

**Independent Test**: View a unified diff with `@@` headers and `---`/`+++` file markers, verify distinct styling

### Implementation for User Story 2

- [x] T008 [US2] Verify hunk headers (`@@`) are highlighted by tree-sitter-diff HIGHLIGHTS_QUERY
- [x] T009 [US2] Verify file markers (`---`/`+++`) are highlighted correctly
- [x] T010 [US2] If capture names missing from COLOR_MAP: add new entries to HIGHLIGHT_NAMES and COLOR_MAP in src/highlight/mod.rs
- [x] T011 [US2] Manual test: unified diff with all elements renders correctly

**Checkpoint**: Full unified diff format renders with appropriate highlighting

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, feature flag verification, code quality

- [x] T012 [P] Verify empty diff blocks render without error
- [x] T013 [P] Verify malformed diff content doesn't crash (best-effort highlighting)
- [x] T014 Test build without feature: `cargo build --no-default-features --features network`
- [x] T015 Verify diff blocks render as plain text when feature disabled
- [x] T016 Add "patch" as alias in match arm (diff | patch => ...) in src/highlight/mod.rs
- [x] T017 Run `cargo clippy` and fix any pedantic warnings in new code
- [x] T018 Run `cargo test` to ensure no regressions

**Checkpoint**: Feature complete with all edge cases handled

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Skipped - no new infrastructure needed
- **User Story 1 (Phase 3)**: Depends on Setup completion
- **User Story 2 (Phase 4)**: Depends on User Story 1 (same file)
- **Polish (Phase 5)**: Depends on User Story 1 completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Setup - creates core highlighting
- **User Story 2 (P2)**: Depends on US1 - verifies/extends US1 implementation

### Within Each Phase

- T001 and T002 must both complete before T003
- T003 must complete before T004 and T005 (module must exist before registration)
- T004 and T005 can run in sequence (same file, different locations)
- T006 and T007 are verification steps after implementation

### Parallel Opportunities

- T001 and T002 modify same file but different sections (can be combined)
- T012 and T013 are independent verification tasks [P]
- T014 and T015 are sequential (build then test)

---

## Parallel Example: Setup Phase

```bash
# T001 and T002 modify Cargo.toml - execute sequentially or as single edit
Task: "Add tree-sitter-diff dependency and feature flag in Cargo.toml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 3: User Story 1 (T003-T007)
3. **STOP and VALIDATE**: Test with simple diff block
4. Ready for use immediately

### Full Implementation

1. Setup (T001-T002) → Dependency available
2. User Story 1 (T003-T007) → Basic highlighting works
3. User Story 2 (T008-T011) → Full unified diff support
4. Polish (T012-T018) → Edge cases and quality

---

## Summary

| Phase | Tasks | Purpose |
|-------|-------|---------|
| Setup | T001-T002 | Add dependency |
| User Story 1 | T003-T007 | Core diff highlighting |
| User Story 2 | T008-T011 | Unified diff elements |
| Polish | T012-T018 | Edge cases, quality |

**Total Tasks**: 18
**Parallel Opportunities**: 2 (T012, T013)
**MVP Scope**: Tasks T001-T007 (7 tasks)
