# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Fork Context

This is a fork of henriklovhaug/md-tui. Read `memory/upstream-strategy.md` for critical context on:
- Dual-track development (pedantic locally, non-pedantic for upstream PRs)
- How to create PRs without conflicts
- How to merge upstream changes
- Why cherry-picking from main to PR branches doesn't work

## Project Overview

MD-TUI is a terminal user interface (TUI) markdown viewer written in Rust. The binary is named `mdt`. It provides rich formatting, syntax highlighting via tree-sitter, image rendering, and interactive navigation.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (with LTO)
cargo run -- <markdown-file>   # Run with a file
cargo run --                   # Read from stdin
cargo test                     # Run all tests
cargo test -- --nocapture      # Run tests with output
```

### Feature Flags

- `tree-sitter` (default): Syntax highlighting for 18 languages
- `network` (default): Remote image loading via ureq

```bash
cargo run --no-default-features --features network    # Without syntax highlighting
cargo run --no-default-features --features tree-sitter # Without network
```

## Architecture

### Core Data Flow

1. **Entry (main.rs)**: Initializes ratatui terminal, spawns file discovery thread, runs event loop
2. **Parsing (parser.rs + md.pest)**: Pest PEG parser converts markdown → `ParseNode` tree → `Component` tree
3. **Rendering (nodes/)**: Components implement ratatui's `Widget` trait, rendered with scroll/clip calculations
4. **Events (event_handler.rs)**: Mode-aware keyboard handling (FileTree mode vs View mode)

### Key Modules

- `parser.rs` + `md.pest`: Markdown parsing using Pest PEG grammar
- `nodes/root.rs`: `ComponentRoot` - top-level document container
- `nodes/textcomponent.rs`: Main text rendering (paragraphs, tables, code blocks, etc.)
- `nodes/word.rs`: Individual styled text units with metadata
- `pages/file_explorer.rs`: File tree browser UI
- `event_handler.rs`: All keyboard input handling
- `highlight/`: Tree-sitter syntax highlighting (one file per language)
- `util/colors.rs` + `util/general.rs`: Configuration loading

### Key Types

```rust
enum Component { TextComponent(TextComponent), Image(ImageComponent) }
enum TextNode { Paragraph, Heading, Table, CodeBlock, Quote, Task, List, ... }
struct App { vertical_scroll, mode, boxes, history, ... }  // Central state
```

### Configuration

Config file: `~/.config/mdt/config.toml`
Environment variables: `MDT_` prefix (e.g., `MDT_WIDTH=120`)

## Development Patterns

### Adding Syntax Highlighting

1. Create `src/highlight/<language>.rs` using tree-sitter
2. Add conditional module in `src/highlight/mod.rs`
3. Add language routing in `highlight_code()` function
4. Add tree-sitter crate with optional feature flag in Cargo.toml

### Modifying Markdown Grammar

Edit `src/md.pest` PEG grammar, then test with `cargo test` and real markdown files.

## Code Style

This fork uses `clippy::pedantic` lint level. Upstream does not. See `memory/upstream-strategy.md` for how to handle this when contributing upstream.

### Path Qualifications

Use the shortest path that compiles. If a type is already imported, use the short name:

```rust
// Good - Word is already imported
.filter(Word::is_renderable)

// Bad - unnecessary qualification
.filter(super::word::Word::is_renderable)
```

### Rustdoc Headers

In rustdoc Markdown, any header (such as `# Panics`) must be followed by a blank `///` line:

```rust
/// # Panics
///
/// Panics if the lock is poisoned.
```

Not:

```rust
/// # Panics
/// Panics if the lock is poisoned.
```

## Notes

- All components use ratatui's `Widget` trait
- Lazy cloning optimization - components only cloned when visible
- File watching via `notify` crate for live reload
- Panic hook ensures terminal restoration on crash
- Character boundary safety for non-ASCII text (Chinese, etc.)

## Active Technologies
- Rust 2024 edition (rustc 1.92.0) + ratatui 0.29.0, config 0.15.17, crossterm 0.29.0 (001-hide-help-bar)
- Config file at `~/.config/mdt/config.toml` (existing pattern) (001-hide-help-bar)
- Rust 2024 edition + crossterm 0.29.0 (KeyEvent, KeyModifiers), config 0.15.17 (TOML parsing), lazy_static (005-enhanced-keybindings)
- TOML config file (~/.config/mdt/config.toml) (005-enhanced-keybindings)
- Rust 1.92.0 (2024 edition) + ratatui 0.29.0, crossterm 0.29.0, pest (parser), tree-sitter (syntax highlighting) (006-fix-pedantic-lint)
- N/A (file-based markdown viewer) (006-fix-pedantic-lint)

## Recent Changes
- 001-hide-help-bar: Added Rust 2024 edition (rustc 1.92.0) + ratatui 0.29.0, config 0.15.17, crossterm 0.29.0
