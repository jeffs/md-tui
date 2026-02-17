use std::cmp;

use itertools::Itertools;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use ratatui::style::Color;
use tree_sitter_highlight::HighlightEvent;

use crate::{
    highlight::{COLOR_MAP, HighlightInfo, highlight_code},
    nodes::word::MetaData,
};

use super::word::{Word, WordType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextNode {
    Image,
    Paragraph,
    LineBreak,
    Heading,
    Task,
    List,
    Footnote,
    /// (`widths_by_column`, `heights_by_row`)
    Table(Vec<u16>, Vec<u16>),
    CodeBlock,
    Quote,
    HorizontalSeparator,
}

#[derive(Debug, Clone)]
pub struct TextComponent {
    kind: TextNode,
    content: Vec<Vec<Word>>,
    meta_info: Vec<Word>,
    height: u16,
    offset: u16,
    scroll_offset: u16,
    focused: bool,
    focused_index: usize,
}

impl TextComponent {
    #[must_use]
    pub fn new(kind: TextNode, content: Vec<Word>) -> Self {
        let meta_info: Vec<Word> = content
            .iter()
            .filter(|c| !c.is_renderable() || c.kind() == WordType::FootnoteInline)
            .cloned()
            .collect();

        let content = content.into_iter().filter(Word::is_renderable).collect();

        Self {
            kind,
            content: vec![content],
            meta_info,
            height: 0,
            offset: 0,
            scroll_offset: 0,
            focused: false,
            focused_index: 0,
        }
    }

    #[must_use]
    pub fn new_formatted(kind: TextNode, content: Vec<Vec<Word>>) -> Self {
        let meta_info: Vec<Word> = content
            .iter()
            .flatten()
            .filter(|c| !c.is_renderable())
            .cloned()
            .collect();

        let content = content
            .into_iter()
            .map(|c| c.into_iter().filter(Word::is_renderable).collect::<Vec<Word>>())
            .filter(|c| !c.is_empty())
            .collect::<Vec<Vec<Word>>>();

        Self {
            kind,
            height: content.len() as u16,
            meta_info,
            content,
            offset: 0,
            scroll_offset: 0,
            focused: false,
            focused_index: 0,
        }
    }

    #[must_use]
    pub fn kind(&self) -> TextNode {
        self.kind.clone()
    }

    /// Returns true if this is an indented (sub-item) list.
    #[must_use]
    pub fn is_indented_list(&self) -> bool {
        if self.kind != TextNode::List {
            return false;
        }
        // First meta_info word with empty trim is the indent;
        // check if non-empty (meaning it has whitespace = indented)
        self.meta_info
            .iter()
            .find(|w| w.content().trim().is_empty())
            .is_some_and(|w| !w.content().is_empty())
    }

    #[must_use]
    pub fn content(&self) -> &Vec<Vec<Word>> {
        &self.content
    }

    #[must_use]
    pub fn content_as_lines(&self) -> Vec<String> {
        if let TextNode::Table(widths, _) = self.kind() {
            let column_count = widths.len();
            if column_count == 0 {
                return Vec::new();
            }

            let moved_content = self.content.chunks(column_count).collect::<Vec<_>>();

            let mut lines = Vec::new();

            moved_content.iter().for_each(|line| {
                let temp = line
                    .iter()
                    .map(|c| c.iter().map(Word::content).join(""))
                    .join(" ");
                lines.push(temp);
            });

            lines
        } else {
            self.content
                .iter()
                .map(|c| c.iter().map(Word::content).collect::<Vec<_>>().join(""))
                .collect()
        }
    }

    #[must_use]
    pub fn content_as_bytes(&self) -> Vec<u8> {
        match self.kind() {
            TextNode::CodeBlock => self.content_as_lines().join("").as_bytes().to_vec(),
            _ => {
                let strings = self.content_as_lines();
                let string = strings.join("\n");
                string.as_bytes().to_vec()
            }
        }
    }

    #[must_use]
    pub fn content_owned(self) -> Vec<Vec<Word>> {
        self.content
    }

    #[must_use]
    pub fn meta_info(&self) -> &Vec<Word> {
        &self.meta_info
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[must_use]
    pub fn y_offset(&self) -> u16 {
        self.offset
    }

    #[must_use]
    pub fn scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    pub fn set_y_offset(&mut self, y_offset: u16) {
        self.offset = y_offset;
    }

    pub fn set_scroll_offset(&mut self, offset: u16) {
        self.scroll_offset = offset;
    }

    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn deselect(&mut self) {
        self.focused = false;
        self.focused_index = 0;
        self.content
            .iter_mut()
            .flatten()
            .filter(|c| c.kind() == WordType::Selected)
            .for_each(|c| {
                c.clear_kind();
            });
    }

    pub fn visually_select(&mut self, index: usize) -> Result<(), String> {
        self.focused = true;
        self.focused_index = index;

        if index >= self.num_links() {
            return Err(format!(
                "Index out of bounds: {} >= {}",
                index,
                self.num_links()
            ));
        }

        // Transform nth link to selected
        self.link_words_mut()
            .get_mut(index)
            .ok_or("index out of bounds")?
            .iter_mut()
            .for_each(|c| {
                c.set_kind(WordType::Selected);
            });
        Ok(())
    }

    fn link_words_mut(&mut self) -> Vec<Vec<&mut Word>> {
        let mut selection: Vec<Vec<&mut Word>> = Vec::new();
        let mut iter = self.content.iter_mut().flatten().peekable();
        while let Some(e) = iter.peek() {
            if matches!(e.kind(), WordType::Link | WordType::FootnoteInline) {
                selection.push(
                    iter.by_ref()
                        .take_while(|c| {
                            matches!(c.kind(), WordType::Link | WordType::FootnoteInline)
                        })
                        .collect(),
                );
            } else {
                iter.next();
            }
        }
        selection
    }

    #[must_use]
    pub fn get_footnote(&self, search: &str) -> String {
        self.content()
            .iter()
            .flatten()
            .skip_while(|c| c.kind() != WordType::FootnoteData && c.content() != search)
            .take_while(|c| c.kind() == WordType::Footnote)
            .map(Word::content)
            .collect()
    }

    pub fn highlight_link(&self) -> Result<&str, String> {
        Ok(self
            .meta_info()
            .iter()
            .filter(|c| matches!(c.kind(), WordType::LinkData | WordType::FootnoteInline))
            .nth(self.focused_index)
            .ok_or("index out of bounds")?
            .content())
    }

    #[must_use]
    pub fn num_links(&self) -> usize {
        self.meta_info
            .iter()
            .filter(|c| matches!(c.kind(), WordType::LinkData | WordType::FootnoteInline))
            .count()
    }

    #[must_use]
    pub fn selected_heights(&self) -> Vec<usize> {
        let mut heights = Vec::new();

        if let TextNode::Table(widths, _) = self.kind() {
            let column_count = widths.len();
            let iter = self.content.chunks(column_count).enumerate();

            for (i, line) in iter {
                if line
                    .iter()
                    .flatten()
                    .any(|c| c.kind() == WordType::Selected)
                {
                    heights.push(i);
                }
            }
            return heights;
        }

        for (i, line) in self.content.iter().enumerate() {
            if line.iter().any(|c| c.kind() == WordType::Selected) {
                heights.push(i);
            }
        }
        heights
    }

    pub fn words_mut(&mut self) -> Vec<&mut Word> {
        self.content.iter_mut().flatten().collect()
    }

    pub fn transform(&mut self, width: u16) {
        match self.kind {
            TextNode::List => {
                transform_list(self, width);
            }
            TextNode::CodeBlock => {
                transform_codeblock(self);
            }
            TextNode::Paragraph | TextNode::Task | TextNode::Quote => {
                transform_paragraph(self, width);
            }
            TextNode::LineBreak | TextNode::Heading => {
                self.height = 1;
            }
            TextNode::Table(_, _) => {
                transform_table(self, width);
            }
            TextNode::HorizontalSeparator => self.height = 1,
            TextNode::Image => unreachable!("Image should not be transformed"),
            TextNode::Footnote => self.height = 0,
        }
    }
}

fn word_wrapping<'a>(
    words: impl IntoIterator<Item = &'a Word>,
    width: usize,
    allow_hyphen: bool,
) -> Vec<Vec<Word>> {
    let enable_hyphen = allow_hyphen && width > 4;

    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut line_len = 0;
    for word in words {
        // A leading newline in word content means a hard line break
        // (Claude flavor preserves \n; CommonMark converts it to
        // space before reaching here, so this only fires for Claude).
        if word.content().starts_with('\n') {
            lines.push(std::mem::take(&mut line));
            line_len = 0;
            let trimmed = word.content().trim_start_matches('\n');
            if !trimmed.is_empty() {
                let mut w = word.clone();
                w.set_content(trimmed.to_owned());
                line_len = display_width(trimmed);
                line.push(w);
            }
            continue;
        }

        let word_len = display_width(word.content());
        if line_len + word_len <= width {
            line_len += word_len;
            line.push(word.clone());
        } else if word_len <= width {
            lines.push(line);
            let mut word = word.clone();
            let content = word.content().trim_start().to_owned();
            word.set_content(content);

            line_len = display_width(word.content());
            line = vec![word];
        } else {
            let content = word.content().to_owned();

            if width - line_len < 4 {
                line_len = 0;
                lines.push(line);
                line = Vec::new();
            }

            let split_width = if enable_hyphen && !content.ends_with('-') {
                width - line_len - 1
            } else {
                width - line_len
            };

            let (mut content, mut newline_content) = split_by_width(&content, split_width);
            if enable_hyphen && !content.ends_with('-') && !content.is_empty() {
                if let Some(last_char) = content.pop() {
                    newline_content.insert(0, last_char);
                }
                content.push('-');
            }

            line.push(Word::new(content, word.kind()));
            lines.push(line);

            while display_width(&newline_content) > width {
                let split_width = if enable_hyphen && !newline_content.ends_with('-') {
                    width - 1
                } else {
                    width
                };
                let (mut content, mut next_newline_content) =
                    split_by_width(&newline_content, split_width);
                if enable_hyphen && !newline_content.ends_with('-') && !content.is_empty() {
                    if let Some(last_char) = content.pop() {
                        next_newline_content.insert(0, last_char);
                    }
                    content.push('-');
                }

                line = vec![Word::new(content, word.kind())];
                lines.push(line);
                newline_content = next_newline_content;
            }

            if newline_content.is_empty() {
                line_len = 0;
                line = Vec::new();
            } else {
                line_len = display_width(&newline_content);
                line = vec![Word::new(newline_content, word.kind())];
            }
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    lines
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn split_by_width(text: &str, max_width: usize) -> (String, String) {
    if max_width == 0 {
        return (String::new(), text.to_string());
    }

    let mut width = 0;
    let mut split_idx = 0;
    // Track the byte index where the visible width reaches (or just exceeds) max_width.
    for (i, c) in text.char_indices() {
        let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
        if width + char_width > max_width {
            if split_idx == 0 {
                split_idx = i + c.len_utf8();
            }
            break;
        }
        width += char_width;
        split_idx = i + c.len_utf8();
        if width == max_width {
            break;
        }
    }

    let (head, tail) = text.split_at(split_idx);
    (head.to_string(), tail.to_string())
}

fn transform_paragraph(component: &mut TextComponent, width: u16) {
    let width = match component.kind {
        TextNode::Paragraph => width as usize - 1,
        TextNode::Task => width as usize - 4,
        TextNode::Quote => width as usize - 2,
        _ => unreachable!(),
    };

    let mut lines = word_wrapping(component.content.iter().flatten(), width, true);

    if component.kind() == TextNode::Quote {
        let is_special_quote = !component.meta_info.is_empty();

        for line in lines.iter_mut().skip(usize::from(is_special_quote)) {
            line.insert(0, Word::new(" ".to_string(), WordType::Normal));
        }
    }

    component.height = lines.len() as u16;
    component.content = lines;
}

fn transform_codeblock(component: &mut TextComponent) {
    let language = if let Some(word) = component.meta_info().first() {
        word.content()
    } else {
        ""
    };

    let highlight = highlight_code(language, &component.content_as_bytes());

    let content = component.content_as_lines().join("");

    let mut new_content = Vec::new();

    if language.is_empty() {
        component.content.insert(
            0,
            vec![Word::new(String::new(), WordType::CodeBlock(Color::Reset))],
        );
    }
    match highlight {
        HighlightInfo::Highlighted(e) => {
            let mut color = Color::Reset;
            for event in e {
                match event {
                    HighlightEvent::Source { start, end } => {
                        let word =
                            Word::new(content[start..end].to_string(), WordType::CodeBlock(color));
                        new_content.push(word);
                    }
                    HighlightEvent::HighlightStart(index) => {
                        color = COLOR_MAP[index.0];
                    }
                    HighlightEvent::HighlightEnd => color = Color::Reset,
                }
            }

            // Find all the new lines to split the content correctly
            let mut final_content = Vec::new();
            let mut inner_content = Vec::new();
            for word in new_content {
                if word.content().contains('\n') {
                    let mut start = 0;
                    let mut end;
                    for (i, c) in word.content().char_indices() {
                        if c == '\n' {
                            end = i;
                            let new_word =
                                Word::new(word.content()[start..end].to_string(), word.kind());
                            inner_content.push(new_word);
                            start = i + 1;
                            final_content.push(inner_content);
                            inner_content = Vec::new();
                        } else if i == word.content().len() - 1 {
                            let new_word =
                                Word::new(word.content()[start..].to_string(), word.kind());
                            inner_content.push(new_word);
                        }
                    }
                } else {
                    inner_content.push(word);
                }
            }

            final_content.push(vec![Word::new(String::new(), WordType::CodeBlock(color))]);

            component.content = final_content;
        }
        HighlightInfo::Unhighlighted => (),
    }

    let height = component.content.len() as u16;
    component.height = height;
}

fn transform_list(component: &mut TextComponent, width: u16) {
    let mut len = 0;
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let indent_iter = component
        .meta_info
        .iter()
        .filter(|c| c.content().trim() == "");
    let list_type_iter = component.meta_info.iter().filter(|c| {
        matches!(
            c.kind(),
            WordType::MetaInfo(MetaData::OList | MetaData::UList)
        )
    });

    let mut zip_iter = indent_iter.zip(list_type_iter);

    let mut o_list_counter_stack = vec![0];
    let mut max_stack_len = 1;
    let mut indent = 0;
    let mut extra_indent = 0;
    let mut tmp = indent;
    for word in component.content.iter_mut().flatten() {
        let word_len = display_width(word.content());
        if word_len + len < width as usize && word.kind() != WordType::ListMarker {
            len += word_len;
            line.push(word.clone());
        } else {
            let filler_content = if word.kind() == WordType::ListMarker {
                indent = if let Some((meta, list_type)) = zip_iter.next() {
                    match tmp.cmp(&display_width(meta.content())) {
                        cmp::Ordering::Less => {
                            o_list_counter_stack.push(0);
                            max_stack_len += 1;
                        }
                        cmp::Ordering::Greater => {
                            o_list_counter_stack.pop();
                        }
                        cmp::Ordering::Equal => (),
                    }
                    if list_type.kind() == WordType::MetaInfo(MetaData::OList) {
                        let counter = o_list_counter_stack
                            .last_mut()
                            .expect("List parse error. Stack is empty");

                        *counter += 1;

                        word.set_content(format!("{counter}. "));

                        extra_indent = 1; // Ordered list is longer than unordered and needs extra space
                    } else {
                        extra_indent = 0;
                    }
                    tmp = display_width(meta.content());
                    tmp
                } else {
                    0
                };

                " ".repeat(indent)
            } else {
                " ".repeat(indent + 2 + extra_indent)
            };

            let filler = Word::new(filler_content, WordType::Normal);

            lines.push(line);
            let content = word.content().trim_start().to_owned();
            word.set_content(content);
            len = display_width(word.content()) + display_width(filler.content());
            line = vec![filler, word.to_owned()];
        }
    }
    lines.push(line);
    // Remove empty lines
    lines.retain(|l| l.iter().any(|c| c.content() != ""));

    // Find out if there are ordered indexes longer than 3 chars. F.ex. `1. ` is three chars, but `10. ` is four chars.
    // To align the list on the same column, we need to find the longest index and add the difference to the shorter indexes.
    let mut indent_correction = vec![0; max_stack_len];
    let mut indent_index: u32 = 0;
    let mut indent_len = 0;

    for line in &lines {
        if !line[1]
            .content()
            .strip_prefix(['1', '2', '3', '4', '5', '6', '7', '8', '9'])
            .is_some_and(|c| c.ends_with(". "))
        {
            continue;
        }

        match indent_len.cmp(&display_width(line[0].content())) {
            cmp::Ordering::Less => {
                indent_index += 1;
                indent_len = display_width(line[0].content());
            }
            cmp::Ordering::Greater => {
                indent_index = indent_index.saturating_sub(1);
                indent_len = display_width(line[0].content());
            }
            cmp::Ordering::Equal => (),
        }

        indent_correction[indent_index as usize] = cmp::max(
            indent_correction[indent_index as usize],
            display_width(line[1].content()),
        );
    }

    // Finally, apply the indent correction to the list for each ordered index which is shorter
    // than the longest index.

    indent_index = 0;
    indent_len = 0;
    let mut unordered_list_skip = true; // Skip unordered list items. They are already aligned.

    for line in &mut lines {
        if line[1]
            .content()
            .strip_prefix(['1', '2', '3', '4', '5', '6', '7', '8', '9'])
            .is_some_and(|c| c.ends_with(". "))
        {
            unordered_list_skip = false;
        }

        if line[1].content() == "• " || unordered_list_skip {
            unordered_list_skip = true;
            continue;
        }

        let amount = if line[1]
            .content()
            .strip_prefix(['1', '2', '3', '4', '5', '6', '7', '8', '9'])
            .is_some_and(|c| c.ends_with(". "))
        {
            match indent_len.cmp(&display_width(line[0].content())) {
                cmp::Ordering::Less => {
                    indent_index += 1;
                    indent_len = display_width(line[0].content());
                }
                cmp::Ordering::Greater => {
                    indent_index = indent_index.saturating_sub(1);
                    indent_len = display_width(line[0].content());
                }
                cmp::Ordering::Equal => (),
            }
            indent_correction[indent_index as usize]
                .saturating_sub(display_width(line[1].content()))
                + display_width(line[0].content())
        } else {
            // -3 because that is the length of the shortest ordered index (1. )
            (indent_correction[indent_index as usize] + display_width(line[0].content()))
                .saturating_sub(3)
        };

        line[0].set_content(" ".repeat(amount));
    }

    component.height = lines.len() as u16;
    component.content = lines;
}

fn transform_table(component: &mut TextComponent, width: u16) {
    let content = &mut component.content;

    let column_count = component
        .meta_info
        .iter()
        .filter(|w| w.kind() == WordType::MetaInfo(MetaData::ColumnsCount))
        .count();

    if !content.len().is_multiple_of(column_count) || column_count == 0 {
        component.height = 1;
        component.kind = TextNode::Table(vec![], vec![]);
        return;
    }

    assert!(
        content.len().is_multiple_of(column_count),
        "Invalid table cell distribution: content.len() = {}, column_count = {}",
        content.len(),
        column_count
    );

    let row_count = content.len() / column_count;

    ///////////////////////////
    // Find unbalanced width //
    ///////////////////////////
    let widths = {
        let mut widths = vec![0; column_count];
        content.chunks(column_count).for_each(|row| {
            row.iter().enumerate().for_each(|(col_i, entry)| {
                let len = content_entry_len(entry);
                if len > widths[col_i] as usize {
                    widths[col_i] = len as u16;
                }
            });
        });

        widths
    };

    let styling_width = column_count as u16;
    let unbalanced_cells_width = widths.iter().sum::<u16>();

    /////////////////////////////////////
    // Return if unbalanced width fits //
    /////////////////////////////////////
    if width >= unbalanced_cells_width + styling_width {
        component.height = (content.len() / column_count) as u16;
        component.kind = TextNode::Table(widths, vec![1; component.height as usize]);
        return;
    }

    //////////////////////////////
    // Find overflowing columns //
    //////////////////////////////
    let overflow_threshold = (width - styling_width) / column_count as u16;
    let mut overflowing_columns = vec![];

    let (overflowing_width, non_overflowing_width) = {
        let mut overflowing_width = 0;
        let mut non_overflowing_width = 0;

        for (column_i, column_width) in widths.iter().enumerate() {
            if *column_width > overflow_threshold {
                overflowing_columns.push((column_i, column_width));

                overflowing_width += column_width;
            } else {
                non_overflowing_width += column_width;
            }
        }

        (overflowing_width, non_overflowing_width)
    };

    assert!(
        !overflowing_columns.is_empty(),
        "table overflow should not be handled when there are no overflowing columns"
    );

    /////////////////////////////////////////////
    // Assign new width to overflowing columns //
    /////////////////////////////////////////////
    let mut available_balanced_width = width - non_overflowing_width - styling_width;
    let mut available_overflowing_width = overflowing_width;

    let overflowing_column_min_width =
        (available_balanced_width / (2 * overflowing_columns.len() as u16)).max(1);

    let mut widths_balanced: Vec<u16> = widths.clone();
    for (column_i, old_column_width) in overflowing_columns
        .iter()
        // Sorting ensures the smallest overflowing cells receive minimum area without the
        // need for recalculating the larger cells
        .sorted_by(|a, b| Ord::cmp(a.1, b.1))
    {
        // Ensure the longest cell gets the most amount of area
        let ratio = f32::from(**old_column_width) / f32::from(available_overflowing_width);
        let mut balanced_column_width =
            (ratio * f32::from(available_balanced_width)).floor() as u16;

        if balanced_column_width < overflowing_column_min_width {
            balanced_column_width = overflowing_column_min_width;
            available_overflowing_width -= **old_column_width;
            available_balanced_width -= balanced_column_width;
        }

        widths_balanced[*column_i] = balanced_column_width;
    }

    ////////////////////////////////////////
    // Wrap words based on balanced width //
    ////////////////////////////////////////
    let mut heights = vec![1; row_count];
    for (row_i, row) in content
        .iter_mut()
        .chunks(column_count)
        .into_iter()
        .enumerate()
    {
        for (column_i, entry) in row.into_iter().enumerate() {
            let lines = word_wrapping(
                entry.drain(..).as_ref(),
                widths_balanced[column_i] as usize,
                true,
            );

            if heights[row_i] < lines.len() as u16 {
                heights[row_i] = lines.len() as u16;
            }

            let _drop = std::mem::replace(entry, lines.into_iter().flatten().collect());
        }
    }

    component.height = heights.iter().copied().sum::<u16>();

    component.kind = TextNode::Table(widths_balanced, heights);
}

#[must_use]
pub fn content_entry_len(words: &[Word]) -> usize {
    words.iter().map(|word| display_width(word.content())).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(s: &str, kind: WordType) -> Word {
        Word::new(s.to_string(), kind)
    }

    #[test]
    fn new_filters_meta_info() {
        let words = vec![
            word("hello", WordType::Normal),
            word(" ", WordType::Normal),
            word("world", WordType::Bold),
            word("http://x.com", WordType::LinkData),
            word("fn1", WordType::FootnoteData),
            word("[1]", WordType::FootnoteInline),
            word("lang", WordType::MetaInfo(MetaData::PLanguage)),
        ];

        let tc = TextComponent::new(TextNode::Paragraph, words);

        // Renderable content: Normal("hello"), Normal(" "), Bold("world"), FootnoteInline("[1]")
        // FootnoteInline is renderable (is_renderable returns true) but also appears in meta_info
        let flat: Vec<&str> = tc.content().iter().flatten().map(|w| w.content()).collect();
        assert_eq!(flat, vec!["hello", " ", "world", "[1]"]);

        // meta_info: non-renderable (LinkData, FootnoteData, MetaInfo) + FootnoteInline
        let meta_kinds: Vec<WordType> = tc.meta_info().iter().map(|w| w.kind()).collect();
        assert!(meta_kinds.contains(&WordType::LinkData));
        assert!(meta_kinds.contains(&WordType::FootnoteData));
        assert!(meta_kinds.contains(&WordType::FootnoteInline));
        assert!(meta_kinds.contains(&WordType::MetaInfo(MetaData::PLanguage)));
        // Normal and Bold are renderable and not FootnoteInline, so excluded from meta_info
        assert!(!meta_kinds.contains(&WordType::Normal));
        assert!(!meta_kinds.contains(&WordType::Bold));
    }

    #[test]
    fn new_formatted_sets_height() {
        let lines = vec![
            vec![word("line1", WordType::Normal)],
            vec![word("line2", WordType::Normal)],
            vec![word("line3", WordType::Normal)],
        ];
        let tc = TextComponent::new_formatted(TextNode::Paragraph, lines);
        assert_eq!(tc.height(), 3);
    }

    #[test]
    fn new_formatted_sets_height_single_line() {
        let lines = vec![vec![word("only", WordType::Normal)]];
        let tc = TextComponent::new_formatted(TextNode::Paragraph, lines);
        assert_eq!(tc.height(), 1);
    }

    #[test]
    fn content_as_lines_paragraph() {
        let lines = vec![
            vec![
                word("Hello", WordType::Normal),
                word(" ", WordType::Normal),
                word("world", WordType::Normal),
            ],
            vec![word("second", WordType::Bold)],
        ];
        let tc = TextComponent::new_formatted(TextNode::Paragraph, lines);
        let result = tc.content_as_lines();
        assert_eq!(result, vec!["Hello world", "second"]);
    }

    #[test]
    fn content_as_lines_table() {
        // 2 columns, 2 rows = 4 content entries
        let widths = vec![10, 10];
        let heights = vec![1, 1];
        let tc = TextComponent::new_formatted(
            TextNode::Table(widths, heights),
            vec![
                vec![word("a1", WordType::Normal)],
                vec![word("b1", WordType::Normal)],
                vec![word("a2", WordType::Normal)],
                vec![word("b2", WordType::Normal)],
            ],
        );
        let result = tc.content_as_lines();
        // Each row: columns joined by " "
        assert_eq!(result, vec!["a1 b1", "a2 b2"]);
    }

    #[test]
    fn content_as_lines_table_zero_columns() {
        let tc = TextComponent::new_formatted(
            TextNode::Table(vec![], vec![]),
            vec![vec![word("x", WordType::Normal)]],
        );
        let result = tc.content_as_lines();
        assert!(result.is_empty());
    }

    #[test]
    fn num_links_counts_link_and_footnote() {
        let words = vec![
            word("text", WordType::Normal),
            word("click", WordType::Link),
            word("http://x.com", WordType::LinkData),
            word("[1]", WordType::FootnoteInline),
            word("bold", WordType::Bold),
            word("http://y.com", WordType::LinkData),
        ];
        let tc = TextComponent::new(TextNode::Paragraph, words);
        // LinkData and FootnoteInline in meta_info count
        assert_eq!(tc.num_links(), 3); // 2 LinkData + 1 FootnoteInline
    }

    #[test]
    fn num_links_zero_when_none() {
        let words = vec![
            word("plain", WordType::Normal),
            word("text", WordType::Bold),
        ];
        let tc = TextComponent::new(TextNode::Paragraph, words);
        assert_eq!(tc.num_links(), 0);
    }

    #[test]
    fn visually_select_and_deselect() {
        // Create content with a link span
        let words = vec![
            word("before", WordType::Normal),
            word(" ", WordType::Normal),
            word("click", WordType::Link),
            word("here", WordType::Link),
            word(" ", WordType::Normal),
            word("after", WordType::Normal),
            word("http://x.com", WordType::LinkData), // meta_info for the link
        ];
        let mut tc = TextComponent::new(TextNode::Paragraph, words);

        // Select the first (only) link group
        assert!(tc.visually_select(0).is_ok());
        assert!(tc.is_focused());

        // The link words should now be Selected
        let selected: Vec<&Word> = tc
            .content()
            .iter()
            .flatten()
            .filter(|w| w.kind() == WordType::Selected)
            .collect();
        assert_eq!(selected.len(), 2); // "click" and "here"
        assert_eq!(selected[0].content(), "click");
        assert_eq!(selected[1].content(), "here");

        // Deselect should restore original types
        tc.deselect();
        assert!(!tc.is_focused());
        let link_words: Vec<&Word> = tc
            .content()
            .iter()
            .flatten()
            .filter(|w| w.kind() == WordType::Link)
            .collect();
        assert_eq!(link_words.len(), 2);
    }

    #[test]
    fn visually_select_out_of_bounds() {
        let words = vec![
            word("text", WordType::Normal),
            word("link", WordType::Link),
            word("url", WordType::LinkData),
        ];
        let mut tc = TextComponent::new(TextNode::Paragraph, words);

        // Only 1 link; selecting index 1 is out of bounds
        let result = tc.visually_select(1);
        assert!(result.is_err());
    }

    #[test]
    fn visually_select_out_of_bounds_no_links() {
        let words = vec![word("plain", WordType::Normal)];
        let mut tc = TextComponent::new(TextNode::Paragraph, words);
        let result = tc.visually_select(0);
        assert!(result.is_err());
    }

    #[test]
    fn is_indented_list_true() {
        // An indented list has meta_info containing a word whose content is whitespace-only
        // but non-empty (e.g., "  " for indent)
        let words = vec![
            word("  ", WordType::MetaInfo(MetaData::Other)), // indent marker: whitespace, non-empty
            word("•", WordType::ListMarker),
            word("item", WordType::Normal),
        ];
        let tc = TextComponent::new(TextNode::List, words);
        assert!(tc.is_indented_list());
    }

    #[test]
    fn is_indented_list_false_for_paragraph() {
        let words = vec![word("text", WordType::Normal)];
        let tc = TextComponent::new(TextNode::Paragraph, words);
        assert!(!tc.is_indented_list());
    }

    #[test]
    fn is_indented_list_false_for_non_indented_list() {
        // A non-indented list has no whitespace-only meta_info word,
        // or has an empty string for the indent marker
        let words = vec![
            word("", WordType::MetaInfo(MetaData::Other)), // empty = no indent
            word("•", WordType::ListMarker),
            word("item", WordType::Normal),
        ];
        let tc = TextComponent::new(TextNode::List, words);
        assert!(!tc.is_indented_list());
    }

    // ── T04: word_wrapping and split_by_width tests ──

    #[test]
    fn word_wrapping_fits_one_line() {
        let words = vec![
            word("Hello", WordType::Normal),
            word(" ", WordType::Normal),
            word("world", WordType::Normal),
        ];
        // "Hello world" = 11 chars, width 20 → fits on one line
        let lines = word_wrapping(words.iter(), 20, true);
        assert_eq!(lines.len(), 1);
        let content: String = lines[0].iter().map(|w| w.content().to_string()).collect();
        assert_eq!(content, "Hello world");
    }

    #[test]
    fn word_wrapping_breaks_at_boundary() {
        // "aaaa" (4) + " " (1) + "bbb" (3) = 8; width=5 should break
        let words = vec![
            word("aaaa", WordType::Normal),
            word(" ", WordType::Normal),
            word("bbb", WordType::Normal),
        ];
        let lines = word_wrapping(words.iter(), 5, true);
        assert_eq!(lines.len(), 2);
        // First line: "aaaa" + " " = 5
        let line0: String = lines[0].iter().map(|w| w.content().to_string()).collect();
        assert_eq!(line0, "aaaa ");
        // Second line: "bbb" (trimmed leading space)
        let line1: String = lines[1].iter().map(|w| w.content().to_string()).collect();
        assert_eq!(line1, "bbb");
    }

    #[test]
    fn word_wrapping_long_word_hyphenation() {
        // A single word longer than width should be split with hyphen
        let words = vec![word("abcdefghij", WordType::Normal)];
        // width=5, allow_hyphen=true → hyphenation enabled (width > 4)
        let lines = word_wrapping(words.iter(), 5, true);
        assert!(lines.len() > 1, "long word should be split across lines");
        // Each line except possibly the last should end with '-'
        for line in &lines[..lines.len() - 1] {
            let last_word = line.last().unwrap();
            assert!(
                last_word.content().ends_with('-'),
                "split line should end with hyphen, got: {:?}",
                last_word.content()
            );
        }
        // Reconstruct: removing hyphens and joining should recover original content
        let reconstructed: String = lines
            .iter()
            .flat_map(|line| line.iter())
            .map(|w| w.content().trim_end_matches('-'))
            .collect();
        assert_eq!(reconstructed, "abcdefghij");
    }

    #[test]
    fn word_wrapping_no_hyphen_narrow() {
        // width ≤ 4 disables hyphenation even when allow_hyphen=true
        let words = vec![word("abcdefgh", WordType::Normal)];
        let lines = word_wrapping(words.iter(), 4, true);
        assert!(lines.len() > 1, "long word should still be split");
        // No line should end with a hyphen
        for line in &lines {
            for w in line {
                assert!(
                    !w.content().ends_with('-'),
                    "narrow width should not produce hyphens, got: {:?}",
                    w.content()
                );
            }
        }
    }

    #[test]
    fn word_wrapping_preserves_word_types() {
        let words = vec![word("abcdefghijklmnop", WordType::Bold)];
        let lines = word_wrapping(words.iter(), 6, true);
        assert!(lines.len() > 1);
        for line in &lines {
            for w in line {
                assert_eq!(
                    w.kind(),
                    WordType::Bold,
                    "wrapped words should preserve their WordType"
                );
            }
        }
    }

    #[test]
    fn word_wrapping_unicode_cjk() {
        // CJK characters have display width 2 each
        // "你好世界" = 4 chars × 2 width = 8 display width
        let words = vec![word("你好世界", WordType::Normal)];
        // width=4 → can fit 2 CJK chars per line
        let lines = word_wrapping(words.iter(), 4, false);
        assert!(
            lines.len() >= 2,
            "CJK text should wrap; got {} lines",
            lines.len()
        );
        // Verify total content is preserved
        let total: String = lines
            .iter()
            .flat_map(|line| line.iter())
            .map(|w| w.content().to_string())
            .collect();
        assert_eq!(total, "你好世界");
    }

    #[test]
    fn word_wrapping_empty_input() {
        let words: Vec<Word> = vec![];
        let lines = word_wrapping(words.iter(), 80, true);
        assert!(lines.is_empty());
    }

    #[test]
    fn split_by_width_ascii() {
        let (head, tail) = split_by_width("abcdef", 3);
        assert_eq!(head, "abc");
        assert_eq!(tail, "def");
    }

    #[test]
    fn split_by_width_unicode() {
        // "café" — 'é' is 2 bytes in UTF-8 but 1 display width
        let (head, tail) = split_by_width("café", 3);
        assert_eq!(head, "caf");
        assert_eq!(tail, "é");

        // CJK: "你好" — each char is 3 bytes, 2 display width
        let (head, tail) = split_by_width("你好", 2);
        assert_eq!(head, "你");
        assert_eq!(tail, "好");

        // Split at width 3 with CJK: "你" = width 2, "好" starts at width 2
        // width 3 can fit "你" (2) but not "你好" (4), so split after "你"
        let (head, tail) = split_by_width("你好世", 3);
        // "你" = width 2, next "好" would be 2+2=4 > 3, so split after "你"
        assert_eq!(head, "你");
        assert_eq!(tail, "好世");
    }

    #[test]
    fn split_by_width_zero_width() {
        let (head, tail) = split_by_width("hello", 0);
        assert_eq!(head, "");
        assert_eq!(tail, "hello");
    }

    // ── T05: transform_paragraph, transform_list tests ──

    /// Helper: build a list TextComponent from item descriptors.
    /// Each item is (indent_str, is_ordered, content_words).
    /// Mimics what the parser produces via `new_formatted`.
    fn make_list(items: &[(&str, bool, Vec<(&str, WordType)>)]) -> TextComponent {
        let rows: Vec<Vec<Word>> = items
            .iter()
            .map(|(indent, is_ordered, content_words)| {
                let mut ws = Vec::new();
                // indent meta word (content = whitespace, MetaInfo(Other))
                ws.push(word(indent, WordType::MetaInfo(MetaData::Other)));
                if *is_ordered {
                    // ordered: digit marker word from parser
                    ws.push(word("X. ", WordType::ListMarker));
                } else {
                    // unordered: bullet marker
                    ws.push(word("• ", WordType::ListMarker));
                }
                for (text, kind) in content_words {
                    ws.push(word(text, *kind));
                }
                // list type meta word (appended at end, like parser does)
                if *is_ordered {
                    ws.push(word("X", WordType::MetaInfo(MetaData::OList)));
                } else {
                    ws.push(word("X", WordType::MetaInfo(MetaData::UList)));
                }
                ws
            })
            .collect();

        TextComponent::new_formatted(TextNode::List, rows)
    }

    #[test]
    fn transform_paragraph_sets_height() {
        // A paragraph with words that wrap across multiple lines
        let words = vec![
            word("Hello", WordType::Normal),
            word(" ", WordType::Normal),
            word("world", WordType::Normal),
            word(" ", WordType::Normal),
            word("this", WordType::Normal),
            word(" ", WordType::Normal),
            word("is", WordType::Normal),
            word(" ", WordType::Normal),
            word("a", WordType::Normal),
            word(" ", WordType::Normal),
            word("test", WordType::Normal),
        ];
        // "Hello world this is a test" = 26 chars
        // width 15 → transform subtracts 1 for Paragraph → effective 14
        // Line 1: "Hello world " (12) + "th" won't fit... let's trace:
        //   "Hello" (5) + " " (1) + "world" (5) = 11, + " " (1) = 12, + "this" (4) = 16 > 14
        //   → line 1 ends at "world ", line 2 starts with "this"
        //   "this" (4) + " " (1) + "is" (2) + " " (1) + "a" (1) + " " (1) = 10, + "test" (4) = 14
        //   → line 2: "this is a test" fits in 14
        let mut tc = TextComponent::new(TextNode::Paragraph, words);
        tc.transform(15);
        assert_eq!(tc.height(), 2);
        assert_eq!(tc.content().len(), 2);
    }

    #[test]
    fn transform_paragraph_quote_prefix() {
        // Quote transform: width - 2, and prepends " " to continuation lines
        let words = vec![
            word("A", WordType::Normal),
            word(" ", WordType::Normal),
            word("short", WordType::Normal),
            word(" ", WordType::Normal),
            word("quote", WordType::Normal),
            word(" ", WordType::Normal),
            word("that", WordType::Normal),
            word(" ", WordType::Normal),
            word("wraps", WordType::Normal),
        ];
        // "A short quote that wraps" = 24 chars
        // width=15, Quote → effective width = 13
        // Line 1: "A" + " " + "short" + " " + "quote" = 13 → fits
        // Line 2: "that" + " " + "wraps" = 10
        let mut tc = TextComponent::new(TextNode::Quote, words);
        tc.transform(15);
        assert!(tc.height() >= 2, "quote should wrap, height={}", tc.height());

        // Continuation lines (all lines after the first, since no special quote meta)
        // should start with a space word
        for line in tc.content().iter().skip(1) {
            assert!(
                !line.is_empty(),
                "continuation line should not be empty"
            );
            assert_eq!(
                line[0].content(),
                " ",
                "continuation line should start with space prefix"
            );
        }
    }

    #[test]
    fn transform_paragraph_quote_prefix_special_skips_first() {
        // A "special" quote (admonition) has non-empty meta_info.
        // The space prefix is NOT prepended to line 0 (it's the admonition header).
        let words = vec![
            word("note", WordType::MetaInfo(MetaData::Note)), // makes meta_info non-empty
            word("Some", WordType::Normal),
            word(" ", WordType::Normal),
            word("admonition", WordType::Normal),
            word(" ", WordType::Normal),
            word("text", WordType::Normal),
            word(" ", WordType::Normal),
            word("here", WordType::Normal),
        ];
        // Effective width for Quote = width - 2 = 18
        let mut tc = TextComponent::new(TextNode::Quote, words);
        tc.transform(20);

        // For special quotes, skip(1) means skip line 0, then prepend " " from line 1 on.
        // Line 0 should NOT start with the " " prefix.
        if tc.height() >= 2 {
            let first_line = &tc.content()[0];
            // The first word should be the actual content, not a space prefix
            assert_ne!(
                first_line.first().map(|w| w.content()),
                Some(" "),
                "special quote first line should not get space prefix"
            );
        }
    }

    #[test]
    fn transform_paragraph_task_width() {
        // Task transform subtracts 4 from width
        let words = vec![
            word("Do", WordType::Normal),
            word(" ", WordType::Normal),
            word("something", WordType::Normal),
            word(" ", WordType::Normal),
            word("important", WordType::Normal),
            word(" ", WordType::Normal),
            word("now", WordType::Normal),
        ];
        // "Do something important now" = 26 chars
        // width=20, Task → effective = 16
        // Line 1: "Do" + " " + "something" + " " = 13, + "important" = 22 > 16
        //   → line 1: "Do something ", line 2: "important now"
        let mut tc = TextComponent::new(TextNode::Task, words.clone());
        tc.transform(20);
        let task_height = tc.height();

        // Same content as Paragraph at same width should have different height
        // because Paragraph subtracts only 1
        let mut tc_para = TextComponent::new(TextNode::Paragraph, words);
        tc_para.transform(20);
        let para_height = tc_para.height();

        // Task has less effective width, so it should wrap at least as much
        assert!(
            task_height >= para_height,
            "task (eff width=16) should wrap at least as much as paragraph (eff width=19)"
        );
    }

    #[test]
    fn transform_list_ordered_numbering() {
        // Two ordered list items at the same indent level
        let mut tc = make_list(&[
            ("", true, vec![("first", WordType::Normal)]),
            ("", true, vec![(" ", WordType::Normal), ("second", WordType::Normal)]),
        ]);

        tc.transform(40);

        // After transform, ordered items should have sequential numbers
        let markers: Vec<&str> = tc
            .content()
            .iter()
            .filter_map(|line| {
                line.iter().find(|w| w.kind() == WordType::ListMarker)
            })
            .map(|w| w.content())
            .collect();

        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0], "1. ", "first ordered item should be numbered 1");
        assert_eq!(markers[1], "2. ", "second ordered item should be numbered 2");
    }

    #[test]
    fn transform_list_nested_indent() {
        // Outer unordered item, then inner (indented) unordered item
        let mut tc = make_list(&[
            ("", false, vec![("outer", WordType::Normal)]),
            ("  ", false, vec![("inner", WordType::Normal)]),
        ]);

        tc.transform(40);

        // The inner item should have more leading whitespace than the outer
        let indent_widths: Vec<usize> = tc
            .content()
            .iter()
            .map(|line| {
                line.first()
                    .map(|w| display_width(w.content()))
                    .unwrap_or(0)
            })
            .collect();

        assert!(
            indent_widths.len() >= 2,
            "should have at least 2 lines"
        );
        // The nested item should have strictly more indent than the top-level item
        assert!(
            indent_widths[1] > indent_widths[0],
            "nested item indent ({}) should be greater than outer ({})",
            indent_widths[1],
            indent_widths[0]
        );
    }

    // ── T06: transform_table and transform_codeblock tests ──

    /// Helper: build a Table TextComponent matching parser output structure.
    ///
    /// `rows` is `[header_row, data_row, ...]` — each row is `cols`-length vec of cell words.
    /// The parser emits: header cells, then `cols` separator entries (each containing one
    /// `ColumnsCount` meta word), then data cells. `new_formatted` extracts the meta words
    /// to `meta_info` and leaves empty vecs in `content` for separator entries.
    ///
    /// `transform_table` counts `ColumnsCount` meta words to determine column count,
    /// then checks that `content.len() % column_count == 0`.
    fn make_table(cols: usize, rows: Vec<Vec<Vec<Word>>>) -> TextComponent {
        assert!(!rows.is_empty(), "need at least a header row");
        let mut flat: Vec<Vec<Word>> = Vec::new();

        // First row = header
        for cell in &rows[0] {
            flat.push(cell.clone());
        }

        // Separator row: one entry per column, each with a ColumnsCount meta word
        for _ in 0..cols {
            flat.push(vec![Word::new(
                String::new(),
                WordType::MetaInfo(MetaData::ColumnsCount),
            )]);
        }

        // Remaining rows = data
        for row in &rows[1..] {
            assert_eq!(row.len(), cols, "each row must have exactly `cols` cells");
            for cell in row {
                flat.push(cell.clone());
            }
        }

        TextComponent::new_formatted(TextNode::Table(vec![], vec![]), flat)
    }

    #[test]
    fn transform_table_unbalanced_fits() {
        // 2 columns, header + 1 data row. Content is short → fits without balancing.
        // After new_formatted: separator entries (meta-only) are filtered out,
        // leaving 2 header cells + 2 data cells = 4 entries
        // column_count = 2, row_count = 2 (header, data)
        let mut tc = make_table(2, vec![
            vec![vec![word("A", WordType::Normal)], vec![word("B", WordType::Normal)]],
            vec![vec![word("C", WordType::Normal)], vec![word("D", WordType::Normal)]],
        ]);
        // Width 80 is plenty for single-char cells
        tc.transform(80);

        if let TextNode::Table(widths, heights) = tc.kind() {
            assert_eq!(widths.len(), 2, "should have 2 column widths");
            // 2 rows: header and data (separator entries removed as empty after meta extraction)
            assert_eq!(heights.len(), 2, "should have 2 row heights (header + data)");
            // Each column's natural width is 1 (single char)
            assert_eq!(widths[0], 1);
            assert_eq!(widths[1], 1);
            // No wrapping needed → each row height is 1
            assert!(heights.iter().all(|&h| h == 1));
        } else {
            panic!("expected Table variant, got {:?}", tc.kind());
        }

        assert_eq!(tc.height(), 2, "2 rows × 1 line each = 2");
    }

    #[test]
    fn transform_table_overflow_balanced() {
        // 2 columns where one column is very wide, forcing overflow balancing.
        // Column 0: short (3 chars), Column 1: long (40 chars)
        let long_word = "a]".repeat(20); // 40 chars
        let mut tc = make_table(2, vec![
            vec![
                vec![word("Hi!", WordType::Normal)],
                vec![word(&long_word, WordType::Normal)],
            ],
        ]);
        // Width 30: natural widths = 3 + 40 = 43 + styling(2) = 45 > 30 → balancing required
        tc.transform(30);

        if let TextNode::Table(widths, _) = tc.kind() {
            assert_eq!(widths.len(), 2);
            // The short column should keep roughly its natural width
            // The long column should be reduced
            let total: u16 = widths.iter().sum();
            let styling_width = 2u16; // column_count
            assert!(
                total + styling_width <= 30,
                "balanced widths ({total}) + styling ({styling_width}) should fit in width 30"
            );
            // The long column must be narrower than its natural 40
            assert!(widths[1] < 40, "overflowing column should be reduced");
        } else {
            panic!("expected Table variant, got {:?}", tc.kind());
        }
    }

    #[test]
    fn transform_table_wraps_cell_content() {
        // A table where the long column must word-wrap, producing multi-line rows.
        let mut tc = make_table(2, vec![
            vec![
                vec![word("X", WordType::Normal)],
                vec![
                    word("This", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("is", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("a", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("very", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("long", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("cell", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("that", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("must", WordType::Normal),
                    word(" ", WordType::Normal),
                    word("wrap", WordType::Normal),
                ],
            ],
        ]);
        // Width 20: the long cell ("This is a very long cell that must wrap" = 39 chars)
        // can't possibly fit in ~18 usable chars, so it must wrap across multiple lines.
        tc.transform(20);

        if let TextNode::Table(_, heights) = tc.kind() {
            assert!(!heights.is_empty());
            // The row with the long cell should wrap to more than 1 line
            assert!(
                heights[0] > 1,
                "long cell should cause row to wrap, got height {}",
                heights[0]
            );
        } else {
            panic!("expected Table variant, got {:?}", tc.kind());
        }

        // Total height should reflect the wrapped row
        assert!(tc.height() > 1);
    }

    #[test]
    fn transform_table_zero_columns() {
        // A degenerate table with 0 ColumnsCount meta words → malformed.
        // Build directly via new_formatted with no ColumnsCount meta.
        let tc_content = vec![vec![word("x", WordType::Normal)]];
        let mut tc = TextComponent::new_formatted(TextNode::Table(vec![], vec![]), tc_content);
        // No ColumnsCount meta_info → column_count = 0
        tc.transform(80);

        assert_eq!(tc.height(), 1, "malformed table should have height 1");
        if let TextNode::Table(widths, heights) = tc.kind() {
            assert!(widths.is_empty(), "malformed table should have empty widths");
            assert!(heights.is_empty(), "malformed table should have empty heights");
        } else {
            panic!("expected Table variant");
        }
    }

    #[test]
    fn transform_codeblock_preserves_lines() {
        // A code block with no language (unhighlighted path).
        // Content: two lines separated into two inner Vecs.
        let lines = vec![
            vec![word("line one\n", WordType::CodeBlock(Color::Reset))],
            vec![word("line two\n", WordType::CodeBlock(Color::Reset))],
        ];
        let mut tc = TextComponent::new_formatted(TextNode::CodeBlock, lines);
        tc.transform(80);

        // For unhighlighted code (no language), transform_codeblock inserts an empty
        // CodeBlock word at position 0, then sets height = content.len().
        // The original 2 lines + 1 inserted empty leader = 3
        assert_eq!(
            tc.height(),
            tc.content().len() as u16,
            "height should equal number of content lines"
        );
        assert_eq!(tc.height(), 3, "2 original lines + 1 inserted empty leader");

        // The inserted line at index 0 should be an empty CodeBlock word
        let first_line = &tc.content()[0];
        assert_eq!(first_line.len(), 1);
        assert_eq!(first_line[0].content(), "");
        assert!(matches!(first_line[0].kind(), WordType::CodeBlock(_)));

        // Original content should still be present
        let second_content: String = tc.content()[1].iter().map(|w| w.content().to_string()).collect();
        assert_eq!(second_content, "line one\n");
    }

    #[test]
    fn transform_codeblock_height() {
        // Code block with a language that won't match any tree-sitter (unhighlighted).
        // Include a PLanguage meta word in the first line so new_formatted extracts it.
        let lines = vec![
            vec![
                word("unknownlang", WordType::MetaInfo(MetaData::PLanguage)),
                word("alpha\n", WordType::CodeBlock(Color::Reset)),
            ],
            vec![word("beta\n", WordType::CodeBlock(Color::Reset))],
            vec![word("gamma\n", WordType::CodeBlock(Color::Reset))],
        ];
        let mut tc = TextComponent::new_formatted(TextNode::CodeBlock, lines);

        // Verify the language meta was extracted
        assert!(
            tc.meta_info().iter().any(|w| w.content() == "unknownlang"),
            "PLanguage should be in meta_info"
        );

        tc.transform(80);

        // "unknownlang" won't match any highlight_code branch → Unhighlighted.
        // In the Unhighlighted path, content stays as-is and height = content.len().
        assert_eq!(tc.height(), tc.content().len() as u16);
        // Original 3 lines should be preserved (no insertion for non-empty language)
        assert_eq!(tc.height(), 3);
    }

    #[test]
    fn transform_list_mixed_ordered_unordered() {
        // An unordered item followed by an ordered item at the same indent
        let mut tc = make_list(&[
            ("", false, vec![("bullet", WordType::Normal)]),
            ("", true, vec![("numbered", WordType::Normal)]),
        ]);

        tc.transform(40);

        // Should produce two lines, one with bullet marker and one with number
        let markers: Vec<String> = tc
            .content()
            .iter()
            .filter_map(|line| {
                line.iter()
                    .find(|w| w.kind() == WordType::ListMarker)
                    .map(|w| w.content().to_string())
            })
            .collect();

        assert_eq!(markers.len(), 2, "should have 2 list markers");
        assert_eq!(markers[0], "• ", "first item should be bullet");
        assert!(
            markers[1].ends_with(". "),
            "second item should be numbered, got: {:?}",
            markers[1]
        );
    }
}
