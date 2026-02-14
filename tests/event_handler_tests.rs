use std::sync::{Once, mpsc};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use notify::{Config, PollWatcher};

use md_tui::event_handler::{KeyBoardAction, handle_keyboard_input, keyboard_mode_file_tree};
use md_tui::nodes::root::ComponentRoot;
use md_tui::pages::file_explorer::{FileTree, MdFile};
use md_tui::parser::parse_markdown;
use md_tui::util::{App, Boxes, Mode};

static INIT: Once = Once::new();

fn init_env() {
    INIT.call_once(|| {
        unsafe {
            std::env::set_var("MDT_FLAVOR", "commonmark");
            std::env::set_var("MDT_WIDTH", "80");
        }
    });
}

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn key_code(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn make_watcher() -> PollWatcher {
    let (tx, _rx) = mpsc::channel();
    PollWatcher::new(tx, Config::default().with_poll_interval(Duration::from_secs(60))).unwrap()
}

fn make_file_tree(files: &[(&str, &str)]) -> FileTree {
    let mut ft = FileTree::new();
    for (path, name) in files {
        ft.add_file(MdFile::new(path.to_string(), name.to_string()));
    }
    ft
}

fn empty_root() -> ComponentRoot {
    ComponentRoot::new(None, Vec::new())
}

const HEIGHT: u16 = 24;

// --- FileTree / Boxes::None tests ---

#[test]
fn filetree_j_moves_down() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[
        ("a.md", "A"),
        ("b.md", "B"),
        ("c.md", "C"),
    ]);
    let mut w = make_watcher();

    // Initially selected index is 0 (first file auto-selected by add_file)
    assert_eq!(ft.selected().map(MdFile::name), Some("A"));

    handle_keyboard_input(&key('j'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(ft.selected().map(MdFile::name), Some("B"));
}

#[test]
fn filetree_k_moves_up() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[
        ("a.md", "A"),
        ("b.md", "B"),
        ("c.md", "C"),
    ]);
    let mut w = make_watcher();

    // Move down first, then up
    handle_keyboard_input(&key('j'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(ft.selected().map(MdFile::name), Some("B"));

    handle_keyboard_input(&key('k'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(ft.selected().map(MdFile::name), Some("A"));
}

#[test]
fn filetree_enter_opens_file() {
    init_env();

    // Write a real markdown file to a temp directory
    let dir = std::env::temp_dir().join("mdt_test_filetree_enter");
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("test.md");
    std::fs::write(&file_path, "# Test\n\nSome content here.").unwrap();

    let path_str = file_path.to_str().unwrap().to_string();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.set_width(80);
    let mut md = empty_root();
    let mut ft = FileTree::new();
    ft.add_file(MdFile::new(path_str.clone(), "test.md".to_string()));
    let mut w = make_watcher();

    handle_keyboard_input(&key_code(KeyCode::Enter), &mut app, &mut md, &mut ft, HEIGHT, &mut w);

    assert_eq!(app.mode, Mode::View);
    // The markdown was parsed — it should have components
    assert!(md.height() > 0);

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn filetree_f_opens_search() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("a.md", "A")]);
    let mut w = make_watcher();

    handle_keyboard_input(&key('f'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.boxes, Boxes::Search);
}

#[test]
fn filetree_q_exits() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("a.md", "A")]);
    let mut w = make_watcher();

    let result = handle_keyboard_input(&key('q'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(matches!(result, KeyBoardAction::Exit));
}

// --- FileTree / Boxes::Search tests ---

#[test]
fn filetree_search_char_filters() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.boxes = Boxes::Search;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[
        ("alpha.md", "Alpha"),
        ("beta.md", "Beta"),
    ]);
    let mut w = make_watcher();

    // Typing 'a' should filter and insert into search box
    keyboard_mode_file_tree(&key('a'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.search_box.content(), Some("a"));

    // File tree should be filtered — only "Alpha" matches "a" (case-insensitive)
    let visible: Vec<&str> = ft.files().iter().map(|f| f.name()).collect();
    assert!(visible.contains(&"Alpha"));
}

#[test]
fn filetree_search_backspace_empty_closes() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.boxes = Boxes::Search;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("a.md", "A")]);
    let mut w = make_watcher();

    // Search box is empty, backspace should close search
    keyboard_mode_file_tree(
        &key_code(KeyCode::Backspace),
        &mut app,
        &mut md,
        &mut ft,
        HEIGHT,
        &mut w,
    );
    assert_eq!(app.boxes, Boxes::None);
}

