use std::sync::Once;

use md_tui::nodes::root::ComponentRoot;
use md_tui::nodes::textcomponent::TextNode;
use md_tui::nodes::word::{MetaData, WordType};
use md_tui::parser::parse_markdown;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        unsafe {
            std::env::set_var("MDT_FLAVOR", "commonmark");
            std::env::set_var("MDT_WIDTH", "80");
        }
    });
}

fn parse(input: &str) -> ComponentRoot {
    setup();
    parse_markdown(None, input, 78)
}

/// Helper: collect only non-LineBreak text components
fn content_components(root: &ComponentRoot) -> Vec<&md_tui::nodes::textcomponent::TextComponent> {
    root.components()
        .into_iter()
        .filter(|c| c.kind() != TextNode::LineBreak)
        .collect()
}

/// Helper: flatten all renderable words from a component into (content, kind) pairs
fn words_of(
    comp: &md_tui::nodes::textcomponent::TextComponent,
) -> Vec<(&str, WordType)> {
    comp.content()
        .iter()
        .flatten()
        .map(|w| (w.content(), w.kind()))
        .collect()
}

// ── parse_empty ──────────────────────────────────────────────────────

#[test]
fn parse_empty() {
    let root = parse("");
    assert_eq!(root.components().len(), 0, "empty input => 0 components");
}

// ── parse_paragraph ──────────────────────────────────────────────────

#[test]
fn parse_paragraph() {
    let root = parse("Hello world");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let text: String = words.iter().map(|(c, _)| *c).collect();
    assert!(
        text.contains("Hello"),
        "paragraph should contain 'Hello', got: {text}"
    );
    assert!(
        text.contains("world"),
        "paragraph should contain 'world', got: {text}"
    );
    assert!(
        words.iter().all(|(_, k)| *k == WordType::Normal),
        "all words should be Normal"
    );
}

// ── parse_heading_h1 ─────────────────────────────────────────────────

#[test]
fn parse_heading_h1() {
    let root = parse("# Title");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Heading);

    // Meta should contain HeadingLevel(1)
    let meta = comps[0].meta_info();
    let has_h1 = meta
        .iter()
        .any(|w| w.kind() == WordType::MetaInfo(MetaData::HeadingLevel(1)));
    assert!(has_h1, "H1 should have HeadingLevel(1) meta");

    // Content should contain "Title"
    let text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(text.contains("Title"), "H1 content should contain 'Title'");
}

// ── parse_heading_h2 ─────────────────────────────────────────────────

#[test]
fn parse_heading_h2() {
    let root = parse("## Subtitle");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Heading);

    let meta = comps[0].meta_info();
    let has_h2 = meta
        .iter()
        .any(|w| w.kind() == WordType::MetaInfo(MetaData::HeadingLevel(2)));
    assert!(has_h2, "H2 should have HeadingLevel(2) meta");

    // H2+ get "## " prefix in content
    let text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        text.contains("## "),
        "H2 content should start with '## ', got: {text}"
    );
    assert!(text.contains("Subtitle"));
}

// ── parse_heading_h3_through_h6 ──────────────────────────────────────

#[test]
fn parse_heading_h3_through_h6() {
    for level in 3u8..=6 {
        let prefix = "#".repeat(level as usize);
        let input = format!("{prefix} Heading{level}");
        let root = parse(&input);
        let comps = content_components(&root);

        assert_eq!(comps.len(), 1, "level {level} should produce 1 component");
        assert_eq!(comps[0].kind(), TextNode::Heading);

        let has_level = comps[0]
            .meta_info()
            .iter()
            .any(|w| w.kind() == WordType::MetaInfo(MetaData::HeadingLevel(level)));
        assert!(has_level, "should have HeadingLevel({level}) meta");

        let text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
        assert!(
            text.contains(&format!("Heading{level}")),
            "level {level} content should contain 'Heading{level}', got: {text}"
        );
    }
}

// ── parse_bold ───────────────────────────────────────────────────────

