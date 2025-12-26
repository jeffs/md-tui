# Data Model: Hide Help Bar

**Feature**: 001-hide-help-bar
**Date**: 2025-12-26

## Entities

### GeneralConfig (modified)

The existing `GeneralConfig` struct gains one new field.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| width | u16 | 100 | Terminal width for content |
| gitignore | bool | false | Respect gitignore in file tree |
| centering | Centering | Left | Content alignment |
| **help_bar** | **bool** | **true** | **Show Help bar hint ("? - Help")** |

### Configuration Sources (existing pattern)

```
Priority (highest to lowest):
1. Environment variable: MDT_HELP_BAR=false
2. Config file: ~/.config/mdt/config.toml → help_bar = false
3. Default: true (show Help bar)
```

## State Transitions

N/A - This is a static configuration read at startup. No runtime state changes.

## Validation Rules

| Rule | Behavior |
|------|----------|
| Invalid boolean value | Use default (true) |
| Missing config | Use default (true) |
| Env var override | Env var wins over config file |

## Relationships

```
GeneralConfig (static, lazy_static)
    └── help_bar: bool
            └── Used by: render_file_tree(), render_markdown() in main.rs
                    └── Controls: collapsed Help bar visibility
                    └── Does NOT affect: expanded Help panel (via ? key)
```