#[test]
fn filetree_search_esc_clears() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.boxes = Boxes::Search;
    app.search_box.insert('x');
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("a.md", "A")]);
    let mut w = make_watcher();

    keyboard_mode_file_tree(
        &key_code(KeyCode::Esc),
        &mut app,
        &mut md,
        &mut ft,
        HEIGHT,
        &mut w,
    );
    assert_eq!(app.boxes, Boxes::None);
    assert_eq!(app.search_box.content(), None); // cleared
}

#[test]
fn filetree_search_enter_consumes() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.boxes = Boxes::Search;
    app.search_box.insert('t');
    app.search_box.insert('e');
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("test.md", "Test")]);
    let mut w = make_watcher();

    keyboard_mode_file_tree(
        &key_code(KeyCode::Enter),
        &mut app,
        &mut md,
        &mut ft,
        HEIGHT,
        &mut w,
    );
    assert_eq!(app.boxes, Boxes::None);
    // search_box was consumed (cleared)
    assert_eq!(app.search_box.content(), None);
}

// --- FileTree / Boxes::Error tests ---

#[test]
fn filetree_error_enter_closes() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.boxes = Boxes::Error;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("a.md", "A")]);
    let mut w = make_watcher();

    keyboard_mode_file_tree(
        &key_code(KeyCode::Enter),
        &mut app,
        &mut md,
        &mut ft,
        HEIGHT,
        &mut w,
    );
    assert_eq!(app.boxes, Boxes::None);
}

#[test]
fn filetree_error_esc_closes() {
    init_env();
    let mut app = App::default();
    app.mode = Mode::FileTree;
    app.boxes = Boxes::Error;
    let mut md = empty_root();
    let mut ft = make_file_tree(&[("a.md", "A")]);
    let mut w = make_watcher();

    keyboard_mode_file_tree(
        &key_code(KeyCode::Esc),
        &mut app,
        &mut md,
        &mut ft,
        HEIGHT,
        &mut w,
    );
    assert_eq!(app.boxes, Boxes::None);
}

// ===========================================================================
// T14: View mode tests
// ===========================================================================

const VIEW_WIDTH: u16 = 80;

/// Markdown with multiple links, a heading, and enough content to scroll.
const MD_WITH_LINKS: &str = "\
# Top Heading

Some paragraph text here.

