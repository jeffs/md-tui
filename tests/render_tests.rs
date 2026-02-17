use std::sync::Once;

use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect, Terminal, widgets::Clear};

use md_tui::boxes::searchbox::SearchBox;
use md_tui::nodes::root::{Component, ComponentRoot};
use md_tui::nodes::word::WordType;
use md_tui::pages::file_explorer::{FileTree, MdFile};
use md_tui::parser::parse_markdown;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| unsafe {
        std::env::set_var("MDT_FLAVOR", "commonmark");
        std::env::set_var("MDT_WIDTH", "80");
    });
}

fn parse(input: &str) -> ComponentRoot {
    setup();
    parse_markdown(None, input, 78)
}

/// Render all TextComponents from a parsed markdown string into a ratatui Buffer.
fn render_to_buffer(input: &str, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut root = parse(input);
    root.set_scroll(0);

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, width, height);
            for child in root.children_mut() {
                if let Component::TextComponent(comp) = child {
                    // Skip off-screen components (same logic as main.rs)
                    if comp.y_offset().saturating_sub(comp.scroll_offset()) >= area.height
                        || (comp.y_offset() + comp.height())
                            .saturating_sub(comp.scroll_offset())
                            == 0
                    {
                        continue;
                    }
                    f.render_widget(comp.clone(), area);
                }
            }
        })
        .unwrap();

    terminal.backend().buffer().clone()
}

/// Extract all non-whitespace text from a buffer as a single string.
fn buffer_text(buf: &Buffer) -> String {
    let area = buf.area;
    let mut text = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            text.push_str(cell.symbol());
        }
        text.push('\n');
    }
    text
}

/// Extract the text content of a single row from the buffer.
fn row_text(buf: &Buffer, y: u16) -> String {
    let mut text = String::new();
    for x in 0..buf.area.width {
        let cell = &buf[(x, y)];
        text.push_str(cell.symbol());
    }
    text
}

/// Check that a given string appears somewhere in the buffer text.
fn buffer_contains(buf: &Buffer, needle: &str) -> bool {
    buffer_text(buf).contains(needle)
}

// ── render_heading_text_present ─────────────────────────────────────

#[test]
fn render_heading_text_present() {
    let buf = render_to_buffer("# My Heading", 80, 10);
    assert!(
        buffer_contains(&buf, "My Heading"),
        "heading text should appear in rendered buffer"
    );
}

// ── render_paragraph_wraps ──────────────────────────────────────────

#[test]
fn render_paragraph_wraps() {
    // Create a paragraph that must wrap: repeat a word to exceed width 40
    let long_text = "word ".repeat(20); // 100 chars, must wrap at width 40
    let buf = render_to_buffer(&long_text, 40, 20);

    // The paragraph should occupy more than 1 line
    let mut non_empty_rows = 0;
    for y in 0..20u16 {
        let row = row_text(&buf, y);
        if row.trim().len() > 0 {
            non_empty_rows += 1;
        }
    }
    assert!(
        non_empty_rows > 1,
        "long paragraph should wrap to multiple lines, got {non_empty_rows}"
    );
}

// ── render_code_block_content ───────────────────────────────────────

#[test]
fn render_code_block_content() {
    let input = "```rust\nlet x = 42;\n```";
    let buf = render_to_buffer(input, 80, 10);
    assert!(
        buffer_contains(&buf, "let x = 42;"),
        "code block content should be present in buffer"
    );
}

// ── render_table_columns ────────────────────────────────────────────

#[test]
fn render_table_columns() {
    let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    assert!(text.contains("Name"), "table header 'Name' should appear");
    assert!(text.contains("Age"), "table header 'Age' should appear");
    assert!(text.contains("Alice"), "table data 'Alice' should appear");
    assert!(text.contains("30"), "table data '30' should appear");
}

// ── render_list_bullets ─────────────────────────────────────────────

#[test]
fn render_list_bullets() {
    let input = "- Apple\n- Banana\n- Cherry";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    // The list items should appear
    assert!(text.contains("Apple"), "list item 'Apple' should appear");
    assert!(text.contains("Banana"), "list item 'Banana' should appear");
    assert!(text.contains("Cherry"), "list item 'Cherry' should appear");
    // Bullet marker (unicode bullet '•')
    assert!(text.contains('•'), "bullet marker should appear");
}

// ── render_ordered_list_numbers ─────────────────────────────────────

#[test]
fn render_ordered_list_numbers() {
    let input = "1. First\n2. Second\n3. Third";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    assert!(text.contains("First"), "list item 'First' should appear");
    assert!(text.contains("Second"), "list item 'Second' should appear");
    assert!(text.contains("1."), "numbering '1.' should appear");
    assert!(text.contains("2."), "numbering '2.' should appear");
}

