use std::{cmp, io};

use crossterm::{
    cursor,
    event::DisableMouseCapture,
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use general::GENERAL_CONFIG;

use crate::boxes::{errorbox::ErrorBox, help_box::HelpBox, linkbox::LinkBox, searchbox::SearchBox};

pub mod colors;
pub mod general;
pub mod keys;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Mode {
    View,
    #[default]
    FileTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Boxes {
    Error,
    Search,
    LinkPreview,
    #[default]
    None,
}

impl From<JumpHistory> for Mode {
    fn from(jump_history: JumpHistory) -> Self {
        match jump_history.history.last() {
            Some(jump) => match jump {
                Jump::File(_) => Mode::View,
                Jump::FileTree => Mode::FileTree,
            },
            None => Mode::FileTree,
        }
    }
}

#[derive(Default, Clone)]
pub struct App {
    pub vertical_scroll: u16,
    width: u16,
    pub selected: bool,
    pub select_index: usize,
    pub mode: Mode,
    pub boxes: Boxes,
    pub history: JumpHistory,
    pub search_box: SearchBox,
    pub message_box: ErrorBox,
    pub help_box: HelpBox,
    pub link_box: LinkBox,
    pub direct_file: bool,
}

impl App {
    pub fn reset(&mut self) {
        self.vertical_scroll = 0;
        self.selected = false;
        self.select_index = 0;
        self.boxes = Boxes::None;
        self.help_box.close();
    }

    pub fn set_width(&mut self, width: u16) -> bool {
        let temp_width = self.width;
        self.width = cmp::min(width, GENERAL_CONFIG.width);
        temp_width != self.width
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }
}

pub enum LinkType<'a> {
    Internal(&'a str),
    External(&'a str),
    MarkdownFile(&'a str),
}

impl<'a> From<&'a str> for LinkType<'a> {
    fn from(s: &'a str) -> Self {
        if s.starts_with('#') {
            return Self::Internal(s);
        }
        if s.ends_with("md") || !s.contains('.') {
            return Self::MarkdownFile(s);
        }
        Self::External(s)
    }
}

pub fn destruct_terminal() {
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    execute!(io::stdout(), cursor::Show).unwrap();
}

#[derive(Debug, Clone)]
pub struct JumpHistory {
    history: Vec<Jump>,
}

impl JumpHistory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn push(&mut self, jump: Jump) {
        self.history.push(jump);
    }

    pub fn pop(&mut self) -> Jump {
        if let Some(jump) = self.history.pop() {
            jump
        } else {
            Jump::FileTree
        }
    }
}

impl Default for JumpHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Jump {
    File(String),
    FileTree,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_history() {
        let mut jump_history = JumpHistory::default();
        jump_history.push(Jump::File("file".to_string()));
        jump_history.push(Jump::File("file2".to_string()));
        jump_history.push(Jump::FileTree);
        assert_eq!(jump_history.pop(), Jump::FileTree);
        assert_eq!(jump_history.pop(), Jump::File("file2".to_string()));
        assert_eq!(jump_history.pop(), Jump::File("file".to_string()));
        assert_eq!(jump_history.pop(), Jump::FileTree);
        assert_eq!(jump_history.pop(), Jump::FileTree);
        assert_eq!(jump_history.pop(), Jump::FileTree);
    }

    // T11: jump_history_empty_pops_filetree — covered by test_jump_history above
    // (the last 3 asserts verify empty-pop → FileTree). Not duplicated per spec.

    #[test]
    fn link_type_internal() {
        let link = LinkType::from("#heading");
        assert!(matches!(link, LinkType::Internal("#heading")));
    }

    #[test]
    fn link_type_external_http() {
        let link = LinkType::from("https://example.com/page.html");
        assert!(matches!(link, LinkType::External("https://example.com/page.html")));
    }

    #[test]
    fn link_type_markdown_file_md() {
        let link = LinkType::from("other.md");
        assert!(matches!(link, LinkType::MarkdownFile("other.md")));
    }

    #[test]
    fn link_type_markdown_file_no_ext() {
        let link = LinkType::from("other");
        assert!(matches!(link, LinkType::MarkdownFile("other")));
    }

    #[test]
    fn link_type_external_dot_rs() {
        let link = LinkType::from("file.rs");
        assert!(matches!(link, LinkType::External("file.rs")));
    }

    #[test]
    fn app_reset_clears_state() {
        let mut app = App::default();
        app.vertical_scroll = 42;
        app.selected = true;
        app.select_index = 7;
        app.boxes = Boxes::Search;

        app.reset();

        assert_eq!(app.vertical_scroll, 0);
        assert!(!app.selected);
        assert_eq!(app.select_index, 0);
        assert_eq!(app.boxes, Boxes::None);
    }

    #[test]
    fn app_set_width_clamps_to_config() {
        let mut app = App::default();
        let max = GENERAL_CONFIG.width;

        // Setting width far above config max should clamp
        app.set_width(u16::MAX);
        assert_eq!(app.width(), max);

        // Setting width below config max should use the given value
        if max > 10 {
            app.set_width(10);
            assert_eq!(app.width(), 10);
        }
    }

    #[test]
    fn app_set_width_returns_changed() {
        let mut app = App::default();

        // First set: 0 → some value, should return true (unless width was already 50)
        let changed = app.set_width(50);
        // Default width is 0, so 0 → min(50, config.width) should change
        assert!(changed, "expected change from default width");

        // Set same value again: no change
        let current = app.width();
        let changed = app.set_width(current);
        assert!(!changed, "expected no change when setting same width");

        // Set different value: should change
        let new = if current > 1 { current - 1 } else { current + 1 };
        let changed = app.set_width(new);
        assert!(changed, "expected change when setting different width");
    }
}