[Link One](https://one.example.com)

[Link Two](https://two.example.com)

[Internal](#top-heading)

More text at the bottom to make the document taller.

And more filler lines.

Even more filler.

Yet another filler paragraph.

Still more content.

Almost there.

Last paragraph.
";

/// Parse the links markdown into a ComponentRoot, setting scroll so offsets
/// are initialized. Returns the root and a configured App in View mode.
fn setup_view() -> (App, ComponentRoot, FileTree, PollWatcher) {
    init_env();
    let mut app = App::default();
    app.set_width(VIEW_WIDTH);
    app.mode = Mode::View;
    let mut md = parse_markdown(None, MD_WITH_LINKS, VIEW_WIDTH - 2);
    md.set_scroll(0);
    let ft = make_file_tree(&[("dummy.md", "Dummy")]);
    let w = make_watcher();
    (app, md, ft, w)
}

// --- View / Boxes::None scroll tests ---

#[test]
fn view_j_scrolls_down() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    let before = app.vertical_scroll;
    handle_keyboard_input(&key('j'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.vertical_scroll, before + 1);
}

#[test]
fn view_k_scrolls_up() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    // Scroll down first so we can scroll up
    app.vertical_scroll = 5;
    handle_keyboard_input(&key('k'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.vertical_scroll, 4);
}

#[test]
fn view_g_to_top() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    app.vertical_scroll = 10;
    handle_keyboard_input(&key('g'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.vertical_scroll, 0);
}

#[test]
fn view_g_to_bottom() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    app.vertical_scroll = 0;
    handle_keyboard_input(&key('G'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.vertical_scroll, md.height().saturating_sub(HEIGHT / 2));
}

#[test]
fn view_s_selects_link() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    assert!(!app.selected);

    handle_keyboard_input(&key('s'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);

    assert!(app.selected, "s should enter link selection mode");
    // A link should now be selected
    assert!(md.num_links() > 0, "fixture must contain links");
}

#[test]
fn view_q_direct_file_exits() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    app.direct_file = true;
    let result = handle_keyboard_input(&key('q'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(matches!(result, KeyBoardAction::Exit));
}

#[test]
fn view_q_returns_to_filetree() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    app.direct_file = false;
    handle_keyboard_input(&key('q'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.mode, Mode::FileTree);
}

#[test]
fn view_f_opens_search() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    handle_keyboard_input(&key('f'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.boxes, Boxes::Search);
}

#[test]
fn view_e_returns_edit() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    let result = handle_keyboard_input(&key('e'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(matches!(result, KeyBoardAction::Edit));
}

#[test]
fn view_t_to_filetree() {
    let (mut app, mut md, mut ft, mut w) = setup_view();
    handle_keyboard_input(&key('t'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.mode, Mode::FileTree);
}

// --- View / Boxes::Search tests ---

#[test]
fn view_search_enter_marks_results() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Open search
    handle_keyboard_input(&key('f'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.boxes, Boxes::Search);

    // Type "paragraph"
    for c in "paragraph".chars() {
        handle_keyboard_input(&key(c), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    }

    // Press Enter to execute search
    handle_keyboard_input(&key_code(KeyCode::Enter), &mut app, &mut md, &mut ft, HEIGHT, &mut w);

    // Search box should close on successful search
    assert_eq!(app.boxes, Boxes::None);

    // There should be search results
    let heights = md.search_results_heights();
    assert!(!heights.is_empty(), "search for 'paragraph' should find results");
}

#[test]
fn view_search_no_results_shows_error() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Open search and type a string with no matches
    app.boxes = Boxes::Search;
    for c in "zzzzzznotfound".chars() {
        handle_keyboard_input(&key(c), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    }

    handle_keyboard_input(&key_code(KeyCode::Enter), &mut app, &mut md, &mut ft, HEIGHT, &mut w);

    // Should show error box
    assert_eq!(app.boxes, Boxes::Error);
}

#[test]
fn view_search_esc_closes() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    app.boxes = Boxes::Search;
    app.search_box.insert('x');

    handle_keyboard_input(&key_code(KeyCode::Esc), &mut app, &mut md, &mut ft, HEIGHT, &mut w);

    assert_eq!(app.boxes, Boxes::None);
    assert_eq!(app.search_box.content(), None);
}

// --- View / selected link navigation tests ---

#[test]
fn view_selected_j_next_link() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Select first link
    handle_keyboard_input(&key('s'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(app.selected);
    let first_index = app.select_index;

    // Move to next link
    handle_keyboard_input(&key('j'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(
        app.select_index > first_index || md.num_links() == 1,
        "j in selected mode should advance to next link"
    );
}

#[test]
fn view_selected_k_prev_link() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Select link, then move forward, then back
    handle_keyboard_input(&key('s'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    handle_keyboard_input(&key('j'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    let after_j = app.select_index;

    handle_keyboard_input(&key('k'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(
        app.select_index < after_j || after_j == 0,
        "k in selected mode should go to previous link"
    );
}

#[test]
fn view_selected_esc_deselects() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Select a link
    handle_keyboard_input(&key('s'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(app.selected);

    // Esc should deselect
    handle_keyboard_input(&key_code(KeyCode::Esc), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(!app.selected);
}

#[test]
fn view_selected_k_hover() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Select a link first
    handle_keyboard_input(&key('s'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert!(app.selected);

    // K (uppercase) should open LinkPreview
    handle_keyboard_input(&key('K'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.boxes, Boxes::LinkPreview);
}

// --- View / Boxes::LinkPreview tests ---

#[test]
fn view_linkpreview_esc_closes() {
    let (mut app, mut md, mut ft, mut w) = setup_view();

    // Get into LinkPreview state
    handle_keyboard_input(&key('s'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    handle_keyboard_input(&key('K'), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.boxes, Boxes::LinkPreview);

    // Esc closes the preview
    handle_keyboard_input(&key_code(KeyCode::Esc), &mut app, &mut md, &mut ft, HEIGHT, &mut w);
    assert_eq!(app.boxes, Boxes::None);
}