#[test]
fn parse_bold() {
    let root = parse("**bold text**");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_bold = words.iter().any(|(_, k)| *k == WordType::Bold);
    assert!(has_bold, "should contain Bold words, got: {words:?}");

    let bold_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::Bold)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        bold_text.contains("bold"),
        "bold content should contain 'bold', got: {bold_text}"
    );
}

// ── parse_italic_star ────────────────────────────────────────────────

#[test]
fn parse_italic_star() {
    let root = parse("*italic*");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_italic = words.iter().any(|(_, k)| *k == WordType::Italic);
    assert!(has_italic, "should contain Italic words, got: {words:?}");

    let italic_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::Italic)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        italic_text.contains("italic"),
        "italic content should contain 'italic', got: {italic_text}"
    );
}

// ── parse_bold_italic ────────────────────────────────────────────────

#[test]
fn parse_bold_italic() {
    let root = parse("***both***");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_bold_italic = words.iter().any(|(_, k)| *k == WordType::BoldItalic);
    assert!(
        has_bold_italic,
        "should contain BoldItalic words, got: {words:?}"
    );

    let bi_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::BoldItalic)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        bi_text.contains("both"),
        "bold-italic content should contain 'both', got: {bi_text}"
    );
}

// ── parse_strikethrough ──────────────────────────────────────────────

#[test]
fn parse_strikethrough() {
    let root = parse("~~struck~~");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_strike = words.iter().any(|(_, k)| *k == WordType::Strikethrough);
    assert!(
        has_strike,
        "should contain Strikethrough words, got: {words:?}"
    );

    let strike_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::Strikethrough)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        strike_text.contains("struck"),
        "strikethrough content should contain 'struck', got: {strike_text}"
    );
}

// ── parse_inline_code ────────────────────────────────────────────────

#[test]
fn parse_inline_code() {
    let root = parse("`code`");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_code = words.iter().any(|(_, k)| *k == WordType::Code);
    assert!(has_code, "should contain Code words, got: {words:?}");

    let code_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::Code)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        code_text.contains("code"),
        "inline code content should contain 'code', got: {code_text}"
    );
}

// ── parse_code_block_fenced ──────────────────────────────────────────

#[test]
fn parse_code_block_fenced() {
    let input = "```rust\nlet x = 1;\n```";
    let root = parse(input);
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::CodeBlock);

    // Meta should contain the language identifier
    let meta = comps[0].meta_info();
    let has_lang = meta.iter().any(|w| {
        w.kind() == WordType::MetaInfo(MetaData::Other) && w.content() == "rust"
    });
    assert!(
        has_lang,
        "fenced code block should have PLanguage meta 'rust', got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    // Content should include the code
    let all_text: String = comps[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(
        all_text.contains("let x = 1;") || all_text.contains("let") && all_text.contains("1"),
        "code block should contain source code, got: {all_text}"
    );
}

// ── parse_code_block_tilde ───────────────────────────────────────────

#[test]
fn parse_code_block_tilde() {
    let input = "~~~\ncode here\n~~~";
    let root = parse(input);
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::CodeBlock);

    let all_text: String = comps[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(
        all_text.contains("code here"),
        "tilde code block should contain 'code here', got: {all_text}"
    );
}

// ── parse_horizontal_separator ───────────────────────────────────────

#[test]
fn parse_horizontal_separator() {
    let root = parse("---");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::HorizontalSeparator);
}

// ── parse_block_separators ───────────────────────────────────────────

#[test]
fn parse_block_separators() {
    let root = parse("para1\n\npara2");
    let comps = content_components(&root);

    // Should produce two paragraphs (LineBreaks are filtered out)
    assert_eq!(
        comps.len(),
        2,
        "two paragraphs separated by blank line should produce 2 content components, got {}",
        comps.len()
    );
    assert_eq!(comps[0].kind(), TextNode::Paragraph);
    assert_eq!(comps[1].kind(), TextNode::Paragraph);

    let text1: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    let text2: String = words_of(comps[1]).iter().map(|(c, _)| *c).collect();
    assert!(text1.contains("para1"), "first paragraph should contain 'para1'");
    assert!(text2.contains("para2"), "second paragraph should contain 'para2'");

    // There should also be a LineBreak in the full component list
    let all_comps = root.components();
    let linebreaks = all_comps
        .iter()
        .filter(|c| c.kind() == TextNode::LineBreak)
        .count();
    assert!(
        linebreaks >= 1,
        "should have at least 1 LineBreak between paragraphs, got {linebreaks}"
    );
}

