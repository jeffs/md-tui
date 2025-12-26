# Feature Specification: Fix Pedantic Clippy Lints

**Feature Branch**: `006-fix-pedantic-lint`
**Created**: 2025-12-26
**Status**: Draft
**Input**: User description: "Fix all pedantic clippy lints for cleaner code"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Clean Build Output (Priority: P1)

As a developer, I want to run `cargo clippy -- -W clippy::pedantic` and see zero warnings so that the codebase maintains high quality standards.

**Why this priority**: This is the core deliverable - eliminating all pedantic lint warnings is the primary goal of this feature.

**Independent Test**: Can be fully tested by running `cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"` and confirming the count is 0.

**Acceptance Scenarios**:

1. **Given** the project compiles successfully, **When** I run `cargo clippy -- -W clippy::pedantic`, **Then** zero warnings are reported
2. **Given** all lints are fixed, **When** I run the standard build and test suite, **Then** all existing tests continue to pass

---

### User Story 2 - Maintainable Code Quality (Priority: P2)

As a developer, I want the lint fixes to follow idiomatic Rust patterns so that the code remains readable and maintainable.

**Why this priority**: Fixes should improve code quality, not just silence warnings with blanket allows.

**Independent Test**: Code review confirms fixes use proper Rust idioms rather than excessive `#[allow(...)]` annotations.

**Acceptance Scenarios**:

1. **Given** a lint warning exists, **When** it is fixed, **Then** the fix uses the idiomatic approach recommended by Clippy (e.g., using `let...else` instead of `if let...else return`)
2. **Given** a lint cannot be reasonably fixed, **When** an expect annotation is used, **Then** the annotation is targeted to the specific item, not broadly applied

---

### User Story 3 - Preserved Functionality (Priority: P1)

As a user of MD-TUI, I want the application to behave identically after lint fixes so that no regressions are introduced.

**Why this priority**: Code refactoring must not change behavior.

**Independent Test**: Run existing test suite and manual verification of core features.

**Acceptance Scenarios**:

1. **Given** the lint fixes are applied, **When** I run `cargo test`, **Then** all existing tests pass
2. **Given** the lint fixes are applied, **When** I run the application with markdown files, **Then** rendering and navigation work as before

---

### Edge Cases

- Functions flagged as "too many lines" may use `#[expect(...)]` rather than being refactored to avoid scope creep
- Cast truncation warnings in display/UI code (usize to u16) are intentional and may need targeted `#[expect(...)]`
- Documentation-related lints (missing panics/errors docs) are addressed with doc comments or targeted `#[expect(...)]`

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Build MUST complete with `cargo clippy -- -W clippy::pedantic` producing zero warnings
- **FR-002**: All existing unit and integration tests MUST continue to pass
- **FR-003**: Code fixes MUST prefer Clippy's suggested idiomatic solutions over suppression annotations
- **FR-004**: When suppression is necessary, MUST use `#[expect(..., reason = "...")]` instead of `#[allow(...)]` so removability is tracked and intent is documented
- **FR-005**: Expect annotations MUST be scoped to specific functions/items, not applied at module level
- **FR-006**: Semantic behavior of all functions MUST remain unchanged

### Categories of Lints to Address

Based on the current codebase analysis, the following lint categories require attention:

- **Cast truncation warnings** (`cast_possible_truncation`, `cast_sign_loss`): Review each cast, add `#[expect(...)]` where truncation is intentional/safe
- **Too many lines** (`too_many_lines`): Use `#[expect(...)]` on specific functions rather than refactoring
- **Needless pass by value** (`needless_pass_by_value`): Change function signatures to take references where appropriate
- **Missing documentation** (`missing_errors_doc`, `missing_panics_doc`): Add doc sections or use `#[expect(...)]` on internal functions
- **Manual let-else** (`manual_let_else`): Convert if-let-else patterns to let-else syntax
- **Match same arms** (`match_same_arms`): Combine arms with identical bodies
- **Needless for_each** (`needless_for_each`): Convert to for loops
- **Case-sensitive file extension** (`case_sensitive_file_extension_comparisons`): Use Path-based extension checking
- **Struct field names** (`struct_field_names`): Use `#[expect(...)]` or rename fields
- **Non-std lazy statics** (`non_std_lazy_statics`): Consider migrating from lazy_static to std::sync::LazyLock
- **Unnecessary wraps** (`unnecessary_wraps`): Simplify return types where Result is unnecessary

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Running `cargo clippy -- -W clippy::pedantic` produces zero warnings
- **SC-002**: Running `cargo test` shows 100% of existing tests passing
- **SC-003**: Running `cargo build --release` completes successfully
- **SC-004**: `#[expect(...)]` annotations are used only for intentional patterns (UI casts, long functions) where idiomatic fixes are impractical or out of scope

## Assumptions

- The current "lint" branch already has some pedantic lint work in progress
- Refactoring large functions is out of scope; "too_many_lines" warnings will be allowed
- Cast truncation in UI code (usize to u16 for terminal coordinates) is intentional and safe given terminal size limits
- lazy_static migration to LazyLock may be deferred if it requires significant refactoring