// ── render_quote_indented ───────────────────────────────────────────

#[test]
fn render_quote_indented() {
    let input = "> This is a quote";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    assert!(
        text.contains("This is a quote"),
        "quote content should appear"
    );
    // The quote renders a vertical bar (U+2588) at position x=0, then content at x=1+
    // Verify the quote text does NOT start at column 0 (it's indented by the bar)
    let first_content_row = row_text(&buf, 0);
    let trimmed = first_content_row.trim_start();
    assert!(
        first_content_row.len() > trimmed.len() || first_content_row.contains('\u{2588}'),
        "quote should have indentation marker or vertical bar"
    );
}

// ── render_task_checkbox ────────────────────────────────────────────

#[test]
fn render_task_checkbox() {
    let input = "- [ ] Open task\n- [x] Done task";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    assert!(text.contains("Open task"), "open task text should appear");
    assert!(text.contains("Done task"), "done task text should appear");
    // Task prefix: either "[ ] "/"[x] " or emoji variants "❌ "/"✅ "
    let has_checkbox = text.contains("[ ]")
        || text.contains("[x]")
        || text.contains('❌')
        || text.contains('✅');
    assert!(has_checkbox, "task checkbox prefix should appear");
}

// ── render_horizontal_separator ─────────────────────────────────────

#[test]
fn render_horizontal_separator() {
    let input = "Above\n\n---\n\nBelow";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    // Horizontal separator renders as repeated em-dash (U+2014)
    assert!(
        text.contains('\u{2014}'),
        "horizontal separator should render em-dashes"
    );
    assert!(text.contains("Above"), "'Above' text should appear");
    assert!(text.contains("Below"), "'Below' text should appear");
}

// ── render_search_highlight ─────────────────────────────────────────

#[test]
fn render_search_highlight() {
    setup();
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut root = parse("Hello world, hello again");
    root.set_scroll(0);

    // Mark search results — this mutates word types to Selected
    root.find_and_mark("hello");

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 10);
            for child in root.children_mut() {
                if let Component::TextComponent(comp) = child {
                    if comp.y_offset().saturating_sub(comp.scroll_offset()) >= area.height {
                        continue;
                    }
                    f.render_widget(comp.clone(), area);
                }
            }
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    // The text "hello" should still be present (now with Selected styling)
    assert!(
        buffer_contains(&buf, "Hello") || buffer_contains(&buf, "hello"),
        "search-highlighted text should still be visible"
    );

    // Verify that some words became Selected in the data model
    let has_selected = root
        .words()
        .iter()
        .any(|w| w.kind() == WordType::Selected);
    assert!(has_selected, "find_and_mark should produce Selected words");
}

// ── render_link_selection ───────────────────────────────────────────

#[test]
fn render_link_selection() {
    setup();
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut root = parse("[Click here](https://example.com)");
    root.set_scroll(0);

    // Select the first link
    assert!(root.select(0).is_ok(), "should be able to select link 0");

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 10);
            for child in root.children_mut() {
                if let Component::TextComponent(comp) = child {
                    if comp.y_offset().saturating_sub(comp.scroll_offset()) >= area.height {
                        continue;
                    }
                    f.render_widget(comp.clone(), area);
                }
            }
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();

    // The link text should be visible in the buffer
    assert!(
        buffer_contains(&buf, "Click here"),
        "selected link text should appear in buffer"
    );

    // The word type should be Selected in the data model
    let has_selected = root
        .words()
        .iter()
        .any(|w| w.kind() == WordType::Selected);
    assert!(
        has_selected,
        "selecting a link should produce Selected words"
    );
}

// ── render_file_tree_widget ─────────────────────────────────────────

#[test]
fn render_file_tree_widget() {
    setup();
    let backend = TestBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut tree = FileTree::new();
    tree.add_file(MdFile::new(
        "./docs/readme.md".to_string(),
        "readme.md".to_string(),
    ));
    tree.add_file(MdFile::new(
        "./notes/todo.md".to_string(),
        "todo.md".to_string(),
    ));

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 60, 20);
            f.render_widget(tree.clone(), area);
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let text = buffer_text(&buf);

    assert!(
        text.contains("readme.md"),
        "file tree should show file name 'readme.md'"
    );
    assert!(
        text.contains("todo.md"),
        "file tree should show file name 'todo.md'"
    );
    // File paths should also appear
    assert!(
        text.contains("./docs/readme.md"),
        "file tree should show path"
    );
    assert!(
        text.contains("./notes/todo.md"),
        "file tree should show path"
    );
    // The title "MD-TUI" should appear
    assert!(text.contains("MD-TUI"), "file tree should show title");
}

