# Feature Specification: Hide Help Bar

**Feature Branch**: `001-hide-help-bar`
**Created**: 2025-12-26
**Status**: Draft
**Input**: User description: "Add configurable Help bar visibility to terminal markdown viewer"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Hide Help Bar via Configuration File (Priority: P1)

An experienced user who frequently uses mdt knows the `?` keyboard shortcut for help and wants to maximize their terminal space for viewing markdown content. They configure the Help bar to be hidden by adding a setting to their config file.

**Why this priority**: This is the core functionality requested. Config file settings provide persistent user preferences, which is the most common way users customize CLI tools.

**Independent Test**: Can be fully tested by creating a config file with the hide setting, launching mdt with a markdown file, and verifying the Help bar is not displayed while the `?` key still functions to show full help.

**Acceptance Scenarios**:

1. **Given** a config file at `~/.config/mdt/config.toml` with `help_bar = false`, **When** the user launches mdt with a markdown file, **Then** the Help bar is not displayed and the full terminal height is used for content.
2. **Given** no config file exists or `help_bar` is not specified, **When** the user launches mdt, **Then** the Help bar is displayed (preserving current default behavior).
3. **Given** the Help bar is hidden via configuration, **When** the user presses `?`, **Then** the expanded help panel still appears showing all keybindings.

---

### User Story 2 - Hide Help Bar via Environment Variable (Priority: P2)

A user wants to temporarily hide the Help bar for a specific session or set it system-wide through their shell profile using an environment variable, following the existing mdt pattern of `MDT_` prefixed environment variables.

**Why this priority**: Environment variables provide flexibility for session-specific overrides without modifying configuration files, following the established pattern in the codebase.

**Independent Test**: Can be fully tested by setting `MDT_HELP_BAR=false` environment variable, launching mdt, and verifying the Help bar is hidden.

**Acceptance Scenarios**:

1. **Given** the environment variable `MDT_HELP_BAR=false` is set, **When** the user launches mdt, **Then** the Help bar is not displayed.
2. **Given** both config file has `help_bar = true` and environment variable `MDT_HELP_BAR=false` is set, **When** the user launches mdt, **Then** the environment variable takes precedence and the Help bar is hidden.

---

### Edge Cases

- What happens when an invalid value is provided for `help_bar` in config (e.g., a string instead of boolean)? The system should use the default (show Help bar) and optionally warn.
- What happens when the terminal height is very small (< 10 lines)? The Help bar behavior should be consistent with the user's preference regardless of terminal size.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support a `help_bar` boolean configuration option in the config file (`~/.config/mdt/config.toml`) to control Help bar visibility.
- **FR-002**: System MUST support an `MDT_HELP_BAR` environment variable to control Help bar visibility, following the existing `MDT_` prefix convention.
- **FR-003**: System MUST maintain the current default behavior of showing the Help bar when no configuration is specified.
- **FR-004**: System MUST preserve `?` key functionality to show the expanded help panel regardless of Help bar visibility setting.
- **FR-005**: System MUST apply configuration precedence in order: environment variable > config file > default (show), consistent with existing settings.
- **FR-006**: System MUST reclaim the screen space used by the Help bar (typically 3 lines when collapsed) for content display when the Help bar is hidden.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view 3 additional lines of markdown content when the Help bar is hidden (matching the space currently used by the collapsed Help bar).
- **SC-002**: Configuration takes effect immediately upon application launch without requiring additional user action.
- **SC-003**: The `?` key continues to display the full help panel regardless of Help bar visibility setting, ensuring users can always access help when needed.
- **SC-004**: 100% of existing mdt users experience no change in default behavior (Help bar remains visible unless explicitly configured otherwise).

## Assumptions

- The configuration option name `help_bar` follows the snake_case convention used in the existing config file.
- The environment variable `MDT_HELP_BAR` follows the existing pattern of `MDT_` prefix with underscore separator, handled automatically by the existing config system.
- Boolean parsing for config/env will accept standard truthy/falsy values (true/false, 1/0, yes/no).

## Out of Scope

- Runtime toggling of Help bar visibility (user explicitly stated this is not needed).
- Customizing the Help bar content or appearance.
- Partial hiding (e.g., making it smaller rather than fully hidden).