// ── T08: Complex element parser tests ───────────────────────────────

// ── parse_unordered_list ────────────────────────────────────────────

#[test]
fn parse_unordered_list() {
    let root = parse("- item1\n- item2");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1, "should produce 1 list component");
    assert_eq!(comps[0].kind(), TextNode::List);

    // Should have UList meta markers
    let meta = comps[0].meta_info();
    let ulist_count = meta
        .iter()
        .filter(|w| w.kind() == WordType::MetaInfo(MetaData::UList))
        .count();
    assert!(
        ulist_count >= 2,
        "should have at least 2 UList meta markers, got {ulist_count}"
    );

    // Content should contain both items
    let all_text: String = comps[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(all_text.contains("item1"), "should contain 'item1', got: {all_text}");
    assert!(all_text.contains("item2"), "should contain 'item2', got: {all_text}");

    // Should have bullet markers
    let has_bullet = comps[0]
        .content()
        .iter()
        .flatten()
        .any(|w| w.kind() == WordType::ListMarker);
    assert!(has_bullet, "unordered list should have ListMarker words");
}

// ── parse_ordered_list ──────────────────────────────────────────────

#[test]
fn parse_ordered_list() {
    let root = parse("1. first\n2. second");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1, "should produce 1 list component");
    assert_eq!(comps[0].kind(), TextNode::List);

    // Should have OList meta markers
    let meta = comps[0].meta_info();
    let olist_count = meta
        .iter()
        .filter(|w| w.kind() == WordType::MetaInfo(MetaData::OList))
        .count();
    assert!(
        olist_count >= 2,
        "should have at least 2 OList meta markers, got {olist_count}"
    );

    let all_text: String = comps[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(all_text.contains("first"), "should contain 'first', got: {all_text}");
    assert!(all_text.contains("second"), "should contain 'second', got: {all_text}");
}

// ── parse_nested_list ───────────────────────────────────────────────

#[test]
fn parse_nested_list() {
    let root = parse("- outer\n  - inner");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1, "nested list should produce 1 list component");
    assert_eq!(comps[0].kind(), TextNode::List);

    let all_text: String = comps[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(all_text.contains("outer"), "should contain 'outer', got: {all_text}");
    assert!(all_text.contains("inner"), "should contain 'inner', got: {all_text}");

    // Content should have more than one line (nested produces multiple rows)
    assert!(
        comps[0].content().len() >= 2,
        "nested list should have at least 2 content rows, got {}",
        comps[0].content().len()
    );
}

// ── parse_task_open ─────────────────────────────────────────────────

#[test]
fn parse_task_open() {
    let root = parse("- [ ] Todo");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Task);

    // Meta should contain TaskOpen marker (MetaData::Other from MdParseEnum::TaskOpen)
    let meta = comps[0].meta_info();
    let has_task_open = meta.iter().any(|w| {
        w.kind() == WordType::MetaInfo(MetaData::Other) && w.content().contains("- [ ]")
    });
    assert!(
        has_task_open,
        "open task should have TaskOpen meta, got meta: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    let all_text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("Todo"),
        "task content should contain 'Todo', got: {all_text}"
    );
}

// ── parse_task_closed ───────────────────────────────────────────────

#[test]
fn parse_task_closed() {
    let root = parse("- [x] Done");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Task);

    // Meta should contain TaskClosed marker
    let meta = comps[0].meta_info();
    let has_task_closed = meta.iter().any(|w| {
        w.kind() == WordType::MetaInfo(MetaData::Other) && w.content().contains("- [x]")
    });
    assert!(
        has_task_closed,
        "closed task should have TaskClosed meta, got meta: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    let all_text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("Done"),
        "task content should contain 'Done', got: {all_text}"
    );
}

