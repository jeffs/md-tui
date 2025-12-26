# Quickstart: Hide Help Bar

**Feature**: 001-hide-help-bar

## Configuration

### Option 1: Config File (Persistent)

Create or edit `~/.config/mdt/config.toml`:

```toml
help_bar = false
```

### Option 2: Environment Variable (Session/System)

```bash
# Single session
export MDT_HELP_BAR=false
mdt README.md

# Or inline
MDT_HELP_BAR=false mdt README.md
```

## Verification

1. Run `mdt` with a markdown file
2. The "? - Help" bar at the bottom should NOT appear
3. Press `?` - the expanded help panel should still appear
4. Content area should use the full terminal height (minus standard margins)

## Reverting

Remove the `help_bar = false` line from config, or unset the environment variable:

```bash
unset MDT_HELP_BAR
```

Or explicitly set to true:

```toml
help_bar = true
```
