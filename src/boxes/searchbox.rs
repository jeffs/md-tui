use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

#[derive(Debug, Clone)]
pub struct SearchBox {
    pub text: String,
    pub cursor: usize,
    height: u16,
    width: u16,
    x: u16,
    y: u16,
}

impl SearchBox {
    #[must_use]
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            height: 2,
            width: 20,
            x: 0,
            y: 0,
        }
    }

    pub fn insert(&mut self, c: char) {
        self.text.push(c);
        self.cursor += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    #[must_use]
    pub fn dimensions(&self) -> (u16, u16) {
        (self.height, self.width)
    }

    pub fn consume(&mut self) -> String {
        let text = self.text.clone();
        self.clear();
        text
    }

    #[must_use]
    pub fn content_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn content(&self) -> Option<&str> {
        if self.text.is_empty() {
            None
        } else {
            Some(&self.text)
        }
    }

    pub fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_width(&mut self, width: u16) {
        self.width = width;
    }

    #[must_use]
    pub fn x(&self) -> u16 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> u16 {
        self.y
    }
}

impl Default for SearchBox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searchbox_insert_delete() {
        let mut sb = SearchBox::new();
        sb.insert('a');
        sb.insert('b');
        sb.insert('c');
        assert_eq!(sb.content_str(), "abc");
        assert_eq!(sb.cursor, 3);

        sb.delete();
        assert_eq!(sb.content_str(), "ab");
        assert_eq!(sb.cursor, 2);

        // delete on empty is a no-op
        sb.delete();
        sb.delete();
        sb.delete();
        assert_eq!(sb.content_str(), "");
        assert_eq!(sb.cursor, 0);
    }

    #[test]
    fn searchbox_clear() {
        let mut sb = SearchBox::new();
        sb.insert('x');
        sb.insert('y');
        assert_eq!(sb.cursor, 2);

        sb.clear();
        assert_eq!(sb.content_str(), "");
        assert_eq!(sb.cursor, 0);
    }

    #[test]
    fn searchbox_consume() {
        let mut sb = SearchBox::new();
        sb.insert('h');
        sb.insert('i');

        let consumed = sb.consume();
        assert_eq!(consumed, "hi");
        // consume clears the box
        assert_eq!(sb.content_str(), "");
        assert_eq!(sb.cursor, 0);
    }

    #[test]
    fn searchbox_content_empty_is_none() {
        let sb = SearchBox::new();
        assert_eq!(sb.content(), None);
    }

    #[test]
    fn searchbox_content_nonempty_is_some() {
        let mut sb = SearchBox::new();
        sb.insert('z');
        assert_eq!(sb.content(), Some("z"));
    }
}

impl Widget for SearchBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(self.text)
            .block(Block::default().borders(Borders::BOTTOM))
            .wrap(Wrap { trim: true });
        paragraph.render(area, buf);
    }
}
