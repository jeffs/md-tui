# Research: Hide Help Bar

**Feature**: 001-hide-help-bar
**Date**: 2025-12-26

## Configuration Pattern Analysis

### Decision: Use existing `GeneralConfig` with `help_bar` boolean field

**Rationale**: The existing `GeneralConfig` struct in `src/util/general.rs` already handles:
- Config file loading from `~/.config/mdt/config.toml`
- Environment variable override via `MDT_` prefix
- Default values via `unwrap_or()`

Adding `help_bar: bool` follows the exact pattern used for `width`, `gitignore`, and `centering`.

**Alternatives considered**:
- Separate config struct: Rejected - unnecessary complexity
- Runtime-only flag: Rejected - doesn't meet persistent config requirement

### Existing Config Pattern

```rust
GeneralConfig {
    width: settings.get::<u16>("width").unwrap_or(100),
    gitignore: settings.get::<bool>("gitignore").unwrap_or(false),
    centering: settings.get::<Centering>("alignment").unwrap_or(Centering::Left),
}
```

**New field pattern**:
```rust
help_bar: settings.get::<bool>("help_bar").unwrap_or(true),
```

Default is `true` to preserve existing behavior (Help bar visible).

## Help Bar Rendering Analysis

### Decision: Conditional rendering in `main.rs` based on config

**Rationale**: The Help bar is rendered in two functions:
1. `render_file_tree()` (lines 259-293) - FileTree mode
2. `render_markdown()` (lines 365-404) - View mode

Both follow the same pattern:
1. Calculate Help bar area (3 lines when collapsed, larger when expanded)
2. Render `Clear` widget to create background
3. Render `app.help_box` widget

When `help_bar = false`:
- Skip rendering the collapsed "? - Help" hint
- Skip the `Clear` background in the Help bar area
- Reclaim the 3-5 lines for content area
- The `?` key should still toggle the expanded help panel (this works via `app.help_box.toggle()` in event_handler.rs, unchanged)

**Key layout values to modify**:
- `height: size.height - 5` → `height: size.height - 2` (or similar) when hidden
- Skip `Clear` and `help_box` rendering for collapsed state

### Decision: Pass visibility flag to rendering, not to HelpBox widget

**Rationale**: The HelpBox widget itself doesn't need to know about visibility config. The visibility check happens at the render call site in main.rs. The HelpBox still handles expanded/collapsed state for when the user presses `?`.

**Alternatives considered**:
- Modify HelpBox to have visibility flag: Rejected - HelpBox handles expand/collapse, not visibility
- New "hidden" state in HelpBox: Rejected - overcomplicates the simple on/off visibility

## Implementation Approach

### Files to modify:

1. **`src/util/general.rs`**: Add `help_bar: bool` field to `GeneralConfig`
2. **`src/main.rs`**: Conditional Help bar rendering in both render functions

### No changes needed to:
- `src/boxes/help_box.rs` - Widget logic unchanged
- `src/event_handler.rs` - `?` key handling unchanged
- `src/util.rs` - App struct unchanged

## Risk Analysis

| Risk | Mitigation |
|------|------------|
| Breaking existing users | Default `true` preserves behavior |
| Expanded help panel breaks when hidden | Toggle still works; only collapsed hint is hidden |
| Layout calculation errors | Adjust content height conditionally |