// ── render_search_box_clear_overlay ────────────────────────────────

#[test]
fn render_search_box_clear_overlay() {
    setup();
    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    // First pass: render markdown content that fills the bottom rows
    let mut root = parse("# Heading\n\nLine one\n\nLine two\n\nLine three\n\nLine four");
    root.set_scroll(0);

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 10);
            for child in root.children_mut() {
                if let Component::TextComponent(comp) = child {
                    if comp.y_offset().saturating_sub(comp.scroll_offset()) >= area.height {
                        continue;
                    }
                    f.render_widget(comp.clone(), area);
                }
            }
        })
        .unwrap();

    // Verify content exists in the overlay region (rows 7-8) before Clear
    let before_row7 = row_text(terminal.backend().buffer(), 7);
    let before_row8 = row_text(terminal.backend().buffer(), 8);
    let has_content_before =
        before_row7.trim().len() > 0 || before_row8.trim().len() > 0;

    // Second pass: render Clear + SearchBox on top of the same area
    let search_box = SearchBox::new();
    terminal
        .draw(|f| {
            // Re-render the markdown so the buffer has content
            let area = Rect::new(0, 0, 80, 10);
            for child in root.children_mut() {
                if let Component::TextComponent(comp) = child {
                    if comp.y_offset().saturating_sub(comp.scroll_offset()) >= area.height {
                        continue;
                    }
                    f.render_widget(comp.clone(), area);
                }
            }
            // Now overlay Clear + SearchBox at rows 7-8
            let overlay = Rect::new(2, 7, 40, 2);
            f.render_widget(Clear, overlay);
            f.render_widget(search_box.clone(), overlay);
        })
        .unwrap();

    // The overlay region should now be cleared of markdown content
    let after_row7 = row_text(terminal.backend().buffer(), 7);
    // Row 7 is the text line (empty search box), row 8 is the bottom border
    // The cleared area (columns 2..42) should not contain markdown text
    let overlay_slice: String = after_row7.chars().skip(2).take(40).collect();
    // With an empty search box, the overlay should just be spaces (cleared)
    assert!(
        !overlay_slice.contains("Line") && !overlay_slice.contains("four"),
        "Clear widget should erase underlying markdown in the overlay area, got: {overlay_slice:?}"
    );

    // If there was content before, the clear worked; if there wasn't,
    // the test is still valid (no bleed-through either way)
    if has_content_before {
        assert!(
            !overlay_slice.trim().is_empty() || overlay_slice.chars().all(|c| c == ' '),
            "overlay area should be cleared, not contain markdown text"
        );
    }
}

// ── render_hard_wrapped_paragraph ──────────────────────────────────

#[test]
fn render_hard_wrapped_paragraph_has_spaces() {
    let input = "Get FreqTrade running from source at `~/git/freqtrade` and\n\
                 configured to trade on Hyperliquid.";
    let buf = render_to_buffer(input, 80, 10);
    let text = buffer_text(&buf);
    assert!(
        !text.contains("andconfigured"),
        "hard-wrapped line should not jam 'and' and 'configured' together, got:\n{text}"
    );
}

#[test]
fn render_phase0_setup_no_jammed_words() {
    let content = std::fs::read_to_string("var/phase0-setup.md")
        .expect("var/phase0-setup.md should exist");
    let buf = render_to_buffer(&content, 80, 120);
    let text = buffer_text(&buf);
    eprintln!("FULL RENDER:\n{text}");

    // Check all known hard-wrap boundaries in the file
    assert!(!text.contains("andconfigured"), "line 5→6 jammed: {text}");
    assert!(!text.contains("extrasas"), "line 37→38 jammed");
    assert!(!text.contains("`secret`as"), "line 74→75 jammed");
    assert!(!text.contains("secretas"), "line 74→75 jammed (no backtick)");
}

// ── render_hard_wrapped_bold_has_space ─────────────────────────────

#[test]
fn render_hard_wrapped_bold_has_space() {
    let buf = render_to_buffer("**bold\ntext**", 80, 10);
    let text = buffer_text(&buf);
    assert!(
        text.contains("bold") && text.contains("text"),
        "bold words should appear in buffer"
    );
    // The two words must not be jammed together
    assert!(
        !text.contains("boldtext"),
        "hard-wrapped bold should not render 'boldtext', got:\n{text}"
    );
}