// ── parse_link_standard ─────────────────────────────────────────────

#[test]
fn parse_link_standard() {
    let root = parse("[text](url)");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    // Should have Link words
    let words = words_of(comps[0]);
    let has_link = words.iter().any(|(_, k)| *k == WordType::Link);
    assert!(has_link, "should contain Link words, got: {words:?}");

    let link_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::Link)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        link_text.contains("text"),
        "link should contain 'text', got: {link_text}"
    );

    // Should have LinkData meta
    let meta = comps[0].meta_info();
    let has_link_data = meta.iter().any(|w| w.kind() == WordType::LinkData);
    assert!(
        has_link_data,
        "should have LinkData in meta, got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    let link_data_text: String = meta
        .iter()
        .filter(|w| w.kind() == WordType::LinkData)
        .map(|w| w.content())
        .collect();
    assert!(
        link_data_text.contains("url"),
        "LinkData should contain 'url', got: {link_data_text}"
    );
}

// ── parse_link_wiki ─────────────────────────────────────────────────

#[test]
fn parse_link_wiki() {
    let root = parse("[[page]]");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_link = words.iter().any(|(_, k)| *k == WordType::Link);
    assert!(has_link, "wiki link should produce Link words, got: {words:?}");

    // WikiLink generates LinkData meta with the page name
    let meta = comps[0].meta_info();
    let has_link_data = meta.iter().any(|w| w.kind() == WordType::LinkData);
    assert!(
        has_link_data,
        "wiki link should have LinkData meta, got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );
}

// ── parse_link_inline ───────────────────────────────────────────────

#[test]
fn parse_link_inline() {
    let root = parse("<https://example.com>");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let words = words_of(comps[0]);
    let has_link = words.iter().any(|(_, k)| *k == WordType::Link);
    assert!(has_link, "inline link should produce Link words, got: {words:?}");

    // Should contain the URL
    let link_text: String = words
        .iter()
        .filter(|(_, k)| *k == WordType::Link)
        .map(|(c, _)| *c)
        .collect();
    assert!(
        link_text.contains("example.com"),
        "inline link should contain 'example.com', got: {link_text}"
    );

    // InlineLink also produces LinkData meta
    let meta = comps[0].meta_info();
    let has_link_data = meta.iter().any(|w| w.kind() == WordType::LinkData);
    assert!(
        has_link_data,
        "inline link should have LinkData meta, got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );
}

// ── parse_table ─────────────────────────────────────────────────────

#[test]
fn parse_table() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 |";
    let root = parse(input);
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1, "table should produce 1 component");

    // Table kind has Table(widths, heights) variant
    match comps[0].kind() {
        TextNode::Table(_, _) => {} // expected
        other => panic!("expected Table, got {other:?}"),
    }

    // Meta should contain ColumnsCount
    let meta = comps[0].meta_info();
    let has_columns = meta
        .iter()
        .any(|w| w.kind() == WordType::MetaInfo(MetaData::ColumnsCount));
    assert!(
        has_columns,
        "table should have ColumnsCount meta, got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    // Content should contain cell values
    let all_text: String = comps[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(all_text.contains('a'), "table should contain 'a', got: {all_text}");
    assert!(all_text.contains('b'), "table should contain 'b', got: {all_text}");
    assert!(all_text.contains('1'), "table should contain '1', got: {all_text}");
    assert!(all_text.contains('2'), "table should contain '2', got: {all_text}");
}

// ── parse_quote ─────────────────────────────────────────────────────

#[test]
fn parse_quote() {
    let root = parse("> quoted text");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Quote);

    let all_text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("quoted"),
        "quote should contain 'quoted', got: {all_text}"
    );
    assert!(
        all_text.contains("text"),
        "quote should contain 'text', got: {all_text}"
    );
}

