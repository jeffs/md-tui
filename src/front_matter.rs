//! Front matter: a metadata block at the very start of a document,
//! delimited by `---` lines for YAML or `+++` lines for TOML.
//!
//! The block is presented as a fenced code block in its own language.
//! The rewrite keeps every line in place, so source line numbers are
//! unchanged.

use std::borrow::Cow;

/// A metadata language, identified by its delimiter line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Yaml,
    Toml,
}

impl Language {
    fn from_delimiter(line: &str) -> Option<Self> {
        match line {
            "---" => Some(Self::Yaml),
            "+++" => Some(Self::Toml),
            _ => None,
        }
    }

    /// The code block info string, which selects syntax highlighting.
    fn name(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    /// Whether `line` ends a block that began with this language's
    /// delimiter. YAML also allows its document end marker, `...`.
    fn closes(self, line: &str) -> bool {
        Self::from_delimiter(line) == Some(self) || (self == Self::Yaml && line == "...")
    }
}

/// Front matter found at the start of a document.
struct FrontMatter<'a> {
    language: Language,
    /// The lines between the delimiters, each with its line ending.
    body: &'a str,
    /// Everything after the closing delimiter line's content, starting
    /// with that line's line ending (if any).
    rest: &'a str,
}

impl<'a> FrontMatter<'a> {
    fn find(content: &'a str) -> Option<Self> {
        let mut lines = content.split_inclusive('\n');
        let opening = lines.next()?;
        let language = Language::from_delimiter(trim_line_ending(opening))?;
        let body_start = opening.len();
        let mut offset = body_start;
        for line in lines {
            let text = trim_line_ending(line);
            if language.closes(text) {
                return Some(Self {
                    language,
                    body: &content[body_start..offset],
                    rest: &content[offset + text.len()..],
                });
            }
            offset += line.len();
        }
        None
    }

    /// The equivalent fenced code block, followed by the rest of the
    /// document. Returns `None` if the body cannot be fenced.
    fn as_code_block(&self) -> Option<String> {
        let fence = ["```", "~~~"]
            .into_iter()
            .find(|fence| !self.body.contains(fence))?;
        if self.body.is_empty() {
            // An empty code block does not parse; keep the delimiter
            // lines as blank lines instead.
            return Some(format!("\n{}", self.rest));
        }
        Some(format!(
            "{fence}{}\n{}{fence}{}",
            self.language.name(),
            self.body,
            self.rest
        ))
    }
}

fn trim_line_ending(line: &str) -> &str {
    line.trim_end_matches('\n').trim_end_matches('\r')
}

/// Returns `content` with any front matter rewritten as a fenced code
/// block.
#[must_use]
pub fn as_code_block(content: &str) -> Cow<'_, str> {
    FrontMatter::find(content)
        .and_then(|front_matter| front_matter.as_code_block())
        .map_or(Cow::Borrowed(content), Cow::Owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml() {
        assert_eq!(
            as_code_block("---\ntitle: Guide\n---\n# Guide\n"),
            "```yaml\ntitle: Guide\n```\n# Guide\n"
        );
    }

    #[test]
    fn yaml_closed_by_document_end_marker() {
        assert_eq!(
            as_code_block("---\ntitle: Guide\n...\nText\n"),
            "```yaml\ntitle: Guide\n```\nText\n"
        );
    }

    #[test]
    fn toml() {
        assert_eq!(
            as_code_block("+++\ntitle = \"Guide\"\n+++\nText"),
            "```toml\ntitle = \"Guide\"\n```\nText"
        );
    }

    #[test]
    fn crlf_line_endings() {
        assert_eq!(
            as_code_block("---\r\na: 1\r\n---\r\nText\r\n"),
            "```yaml\na: 1\r\n```\r\nText\r\n"
        );
    }

    #[test]
    fn closing_delimiter_at_end_of_file() {
        assert_eq!(as_code_block("---\na: 1\n---"), "```yaml\na: 1\n```");
    }

    #[test]
    fn empty_body_becomes_blank_lines() {
        assert_eq!(as_code_block("---\n---\nText\n"), "\n\nText\n");
    }

    #[test]
    fn body_containing_both_fences_is_unchanged() {
        let content = "---\na: ```\nb: ~~~\n---\nText\n";
        assert_eq!(as_code_block(content), content);
    }

    #[test]
    fn body_containing_backticks_uses_tildes() {
        assert_eq!(
            as_code_block("---\na: ```\n---\n"),
            "~~~yaml\na: ```\n~~~\n"
        );
    }

    #[test]
    fn rule_after_first_line_is_not_front_matter() {
        let content = "Text\n\n---\na: 1\n---\n";
        assert_eq!(as_code_block(content), content);
    }

    #[test]
    fn unclosed_is_not_front_matter() {
        let content = "---\nText\n";
        assert_eq!(as_code_block(content), content);
    }

    #[test]
    fn mismatched_delimiters_are_not_front_matter() {
        let content = "---\na: 1\n+++\n";
        assert_eq!(as_code_block(content), content);
    }
}
