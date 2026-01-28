# Adding Syntax Highlighting for New Languages

This guide explains how to add tree-sitter based syntax highlighting for new
languages to md-tui.

## Prerequisites

You need a tree-sitter grammar crate that:

1. Is compatible with tree-sitter 0.25+ (check md-tui's `Cargo.lock` for exact
   version)
2. Exports a `LANGUAGE` constant (preferred) or `language()` function
3. Has a highlights query (either exported as `HIGHLIGHTS_QUERY` or available
   as `queries/highlights.scm` in the repo)

## Finding a Grammar

Search crates.io or GitHub for `tree-sitter-<language>`. Check:

- **Tree-sitter version**: Must be compatible with md-tui's version (currently
  0.25.x). Older grammars (0.19-0.21) won't work.
- **HIGHLIGHTS_QUERY**: Check docs.rs or the crate's `lib.rs` for this export.
  If commented out, you'll need to embed the query manually.
- **License**: Must be compatible with AGPL-3.0-or-later. MIT, Apache-2.0, and
  BSD licenses are compatible.

### If No Compatible Crate Exists

You can use a git dependency if a GitHub repo has modern tree-sitter support
but no published crate. Example from Cargo.toml:

```toml
tree-sitter-proto = { git = "https://github.com/user/tree-sitter-proto", optional = true }
```

## Implementation Steps

### 1. Add the Dependency to Cargo.toml

Add to `[dependencies]` (keep alphabetical order):

```toml
tree-sitter-foo = { version = "0.x.y", optional = true }
```

Add to `[features]` under `tree-sitter = [...]`:

```toml
tree-sitter = [
  # ... existing languages ...
  "tree-sitter-foo",
  # ... more languages ...
]
```

### 2. Create the Highlighter Module

Create `src/highlight/foo.rs`:

```rust
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::highlight::HIGHLIGHT_NAMES;

pub fn highlight_foo(lines: &[u8]) -> Result<Vec<HighlightEvent>, String> {
    let mut highlighter = Highlighter::new();
    let language = tree_sitter_foo::LANGUAGE;

    let mut config = HighlightConfiguration::new(
        language.into(),
        "foo",
        tree_sitter_foo::HIGHLIGHTS_QUERY,
        "",
        "",
    )
    .unwrap();

    config.configure(&HIGHLIGHT_NAMES);

    if let Ok(lines) = highlighter.highlight(&config, lines, None, |_| None) {
        lines
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    } else {
        Err("Failed to highlight".to_string())
    }
}
```

### 3. Embedding a Highlights Query

If the crate doesn't export `HIGHLIGHTS_QUERY`, embed it as a const:

```rust
// Embedded from https://github.com/user/tree-sitter-foo/blob/main/queries/highlights.scm
// MIT License - Copyright (c) YEAR Author Name
const HIGHLIGHTS_QUERY: &str = r#"
(keyword) @keyword
(string) @string
(comment) @property
"#;
```

**Important**: Include proper attribution with URL, license, and copyright
holder.

### 4. Map Query Captures to HIGHLIGHT_NAMES

The highlights query uses captures like `@keyword`, `@string`, etc. These must
match entries in `HIGHLIGHT_NAMES` (defined in `mod.rs`):

```rust
static HIGHLIGHT_NAMES: [&str; 18] = [
    "attribute",           // index 0  -> Yellow
    "constant",            // index 1  -> Yellow
    "function.builtin",    // index 2  -> Green
    "function",            // index 3  -> Green
    "keyword",             // index 4  -> Red
    "operator",            // index 5  -> Red
    "property",            // index 6  -> Blue
    "punctuation",         // index 7  -> Blue
    "punctuation.bracket", // index 8  -> Blue
    "punctuation.delimiter", // index 9 -> Blue
    "string",              // index 10 -> Magenta
    "string.special",      // index 11 -> Magenta
    "tag",                 // index 12 -> Cyan
    "type",                // index 13 -> Cyan
    "type.builtin",        // index 14 -> Cyan
    "variable",            // index 15 -> Reset
    "variable.builtin",    // index 16 -> Reset
    "variable.parameter",  // index 17 -> Reset
];
```

If the query uses captures not in this list (e.g., `@comment`, `@number`), map
them to existing names:

| Original Capture   | Substitute With |
|--------------------|-----------------|
| `@comment`         | `@property`     |
| `@number`          | `@constant`     |
| `@constant.builtin`| `@constant`     |
| `@boolean`         | `@constant`     |

### 5. Register the Module in mod.rs

Add the conditional module declaration (keep alphabetical):

```rust
#[cfg(feature = "tree-sitter-foo")]
mod foo;
```

Add the router arm in `highlight_code()`:

```rust
#[cfg(feature = "tree-sitter-foo")]
"foo" | "foolang" => {
    HighlightInfo::Highlighted(foo::highlight_foo(lines).unwrap())
}
```

Include common aliases (e.g., `"js"` for JavaScript, `"yml"` for YAML).

### 6. Build and Test

```bash
cargo build
cargo test
```

Test with a markdown file containing a fenced code block:

````markdown
```foo
your code here
```
````

## Checklist

- [ ] Cargo.toml: Added optional dependency
- [ ] Cargo.toml: Added feature flag to `tree-sitter` list
- [ ] Created `src/highlight/<lang>.rs`
- [ ] Proper copyright attribution for any embedded queries
- [ ] mod.rs: Added `#[cfg(feature = "...")]` module declaration
- [ ] mod.rs: Added router arm with language aliases
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] Manual test with sample code block

## Example: Protobuf

See `src/highlight/protobuf.rs` for a complete example that:

- Uses a git dependency (no crates.io release)
- Embeds a highlights query with proper attribution
- Maps captures to available `HIGHLIGHT_NAMES`