// ── parse_quote_admonition_note ─────────────────────────────────────

#[test]
fn parse_quote_admonition_note() {
    let root = parse("> [!note]\n> some text");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Quote);

    // Should have Note admonition meta
    let meta = comps[0].meta_info();
    let has_note = meta
        .iter()
        .any(|w| w.kind() == WordType::MetaInfo(MetaData::Note));
    assert!(
        has_note,
        "note admonition should have Note meta, got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    let all_text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("some") && all_text.contains("text"),
        "note admonition should contain body text, got: {all_text}"
    );
}

// ── parse_quote_admonition_warning ──────────────────────────────────

#[test]
fn parse_quote_admonition_warning() {
    let root = parse("> [!warning]\n> danger ahead");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Quote);

    let meta = comps[0].meta_info();
    let has_warning = meta
        .iter()
        .any(|w| w.kind() == WordType::MetaInfo(MetaData::Warning));
    assert!(
        has_warning,
        "warning admonition should have Warning meta, got: {:?}",
        meta.iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>()
    );

    let all_text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("danger") && all_text.contains("ahead"),
        "warning admonition should contain body text, got: {all_text}"
    );
}

// ── T09: Edge cases and combinations ────────────────────────────────

// ── parse_footnote ──────────────────────────────────────────────────

#[test]
fn parse_footnote() {
    let input = "text[^1]\n\n[^1]: footnote content";
    let root = parse(input);
    let all = root.components();

    // Should have a paragraph with FootnoteInline reference
    let paragraphs: Vec<_> = all.iter().filter(|c| c.kind() == TextNode::Paragraph).collect();
    assert!(
        !paragraphs.is_empty(),
        "should have at least one paragraph"
    );

    // Check the paragraph contains a FootnoteInline word
    let has_footnote_inline = paragraphs.iter().any(|p| {
        p.meta_info()
            .iter()
            .any(|w| w.kind() == WordType::FootnoteInline)
    });
    assert!(
        has_footnote_inline,
        "paragraph should contain FootnoteInline meta, got metas: {:?}",
        paragraphs
            .iter()
            .flat_map(|p| p.meta_info().iter().map(|w| (w.content(), w.kind())).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );

    // Should also have a Footnote component
    let footnotes: Vec<_> = all.iter().filter(|c| c.kind() == TextNode::Footnote).collect();
    assert_eq!(
        footnotes.len(),
        1,
        "should have exactly 1 Footnote component, got {}",
        footnotes.len()
    );

    // Footnote content should contain the footnote text
    let fn_words: String = footnotes[0]
        .content()
        .iter()
        .flatten()
        .map(|w| w.content())
        .collect();
    assert!(
        fn_words.contains("footnote") && fn_words.contains("content"),
        "footnote should contain 'footnote content', got: {fn_words}"
    );
}

// ── parse_image_missing ─────────────────────────────────────────────

#[test]
fn parse_image_missing() {
    let root = parse("![alt](nonexistent.png)");
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    let all_text: String = words_of(comps[0]).iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("Image") && all_text.contains("not") && all_text.contains("found"),
        "missing image should produce 'Image not found/fetched', got: {all_text}"
    );
    assert!(
        all_text.contains("[alt]"),
        "should contain alt text in brackets, got: {all_text}"
    );
}

// ── parse_comment_stripped ──────────────────────────────────────────

#[test]
fn parse_comment_stripped() {
    let input = "before <!-- comment --> after";
    let root = parse(input);
    let comps = content_components(&root);

    // Should produce content with "before" and "after"
    let all_text: String = comps
        .iter()
        .flat_map(|c| words_of(c))
        .map(|(c, _)| c)
        .collect::<Vec<_>>()
        .join("");
    assert!(
        all_text.contains("before"),
        "should contain 'before', got: {all_text}"
    );
    assert!(
        all_text.contains("after"),
        "should contain 'after', got: {all_text}"
    );
    // Comment text should NOT appear in rendered words
    assert!(
        !all_text.contains("comment"),
        "comment text should be stripped, got: {all_text}"
    );
}

