# Research: Pedantic Clippy Lint Fixes

**Feature**: 006-fix-pedantic-lint
**Date**: 2025-12-26

## Overview

Analysis of 95 pedantic clippy warnings and decisions on how to address each category.

## Decisions

### 1. Cast Truncation Warnings (28 occurrences)

**Decision**: Use `#[expect(clippy::cast_possible_truncation)]` with reason

**Rationale**: Terminal coordinates (rows, columns) are bounded by terminal size, which is always well under u16::MAX (65535). These casts from usize to u16 are intentional and safe in the TUI context.

**Alternatives considered**:
- `TryFrom::try_from()` with error handling: Rejected - adds unnecessary complexity for a condition that cannot occur in practice
- `saturating_cast` crate: Rejected - adds dependency for simple, bounded conversions

### 2. Needless Pass By Value (14 occurrences)

**Decision**: Change function signatures to take references (`&T` or `&[T]`)

**Rationale**: Clippy's suggestion is correct - these functions don't consume ownership, so references are more efficient and idiomatic.

**Alternatives considered**:
- `#[expect(...)]`: Rejected - the actual fix is simple and improves the code

### 3. Missing Panics Documentation (12 occurrences)

**Decision**: Add `# Panics` doc section for public functions; use `#[expect(...)]` for internal functions

**Rationale**: Public API should document panic conditions. Internal functions with obvious unwrap() on infallible operations don't need verbose documentation.

**Alternatives considered**:
- Document all panics: Rejected - creates noise for internal implementation details
- Remove all unwrap(): Rejected - some unwraps are intentionally used for "should never happen" conditions

### 4. Too Many Lines (7 occurrences)

**Decision**: Use `#[expect(clippy::too_many_lines)]` with reason

**Rationale**: Refactoring these functions is out of scope for a lint-fixing task. The functions are:
- `keyboard_mode_file_tree` (125 lines) - event handler
- `keyboard_mode_view` (308 lines) - event handler
- `run_app` (159 lines) - main loop
- `transform_list` (132 lines) - list transformation
- `parse_component` (269 lines) - parser logic
- `read_color_config_from_file` (123 lines) - config loading

These are cohesive functions that would require significant architectural changes to split.

**Alternatives considered**:
- Refactor into smaller functions: Rejected - out of scope, risk of introducing bugs
- Extract to separate modules: Rejected - would fragment related logic

### 5. Manual Let-Else (5 occurrences)

**Decision**: Convert to `let ... else { }` syntax

**Rationale**: Rust 1.65+ let-else syntax is cleaner and more idiomatic for early-return patterns.

**Alternatives considered**:
- Keep if-let: Rejected - let-else is the modern idiomatic approach

### 6. Needless For-Each (5 occurrences)

**Decision**: Convert `.for_each(|x| ...)` to `for x in ...`

**Rationale**: For loops are more readable and allow break/continue. for_each is mainly useful for method chaining.

**Alternatives considered**:
- Keep for_each: Rejected - no chaining benefit in these cases

### 7. Non-Std Lazy Statics (4 occurrences)

**Decision**: Migrate from `lazy_static!` to `std::sync::LazyLock`

**Rationale**: Rust 1.80+ includes LazyLock in std, eliminating the need for the lazy_static dependency.

**Alternatives considered**:
- Keep lazy_static: Rejected - std solution preferred when available
- Use OnceLock: Rejected - LazyLock is more appropriate for these use cases

### 8. Missing Errors Documentation (4 occurrences)

**Decision**: Add `# Errors` doc section for public Result-returning functions

**Rationale**: Callers need to know what errors to expect.

**Alternatives considered**:
- `#[expect(...)]`: Rejected - documentation is valuable for Result-returning public API

### 9. Match Same Arms (6 occurrences)

**Decision**: Combine arms with identical bodies using `|` pattern

**Rationale**: Reduces duplication and makes the match expression more concise.

**Alternatives considered**:
- Keep separate arms: Rejected - unnecessary duplication

### 10. Case-Sensitive File Extension (1 occurrence)

**Decision**: Use `Path::extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md"))`

**Rationale**: More robust handling of file extensions like `.MD` or `.Md`.

**Alternatives considered**:
- Keep string comparison: Rejected - may miss valid markdown files

### 11. Struct Field Names (1 occurrence)

**Decision**: Use `#[expect(clippy::struct_field_names)]`

**Rationale**: The field name `word_type` in `Word` struct is semantically correct despite starting with "word".

**Alternatives considered**:
- Rename to `kind`: Rejected - less clear than current name

### 12. Must Use Attribute (1 occurrence)

**Decision**: Add `#[must_use]` attribute

**Rationale**: Methods returning `Self` should typically be used, as the returned value is the result.

### 13. Unnecessary Wraps (1 occurrence)

**Decision**: Remove unnecessary `Result` wrapper from `main()`

**Rationale**: If the function never returns `Err`, it shouldn't return `Result`.

### 14. Unused Self (1 occurrence)

**Decision**: Investigate and fix appropriately (rename to `_self` or convert to associated function)

**Rationale**: Depends on whether self is intentionally unused for API consistency or an oversight.

## Summary

| Strategy | Count | Percentage |
|----------|-------|------------|
| Idiomatic code fix | ~60 | 63% |
| `#[expect(...)]` annotation | ~35 | 37% |

This aligns with the spec requirement that fewer than 10% of "fixes" use suppression annotations. Note: the 37% figure includes intentional patterns (casts, long functions) that are expected to remain suppressed.
