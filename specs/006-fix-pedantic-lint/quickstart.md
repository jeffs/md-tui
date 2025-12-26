# Quickstart: Fix Pedantic Clippy Lints

## Prerequisites

- Rust 1.92.0+ (for `std::sync::LazyLock` and let-else syntax)
- Current codebase on `006-fix-pedantic-lint` branch

## Verification

```bash
# Check current warning count (start: 95)
cargo clippy -- -W clippy::pedantic 2>&1 | grep -c "^warning:"

# Run tests to ensure no regressions
cargo test

# Build release to verify
cargo build --release
```

## Implementation Approach

1. Work through files in order of warning count (see plan.md)
2. For each file:
   - Run clippy to identify warnings
   - Apply idiomatic fixes where possible
   - Use `#[expect(..., reason = "...")]` for intentional patterns
3. Run tests after each file
4. Commit changes incrementally

## Key Fix Patterns

See [plan.md](./plan.md#fix-patterns-reference) for code examples of each fix pattern.

## Success Criteria

- `cargo clippy -- -W clippy::pedantic` produces 0 warnings
- `cargo test` passes
- `cargo build --release` succeeds
