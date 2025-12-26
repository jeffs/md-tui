# Research: Enhanced Keybindings

**Feature**: 005-enhanced-keybindings
**Date**: 2025-12-26

## Crossterm KeyEvent and KeyModifiers

**Decision**: Use crossterm's `KeyEvent` struct which contains both `code: KeyCode` and `modifiers: KeyModifiers`.

**Rationale**: crossterm is already a dependency (v0.29.0) and provides native support for modifier key detection. The `KeyModifiers` bitflags include `CONTROL`, `SHIFT`, `ALT`, etc.

**Alternatives considered**:
- Manual terminal escape sequence parsing: Rejected (reinventing the wheel, crossterm handles this)
- termion crate: Rejected (would add a new dependency when crossterm already works)

## Config Crate Value Parsing

**Decision**: Use `config::Value` enum to detect string vs array, then parse accordingly.

**Rationale**: The config crate supports polymorphic values. We can use `settings.get::<config::Value>("key_name")` and match on the result type.

**Implementation pattern**:
```rust
match settings.get::<config::Value>("up") {
    Ok(config::Value::new(None, config::ValueKind::String(s))) => parse_single(&s),
    Ok(config::Value::new(None, config::ValueKind::Array(arr))) => parse_array(&arr),
    _ => default_bindings(),
}
```

**Alternatives considered**:
- Custom serde deserializer: More complex, not necessary for this use case
- Try string first, then array: Less elegant, would require error handling gymnastics

## Key String Parsing

**Decision**: Parse key strings with format `[C-]key` where:
- Modifier prefix: `C-` (case-insensitive)
- Key values: single character, or named keys (`space`, `tab`, `enter`, `esc`)

**Rationale**: The `C-` notation is consistent with Emacs and Helix conventions, making it familiar to users of those editors.

**Named key mappings**:
| Config String | KeyCode |
|---------------|---------|
| `space` or ` ` | `KeyCode::Char(' ')` |
| `tab` | `KeyCode::Tab` |
| `enter` | `KeyCode::Enter` |
| `esc` | `KeyCode::Esc` |
| `backspace` | `KeyCode::Backspace` |
| Single char | `KeyCode::Char(c)` |

**Alternatives considered**:
- `ctrl+key` / `control+key`: More verbose, less standard
- XML-style `<C-e>`: Requires angle brackets, vim-specific
- Separate modifier field in TOML: More verbose, harder to read

## Reserved Terminal Sequences

**Decision**: Do not attempt to handle Ctrl+c or Ctrl+z in the application - let the terminal process them.

**Rationale**: These are typically handled by the terminal/OS before reaching the application:
- Ctrl+c → SIGINT (process termination)
- Ctrl+z → SIGTSTP (process suspend)

The raw mode used by crossterm does intercept these, but overriding them would violate user expectations for terminal applications.

**Implementation**: Simply don't bind these by default, and if a user configures them, they'll work but may have unexpected side effects depending on their terminal settings.

## Backwards Compatibility Strategy

**Decision**: Parse single string values as single-element binding lists.

**Rationale**: Treating `up = "k"` as equivalent to `up = ["k"]` internally simplifies the action matching logic. All config handling uses `Vec<KeyBinding>`, whether from string or array input.

**Migration path**: None required. Existing configs work unchanged.
