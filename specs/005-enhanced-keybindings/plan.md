# Implementation Plan: Enhanced Keybindings

**Branch**: `005-enhanced-keybindings` | **Date**: 2025-12-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-enhanced-keybindings/spec.md`

## Summary

Enhance the keybinding system to support (1) multiple keys per action via array syntax and (2) control key modifiers via `ctrl+key` syntax. The implementation requires refactoring `KeyConfig` from single `char` fields to `Vec<KeyBinding>`, updating config parsing to handle both string and array values, and modifying `key_to_action()` to accept `KeyEvent` (with modifiers) instead of `KeyCode`.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: crossterm 0.29.0 (KeyEvent, KeyModifiers), config 0.15.17 (TOML parsing), lazy_static
**Storage**: TOML config file (~/.config/mdt/config.toml)
**Testing**: cargo test (unit tests in src/util/keys.rs)
**Target Platform**: Cross-platform terminal (macOS, Linux, Windows)
**Project Type**: Single CLI application
**Performance Goals**: Key response <1ms (no perceptible delay)
**Constraints**: Backwards compatible with existing single-char config format
**Scale/Scope**: 18 configurable actions, typically 1-3 bindings per action

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

No project constitution defined. Proceeding with standard best practices:
- [x] Backwards compatibility preserved
- [x] No breaking changes to public API
- [x] Tests cover new functionality
- [x] Error handling graceful (fallback to defaults)

## Project Structure

### Documentation (this feature)

```text
specs/005-enhanced-keybindings/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── util/
│   └── keys.rs          # PRIMARY: KeyBinding, KeyConfig, key_to_action()
├── event_handler.rs     # Update to pass KeyEvent instead of KeyCode
└── main.rs              # Update event loop to pass full KeyEvent
```

**Structure Decision**: Existing single-project structure. Changes isolated to `src/util/keys.rs` with minor updates to `event_handler.rs` and `main.rs` for KeyEvent propagation.

## Complexity Tracking

No constitution violations to justify.

## Design Decisions

### Key Binding Representation

A `KeyBinding` struct represents a single key combination:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}
```

### Config Parsing Strategy

Use the `config` crate's `Value` type to handle polymorphic input:
- String value → parse as single KeyBinding
- Array value → parse as Vec<KeyBinding>

Parsing logic for key strings:
1. Check for modifier prefix: `ctrl+`, `ctrl-`, `control+`, `control-`
2. Parse the key portion: single char, or named key (`space`, `tab`, etc.)
3. Construct KeyBinding with appropriate KeyCode and KeyModifiers

### Action Matching

Change `key_to_action(key: KeyCode)` to `key_to_action(event: &KeyEvent)`:
- Compare both `event.code` and `event.modifiers` against stored bindings
- First matching binding wins (preserves existing behavior)

### Reserved Keys

Skip binding check for terminal-reserved sequences:
- Ctrl+c (SIGINT)
- Ctrl+z (SIGTSTP)
- These are handled by the terminal before reaching the application

## Files to Modify

| File | Changes |
|------|---------|
| `src/util/keys.rs` | New KeyBinding struct, update KeyConfig fields, new parsing logic, update key_to_action signature |
| `src/event_handler.rs` | Update handle_keyboard_input to accept KeyEvent, pass event to key_to_action |
| `src/main.rs` | Pass full KeyEvent instead of key.code |
| `README.md` | Document new config syntax |

## Implementation Approach

1. **Phase 1: KeyBinding Type** - Create the new struct and parsing functions
2. **Phase 2: KeyConfig Refactor** - Change fields from char to Vec<KeyBinding>
3. **Phase 3: Config Parsing** - Handle string/array polymorphism in lazy_static block
4. **Phase 4: Event Flow** - Update call chain to pass KeyEvent
5. **Phase 5: Action Matching** - Update key_to_action to check modifiers
6. **Phase 6: Documentation** - Update README with new syntax examples