// ── parse_multiple_blocks ───────────────────────────────────────────

#[test]
fn parse_multiple_blocks() {
    let input = "# Title\n\nSome paragraph text.\n\n- item1\n- item2";
    let root = parse(input);
    let comps = content_components(&root);

    // Should have heading, paragraph, and list (in that order)
    assert!(
        comps.len() >= 3,
        "should have at least 3 content components (heading, paragraph, list), got {}",
        comps.len()
    );

    assert_eq!(comps[0].kind(), TextNode::Heading, "first should be Heading");
    assert_eq!(
        comps[1].kind(),
        TextNode::Paragraph,
        "second should be Paragraph"
    );
    assert_eq!(comps[2].kind(), TextNode::List, "third should be List");

    // Verify the full component list has LineBreaks between content components
    let all = root.components();
    let linebreaks = all.iter().filter(|c| c.kind() == TextNode::LineBreak).count();
    assert!(
        linebreaks >= 2,
        "should have at least 2 LineBreaks between 3 blocks, got {linebreaks}"
    );
}

// ── parse_escaped_bold ──────────────────────────────────────────────

#[test]
fn parse_escaped_bold() {
    // The grammar's bold rule has `!"\\"` guard — a backslash before ** prevents bold parsing
    let input = "\\**not bold**";
    let root = parse(input);
    let comps = content_components(&root);
    assert!(!comps.is_empty(), "should produce at least one component");

    let words = words_of(comps[0]);
    let has_bold = words.iter().any(|(_, k)| *k == WordType::Bold);
    assert!(
        !has_bold,
        "escaped bold should NOT produce Bold words, got: {words:?}"
    );
}

// ── parse_latex ─────────────────────────────────────────────────────

#[test]
fn parse_latex() {
    let input = "$x^2$";
    let root = parse(input);
    let comps = content_components(&root);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), TextNode::Paragraph);

    // Latex is parsed as Normal words (latex_word maps to MdParseEnum::Word → WordType::Normal)
    let words = words_of(comps[0]);
    let all_text: String = words.iter().map(|(c, _)| *c).collect();
    assert!(
        all_text.contains("x"),
        "latex should contain variable 'x', got: {all_text}"
    );
    assert!(
        words.iter().all(|(_, k)| *k == WordType::Normal),
        "latex words should be Normal type, got: {words:?}"
    );
}

// ── parse_kitchen_sink_no_panic ─────────────────────────────────────

#[test]
fn parse_kitchen_sink_no_panic() {
    let fixture = std::fs::read_to_string("tests/fixtures/kitchen_sink.md")
        .expect("kitchen_sink.md fixture should exist");
    let root = parse(&fixture);
    let comps = content_components(&root);

    assert!(
        !comps.is_empty(),
        "kitchen sink should produce at least 1 content component, got 0"
    );

    // Verify we get a good variety of component types
    let kinds: std::collections::HashSet<_> = comps.iter().map(|c| {
        // Normalize Table variants since they carry dynamic data
        match c.kind() {
            TextNode::Table(_, _) => "Table",
            TextNode::Paragraph => "Paragraph",
            TextNode::Heading => "Heading",
            TextNode::Quote => "Quote",
            TextNode::List => "List",
            TextNode::Task => "Task",
            TextNode::CodeBlock => "CodeBlock",
            TextNode::HorizontalSeparator => "HorizontalSeparator",
            TextNode::Footnote => "Footnote",
            TextNode::LineBreak => "LineBreak",
            TextNode::Image => "Image",
        }
    }).collect();

    // kitchen_sink.md should produce at least headings, paragraphs, quotes, lists, tasks,
    // code blocks, tables, and a horizontal separator
    let expected = ["Heading", "Paragraph", "Quote", "List", "Task", "CodeBlock", "HorizontalSeparator"];
    for kind in &expected {
        assert!(
            kinds.contains(kind),
            "kitchen sink should contain {kind} component, found: {kinds:?}"
        );
    }
}
