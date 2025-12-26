# Data Model: Enhanced Keybindings

**Feature**: 005-enhanced-keybindings
**Date**: 2025-12-26

## Entities

### KeyBinding

Represents a single key combination (key + optional modifiers).

| Field | Type | Description |
|-------|------|-------------|
| `key` | `KeyCode` | The key (from crossterm) |
| `modifiers` | `KeyModifiers` | Modifier flags (CONTROL, SHIFT, ALT, etc.) |

**Validation rules**:
- `key` must be a valid KeyCode variant
- `modifiers` defaults to `KeyModifiers::NONE` for plain keys
- For Ctrl-modified keys, `modifiers` includes `KeyModifiers::CONTROL`

**Equality**: Two KeyBindings match if both `key` and `modifiers` are equal.

### KeyConfig

Stores all configurable keybindings. Each field is now a `Vec<KeyBinding>` instead of a single `char`.

| Field | Type | Default |
|-------|------|---------|
| `up` | `Vec<KeyBinding>` | `['k']` |
| `down` | `Vec<KeyBinding>` | `['j']` |
| `page_up` | `Vec<KeyBinding>` | `['u']` |
| `page_down` | `Vec<KeyBinding>` | `['d']` |
| `half_page_up` | `Vec<KeyBinding>` | `['h']` |
| `half_page_down` | `Vec<KeyBinding>` | `['l']` |
| `search` | `Vec<KeyBinding>` | `['f']` |
| `search_next` | `Vec<KeyBinding>` | `['n']` |
| `search_previous` | `Vec<KeyBinding>` | `['N']` |
| `select_link` | `Vec<KeyBinding>` | `['s']` |
| `select_link_alt` | `Vec<KeyBinding>` | `['S']` |
| `edit` | `Vec<KeyBinding>` | `['e']` |
| `hover` | `Vec<KeyBinding>` | `['K']` |
| `top` | `Vec<KeyBinding>` | `['g']` |
| `bottom` | `Vec<KeyBinding>` | `['G']` |
| `back` | `Vec<KeyBinding>` | `['b']` |
| `file_tree` | `Vec<KeyBinding>` | `['t']` |
| `sort` | `Vec<KeyBinding>` | `['o']` |

### Action (existing enum, unchanged)

```rust
pub enum Action {
    Up, Down, PageUp, PageDown, HalfPageUp, HalfPageDown,
    Search, SelectLink, SelectLinkAlt, SearchNext, SearchPrevious,
    Edit, Hover, Enter, Escape, ToTop, ToBottom, Help, Back,
    ToFileTree, Sort, None,
}
```

## Config File Format

### TOML Syntax

```toml
# Single key (backwards compatible)
up = "k"

# Multiple keys (new array syntax)
page_down = ["d", " "]

# Control modifier (new)
down = ["j", "C-e"]
up = ["k", "C-y"]

# Named keys
page_down = ["d", "space"]

# Mixed
half_page_down = ["l", "C-d", " "]
```

### Parsing Rules

1. **String value** → Parse as single KeyBinding, wrap in Vec
2. **Array value** → Parse each element as KeyBinding, collect into Vec
3. **Invalid value** → Use default bindings for that action
4. **Empty array** → Use default bindings for that action

### Key String Format

```
[C-]key
```

Where:
- `C-` (optional): Control modifier prefix (case-insensitive)
- `key`: single character or named key

**Named keys**: `space`, `tab`, `enter`, `esc`, `backspace`

## State Transitions

N/A - KeyConfig is loaded once at startup and is immutable.

## Relationships

```
KeyConfig --contains--> Vec<KeyBinding> (one per action)
KeyBinding --wraps--> (KeyCode, KeyModifiers)
key_to_action() --reads--> KeyConfig
key_to_action() --receives--> KeyEvent
key_to_action() --returns--> Action
```
