# Adding Tree-Sitter Language Support to mdt

This guide covers everything you need to add syntax highlighting for a new
programming language.

## Architecture overview

Markdown code fences carry a language tag (` ```rust `). When mdt encounters
one, the pipeline is:

1. **Pest parser** (`src/md.pest`) captures the language name from the fence.
2. **`TextComponent::transform`** (`src/nodes/textcomponent.rs:298`) calls
   `transform_codeblock`, which passes the language name and raw bytes to
   `highlight_code`.
3. **`highlight_code`** (`src/highlight/mod.rs:94`) dispatches on the language
   string to a per-language module (e.g. `rust::highlight_rust`).
4. The per-language module creates a `tree_sitter_highlight::Highlighter`,
   feeds it the grammar's `LANGUAGE` and `HIGHLIGHTS_QUERY` constants, and
   returns a `Vec<HighlightEvent>`.
5. Back in `transform_codeblock` (`src/nodes/textcomponent.rs:419`), events
   are consumed: `HighlightStart(index)` sets the current color via
   `COLOR_MAP[index.0]`, `Source { start, end }` emits a `Word` with that
   color, and `HighlightEnd` resets to `Color::Reset`.
6. **Rendering** (`src/pages/markdown_renderer.rs`) turns each `Word` into a
   styled `ratatui::text::Span` for terminal output.

The key types:

| Type | Location | Role |
|------|----------|------|
| `HIGHLIGHT_NAMES` | `src/highlight/mod.rs:45` | 18 capture names tree-sitter maps syntax nodes to |
| `COLOR_MAP` | `src/highlight/mod.rs:66` | Parallel array mapping each capture name to a terminal color |
| `HighlightInfo` | `src/highlight/mod.rs:87` | `Highlighted(Vec<HighlightEvent>)` or `Unhighlighted` |
| `WordType::CodeBlock(Color)` | `src/nodes/word.rs` | Carries per-token color through to rendering |

## The four files you touch

Using Ruby as a running example:

### 1. `Cargo.toml` -- add the dependency and feature

```toml
# In [features], add to the tree-sitter list:
tree-sitter = [
  # ... existing entries ...
  "tree-sitter-ruby",
]

# In [dependencies], add:
tree-sitter-ruby = { version = "...", optional = true }
```

Every tree-sitter grammar is an **optional** dependency gated by a feature
flag so users can compile without languages they don't need.

### 2. `src/highlight/ruby.rs` -- create the language module

Every language module follows the same ~30-line template. The standard
("simple") form is:

```rust
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::highlight::HIGHLIGHT_NAMES;

pub fn highlight_ruby(lines: &[u8]) -> Result<Vec<HighlightEvent>, String> {
    let mut highlighter = Highlighter::new();
    let language = tree_sitter_ruby::LANGUAGE;

    let mut config = HighlightConfiguration::new(
        language.into(),
        "ruby",                             // language name string
        tree_sitter_ruby::HIGHLIGHTS_QUERY, // highlights query from the crate
        "",                                 // injection query (unused)
        "",                                 // locals query (unused)
    )
    .unwrap();

    config.configure(&HIGHLIGHT_NAMES);

    let highlights: Result<Vec<HighlightEvent>, String> =
        if let Ok(lines) = highlighter.highlight(&config, lines, None, |_| None) {
            lines
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())
        } else {
            Err("Failed to highlight".to_string())
        };

    highlights
}
```

#### Variations to watch for

Not every crate follows the exact same API surface. The three patterns seen
in the existing code:

| Pattern | Crate | Example |
|---------|-------|---------|
| **Simple** -- `LANGUAGE` + `HIGHLIGHTS_QUERY` | Most crates (rust, python, go, ...) | `tree_sitter_rust::LANGUAGE`, `tree_sitter_rust::HIGHLIGHTS_QUERY` |
| **Multi-language crate** -- named language constant | `tree-sitter-typescript`, `tree-sitter-ocaml`, `tree-sitter-php` | `tree_sitter_typescript::LANGUAGE_TYPESCRIPT`, `tree_sitter_typescript::LANGUAGE_TSX`, `tree_sitter_ocaml::LANGUAGE_OCAML_TYPE`, `tree_sitter_php::LANGUAGE_PHP` |
| **Singular vs plural query name** | Some older crates | `HIGHLIGHT_QUERY` (singular, e.g. bash, c, cpp, javascript) vs `HIGHLIGHTS_QUERY` (plural, e.g. rust, python, go) |

Before writing your module, check the crate's docs or source to find the
exact constant names. A quick way:

```
cargo doc -p tree-sitter-ruby --open
```

Or look at the crate's `lib.rs` on crates.io / GitHub.

### 3. `src/highlight/mod.rs` -- wire it in

Two additions:

```rust
// At the top, with the other module declarations:
#[cfg(feature = "tree-sitter-ruby")]
mod ruby;

