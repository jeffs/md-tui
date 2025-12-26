# Implementation Plan: Hide Help Bar

**Branch**: `001-hide-help-bar` | **Date**: 2025-12-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-hide-help-bar/spec.md`

## Summary

Add a `help_bar` boolean configuration option to control visibility of the Help bar hint ("? - Help") at the bottom of the terminal. When hidden, the space is reclaimed for content display. The `?` key still functions to show the expanded help panel. Leverages existing `config` crate infrastructure for config file and environment variable support.

## Technical Context

**Language/Version**: Rust 2024 edition (rustc 1.92.0)
**Primary Dependencies**: ratatui 0.29.0, config 0.15.17, crossterm 0.29.0
**Storage**: Config file at `~/.config/mdt/config.toml` (existing pattern)
**Testing**: `cargo test` (no dedicated test directory currently exists)
**Target Platform**: Linux, macOS (aarch64 and x86_64)
**Project Type**: Single CLI application
**Performance Goals**: N/A (startup configuration, no runtime impact)
**Constraints**: Must preserve default behavior (Help bar visible)
**Scale/Scope**: Single boolean config option affecting UI layout

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution template has not been customized for this project. Proceeding with standard development practices:

- [x] **Simplicity**: Single boolean config option, minimal code change
- [x] **Backwards Compatibility**: Default behavior preserved (show Help bar)
- [x] **Consistency**: Follows existing config patterns (`GENERAL_CONFIG`, `MDT_` prefix)
- [x] **Testability**: Can be verified manually; config parsing uses existing infrastructure

No gate violations. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/001-hide-help-bar/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (minimal for this feature)
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── boxes/
│   └── help_box.rs      # HelpBox widget (rendering logic)
├── util/
│   └── general.rs       # GeneralConfig struct (add help_bar field)
├── main.rs              # Layout calculations (conditional Help bar rendering)
└── util.rs              # App struct (may need visibility flag)
```

**Structure Decision**: Existing single-project Rust CLI structure. Changes localized to 2-3 files following established patterns.

## Complexity Tracking

No violations to justify. This is a minimal feature addition using existing infrastructure.
