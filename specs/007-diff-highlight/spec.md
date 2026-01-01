# Feature Specification: Diff Syntax Highlighting

**Feature Branch**: `007-diff-highlight`
**Created**: 2025-12-31
**Status**: Draft
**Input**: User description: "Add syntax highlighting for diff code blocks."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Highlighted Diff Blocks (Priority: P1)

A user viewing a markdown document containing diff/patch code blocks wants the diff content to be syntax highlighted, making it easier to distinguish added lines, removed lines, and context.

**Why this priority**: This is the core functionality of the feature. Without diff highlighting, the feature provides no value.

**Independent Test**: Can be fully tested by opening a markdown file with a diff code block and verifying that additions, deletions, and context lines are visually distinct.

**Acceptance Scenarios**:

1. **Given** a markdown file with a fenced code block tagged as `diff`, **When** the user opens the file in mdt, **Then** the diff content is displayed with syntax highlighting applied.
2. **Given** a diff block with added lines (starting with `+`), **When** rendered, **Then** added lines are visually distinguished from other content.
3. **Given** a diff block with removed lines (starting with `-`), **When** rendered, **Then** removed lines are visually distinguished from other content.
4. **Given** a diff block with context lines (no prefix or starting with space), **When** rendered, **Then** context lines appear in a neutral style distinct from additions and removals.

---

### User Story 2 - View Patch Format Diffs (Priority: P2)

A user viewing unified diff output (with `@@` hunk headers and `---`/`+++` file markers) wants the structural elements highlighted distinctly from the content changes.

**Why this priority**: Many diffs include hunk headers and file markers. Highlighting these improves readability but is secondary to the core add/remove highlighting.

**Independent Test**: Can be tested by viewing a markdown file containing a unified diff with hunk headers and file path lines.

**Acceptance Scenarios**:

1. **Given** a diff block with hunk headers (`@@ -1,3 +1,4 @@`), **When** rendered, **Then** hunk headers are visually distinct.
2. **Given** a diff block with file markers (`--- a/file` and `+++ b/file`), **When** rendered, **Then** file markers are highlighted appropriately.

---

### Edge Cases

- What happens when a diff code block is empty? The block should render without error.
- What happens when the `tree-sitter-diff` feature is disabled at compile time? Diff blocks should render as plain text (unhighlighted), consistent with other disabled languages.
- How does the system handle malformed diff content (e.g., lines with invalid prefixes)? The highlighter should apply best-effort highlighting without crashing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST recognize fenced code blocks tagged with `diff` as candidates for syntax highlighting.
- **FR-002**: System MUST apply distinct visual styling to added lines (lines beginning with `+`).
- **FR-003**: System MUST apply distinct visual styling to removed lines (lines beginning with `-`).
- **FR-004**: System MUST apply distinct visual styling to hunk headers (lines beginning with `@@`).
- **FR-005**: System MUST apply distinct visual styling to file markers (`---` and `+++` lines at diff start).
- **FR-006**: System MUST fall back to unhighlighted display when the diff highlighting feature is disabled at compile time.
- **FR-007**: System MUST handle empty or malformed diff blocks without crashing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can visually distinguish added, removed, and context lines in diff blocks at a glance.
- **SC-002**: Diff code blocks render without errors for valid unified diff format content.
- **SC-003**: Feature follows existing syntax highlighting patterns, requiring no additional user configuration.
- **SC-004**: Builds without the tree-sitter-diff feature compile and run correctly, displaying diff blocks as plain text.
