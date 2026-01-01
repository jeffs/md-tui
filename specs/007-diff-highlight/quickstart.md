# Quickstart: Diff Syntax Highlighting

This guide provides step-by-step implementation instructions for adding diff syntax highlighting to md-tui.

## Prerequisites

- Rust toolchain (rustc 1.92.0+)
- Existing md-tui codebase checked out

## Implementation Steps

### Step 1: Add Dependency to Cargo.toml

Add the tree-sitter-diff crate as an optional dependency:

```toml
# In [dependencies] section, after tree-sitter-yaml
tree-sitter-diff = { version = "0.1.0", optional = true }
```

Add the feature flag to the tree-sitter feature group:

```toml
# In [features] section, add to tree-sitter list
tree-sitter = [
  # ... existing entries ...
  "tree-sitter-diff"
]
```

### Step 2: Create src/highlight/diff.rs

Create a new file following the pattern from `rust.rs`:

```rust
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::highlight::HIGHLIGHT_NAMES;

pub fn highlight_diff(lines: &[u8]) -> Result<Vec<HighlightEvent>, String> {
    let mut highlighter = Highlighter::new();
    let language = tree_sitter_diff::LANGUAGE;

    let mut config = HighlightConfiguration::new(
        language.into(),
        "diff",
        tree_sitter_diff::HIGHLIGHTS_QUERY,
        "",
        "",
    )
    .unwrap();

    config.configure(&HIGHLIGHT_NAMES);

    highlighter
        .highlight(&config, lines, None, |_| None)
        .map_err(|_| "Failed to highlight".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
```

### Step 3: Register in src/highlight/mod.rs

Add the conditional module declaration (after yaml):

```rust
#[cfg(feature = "tree-sitter-diff")]
mod diff;
```

Add the match arm in `highlight_code()` function (after yaml):

```rust
#[cfg(feature = "tree-sitter-diff")]
"diff" | "patch" => HighlightInfo::Highlighted(diff::highlight_diff(lines).unwrap()),
```

## Verification

### Build Test

```bash
cargo build
```

### Run with Test File

Create a test markdown file with a diff block:

~~~markdown
```diff
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,4 @@
 context line
-removed line
+added line
 more context
```
~~~

Run:

```bash
cargo run -- test.md
```

### Test Without Feature

Verify graceful degradation:

```bash
cargo build --no-default-features --features network
cargo run --no-default-features --features network -- test.md
```

Diff blocks should render as plain text.

## Troubleshooting

### Colors Not Showing

If diff content renders but without colors, the HIGHLIGHTS_QUERY capture names may not map to the existing COLOR_MAP. Check what captures tree-sitter-diff uses and compare to HIGHLIGHT_NAMES in mod.rs.

### Compilation Errors

Ensure tree-sitter-diff version is compatible with tree-sitter-highlight 0.25.10. The 0.1.0 version uses the standard tree-sitter grammar format.
