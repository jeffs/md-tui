use image::ImageReader;
use itertools::Itertools;
use pest::{
    Parser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use ratatui::style::Color;

use crate::nodes::{
    image::ImageComponent,
    root::{Component, ComponentRoot},
    textcomponent::{TextComponent, TextNode},
    word::{MetaData, Word, WordType},
};
use crate::util::general::{Flavor, GENERAL_CONFIG};

#[derive(Parser)]
#[grammar = "md.pest"]
pub struct MdParser;

/// # Panics
///
/// Panics if the parsed markdown text has no root element.
#[must_use]
pub fn parse_markdown(name: Option<&str>, content: &str, width: u16) -> ComponentRoot {
    let root: Pairs<'_, Rule> = if let Ok(file) = MdParser::parse(Rule::txt, content) {
        file
    } else {
        return ComponentRoot::new(name.map(str::to_string), Vec::new());
    };

    let root_pair = root.into_iter().next().unwrap();

    let children: Vec<ParseNode> = parse_text(root_pair)
        .children_owned()
        .into_iter()
        .dedup()
        .collect();

    let parse_root = ParseRoot::new(name.map(str::to_string), children);

    let mut root = node_to_component(parse_root).add_missing_components();

    root.transform(width);
    root
}

fn parse_text(pair: Pair<'_, Rule>) -> ParseNode {
    let rule = pair.as_rule();
    let raw = pair.as_str();
    let content = if rule == Rule::code_line {
        raw.replace('\t', "    ").replace('\r', "")
    } else if GENERAL_CONFIG.flavor == Flavor::Claude {
        // Claude flavor: preserve newlines as hard line breaks
        raw.replace('\r', "")
    } else {
        // CommonMark: collapse newlines to spaces for text reflow
        raw.replace('\n', " ")
    };
    let mut component = ParseNode::new(rule.into(), content);
    let children = parse_node_children(pair.into_inner());
    component.add_children(children);
    component
}

fn parse_node_children(pair: Pairs<'_, Rule>) -> Vec<ParseNode> {
    let mut children = Vec::new();
    for inner_pair in pair {
        children.push(parse_text(inner_pair));
    }
    children
}

fn node_to_component(root: ParseRoot) -> ComponentRoot {
    let mut children = Vec::new();
    let name = root.file_name().clone();
    for component in root.children_owned() {
        let comp = parse_component(component);
        children.push(comp);
    }

    ComponentRoot::new(name, children)
}

fn is_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

