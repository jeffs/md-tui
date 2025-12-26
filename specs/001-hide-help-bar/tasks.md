# Tasks: Hide Help Bar

**Input**: Design documents from `/specs/001-hide-help-bar/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: No automated tests requested. Manual verification per acceptance scenarios.

**Organization**: Tasks grouped by user story for independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root
- Paths follow existing md-tui structure

---

## Phase 1: Setup

**Purpose**: No project setup needed - existing codebase with established patterns.

*(No tasks - existing project)*

---

## Phase 2: Foundational (Configuration Infrastructure)

**Purpose**: Add `help_bar` config field - required before UI changes.

- [x] T001 Add `help_bar: bool` field to `GeneralConfig` struct in src/util/general.rs

**Checkpoint**: Configuration field available via `GENERAL_CONFIG.help_bar`

---

## Phase 3: User Story 1 - Hide Help Bar via Configuration File (Priority: P1)

**Goal**: Users can hide the Help bar by adding `help_bar = false` to their config file.

**Independent Test**: Create `~/.config/mdt/config.toml` with `help_bar = false`, run `mdt <file>`, verify Help bar hidden, press `?` to confirm expanded help still works.

### Implementation for User Story 1

- [x] T002 [US1] Modify `render_file_tree()` to conditionally render Help bar based on `GENERAL_CONFIG.help_bar` in src/main.rs
- [x] T003 [US1] Modify `render_markdown()` to conditionally render Help bar based on `GENERAL_CONFIG.help_bar` in src/main.rs
- [x] T004 [US1] Adjust content area height calculation in `render_markdown()` when Help bar hidden in src/main.rs
- [x] T005 [US1] Adjust content area height calculation in `render_file_tree()` when Help bar hidden in src/main.rs

**Checkpoint**: Config file `help_bar = false` hides Help bar hint. `?` key still shows expanded help.

---

## Phase 4: User Story 2 - Hide Help Bar via Environment Variable (Priority: P2)

**Goal**: Users can hide Help bar via `MDT_HELP_BAR=false` environment variable.

**Independent Test**: Run `MDT_HELP_BAR=false mdt <file>`, verify Help bar hidden.

### Implementation for User Story 2

*(No additional code changes needed - handled automatically by existing `config` crate infrastructure)*

- [x] T006 [US2] Verify environment variable override works with `MDT_HELP_BAR=false` (manual test)
- [x] T007 [US2] Verify precedence: env var overrides config file setting (manual test)

**Checkpoint**: Environment variable `MDT_HELP_BAR=false` hides Help bar, takes precedence over config file.

---

## Phase 5: Polish & Documentation

**Purpose**: Update documentation and verify edge cases.

- [x] T008 Update README.md with `help_bar` configuration option
- [x] T009 Run quickstart.md validation scenarios
- [x] T010 Verify edge case: invalid config value defaults to showing Help bar
- [x] T011 Verify edge case: Help bar behavior consistent in small terminals

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: N/A - existing project
- **Phase 2 (Foundational)**: No dependencies - start immediately
- **Phase 3 (US1)**: Depends on T001 completion
- **Phase 4 (US2)**: Depends on T001 completion (can run in parallel with US1)
- **Phase 5 (Polish)**: Depends on US1 and US2 completion

### User Story Dependencies

- **User Story 1 (P1)**: Requires T001 (config field). Core implementation.
- **User Story 2 (P2)**: Requires T001 (config field). No additional code - uses existing config crate env var handling.

### Within Each User Story

- T002, T003 can run in parallel (different functions, same file)
- T004 depends on T003 (same function)
- T005 depends on T002 (same function)

### Parallel Opportunities

- T002 and T003 modify different functions - can be done together
- US1 and US2 can be verified in parallel after T001 completes
- T006 and T007 are independent manual tests

---

## Parallel Example: User Story 1

```bash
# After T001 completes, launch these in parallel:
Task: "Modify render_file_tree() conditional rendering in src/main.rs"
Task: "Modify render_markdown() conditional rendering in src/main.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001: Add config field
2. Complete T002-T005: UI conditional rendering
3. **STOP and VALIDATE**: Test with config file `help_bar = false`
4. Verify `?` key still works

### Incremental Delivery

1. T001 → Config field ready
2. US1 (T002-T005) → Test with config file → Core functionality complete
3. US2 (T006-T007) → Test with env var → Full feature complete
4. Polish (T008-T011) → Documentation and edge cases

---

## Notes

- This is a minimal feature (5 implementation tasks + 6 verification/doc tasks)
- No new dependencies required
- No changes to HelpBox widget or event handling
- Existing `config` crate handles env var override automatically (US2 is essentially free)
- Default `true` preserves existing behavior for all users
