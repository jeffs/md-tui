use crate::search::{compare_heading, find_and_mark};

use super::{
    image::ImageComponent,
    textcomponent::{TextComponent, TextNode},
    word::{Word, WordType},
};

pub struct ComponentRoot {
    file_name: Option<String>,
    components: Vec<Component>,
    is_focused: bool,
}

impl ComponentRoot {
    #[must_use]
    pub fn new(file_name: Option<String>, components: Vec<Component>) -> Self {
        Self {
            file_name,
            components,
            is_focused: false,
        }
    }

    #[must_use]
    pub fn children(&self) -> Vec<&Component> {
        self.components.iter().collect()
    }

    pub fn children_mut(&mut self) -> Vec<&mut Component> {
        self.components.iter_mut().collect()
    }

    #[must_use]
    pub fn components(&self) -> Vec<&TextComponent> {
        self.components
            .iter()
            .filter_map(|c| match c {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .collect()
    }

    pub fn components_mut(&mut self) -> Vec<&mut TextComponent> {
        self.components
            .iter_mut()
            .filter_map(|c| match c {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .collect()
    }

    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    #[must_use]
    pub fn words(&self) -> Vec<&Word> {
        self.components
            .iter()
            .filter_map(|c| match c {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .flat_map(|c| c.content().iter().flatten())
            .collect()
    }

    pub fn find_and_mark(&mut self, search: &str) {
        let mut words = self
            .components
            .iter_mut()
            .filter_map(|c| match c {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .flat_map(|c| c.words_mut())
            .collect::<Vec<_>>();
        find_and_mark(search, &mut words);
    }

    #[must_use]
    pub fn search_results_heights(&self) -> Vec<usize> {
        self.components
            .iter()
            .filter_map(|c| match c {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .flat_map(|c| {
                let mut heights = c.selected_heights();
                heights.iter_mut().for_each(|h| *h += c.y_offset() as usize);
                heights
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.file_name = None;
        self.components.clear();
    }

    pub fn select(&mut self, index: usize) -> Result<u16, String> {
        self.deselect();
        self.is_focused = true;
        let mut count = 0;
        for comp in self.components.iter_mut().filter_map(|f| match f {
            Component::TextComponent(comp) => Some(comp),
            Component::Image(_) => None,
        }) {
            let link_inside_comp = index - count < comp.num_links();
            if link_inside_comp {
                comp.visually_select(index - count)?;
                return Ok(comp.y_offset());
            }
            count += comp.num_links();
        }
        Err(format!("Index out of bounds: {index} >= {count}"))
    }

    pub fn deselect(&mut self) {
        self.is_focused = false;
        for comp in self.components.iter_mut().filter_map(|f| match f {
            Component::TextComponent(comp) => Some(comp),
            Component::Image(_) => None,
        }) {
            comp.deselect();
        }
    }

    #[must_use]
    pub fn find_footnote(&self, search: &str) -> String {
        let footnote = self
            .components
            .iter()
            .filter_map(|f| match f {
                Component::TextComponent(text_component) => {
                    if text_component.kind() == TextNode::Footnote {
                        Some(text_component)
                    } else {
                        None
                    }
                }
                Component::Image(_) => None,
            })
            .filter(|f| {
                if let Some(foot_ref) = f.meta_info().iter().next() {
                    foot_ref.content() == search
                } else {
                    false
                }
            })
            .flat_map(|f| f.content().iter().flatten())
            .filter(|f| f.kind() == WordType::Footnote)
            .map(Word::content)
            .collect::<String>();

        if footnote.is_empty() {
            String::from("Footnote not found")
        } else {
            footnote
        }
    }

    #[must_use]
    pub fn link_index_and_height(&self) -> Vec<(usize, u16)> {
        let mut indexes = Vec::new();
        let mut count = 0;
        self.components
            .iter()
            .filter_map(|f| match f {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .for_each(|comp| {
                let height = comp.y_offset();
                comp.content().iter().enumerate().for_each(|(index, row)| {
                    row.iter().for_each(|c| {
                        if matches!(
                            c.kind(),
                            WordType::Link | WordType::Selected | WordType::FootnoteInline
                        ) {
                            indexes.push((count, height + index as u16));
                            count += 1;
                        }
                    });
                });
            });

        indexes
    }

    /// Sets the y offset of the components
    pub fn set_scroll(&mut self, scroll: u16) {
        let mut y_offset = 0;
        for component in &mut self.components {
            component.set_y_offset(y_offset);
            component.set_scroll_offset(scroll);
            y_offset += component.height();
        }
    }

    pub fn heading_offset(&self, heading: &str) -> Result<u16, String> {
        let mut y_offset = 0;
        for component in &self.components {
            match component {
                Component::TextComponent(comp) => {
                    if comp.kind() == TextNode::Heading
                        && compare_heading(&heading[1..], comp.content())
                    {
                        return Ok(y_offset);
                    }
                    y_offset += comp.height();
                }
                Component::Image(e) => y_offset += e.height(),
            }
        }
        Err(format!("Heading not found: {heading}"))
    }

    /// Return the content of the components, where each element a line
    #[must_use]
    pub fn content(&self) -> Vec<String> {
        self.components()
            .iter()
            .flat_map(|c| c.content_as_lines())
            .collect()
    }

    #[must_use]
    pub fn selected(&self) -> &str {
        let block = self
            .components
            .iter()
            .filter_map(|f| match f {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .find(|c| c.is_focused())
            .unwrap();
        block.highlight_link().unwrap()
    }

    #[must_use]
    pub fn selected_underlying_type(&self) -> WordType {
        let selected = self
            .components
            .iter()
            .filter_map(|f| match f {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .find(|c| c.is_focused())
            .unwrap()
            .content()
            .iter()
            .flatten()
            .filter(|c| c.kind() == WordType::Selected)
            .collect::<Vec<_>>();

        selected.first().unwrap().previous_type()
    }

    /// Transforms the content of the components to fit the given width
    pub fn transform(&mut self, width: u16) {
        for component in self.components_mut() {
            component.transform(width);
        }
    }

    /// Because of the parsing, every table has a missing newline at the end
    #[must_use]
    pub fn add_missing_components(self) -> Self {
        let mut components = Vec::new();
        let mut iter = self.components.into_iter().peekable();
        while let Some(component) = iter.next() {
            let kind = component.kind();
            components.push(component);
            if let Some(next) = iter.peek()
                && kind != TextNode::LineBreak
                && next.kind() != TextNode::LineBreak
            {
                // Don't insert LineBreak between Task and its
                // indented subitems
                let is_task_with_subitems = kind == TextNode::Task
                    && matches!(
                        next,
                        Component::TextComponent(tc)
                            if tc.is_indented_list()
                    );

                if !is_task_with_subitems {
                    components.push(Component::TextComponent(
                        TextComponent::new(
                            TextNode::LineBreak,
                            Vec::new(),
                        ),
                    ));
                }
            }
        }
        Self {
            file_name: self.file_name,
            components,
            is_focused: self.is_focused,
        }
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.components.iter().map(ComponentProps::height).sum()
    }

    #[must_use]
    pub fn num_links(&self) -> usize {
        self.components
            .iter()
            .filter_map(|f| match f {
                Component::TextComponent(comp) => Some(comp),
                Component::Image(_) => None,
            })
            .map(TextComponent::num_links)
            .sum()
    }
}

pub trait ComponentProps {
    fn height(&self) -> u16;
    fn set_y_offset(&mut self, y_offset: u16);
    fn set_scroll_offset(&mut self, scroll: u16);
    fn kind(&self) -> TextNode;
}

pub enum Component {
    TextComponent(TextComponent),
    Image(ImageComponent),
}

impl From<TextComponent> for Component {
    fn from(comp: TextComponent) -> Self {
        Component::TextComponent(comp)
    }
}

impl ComponentProps for Component {
    fn height(&self) -> u16 {
        match self {
            Component::TextComponent(comp) => comp.height(),
            Component::Image(comp) => comp.height(),
        }
    }

    fn set_y_offset(&mut self, y_offset: u16) {
        match self {
            Component::TextComponent(comp) => comp.set_y_offset(y_offset),
            Component::Image(comp) => comp.set_y_offset(y_offset),
        }
    }

    fn set_scroll_offset(&mut self, scroll: u16) {
        match self {
            Component::TextComponent(comp) => comp.set_scroll_offset(scroll),
            Component::Image(comp) => comp.set_scroll_offset(scroll),
        }
    }

    fn kind(&self) -> TextNode {
        match self {
            Component::TextComponent(comp) => comp.kind(),
            Component::Image(comp) => comp.kind(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::word::{MetaData, Word, WordType};

    /// Helper: build a TextComponent with pre-formatted content lines and a given height.
    fn tc_formatted(kind: TextNode, lines: Vec<Vec<Word>>) -> TextComponent {
        TextComponent::new_formatted(kind, lines)
    }

    /// Helper: build a simple paragraph TextComponent from a flat word list.
    fn tc(kind: TextNode, words: Vec<Word>) -> TextComponent {
        TextComponent::new(kind, words)
    }

    fn word(s: &str, wt: WordType) -> Word {
        Word::new(s.to_string(), wt)
    }

    fn root(components: Vec<Component>) -> ComponentRoot {
        ComponentRoot::new(None, components)
    }

    // ── height ──────────────────────────────────────────────────

    #[test]
    fn height_sums_children() {
        // new_formatted sets height = number of lines
        let c1 = tc_formatted(
            TextNode::Paragraph,
            vec![
                vec![word("line1", WordType::Normal)],
                vec![word("line2", WordType::Normal)],
            ],
        );
        let c2 = tc_formatted(
            TextNode::Paragraph,
            vec![vec![word("line3", WordType::Normal)]],
        );
        let r = root(vec![c1.into(), c2.into()]);
        assert_eq!(r.height(), 3);
    }

    // ── num_links ───────────────────────────────────────────────

    #[test]
    fn num_links() {
        // num_links counts LinkData and FootnoteInline in meta_info.
        // TextComponent::new filters non-renderable words into meta_info,
        // but FootnoteInline is also copied to meta_info despite being renderable.
        let c1 = tc(
            TextNode::Paragraph,
            vec![
                word("click ", WordType::Normal),
                word("here", WordType::Link),
                word("https://example.com", WordType::LinkData),
            ],
        );
        let c2 = tc(
            TextNode::Paragraph,
            vec![
                word("see", WordType::Normal),
                word("[1]", WordType::FootnoteInline),
            ],
        );
        let r = root(vec![c1.into(), c2.into()]);
        assert_eq!(r.num_links(), 2);
    }

    // ── select / deselect ───────────────────────────────────────

    #[test]
    fn select_deselect() {
        let c = tc(
            TextNode::Paragraph,
            vec![
                word("click ", WordType::Normal),
                word("here", WordType::Link),
                word("https://example.com", WordType::LinkData),
            ],
        );
        let mut r = root(vec![c.into()]);
        // Transform to give height (select returns y_offset, which requires set_scroll first)
        r.set_scroll(0);

        // Select the first (only) link
        assert!(r.select(0).is_ok());

        // The link word should now be Selected
        let selected_words: Vec<_> = r
            .words()
            .into_iter()
            .filter(|w| w.kind() == WordType::Selected)
            .collect();
        assert!(!selected_words.is_empty());

        // Deselect restores original type
        r.deselect();
        let selected_after: Vec<_> = r
            .words()
            .into_iter()
            .filter(|w| w.kind() == WordType::Selected)
            .collect();
        assert!(selected_after.is_empty());
    }

    #[test]
    fn select_out_of_bounds() {
        let c = tc(
            TextNode::Paragraph,
            vec![
                word("text", WordType::Normal),
                word("link", WordType::Link),
                word("url", WordType::LinkData),
            ],
        );
        let mut r = root(vec![c.into()]);
        assert!(r.select(5).is_err());
    }

    // ── words ───────────────────────────────────────────────────

    #[test]
    fn words_flattens_all() {
        let c1 = tc_formatted(
            TextNode::Paragraph,
            vec![
                vec![word("a", WordType::Normal)],
                vec![word("b", WordType::Bold)],
            ],
        );
        let c2 = tc_formatted(
            TextNode::Paragraph,
            vec![vec![word("c", WordType::Italic)]],
        );
        let r = root(vec![c1.into(), c2.into()]);
        let all: Vec<&str> = r.words().iter().map(|w| w.content()).collect();
        assert_eq!(all, vec!["a", "b", "c"]);
    }

    // ── find_footnote ───────────────────────────────────────────

    #[test]
    fn find_footnote_found() {
        // A Footnote component has FootnoteData meta + Footnote-typed words.
        // find_footnote looks for kind == TextNode::Footnote, then checks
        // meta_info first element content matches, then collects Footnote words.
        let c = tc(
            TextNode::Footnote,
            vec![
                word("1", WordType::FootnoteData),
                word("This is the footnote.", WordType::Footnote),
            ],
        );
        let r = root(vec![c.into()]);
        assert_eq!(r.find_footnote("1"), "This is the footnote.");
    }

    #[test]
    fn find_footnote_not_found() {
        let c = tc(
            TextNode::Footnote,
            vec![
                word("1", WordType::FootnoteData),
                word("content", WordType::Footnote),
            ],
        );
        let r = root(vec![c.into()]);
        assert_eq!(r.find_footnote("999"), "Footnote not found");
    }

    // ── heading_offset ──────────────────────────────────────────

    #[test]
    fn heading_offset_found() {
        // heading_offset strips the leading '#' from the search string,
        // then compares with compare_heading. We need a Heading component
        // whose content words match.
        // The heading search expects format "#slug" where slug is built from
        // lowercase words joined with '-'. We pass "#title" and the heading
        // content is ["Title"].
        let h = tc_formatted(
            TextNode::Heading,
            vec![vec![word("Title", WordType::Normal)]],
        );
        let p = tc_formatted(
            TextNode::Paragraph,
            vec![
                vec![word("some", WordType::Normal)],
                vec![word("text", WordType::Normal)],
            ],
        );
        let r = root(vec![h.into(), p.into()]);
        // heading_offset accumulates y from component heights.
        // The heading itself is height 1 (new_formatted with 1 line).
        // It's the first component, so offset should be 0.
        assert_eq!(r.heading_offset("#title"), Ok(0));
    }

    #[test]
    fn heading_offset_not_found() {
        let h = tc_formatted(
            TextNode::Heading,
            vec![vec![word("Title", WordType::Normal)]],
        );
        let r = root(vec![h.into()]);
        assert!(r.heading_offset("#nonexistent").is_err());
    }

    // ── add_missing_components ──────────────────────────────────

    #[test]
    fn add_missing_components_inserts_linebreaks() {
        // Two adjacent paragraphs (neither is LineBreak) should get a
        // LineBreak inserted between them.
        let c1 = tc(TextNode::Paragraph, vec![word("a", WordType::Normal)]);
        let c2 = tc(TextNode::Paragraph, vec![word("b", WordType::Normal)]);
        let r = root(vec![c1.into(), c2.into()]).add_missing_components();

        let kinds: Vec<TextNode> = r.components().iter().map(|c| c.kind()).collect();
        assert_eq!(
            kinds,
            vec![TextNode::Paragraph, TextNode::LineBreak, TextNode::Paragraph]
        );
    }

    #[test]
    fn add_missing_components_task_sublist_no_linebreak() {
        // A Task followed by an indented List should NOT get a LineBreak.
        // is_indented_list requires kind == List and meta_info containing
        // a word whose content is whitespace-only and non-empty.
        let task = tc(TextNode::Task, vec![word("todo", WordType::Normal)]);

        // Build an indented list: it needs a meta_info word with whitespace
        // content (trim is empty but content is non-empty).
        let indented_list = tc(
            TextNode::List,
            vec![
                word("  ", WordType::MetaInfo(MetaData::Other)),
                word("sub", WordType::Normal),
                word("• ", WordType::ListMarker),
                word("  ", WordType::MetaInfo(MetaData::UList)),
            ],
        );
        assert!(
            indented_list.is_indented_list(),
            "precondition: should be recognized as indented list"
        );

        let r = root(vec![task.into(), indented_list.into()]).add_missing_components();
        let kinds: Vec<TextNode> = r.components().iter().map(|c| c.kind()).collect();
        // No LineBreak should be inserted between Task and indented List
        assert_eq!(kinds, vec![TextNode::Task, TextNode::List]);
    }

    // ── set_scroll ──────────────────────────────────────────────

    #[test]
    fn set_scroll_propagates() {
        let c1 = tc_formatted(
            TextNode::Paragraph,
            vec![
                vec![word("line1", WordType::Normal)],
                vec![word("line2", WordType::Normal)],
            ],
        ); // height = 2
        let c2 = tc_formatted(
            TextNode::Paragraph,
            vec![vec![word("line3", WordType::Normal)]],
        ); // height = 1
        let mut r = root(vec![c1.into(), c2.into()]);

        let scroll_value = 5;
        r.set_scroll(scroll_value);

        let comps = r.components();
        // First component: y_offset = 0, scroll_offset = 5
        assert_eq!(comps[0].y_offset(), 0);
        assert_eq!(comps[0].scroll_offset(), scroll_value);
        // Second component: y_offset = 2 (first component's height), scroll_offset = 5
        assert_eq!(comps[1].y_offset(), 2);
        assert_eq!(comps[1].scroll_offset(), scroll_value);
    }

    // ── content ─────────────────────────────────────────────────

    #[test]
    fn content_returns_lines() {
        let c1 = tc_formatted(
            TextNode::Paragraph,
            vec![
                vec![word("hello ", WordType::Normal), word("world", WordType::Bold)],
                vec![word("second", WordType::Normal)],
            ],
        );
        let c2 = tc_formatted(
            TextNode::Paragraph,
            vec![vec![word("third", WordType::Normal)]],
        );
        let r = root(vec![c1.into(), c2.into()]);
        let lines = r.content();
        assert_eq!(lines, vec!["hello world", "second", "third"]);
    }
}