#[expect(
    clippy::too_many_lines,
    reason = "match must handle all Rule variants in one place for exhaustiveness checking"
)]
fn parse_component(parse_node: ParseNode) -> Component {
    match parse_node.kind() {
        MdParseEnum::Image => {
            let leaf_nodes = get_leaf_nodes(parse_node);
            let mut alt_text = String::new();
            let mut image = None;
            for node in leaf_nodes {
                if node.kind() == MdParseEnum::AltText {
                    node.content().clone_into(&mut alt_text);
                } else if is_url(node.content()) {
                    #[cfg(feature = "network")]
                    {
                        let mut buf = Vec::new();
                        image = ureq::get(node.content()).call().ok().and_then(|b| {
                            let noe = b.into_body().read_to_vec();
                            noe.ok().and_then(|b| {
                                buf = b;
                                image::load_from_memory(&buf).ok()
                            })
                        });
                    }
                    #[cfg(not(feature = "network"))]
                    {
                        image = None;
                    }
                } else {
                    image = ImageReader::open(node.content())
                        .ok()
                        .and_then(|r| r.decode().ok());
                }
            }

            if let Some(img) = image.as_ref() {
                let height = img.height();

                let comp = ImageComponent::new(img.to_owned(), height, &alt_text);

                if let Some(comp) = comp {
                    Component::Image(comp)
                } else {
                    let word = [Word::new(format!("[{alt_text}]"), WordType::Normal)];

                    let comp = TextComponent::new(TextNode::Paragraph, word.into());
                    Component::TextComponent(comp)
                }
            } else {
                let word = [
                    Word::new("Image".to_string(), WordType::Normal),
                    Word::new(" ".to_owned(), WordType::Normal),
                    Word::new("not".to_owned(), WordType::Normal),
                    Word::new(" ".to_owned(), WordType::Normal),
                    Word::new("found".to_owned(), WordType::Normal),
                    Word::new("/".to_owned(), WordType::Normal),
                    Word::new("fetched".to_owned(), WordType::Normal),
                    Word::new(" ".to_owned(), WordType::Normal),
                    Word::new(format!("[{alt_text}]"), WordType::Normal),
                ];

                let comp = TextComponent::new(TextNode::Paragraph, word.into());
                Component::TextComponent(comp)
            }
        }

        MdParseEnum::Task => {
            let leaf_nodes = get_leaf_nodes(parse_node);
            let mut words = Vec::new();
            for node in leaf_nodes {
                let word_type = WordType::from(node.kind());

                let mut content: String = node
                    .content()
                    .chars()
                    .dedup_by(|x, y| *x == ' ' && *y == ' ')
                    .collect();

                if matches!(node.kind(), MdParseEnum::WikiLink | MdParseEnum::InlineLink) {
                    let comp = Word::new(content.clone(), WordType::LinkData);
                    words.push(comp);
                }

                if content.starts_with(' ') {
                    content.remove(0);
                    let comp = Word::new(" ".to_owned(), word_type);
                    words.push(comp);
                }
                words.push(Word::new(content, word_type));
            }
            Component::TextComponent(TextComponent::new(TextNode::Task, words))
        }

        MdParseEnum::Quote => {
            let leaf_nodes = get_leaf_nodes(parse_node);
            let mut words = Vec::new();
            for node in leaf_nodes {
                let word_type = WordType::from(node.kind());
                let mut content = node.content().to_owned();

                if matches!(node.kind(), MdParseEnum::WikiLink | MdParseEnum::InlineLink) {
                    let comp = Word::new(content.clone(), WordType::LinkData);
                    words.push(comp);
                }
                if content.starts_with(' ') {
                    content.remove(0);
                    let comp = Word::new(" ".to_owned(), word_type);
                    words.push(comp);
                }
                words.push(Word::new(content, word_type));
            }
            if let Some(w) = words.first_mut() {
                w.set_content(w.content().trim_start().to_owned());
            }
            Component::TextComponent(TextComponent::new(TextNode::Quote, words))
        }

        MdParseEnum::Heading => {
            let indent = parse_node
                .content()
                .chars()
                .take_while(|c| *c == '#')
                .count();
            let leaf_nodes = get_leaf_nodes(parse_node);
            let mut words = Vec::new();

            let indent_u8: u8 = indent
                .try_into()
                .expect("heading level bounded by markdown spec (1-6)");
            words.push(Word::new(
                String::new(),
                WordType::MetaInfo(MetaData::HeadingLevel(indent_u8)),
            ));

            if indent > 1 {
                words.push(Word::new(
                    format!("{} ", "#".repeat(indent)),
                    WordType::Normal,
                ));
            }

            for node in leaf_nodes {
                let word_type = WordType::from(node.kind());
                let mut content = node.content().to_owned();

                if matches!(node.kind(), MdParseEnum::WikiLink | MdParseEnum::InlineLink) {
                    let comp = Word::new(content.clone(), WordType::LinkData);
                    words.push(comp);
                }

                if content.starts_with(' ') {
                    content.remove(0);
                    let comp = Word::new(" ".to_owned(), word_type);
                    words.push(comp);
                }
                words.push(Word::new(content, word_type));
            }
            if let Some(w) = words.first_mut() {
                w.set_content(w.content().trim_start().to_owned());
            }
            Component::TextComponent(TextComponent::new(TextNode::Heading, words))
        }

        MdParseEnum::Paragraph => {
            let leaf_nodes = get_leaf_nodes(parse_node);
            let mut words = Vec::new();
            for node in leaf_nodes {
                let word_type = WordType::from(node.kind());
                let mut content = node.content().to_owned();

                if matches!(node.kind(), MdParseEnum::WikiLink | MdParseEnum::InlineLink) {
                    let comp = Word::new(content.clone(), WordType::LinkData);
                    words.push(comp);
                }

                if content.starts_with(' ') {
                    content.remove(0);
                    let comp = Word::new(" ".to_owned(), word_type);
                    words.push(comp);
                }
                words.push(Word::new(content, word_type));
            }
            if let Some(w) = words.first_mut() {
                w.set_content(w.content().trim_start().to_owned());
            }
            Component::TextComponent(TextComponent::new(TextNode::Paragraph, words))
        }

        MdParseEnum::CodeBlock => {
            let leaf_nodes = get_leaf_nodes(parse_node);
            let mut words = Vec::new();

            let mut space_indented = false;

            for node in leaf_nodes {
                if node.kind() == MdParseEnum::CodeBlockStrSpaceIndented {
                    space_indented = true;
                }
                let word_type = WordType::from(node.kind());
                let content = node.content().to_owned();
                words.push(vec![Word::new(content, word_type)]);
            }

            if space_indented {
                words.push(vec![Word::new(
                    " ".to_owned(),
                    WordType::CodeBlock(Color::Reset),
                )]);
            }

            Component::TextComponent(TextComponent::new_formatted(TextNode::CodeBlock, words))
        }

        MdParseEnum::ListContainer => {
            let mut words = Vec::new();
            for child in parse_node.children_owned() {
                let kind = child.kind();
                let leaf_nodes = get_leaf_nodes(child);
                let mut inner_words = Vec::new();
                for node in leaf_nodes {
                    let word_type = WordType::from(node.kind());

                    let mut content = match node.kind() {
                        MdParseEnum::Indent => node.content().to_owned(),
                        _ => node
                            .content()
                            .chars()
                            .dedup_by(|x, y| *x == ' ' && *y == ' ')
                            .collect(),
                    };

                    if matches!(node.kind(), MdParseEnum::WikiLink | MdParseEnum::InlineLink) {
                        let comp = Word::new(content.clone(), WordType::LinkData);
                        inner_words.push(comp);
                    }
                    if content.starts_with(' ') && node.kind() != MdParseEnum::Indent {
                        content.remove(0);
                        let comp = Word::new(" ".to_owned(), word_type);
                        inner_words.push(comp);
                    }

                    inner_words.push(Word::new(content, word_type));
                }
                if kind == MdParseEnum::UnorderedList {
                    inner_words.push(Word::new(
                        "X".to_owned(),
                        WordType::MetaInfo(MetaData::UList),
                    ));
                    let list_symbol = Word::new("• ".to_owned(), WordType::ListMarker);
                    inner_words.insert(1, list_symbol);
                } else if kind == MdParseEnum::OrderedList {
                    inner_words.push(Word::new(
                        "X".to_owned(),
                        WordType::MetaInfo(MetaData::OList),
                    ));
                }
                words.push(inner_words);
            }
            Component::TextComponent(TextComponent::new_formatted(TextNode::List, words))
        }

        MdParseEnum::Table => {
            let mut words = Vec::new();
            for cell in parse_node.children_owned() {
                if cell.kind() == MdParseEnum::TableSeparator {
                    words.push(vec![Word::new(
                        cell.content().to_owned(),
                        WordType::MetaInfo(MetaData::ColumnsCount),
                    )]);
                    continue;
                }
                let mut inner_words = Vec::new();

                if cell.children().is_empty() {
                    words.push(inner_words);
                    continue;
                }

                for word in get_leaf_nodes(cell) {
                    let word_type = WordType::from(word.kind());
                    let mut content = word.content().to_owned();

                    if matches!(word.kind(), MdParseEnum::WikiLink | MdParseEnum::InlineLink) {
                        let comp = Word::new(content.clone(), WordType::LinkData);
                        inner_words.push(comp);
                    }

                    if content.starts_with(' ') {
                        content.remove(0);
                        let comp = Word::new(" ".to_owned(), word_type);
                        inner_words.push(comp);
                    }

                    inner_words.push(Word::new(content, word_type));
                }
                words.push(inner_words);
            }
            Component::TextComponent(TextComponent::new_formatted(
                TextNode::Table(vec![], vec![]),
                words,
            ))
        }

        MdParseEnum::BlockSeparator => {
            Component::TextComponent(TextComponent::new(TextNode::LineBreak, Vec::new()))
        }
        MdParseEnum::HorizontalSeparator => Component::TextComponent(TextComponent::new(
            TextNode::HorizontalSeparator,
            Vec::new(),
        )),
        MdParseEnum::Footnote => {
            let mut words = Vec::new();
            let foot_ref = parse_node.children().first().unwrap().to_owned();
            words.push(Word::new(foot_ref.content, WordType::FootnoteData));
            let _rest = parse_node
                .children_owned()
                .into_iter()
                .skip(1)
                .map(|e| e.content)
                .collect::<String>();
            words.push(Word::new(_rest, WordType::Footnote));
            Component::TextComponent(TextComponent::new(TextNode::Footnote, words))
        }
        _ => todo!("Not implemented for {:?}", parse_node.kind()),
    }
}

