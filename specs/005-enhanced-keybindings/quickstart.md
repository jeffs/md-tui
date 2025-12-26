# Quickstart: Enhanced Keybindings

**Feature**: 005-enhanced-keybindings

## Test Configuration

Create or edit `~/.config/mdt/config.toml`:

```toml
# Multiple keys for page down (d or space)
page_down = ["d", " "]

# Emacs/Helix-style ctrl scrolling
down = ["j", "C-e"]
up = ["k", "C-y"]

# Half page with C-d/C-u
half_page_down = ["l", "C-d"]
half_page_up = ["h", "C-u"]
```

## Verification Steps

### Test 1: Multiple Keys (P1)

1. Run `mdt README.md`
2. Press `d` - view should scroll down one page
3. Press space - view should scroll down one page (same behavior)
4. **Expected**: Both keys trigger PageDown action

### Test 2: Control Modifiers (P2)

1. Run `mdt README.md`
2. Press `j` - view should scroll down one line
3. Press `C-e` (Ctrl+e) - view should scroll down one line (same behavior)
4. Press `C-y` (Ctrl+y) - view should scroll up one line
5. **Expected**: Control-modified keys work alongside regular keys

### Test 3: Backwards Compatibility (P1)

1. Create minimal config with old syntax:
   ```toml
   up = "k"
   down = "j"
   ```
2. Run `mdt README.md`
3. Press `k` and `j`
4. **Expected**: Navigation works exactly as before

### Test 4: Mixed Syntax

1. Create config mixing old and new syntax:
   ```toml
   up = "k"
   page_down = ["d", "space", "C-d"]
   ```
2. Run `mdt README.md`
3. Test all configured keys
4. **Expected**: Both syntaxes work in same file

### Test 5: Error Handling

1. Create config with invalid binding:
   ```toml
   up = "C-invalid-key"
   ```
2. Run `mdt README.md`
3. Press `k`
4. **Expected**: Default binding (k) works, app doesn't crash

## Reverting

Remove custom keybindings from config file to restore defaults:

```bash
# Edit config
vim ~/.config/mdt/config.toml

# Or remove entirely
rm ~/.config/mdt/config.toml
```

## Common Issues

**Issue**: Ctrl+key not responding
- Check terminal captures the key (some terminals intercept Ctrl sequences)
- Verify config syntax: `C-e` not `c+e` or `ctrl+e`

**Issue**: Space key not working
- Use either `" "` (space in quotes) or `"space"` (named)
- Ensure it's inside the array: `["d", " "]`

**Issue**: Config changes not taking effect
- Restart mdt (config is loaded at startup)
- Check config path: `~/.config/mdt/config.toml`
