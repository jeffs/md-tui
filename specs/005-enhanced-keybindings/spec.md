# Feature Specification: Enhanced Keybindings

**Feature Branch**: `005-enhanced-keybindings`
**Created**: 2025-12-26
**Status**: Draft
**Input**: User description: "Enhance keybinding functionality to: (1) support mapping multiple keys to the same action, such as 'd' and ' ' (space) both meaning PageDown; and (2) support control key modifiers, such as mapping control+e and control+y to the same actions as k and j, respectively. The config format should support both single strings for backwards compatibility and arrays for multiple bindings. Modifier syntax should be intuitive like 'ctrl+e' or 'ctrl-y'."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure Multiple Keys for Same Action (Priority: P1)

A power user who frequently switches between different keyboard layouts or has muscle memory from other applications wants to map multiple keys to the same action. For example, they want both `d` and the space bar to trigger PageDown, allowing them to use whichever key feels natural in the moment.

**Why this priority**: This is the core enhancement enabling flexible keybinding configuration. Multiple keys per action is the foundation that makes the keybinding system more versatile and user-friendly.

**Independent Test**: Can be fully tested by adding an array of keys to the config file (e.g., `page_down = ["d", " "]`), launching mdt, and verifying both keys trigger the same action.

**Acceptance Scenarios**:

1. **Given** a config file with `page_down = ["d", " "]`, **When** the user presses `d` while viewing a markdown file, **Then** the view scrolls down one page.
2. **Given** a config file with `page_down = ["d", " "]`, **When** the user presses space while viewing a markdown file, **Then** the view scrolls down one page (same as pressing `d`).
3. **Given** a config file with `up = ["k", "i"]`, **When** the user presses either `k` or `i`, **Then** the view scrolls up one line.

---

### User Story 2 - Configure Control Key Modifiers (Priority: P2)

A vim user who is accustomed to using `Ctrl+e` and `Ctrl+y` for scrolling wants to configure these familiar keybindings in mdt. They want to add control-modified keys alongside their existing single-character bindings.

**Why this priority**: Control key modifiers significantly expand the available keybinding space and enable vim-style navigation patterns that many terminal users expect.

**Independent Test**: Can be fully tested by adding a control-modified key to the config (e.g., `down = ["j", "ctrl+e"]`), launching mdt, and verifying Ctrl+e triggers the scroll down action.

**Acceptance Scenarios**:

1. **Given** a config file with `down = ["j", "ctrl+e"]`, **When** the user presses `Ctrl+e` while viewing a markdown file, **Then** the view scrolls down one line.
2. **Given** a config file with `up = ["k", "ctrl+y"]`, **When** the user presses `Ctrl+y` while viewing a markdown file, **Then** the view scrolls up one line.
3. **Given** a config file with `page_down = "ctrl+d"`, **When** the user presses `Ctrl+d`, **Then** the view scrolls down one page.

---

### User Story 3 - Backwards Compatibility (Priority: P1)

An existing mdt user who has a working config file with single-character keybindings wants their configuration to continue working without any changes after updating to the new version.

**Why this priority**: Backwards compatibility is critical to avoid breaking existing users' workflows. This is P1 alongside the new feature because both must work together.

**Independent Test**: Can be fully tested by using an existing config file with single-key syntax (e.g., `up = "k"`), launching mdt, and verifying all keybindings work as before.

**Acceptance Scenarios**:

1. **Given** an existing config file with `up = "k"` (single string), **When** the user launches mdt and presses `k`, **Then** the view scrolls up (unchanged behavior).
2. **Given** no config file exists, **When** the user launches mdt, **Then** all default keybindings work as documented.
3. **Given** a config file mixing old syntax (`up = "k"`) and new syntax (`page_down = ["d", " "]`), **When** the user launches mdt, **Then** both syntaxes are honored correctly.

---

### Edge Cases

- What happens when an invalid modifier is specified (e.g., `"ctr+e"` typo)? The system should use the default binding for that action and continue operating.
- What happens when the same key is bound to multiple actions? The first matching action in evaluation order should be used (existing behavior preserved).
- How is the space character represented in config? Both `" "` (space in quotes) and `"space"` (named key) should be accepted.
- What happens with case sensitivity for modifiers? `"ctrl+e"`, `"Ctrl+E"`, and `"CTRL+e"` should all be treated equivalently.
- What happens when a control-modified key conflicts with terminal control sequences (e.g., Ctrl+c)? Reserved terminal sequences should take precedence; the config should not override them.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support binding multiple keys to a single action using array syntax (e.g., `action = ["key1", "key2"]`).
- **FR-002**: System MUST support single-key string syntax for backwards compatibility (e.g., `action = "key"`).
- **FR-003**: System MUST support control key modifiers using intuitive syntax: `ctrl+key`, `ctrl-key`, or `control+key`.
- **FR-004**: System MUST treat modifier syntax as case-insensitive (e.g., `ctrl+e` equals `Ctrl+E`).
- **FR-005**: System MUST support the space key using either `" "` (literal space) or `"space"` (named).
- **FR-006**: System MUST fall back to default bindings when an action has invalid or unparseable key specifications.
- **FR-007**: System MUST preserve existing default keybindings when no config is specified.
- **FR-008**: System MUST allow mixing single-string and array syntax in the same config file.
- **FR-009**: System MUST NOT override terminal-reserved control sequences (Ctrl+c, Ctrl+z).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can configure any action with 1 or more different key combinations.
- **SC-002**: Existing config files with single-key syntax continue to work without modification (100% backwards compatibility).
- **SC-003**: Control-modified keys respond within the same timeframe as single-character keys (no perceptible delay).
- **SC-004**: Configuration errors do not crash the application; defaults are used gracefully.

## Assumptions

- The modifier key support is limited to Control (Ctrl) initially. Alt/Meta and Shift modifiers may be added in future iterations.
- The config file format remains TOML.
- Environment variable configuration follows the same syntax rules as config file values where applicable.
- Key names for special keys follow common conventions: `space`, `tab`, `enter`, etc.

## Out of Scope

- Alt/Meta key modifier support (future enhancement).
- Shift key modifier support (future enhancement).
- Key chord sequences (e.g., pressing `g` then `g` for go-to-top like vim).
- GUI-based keybinding configuration.
