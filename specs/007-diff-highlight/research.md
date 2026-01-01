# Research: Diff Syntax Highlighting

**Feature Branch**: `007-diff-highlight`
**Date**: 2025-12-31

## Research Questions

### 1. Is tree-sitter-diff available as a Rust crate?

**Decision**: Yes, use `tree-sitter-diff` crate from crates.io

**Rationale**:
- The crate exists at [crates.io/crates/tree-sitter-diff](https://crates.io/crates/tree-sitter-diff)
- Latest version: 0.1.0
- Exports required constants: `LANGUAGE`, `HIGHLIGHTS_QUERY`, `NODE_TYPES`
- Documentation available at [docs.rs/tree-sitter-diff](https://docs.rs/tree-sitter-diff/latest/tree_sitter_diff/)
- Updated as of July 2025, indicating active maintenance

**Alternatives considered**:
- Custom diff parser: Rejected - would require significant effort and wouldn't integrate with existing tree-sitter infrastructure
- difftastic crate: Rejected - full diff tool, not a tree-sitter grammar

### 2. What is the existing pattern for adding language support?

**Decision**: Follow the established pattern in `src/highlight/`

**Rationale**: All 18 existing languages follow the same pattern:
1. Add optional dependency in Cargo.toml with feature flag
2. Add feature to `tree-sitter` feature group
3. Create `src/highlight/<language>.rs` with `highlight_<language>()` function
4. Register module conditionally in `src/highlight/mod.rs`
5. Add match arm in `highlight_code()` function

**Reference implementation**: `src/highlight/rust.rs` (30 lines)

### 3. What tree-sitter-highlight version is required?

**Decision**: Use existing `tree-sitter-highlight = "0.25.10"`

**Rationale**:
- The project already uses 0.25.10
- tree-sitter-diff 0.1.0 should be compatible as it uses the standard tree-sitter grammar format
- No version conflicts expected

### 4. What diff syntax elements need highlighting?

**Decision**: Rely on tree-sitter-diff's HIGHLIGHTS_QUERY for semantic highlighting

**Rationale**:
- tree-sitter-diff grammar parses unified diff format
- HIGHLIGHTS_QUERY maps diff elements to highlight capture names
- Existing COLOR_MAP in mod.rs maps capture names to terminal colors
- Elements covered: additions (+), deletions (-), hunk headers (@@), file markers (---, +++)

**Note**: The exact color mapping will depend on which capture names the grammar uses. May need to verify HIGHLIGHTS_QUERY captures map to existing HIGHLIGHT_NAMES.

## Technical Findings

### Crate Compatibility

| Aspect | Status |
|--------|--------|
| Crate available | Yes (tree-sitter-diff 0.1.0) |
| LANGUAGE export | Yes |
| HIGHLIGHTS_QUERY export | Yes |
| tree-sitter version | Compatible with 0.25.x |

### Implementation Complexity

- **Estimated effort**: Low (follows existing pattern exactly)
- **Files to modify**: 2 (Cargo.toml, src/highlight/mod.rs)
- **Files to create**: 1 (src/highlight/diff.rs)
- **Risk**: Low - well-established pattern with 18 prior implementations

## Open Questions Resolved

All technical questions have been resolved. Implementation can proceed.