// Inside highlight_code(), add a match arm:
#[cfg(feature = "tree-sitter-ruby")]
"ruby" | "rb" => HighlightInfo::Highlighted(ruby::highlight_ruby(lines).unwrap()),
```

The match patterns should cover the language names people commonly use in
markdown fences. For example, JavaScript matches `"javascript" | "js"`,
YAML matches `"yaml" | "yml"`, Bash matches `"bash" | "sh"`.

### 4. Verify it compiles and works

```sh
cargo build
```

Then view a markdown file containing a fenced code block with your language:

````markdown
```ruby
class Greeter
  def greet(name)
    puts "Hello, #{name}!"
  end
end
````

## Color mapping reference

`HIGHLIGHT_NAMES` and `COLOR_MAP` are parallel arrays. Tree-sitter queries
assign capture names like `@keyword`, `@function`, `@string` to syntax
nodes. The `config.configure(&HIGHLIGHT_NAMES)` call tells the highlighter
which captures to track and what index each gets. The index then looks up
into `COLOR_MAP`:

| Index | Capture name | Color |
|-------|-------------|-------|
| 0 | `attribute` | Yellow |
| 1 | `constant` | Yellow |
| 2 | `function.builtin` | Green |
| 3 | `function` | Green |
| 4 | `keyword` | Red |
| 5 | `operator` | Red |
| 6 | `property` | Blue |
| 7 | `punctuation` | Blue |
| 8 | `punctuation.bracket` | Blue |
| 9 | `punctuation.delimiter` | Blue |
| 10 | `string` | Magenta |
| 11 | `string.special` | Magenta |
| 12 | `tag` | Cyan |
| 13 | `type` | Cyan |
| 14 | `type.builtin` | Cyan |
| 15 | `variable` | Reset (default fg) |
| 16 | `variable.builtin` | Reset |
| 17 | `variable.parameter` | Reset |

You do **not** need to modify these when adding a language. The highlight
queries bundled inside each `tree-sitter-*` crate already use these standard
capture names. If a grammar's query uses a capture not listed here, it will
simply be ignored (unhighlighted).

## Currently supported languages (19)

| Fence names | Crate | Feature flag |
|-------------|-------|-------------|
| `bash`, `sh` | `tree-sitter-bash` | `tree-sitter-bash` |
| `c` | `tree-sitter-c` | `tree-sitter-c` |
| `cpp` | `tree-sitter-cpp` | `tree-sitter-cpp` |
| `css` | `tree-sitter-css` | `tree-sitter-css` |
| `elixir` | `tree-sitter-elixir` | `tree-sitter-elixir` |
| `go` | `tree-sitter-go` | `tree-sitter-go` |
| `html` | `tree-sitter-html` | `tree-sitter-html` |
| `java` | `tree-sitter-java` | `tree-sitter-java` |
| `javascript`, `js` | `tree-sitter-javascript` | `tree-sitter-javascript` |
| `json` | `tree-sitter-json` | `tree-sitter-json` |
| `lua` | `tree-sitter-lua` | `tree-sitter-lua` |
| `ocaml` | `tree-sitter-ocaml` | `tree-sitter-ocaml` |
| `php` | `tree-sitter-php` | `tree-sitter-php` |
| `python` | `tree-sitter-python` | `tree-sitter-python` |
| `rust` | `tree-sitter-rust` | `tree-sitter-rust` |
| `scala` | `tree-sitter-scala` | `tree-sitter-scala` |
| `tsx` | `tree-sitter-typescript` | `tree-sitter-typescript` |
| `typescript`, `ts` | `tree-sitter-typescript` | `tree-sitter-typescript` |
| `yaml`, `yml` | `tree-sitter-yaml` | `tree-sitter-yaml` |

Note: `tree-sitter-luau-fork` is listed in Cargo.toml features but has no
highlight module wired up yet.

## Known issue

`src/highlight/elixir.rs:12` uses `tree_sitter_cpp::HIGHLIGHT_QUERY` instead
of a query from the elixir crate. This is a copy-paste bug -- the Elixir
grammar is loaded correctly (`tree_sitter_elixir::LANGUAGE`), but the
highlight query comes from the wrong language.