fn get_leaf_nodes(node: ParseNode) -> Vec<ParseNode> {
    let mut leaf_nodes = Vec::new();

    // Insert separator information between links
    if node.kind() == MdParseEnum::Link {
        let comp = if node.content().starts_with(' ') {
            ParseNode::new(MdParseEnum::Word, " ".to_owned())
        } else {
            ParseNode::new(MdParseEnum::Word, String::new())
        };
        leaf_nodes.push(comp);
    }

    if matches!(
        node.kind(),
        MdParseEnum::CodeStr
            | MdParseEnum::ItalicStr
            | MdParseEnum::BoldStr
            | MdParseEnum::BoldItalicStr
            | MdParseEnum::StrikethroughStr
    ) && node.content().starts_with(' ')
    {
        let comp = ParseNode::new(MdParseEnum::Word, " ".to_owned());
        leaf_nodes.push(comp);
    }

    // For Claude flavor: preserve leading newlines in formatted text
    if GENERAL_CONFIG.flavor == Flavor::Claude
        && matches!(
            node.kind(),
            MdParseEnum::CodeStr
                | MdParseEnum::ItalicStr
                | MdParseEnum::BoldStr
                | MdParseEnum::BoldItalicStr
                | MdParseEnum::StrikethroughStr
        )
        && node.content().starts_with('\n')
    {
        let comp = ParseNode::new(MdParseEnum::Word, "\n".to_owned());
        leaf_nodes.push(comp);
    }

    if node.children().is_empty() {
        leaf_nodes.push(node);
    } else {
        for child in node.children_owned() {
            leaf_nodes.append(&mut get_leaf_nodes(child));
        }
    }
    leaf_nodes
}

