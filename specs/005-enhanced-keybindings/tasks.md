# Tasks: Enhanced Keybindings

**Input**: Design documents from `/specs/005-enhanced-keybindings/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Not explicitly requested - test tasks not included. Manual testing via quickstart.md.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No project setup needed - existing Rust project with all dependencies in place.

(No tasks - project already initialized with crossterm, config, lazy_static)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T001 Create KeyBinding struct with key and modifiers fields in src/util/keys.rs
- [x] T002 Add crossterm::event::KeyEvent and KeyModifiers imports in src/util/keys.rs
- [x] T003 Create parse_key_string() function to parse single key strings in src/util/keys.rs
- [x] T004 Create helper function to create KeyBinding from char in src/util/keys.rs
- [x] T005 Update key_to_action() signature from KeyCode to &KeyEvent in src/util/keys.rs
- [x] T006 Update handle_keyboard_input() to accept KeyEvent in src/event_handler.rs
- [x] T007 Update main.rs event loop to pass full KeyEvent instead of key.code

**Checkpoint**: Foundation ready - KeyBinding type exists, event flow passes KeyEvent

---

## Phase 3: User Story 1 + 3 - Multiple Keys & Backwards Compatibility (Priority: P1) 🎯 MVP

**Goal**: Support array syntax for multiple keys per action while preserving single-string backwards compatibility

**Independent Test**: Configure `page_down = ["d", " "]` in config, verify both keys work. Also verify `up = "k"` still works.

### Implementation for User Stories 1 & 3

- [x] T008 [US1] Change KeyConfig fields from char to Vec<KeyBinding> in src/util/keys.rs
- [x] T009 [US1] Create parse_bindings() function to handle Value polymorphism (string vs array) in src/util/keys.rs
- [x] T010 [US1] Update lazy_static KEY_CONFIG block to use parse_bindings() for each action in src/util/keys.rs
- [x] T011 [US1] Create default_binding() helper for fallback defaults in src/util/keys.rs
- [x] T012 [US1] Update key_to_action() to iterate Vec<KeyBinding> and match first in src/util/keys.rs
- [x] T013 [US1] Add named key support (space, tab, enter, esc, backspace) to parse_key_string() in src/util/keys.rs
- [x] T014 [US3] Ensure single string config values parse correctly as single-element Vec in src/util/keys.rs

**Checkpoint**: User Stories 1 & 3 complete - array syntax and backwards compatibility both work

---

## Phase 4: User Story 2 - Control Key Modifiers (Priority: P2)

**Goal**: Support `C-key` syntax for control-modified keybindings

**Independent Test**: Configure `down = ["j", "C-e"]` in config, verify both j and Ctrl+e scroll down.

### Implementation for User Story 2

- [x] T015 [US2] Add C- prefix detection (case-insensitive) to parse_key_string() in src/util/keys.rs
- [x] T016 [US2] Set KeyModifiers::CONTROL when C- prefix detected in src/util/keys.rs
- [x] T017 [US2] Update KeyBinding matching in key_to_action() to compare modifiers in src/util/keys.rs
- [x] T018 [US2] Handle edge case: strip SHIFT modifier for comparison (terminals report Ctrl+Shift for some keys) in src/util/keys.rs

**Checkpoint**: User Story 2 complete - C-e, C-y, C-d etc. all work

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation and cleanup

- [x] T019 [P] Update README.md keyboard configuration section with new array and C- syntax
- [x] T020 [P] Add config examples to README.md showing mixed old/new syntax
- [x] T021 Run quickstart.md manual test scenarios
- [x] T022 Build and verify no compiler warnings with cargo build

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: N/A - no setup needed
- **Foundational (Phase 2)**: BLOCKS all user stories - must complete T001-T007 first
- **User Stories 1+3 (Phase 3)**: Depends on Foundational - core functionality
- **User Story 2 (Phase 4)**: Depends on Phase 3 (builds on parse_key_string)
- **Polish (Phase 5)**: Depends on all user stories complete

### User Story Dependencies

- **User Stories 1+3 (P1)**: Combined because backwards compatibility requires understanding array parsing
- **User Story 2 (P2)**: Builds on top of US1/US3 parsing infrastructure

### Within Each Phase

- T001-T004 can be done together (KeyBinding struct and helpers)
- T005-T007 must be sequential (signature change propagates up call chain)
- T008-T014 mostly sequential (each builds on previous)
- T015-T018 sequential (modifier parsing is incremental)
- T019-T020 can run in parallel (different sections of README)

### Parallel Opportunities

Within Foundational:
- T001, T002, T003, T004 can be done as a group before T005

Within User Story 1+3:
- After T008, T009-T011 depend on it sequentially

Within Polish:
- T019 and T020 can be parallel (both modify README but different sections)

---

## Parallel Example: Foundational Phase

```bash
# First batch (no dependencies):
Task: T001 "Create KeyBinding struct"
Task: T002 "Add imports"
Task: T003 "Create parse_key_string()"
Task: T004 "Create helper function"

# Second batch (depends on first):
Task: T005 "Update key_to_action signature"

# Third batch (depends on T005):
Task: T006 "Update handle_keyboard_input"
Task: T007 "Update main.rs event loop"
```

---

## Implementation Strategy

### MVP First (User Stories 1+3 Only)

1. Complete Phase 2: Foundational (T001-T007)
2. Complete Phase 3: User Stories 1+3 (T008-T014)
3. **STOP and VALIDATE**: Test with `page_down = ["d", " "]` AND `up = "k"`
4. Array syntax and backwards compatibility working = MVP done

### Full Feature

1. Complete MVP above
2. Complete Phase 4: User Story 2 (T015-T018)
3. **VALIDATE**: Test with `down = ["j", "C-e"]`
4. Complete Phase 5: Polish (T019-T022)
5. Full feature complete

### Incremental Commits

Suggested commit points:
- After T007: "refactor: pass KeyEvent through event handling chain"
- After T014: "feat: support array syntax for multiple keybindings"
- After T018: "feat: support C- prefix for control key modifiers"
- After T022: "docs: update README with new keybinding syntax"

---

## Notes

- All work is in src/util/keys.rs except for event flow (T006-T007) and docs (T019-T020)
- No test tasks included - use quickstart.md for manual validation
- Backwards compatibility (US3) is tested alongside US1 since they share implementation
- The existing `cargo test` tests should continue to pass throughout