pub fn print_from_root(root: &ComponentRoot) {
    for child in root.components() {
        print_component(child, 0);
    }
}

fn print_component(component: &TextComponent, _depth: usize) {
    println!(
        "Component: {:?}, height: {}, y_offset: {}",
        component.kind(),
        component.height(),
        component.y_offset()
    );
    for w in component.meta_info() {
        println!("Meta: {}, kind: {:?}", w.content(), w.kind());
    }
    for row in component.content() {
        for w in row {
            println!("Content:{}, kind: {:?}", w.content(), w.kind());
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseRoot {
    file_name: Option<String>,
    children: Vec<ParseNode>,
}

impl ParseRoot {
    #[must_use]
    pub fn new(file_name: Option<String>, children: Vec<ParseNode>) -> Self {
        Self {
            file_name,
            children,
        }
    }

    #[must_use]
    pub fn children(&self) -> &Vec<ParseNode> {
        &self.children
    }

    #[must_use]
    pub fn children_owned(self) -> Vec<ParseNode> {
        self.children
    }

    #[must_use]
    pub fn file_name(&self) -> Option<String> {
        self.file_name.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseNode {
    kind: MdParseEnum,
    content: String,
    children: Vec<ParseNode>,
}

impl ParseNode {
    #[must_use]
    pub fn new(kind: MdParseEnum, content: String) -> Self {
        Self {
            kind,
            content,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn kind(&self) -> MdParseEnum {
        self.kind
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn add_children(&mut self, children: Vec<ParseNode>) {
        self.children.extend(children);
    }

    #[must_use]
    pub fn children(&self) -> &Vec<ParseNode> {
        &self.children
    }

    #[must_use]
    pub fn children_owned(self) -> Vec<ParseNode> {
        self.children
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdParseEnum {
    AltText,
    BlockSeparator,
    Bold,
    BoldItalic,
    BoldItalicStr,
    BoldStr,
    Caution,
    Code,
    CodeBlock,
    CodeBlockStr,
    CodeBlockStrSpaceIndented,
    CodeStr,
    Digit,
    FootnoteRef,
    Footnote,
    Heading,
    HorizontalSeparator,
    Image,
    Imortant,
    Indent,
    InlineLink,
    Italic,
    ItalicStr,
    Link,
    LinkData,
    ListContainer,
    Note,
    OrderedList,
    PLanguage,
    Paragraph,
    Quote,
    Sentence,
    Strikethrough,
    StrikethroughStr,
    Table,
    TableCell,
    TableSeparator,
    Task,
    TaskClosed,
    TaskOpen,
    Tip,
    UnorderedList,
    Warning,
    WikiLink,
    Word,
}

impl From<Rule> for MdParseEnum {
    fn from(value: Rule) -> Self {
        match value {
            Rule::word | Rule::h_word | Rule::latex_word | Rule::t_word => Self::Word,
            Rule::indent => Self::Indent,
            Rule::italic_word => Self::Italic,
            Rule::italic => Self::ItalicStr,
            Rule::bold_word => Self::Bold,
            Rule::bold => Self::BoldStr,
            Rule::bold_italic_word => Self::BoldItalic,
            Rule::bold_italic => Self::BoldItalicStr,
            Rule::strikethrough_word => Self::Strikethrough,
            Rule::strikethrough => Self::StrikethroughStr,
            Rule::code_word => Self::Code,
            Rule::code => Self::CodeStr,
            Rule::programming_language => Self::PLanguage,
            Rule::link_word | Rule::link_line | Rule::link | Rule::wiki_link_word => Self::Link,
            Rule::wiki_link_alone => Self::WikiLink,
            Rule::inline_link | Rule::inline_link_wrapper => Self::InlineLink,
            Rule::o_list_counter | Rule::digit => Self::Digit,
            Rule::task_open => Self::TaskOpen,
            Rule::task_complete => Self::TaskClosed,
            Rule::code_line => Self::CodeBlockStr,
            Rule::indented_code_line | Rule::indented_code_newline => {
                Self::CodeBlockStrSpaceIndented
            }
            Rule::sentence | Rule::t_sentence | Rule::footnote_sentence => Self::Sentence,
            Rule::table_cell => Self::TableCell,
            Rule::table_separator => Self::TableSeparator,
            Rule::u_list => Self::UnorderedList,
            Rule::o_list => Self::OrderedList,
            Rule::h1 | Rule::h2 | Rule::h3 | Rule::h4 | Rule::h5 | Rule::h6 | Rule::heading => {
                Self::Heading
            }
            Rule::list_container => Self::ListContainer,
            Rule::code_block | Rule::indented_code_block => Self::CodeBlock,
            Rule::table => Self::Table,
            Rule::quote => Self::Quote,
            Rule::task => Self::Task,
            Rule::block_sep => Self::BlockSeparator,
            Rule::horizontal_sep => Self::HorizontalSeparator,
            Rule::link_data | Rule::wiki_link_data => Self::LinkData,
            Rule::warning => Self::Warning,
            Rule::note => Self::Note,
            Rule::tip => Self::Tip,
            Rule::important => Self::Imortant,
            Rule::caution => Self::Caution,

            Rule::paragraph
            | Rule::p_char
            | Rule::t_char
            | Rule::link_char
            | Rule::wiki_link_char
            | Rule::normal
            | Rule::t_normal
            | Rule::latex
            | Rule::comment
            | Rule::txt
            | Rule::task_prefix
            | Rule::quote_prefix
            | Rule::code_block_prefix
            | Rule::table_prefix
            | Rule::list_prefix
            | Rule::forbidden_sentence_prefix => Self::Paragraph,
            Rule::image => Self::Image,
            Rule::alt_word | Rule::alt_text => Self::AltText,
            Rule::footnote_ref => Self::FootnoteRef,
            Rule::footnote => Self::Footnote,
            Rule::heading_prefix
            | Rule::horizontal_sep_prefix
            | Rule::blank_line
            | Rule::alt_char
            | Rule::b_char
            | Rule::c_char
            | Rule::c_line_char
            | Rule::comment_char
            | Rule::i_char
            | Rule::latex_char
            | Rule::quote_marking
            | Rule::inline_link_char
            | Rule::s_char
            | Rule::WHITESPACE_S
            | Rule::wiki_link
            | Rule::footnote_ref_container => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_newlines() {
        let content = "Line one\nLine two\nLine three";
        eprintln!("Input: {:?}", content);
        eprintln!("Flavor: {:?}", GENERAL_CONFIG.flavor);

        let root = parse_markdown(None, content, 80);
        eprintln!("Parsed {} components", root.components().len());

        for (i, comp) in root.components().iter().enumerate() {
            eprintln!("Component {}: {:?}", i, comp.kind());
            for (j, row) in comp.content().iter().enumerate() {
                for (k, word) in row.iter().enumerate() {
                    eprintln!("  Row {} Word {}: {:?}", j, k, word.content());
                }
            }
        }
    }

    #[test]
    fn test_parse_technical_context() {
        // This is the content that should show separate lines
        let content = r#"## Technical Context

**Language/Version**: Rust 1.92.0 (2024 edition)
**Primary Dependencies**: ratatui 0.29.0
**Storage**: N/A"#;

        eprintln!("Input:\n{}", content);
        eprintln!("Flavor: {:?}", GENERAL_CONFIG.flavor);

        let root = parse_markdown(None, content, 80);
        eprintln!("Parsed {} components", root.components().len());

        for (i, comp) in root.components().iter().enumerate() {
            eprintln!("\nComponent {}: {:?}", i, comp.kind());
            for (j, row) in comp.content().iter().enumerate() {
                for (k, word) in row.iter().enumerate() {
                    // Show raw content with escapes
                    eprintln!("  Row {} Word {}: {:?} (kind: {:?})", j, k, word.content(), word.kind());
                }
            }
        }
    }

    #[test]
    fn test_ordered_list_with_blank_lines() {
        // CommonMark allows blank lines between list items
        let content = "1. First item\n\n2. Second item\n\n3. Third item";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should parse as a single List component with 3 items
        let list_components: Vec<_> = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .collect();
        assert_eq!(list_components.len(), 1, "Should have exactly one list");

        // The list should have 3 rows (one per item)
        let list = list_components[0];
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 3, "List should have 3 items");

        // Verify the numbers are sequential: 1., 2., 3.
        // The ListMarker words contain the formatted numbers after transform
        for (i, row) in content_rows.iter().enumerate() {
            let marker = row
                .iter()
                .find(|w| w.kind() == WordType::ListMarker)
                .expect("Each row should have a list marker");
            assert_eq!(
                marker.content(),
                format!("{}. ", i + 1),
                "Item {} should be numbered {}",
                i,
                i + 1
            );
        }
    }

    #[test]
    fn test_nested_list_indentation() {
        // Nested lists should preserve indentation
        let content = "- Item 1\n  - Nested 1\n  - Nested 2\n- Item 2";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should parse as a single List component
        let list_components: Vec<_> = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .collect();
        assert_eq!(list_components.len(), 1, "Should have exactly one list");

        let list = list_components[0];

        // Check meta_info for indentation - these are MetaInfo(Other) with whitespace content
        // The transform_list function filters by content().trim() == ""
        let indent_words: Vec<_> = list
            .meta_info()
            .iter()
            .filter(|w| w.content().trim().is_empty())
            .collect();

        // We should have 4 indent entries (one per list item)
        assert_eq!(indent_words.len(), 4, "Should have 4 indent entries");

        // First item: no indent (empty string)
        assert_eq!(indent_words[0].content(), "", "First item should have no indent");
        // Nested items: 2 spaces indent
        assert_eq!(indent_words[1].content(), "  ", "Nested item 1 should have 2-space indent");
        assert_eq!(indent_words[2].content(), "  ", "Nested item 2 should have 2-space indent");
        // Back to top level: no indent
        assert_eq!(indent_words[3].content(), "", "Item 2 should have no indent");
    }

    #[test]
    fn test_four_level_nested_list() {
        // Test 4 levels of nesting - each level indented by 2 spaces
        let content = "- Level 1
  - Level 2
    - Level 3
      - Level 4
      - Level 4 again
    - Level 3 again
  - Level 2 again
- Level 1 again";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should parse as a single List component
        let list_components: Vec<_> = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .collect();
        assert_eq!(list_components.len(), 1, "Should have exactly one list");

        let list = list_components[0];

        // Check indent entries - should have 8 items total
        let indent_words: Vec<_> = list
            .meta_info()
            .iter()
            .filter(|w| w.content().trim().is_empty())
            .collect();

        assert_eq!(indent_words.len(), 8, "Should have 8 indent entries");

        // Verify indentation levels (each level = 2 spaces)
        assert_eq!(indent_words[0].content(), "", "Level 1 = no indent");
        assert_eq!(indent_words[1].content(), "  ", "Level 2 = 2 spaces");
        assert_eq!(indent_words[2].content(), "    ", "Level 3 = 4 spaces");
        assert_eq!(indent_words[3].content(), "      ", "Level 4 = 6 spaces");
        assert_eq!(indent_words[4].content(), "      ", "Level 4 again = 6 spaces");
        assert_eq!(indent_words[5].content(), "    ", "Level 3 again = 4 spaces");
        assert_eq!(indent_words[6].content(), "  ", "Level 2 again = 2 spaces");
        assert_eq!(indent_words[7].content(), "", "Level 1 again = no indent");

        // Verify rendering produces 8 non-empty lines
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 8, "List should render 8 items");

        // Verify distinct bullet markers at each level
        let markers: Vec<_> = content_rows
            .iter()
            .filter_map(|row| {
                row.iter()
                    .find(|w| w.kind() == WordType::ListMarker)
                    .map(|w| w.content().to_string())
            })
            .collect();

        assert_eq!(markers[0], "• ", "Level 1 uses bullet");
        assert_eq!(markers[1], "◦ ", "Level 2 uses white bullet");
        assert_eq!(markers[2], "▪ ", "Level 3 uses black square");
        assert_eq!(markers[3], "▫ ", "Level 4 uses white square");
        assert_eq!(markers[4], "▫ ", "Level 4 uses white square");
        assert_eq!(markers[5], "▪ ", "Level 3 uses black square");
        assert_eq!(markers[6], "◦ ", "Level 2 uses white bullet");
        assert_eq!(markers[7], "• ", "Level 1 uses bullet");
    }

    #[test]
    fn test_four_level_ordered_list() {
        // Test 4 levels of ordered list nesting
        let content = "1. Level 1
   1. Level 2
      1. Level 3
         1. Level 4
         2. Level 4 again
      2. Level 3 again
   2. Level 2 again
2. Level 1 again";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should parse as a single List component
        let list_components: Vec<_> = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .collect();
        assert_eq!(list_components.len(), 1, "Should have exactly one list");

        let list = list_components[0];

        // Verify rendering produces 8 non-empty lines
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 8, "List should render 8 items");

        // Verify counter reset at each nesting level (each level restarts at 1)
        let markers: Vec<_> = content_rows
            .iter()
            .filter_map(|row| {
                row.iter()
                    .find(|w| w.kind() == WordType::ListMarker)
                    .map(|w| w.content().to_string())
            })
            .collect();

        assert_eq!(markers[0], "1. ", "Level 1 first item");
        assert_eq!(markers[1], "1. ", "Level 2 first item (reset)");
        assert_eq!(markers[2], "1. ", "Level 3 first item (reset)");
        assert_eq!(markers[3], "1. ", "Level 4 first item (reset)");
        assert_eq!(markers[4], "2. ", "Level 4 second item");
        assert_eq!(markers[5], "2. ", "Level 3 second item");
        assert_eq!(markers[6], "2. ", "Level 2 second item");
        assert_eq!(markers[7], "2. ", "Level 1 second item");
    }

    #[test]
    fn test_four_level_mixed_list() {
        // Test 4 levels with mixed ordered/unordered lists
        let content = "- Unordered L1
  1. Ordered L2
     - Unordered L3
       1. Ordered L4
       2. Another L4
     - Another L3
  2. Another L2
- Another L1";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should parse as a single List component
        let list_components: Vec<_> = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .collect();
        assert_eq!(list_components.len(), 1, "Should have exactly one list");

        let list = list_components[0];

        // Verify rendering produces 8 non-empty lines
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 8, "List should render 8 items");

        // Verify correct markers for mixed list
        let markers: Vec<_> = content_rows
            .iter()
            .filter_map(|row| {
                row.iter()
                    .find(|w| w.kind() == WordType::ListMarker)
                    .map(|w| w.content().to_string())
            })
            .collect();

        // Level 1 unordered, level 2 ordered, level 3 unordered, level 4 ordered
        assert_eq!(markers[0], "• ", "L1 unordered bullet");
        assert_eq!(markers[1], "1. ", "L2 ordered number");
        assert_eq!(markers[2], "▪ ", "L3 unordered (level 3 bullet)");
        assert_eq!(markers[3], "1. ", "L4 ordered number");
        assert_eq!(markers[4], "2. ", "L4 ordered number");
        assert_eq!(markers[5], "▪ ", "L3 unordered (level 3 bullet)");
        assert_eq!(markers[6], "2. ", "L2 ordered number");
        assert_eq!(markers[7], "• ", "L1 unordered bullet");
    }

    #[test]
    fn test_plus_marker_nested_list() {
        // Test that +, *, and - all work as list markers (common markdown convention)
        let content = "- Level 1 dash
  * Level 2 asterisk
    + Level 3 plus
      - Level 4 dash again";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should parse as a single List component
        let list_components: Vec<_> = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .collect();
        assert_eq!(list_components.len(), 1, "Should have exactly one list");

        let list = list_components[0];

        // Verify rendering produces 4 non-empty lines
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 4, "List should render 4 items");

        // Verify correct bullet markers for each level
        let markers: Vec<_> = content_rows
            .iter()
            .filter_map(|row| {
                row.iter()
                    .find(|w| w.kind() == WordType::ListMarker)
                    .map(|w| w.content().to_string())
            })
            .collect();

        assert_eq!(markers[0], "• ", "Level 1 bullet");
        assert_eq!(markers[1], "◦ ", "Level 2 white bullet");
        assert_eq!(markers[2], "▪ ", "Level 3 black square");
        assert_eq!(markers[3], "▫ ", "Level 4 white square");
    }

    #[test]
    fn test_heading_with_inline_code() {
        let content = "## Type Assertions (`as`) vs Type Guards";
        let root = parse_markdown(None, content, 80);

        // Should parse as a single heading (not split at backticks)
        assert_eq!(root.components().len(), 1, "Should have exactly 1 component");
        assert_eq!(
            root.components()[0].kind(),
            TextNode::Heading,
            "Should be a heading"
        );

        // Should contain the inline code word
        let words: Vec<_> = root.components()[0]
            .content()
            .iter()
            .flatten()
            .collect();
        let has_code_word = words.iter().any(|w| w.kind() == WordType::Code);
        assert!(has_code_word, "Heading should contain inline code");
    }

    #[test]
    fn test_list_without_preceding_blank_line() {
        // Test that lists work without a blank line before them (common in LLM output)
        let content = "Some text
- first item
- second item";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should have a paragraph and a list
        let paragraph_count = components
            .iter()
            .filter(|c| c.kind() == TextNode::Paragraph)
            .count();
        let list_count = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .count();

        assert_eq!(paragraph_count, 1, "Should have 1 paragraph");
        assert_eq!(list_count, 1, "Should have 1 list");

        // List should have 2 items
        let list = components
            .iter()
            .find(|c| c.kind() == TextNode::List)
            .unwrap();
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 2, "List should have 2 items");
    }

    #[test]
    fn test_ordered_list_without_preceding_blank_line() {
        // Test numbered lists without blank lines (common in LLM output)
        let content = "Here are the steps:
1. First step
2. Second step
3. Third step";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should have at least a paragraph and a list
        let paragraph_count = components
            .iter()
            .filter(|c| c.kind() == TextNode::Paragraph)
            .count();
        let list_count = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .count();

        assert_eq!(paragraph_count, 1, "Should have 1 paragraph");
        assert_eq!(list_count, 1, "Should have 1 list");

        // List should have 3 items
        let list = components
            .iter()
            .find(|c| c.kind() == TextNode::List)
            .unwrap();
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 3, "List should have 3 items");
    }

    #[test]
    fn test_asterisk_list_without_preceding_blank_line() {
        // Test asterisk lists without blank lines
        let content = "Items:
* apple
* banana";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should have paragraph, linebreak, and list
        let list_count = components
            .iter()
            .filter(|c| c.kind() == TextNode::List)
            .count();

        assert_eq!(list_count, 1, "Should have 1 list");

        // List should have 2 items
        let list = components
            .iter()
            .find(|c| c.kind() == TextNode::List)
            .unwrap();
        let content_rows: Vec<_> = list.content().iter().filter(|row| !row.is_empty()).collect();
        assert_eq!(content_rows.len(), 2, "List should have 2 items");
    }

    #[test]
    fn test_emphasis_still_works_after_list_fix() {
        // Ensure the !WHITESPACE_S fix for lists doesn't break emphasis
        let content = "This is *italic* and **bold** text.";

        let root = parse_markdown(None, content, 80);
        let components = root.components();

        // Should be a single paragraph
        assert_eq!(components.len(), 1, "Should have 1 component");
        assert_eq!(
            components[0].kind(),
            TextNode::Paragraph,
            "Should be a paragraph"
        );

        // Should contain italic and bold words
        let words: Vec<_> = components[0].content().iter().flatten().collect();
        let has_italic = words.iter().any(|w| w.kind() == WordType::Italic);
        let has_bold = words.iter().any(|w| w.kind() == WordType::Bold);

        assert!(has_italic, "Should contain italic text");
        assert!(has_bold, "Should contain bold text");
    }

    #[test]
    fn test_heading_word_structure_matches_paragraph() {
        // Heading words should be structured like paragraph words
        // (spaces as separate words) for search to work correctly
        let heading = "## I know";
        let paragraph = "I know";

        let heading_root = parse_markdown(None, heading, 80);
        let para_root = parse_markdown(None, paragraph, 80);

        let heading_words: Vec<_> = heading_root.components()[0]
            .content()
            .iter()
            .flatten()
            .map(|w| w.content())
            .collect();
        let para_words: Vec<_> = para_root.components()[0]
            .content()
            .iter()
            .flatten()
            .map(|w| w.content())
            .collect();

        // Heading has "## " prefix, then same structure as paragraph
        assert_eq!(&heading_words[1..], &para_words[..]);
    }

}
