use std::cmp;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, List, ListItem, Paragraph, Row, Table, Widget},
};

use crate::{
    nodes::{
        textcomponent::{TextComponent, TextNode},
        word::{MetaData, Word, WordType},
    },
    util::{
        colors::{color_config, heading_colors},
        general::GENERAL_CONFIG,
    },
};

fn clips_upper_bound(_area: Rect, component: &TextComponent) -> bool {
    component.scroll_offset() > component.y_offset()
}

fn clips_lower_bound(area: Rect, component: &TextComponent) -> bool {
    (component.y_offset() + component.height()).saturating_sub(component.scroll_offset())
        > area.height
}

#[derive(Clone, Copy)]
enum Clipping {
    Both,
    Upper,
    Lower,
    None,
}

impl Widget for TextComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let kind = self.kind();

        let y = self.y_offset().saturating_sub(self.scroll_offset());

        let clips = if clips_upper_bound(area, &self) && clips_lower_bound(area, &self) {
            Clipping::Both
        } else if clips_upper_bound(area, &self) {
            Clipping::Upper
        } else if clips_lower_bound(area, &self) {
            Clipping::Lower
        } else {
            Clipping::None
        };

        let height = match clips {
            Clipping::Both => {
                let new_y = self.y_offset().saturating_sub(self.scroll_offset());
                let new_height = new_y;
                cmp::min(self.height(), area.height.saturating_sub(new_height))
            }

            Clipping::Upper => cmp::min(
                self.height(),
                (self.height() + self.y_offset()).saturating_sub(self.scroll_offset()),
            ),
            Clipping::Lower => {
                let new_y = self.y_offset() - self.scroll_offset();
                let new_height = new_y;
                cmp::min(self.height(), area.height.saturating_sub(new_height))
            }
            Clipping::None => self.height(),
        };

        let meta_info = self
            .meta_info()
            .to_owned()
            .first()
            .cloned()
            .unwrap_or_else(|| Word::new(String::new(), WordType::Normal));

        let area = Rect { height, y, ..area };

        match kind {
            TextNode::Paragraph => render_paragraph(area, buf, self, clips),
            TextNode::Heading => render_heading(area, buf, &self),
            TextNode::Task => render_task(area, buf, self, clips, &meta_info),
            TextNode::List => render_list(area, buf, self, clips),
            TextNode::CodeBlock => render_code_block(area, buf, &self, clips),
            TextNode::Table(widths, heights) => {
                render_table(area, buf, self, clips, &widths, &heights);
            }
            TextNode::Quote => render_quote(area, buf, self, clips),
            TextNode::LineBreak => (),
            TextNode::HorizontalSeperator => render_horizontal_seperator(area, buf),
            TextNode::Image => todo!(),
            TextNode::Footnote => (),
        }
    }
}

fn style_word(word: &Word) -> Span<'_> {
    match word.kind() {
        WordType::MetaInfo(_) | WordType::LinkData | WordType::FootnoteData => unreachable!(),
        WordType::Selected => Span::styled(
            word.content(),
            Style::default()
                .fg(color_config().link_selected_fg_color)
                .bg(color_config().link_selected_bg_color),
        ),
        WordType::Normal => Span::raw(word.content()),
        WordType::Code => Span::styled(
            word.content(),
            Style::default().fg(color_config().code_fg_color),
        )
        .bg(color_config().code_bg_color),
        WordType::Link | WordType::FootnoteInline => Span::styled(
            word.content(),
            Style::default().fg(color_config().link_color),
        ),
        WordType::Italic => Span::styled(
            word.content(),
            Style::default().fg(color_config().italic_color).italic(),
        ),
        WordType::Bold => Span::styled(
            word.content(),
            Style::default().fg(color_config().bold_color).bold(),
        ),
        WordType::Strikethrough | WordType::Footnote => Span::styled(
            word.content(),
            Style::default()
                .fg(color_config().striketrough_color)
                .add_modifier(Modifier::CROSSED_OUT),
        ),
        WordType::White | WordType::ListMarker => {
            Span::styled(word.content(), Style::default().fg(Color::White))
        }
        WordType::BoldItalic => Span::styled(
            word.content(),
            Style::default()
                .fg(color_config().bold_italic_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        ),
        WordType::CodeBlock(e) => Span::styled(word.content(), e),
    }
}

fn render_quote(area: Rect, buf: &mut Buffer, component: TextComponent, clip: Clipping) {
    let top = component
        .scroll_offset()
        .saturating_sub(component.y_offset());

    let meta = component.meta_info().to_owned();

    let mut content = component.content_owned();
    let content = match clip {
        Clipping::Both => {
            content.drain(0..usize::from(top));
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::Upper => {
            let len = content.len();
            let height = area.height;
            let offset = len - usize::from(height);
            let mut content = content;
            content.drain(0..offset);
            content
        }
        Clipping::Lower => {
            let mut content = content;
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::None => content,
    };

    let lines = content
        .iter()
        .map(|c| Line::from(c.iter().map(style_word).collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    let bar_color = if let Some(meta) = meta.first() {
        meta.content()
            .split_whitespace()
            .next()
            .map(str::to_lowercase)
            .map_or(color_config().quote_bg_color, |c| match c.as_str() {
                "[!tip]" => color_config().quote_tip,
                "[!warning]" => color_config().quote_warning,
                "[!caution]" => color_config().quote_caution,
                "[!important]" => color_config().quote_important,
                "[!note]" => color_config().quote_note,
                _ => color_config().quote_default,
            })
    } else {
        Color::White
    };
    let vertical_marker = Span::styled("\u{2588}", Style::default().fg(bar_color));

    let marker_paragraph = Paragraph::new(vec![Line::from(vertical_marker); content.len()])
        .bg(color_config().quote_bg_color);
    marker_paragraph.render(area, buf);

    let paragraph = Paragraph::new(lines)
        .block(Block::default().style(Style::default().bg(color_config().quote_bg_color)));

    let area = Rect {
        x: area.x + 1,
        width: cmp::min(area.width, GENERAL_CONFIG.width) - 1,
        ..area
    };

    paragraph.render(area, buf);
}

fn style_heading(word: &Word, indent: u8) -> Span<'_> {
    let color = match indent {
        2 => heading_colors().level_2,
        3 => heading_colors().level_3,
        4 => heading_colors().level_4,
        5 => heading_colors().level_5,
        6 => heading_colors().level_6,
        _ => color_config().heading_fg_color,
    };
    Span::styled(word.content(), Style::default().fg(color))
}

fn render_heading(area: Rect, buf: &mut Buffer, component: &TextComponent) {
    let indent = if let Some(meta) = component.meta_info().first() {
        match meta.kind() {
            WordType::MetaInfo(MetaData::HeadingLevel(e)) => e,
            _ => 1,
        }
    } else {
        1
    };

    let content: Vec<Span<'_>> = component
        .content()
        .iter()
        .flatten()
        .map(|c| style_heading(c, indent))
        .collect();

    let paragraph = match indent {
        1 => Paragraph::new(Line::from(content))
            .block(Block::default().style(Style::default().bg(color_config().heading_bg_color)))
            .alignment(Alignment::Center),
        _ => Paragraph::new(Line::from(content)),
    };

    paragraph.render(area, buf);
}

fn render_paragraph(area: Rect, buf: &mut Buffer, component: TextComponent, clip: Clipping) {
    let top = component
        .scroll_offset()
        .saturating_sub(component.y_offset());
    let mut content = component.content_owned();
    let content = match clip {
        Clipping::Both => {
            content.drain(0..usize::from(top));
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::Upper => {
            let len = content.len();
            let height = area.height;
            let offset = len - usize::from(height);
            let mut content = content;
            content.drain(0..offset);
            content
        }
        Clipping::Lower => {
            let mut content = content;
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::None => content,
    };

    let lines = content
        .iter()
        .map(|c| Line::from(c.iter().map(style_word).collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines);

    paragraph.render(area, buf);
}

fn render_list(area: Rect, buf: &mut Buffer, component: TextComponent, clip: Clipping) {
    let top = component
        .scroll_offset()
        .saturating_sub(component.y_offset());
    let mut content = component.content_owned();
    let content = match clip {
        Clipping::Both => {
            content.drain(0..usize::from(top));
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::Upper => {
            let len = content.len();
            let height = area.height;
            let offset = len - usize::from(height);
            let mut content = content;
            content.drain(0..offset);
            content
        }
        Clipping::Lower => {
            let mut content = content;
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::None => content,
    };
    let content: Vec<ListItem<'_>> = content
        .iter()
        .map(|c| -> ListItem<'_> {
            ListItem::new(Line::from(c.iter().map(style_word).collect::<Vec<_>>()))
        })
        .collect();

    let list = List::new(content);
    list.render(area, buf);
}

fn render_code_block(area: Rect, buf: &mut Buffer, component: &TextComponent, clip: Clipping) {
    let mut content = component
        .content()
        .iter()
        .map(|c| Line::from(c.iter().map(style_word).collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    match clip {
        Clipping::Both => {
            let top = component.scroll_offset() - component.y_offset();
            content.drain(0..usize::from(top));
            content.drain(usize::from(area.height)..);
        }
        Clipping::Upper => {
            let len = content.len();
            let height = area.height;
            let offset = len - usize::from(height);
            content.drain(0..offset);
        }
        Clipping::Lower => {
            content.drain(usize::from(area.height)..);
        }
        Clipping::None => (),
    }

    let block = Block::default().style(Style::default().bg(color_config().code_block_bg_color));

    block.render(area, buf);

    let area = Rect {
        x: area.x + 1,
        width: area.width - 1,
        ..area
    };

    let paragraph = Paragraph::new(content);

    paragraph.render(area, buf);
}

#[expect(
    clippy::too_many_lines,
    reason = "table cell iteration coordinates row/column state that would be awkward to pass between functions"
)]
fn render_table(
    area: Rect,
    buf: &mut Buffer,
    component: TextComponent,
    clip: Clipping,
    widths: &[u16],
    heights: &[u16],
) {
    let scroll_offset = component.scroll_offset();
    let y_offset = component.y_offset();
    let height = component.height();

    let column_count = widths.len();

    if column_count == 0 {
        Paragraph::new(Line::from("Malformed table").fg(Color::Red)).render(area, buf);
        return;
    }

    let content = component.content_owned();
    let titles = content.chunks(column_count).next().unwrap().to_vec();
    let moved_content = content.chunks(column_count).skip(1).collect::<Vec<_>>();

    let header = Row::new(
        titles
            .iter()
            .enumerate()
            .map(|(column_i, entry)| {
                let mut line = vec![];
                let mut lines = vec![];
                let mut line_len: u16 = 0;
                for word in entry {
                    let word_len: u16 = word
                        .content()
                        .len()
                        .try_into()
                        .expect("word length fits in u16");
                    line_len += word_len;
                    if line_len <= widths[column_i] {
                        line.push(word);
                    } else {
                        lines.push(Line::from(
                            line.into_iter().map(style_word).collect::<Vec<_>>(),
                        ));
                        line = vec![word];
                        line_len -= widths[column_i];
                    }
                }

                lines.push(Line::from(
                    line.into_iter().map(style_word).collect::<Vec<_>>(),
                ));

                Cell::from(lines)
            })
            .collect::<Vec<_>>(),
    )
    .height(heights[0]);

    let (start_i, stop_i) = match clip {
        Clipping::Both => {
            let top = scroll_offset - y_offset;
            (top, top + area.height)
        }
        Clipping::Upper => {
            let offset = height.saturating_sub(area.height);
            (offset, height)
        }
        Clipping::Lower => (0, height.min(area.height)),
        Clipping::None => (0, height),
    };

    let start_i = usize::from(start_i);
    let stop_i = usize::from(stop_i.saturating_sub(heights[0]));
    let mut line_i = 0;
    let rows = moved_content
        .iter()
        .enumerate()
        .filter_map(|(row_i, c)| {
            let row_height = usize::from(heights[row_i + 1]);
            if (line_i + row_height) <= start_i {
                line_i += row_height;
                return None;
            } else if stop_i <= line_i {
                return None;
            }

            let start_cell_line_i = start_i.saturating_sub(line_i);
            let stop_cell_line_i = stop_i.saturating_sub(line_i);
            let n_cell_lines = row_height
                .min(stop_cell_line_i)
                .saturating_sub(start_cell_line_i);

            line_i += row_height;

            Some(
                Row::new(
                    c.iter()
                        .enumerate()
                        .map(|(column_i, entry)| {
                            let mut acc = vec![];
                            let mut lines = vec![];
                            let mut line_len: u16 = 0;
                            for word in entry {
                                let word_len: u16 = word
                                    .content()
                                    .len()
                                    .try_into()
                                    .expect("word length fits in u16");
                                line_len += word_len;
                                if line_len <= widths[column_i] {
                                    acc.push(word);
                                } else {
                                    lines.push(Line::from(
                                        acc.into_iter().map(style_word).collect::<Vec<_>>(),
                                    ));
                                    line_len = word_len;
                                    acc = vec![word];
                                }
                            }

                            lines.push(Line::from(
                                acc.into_iter().map(style_word).collect::<Vec<_>>(),
                            ));

                            lines.append(&mut vec![
                                Line::from("");
                                (start_cell_line_i + n_cell_lines)
                                    .saturating_sub(lines.len())
                            ]);

                            Cell::from(
                                lines
                                    .splice(
                                        start_cell_line_i..(start_cell_line_i + n_cell_lines),
                                        vec![],
                                    )
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .height(n_cell_lines.try_into().expect("cell line count fits in u16")),
            )
        })
        .collect::<Vec<_>>();

    let table = Table::new(rows, widths.to_vec())
        .header(
            header.style(
                Style::default()
                    .fg(color_config().table_header_fg_color)
                    .bg(color_config().table_header_bg_color),
            ),
        )
        .block(Block::default())
        .column_spacing(1);

    table.render(area, buf);
}

fn render_task(
    area: Rect,
    buf: &mut Buffer,
    component: TextComponent,
    clip: Clipping,
    meta_info: &Word,
) {
    let (checked, unchecked) = if GENERAL_CONFIG.emoji_check_marks {
        ("✅ ", "❌ ")
    } else {
        ("[✓] ", "[ ] ")
    };

    let checkbox = if meta_info.content() == "- [ ] " {
        unchecked
    } else {
        checked
    };

    let paragraph = Paragraph::new(checkbox);

    paragraph.render(area, buf);

    let area = Rect {
        x: area.x + 4,
        width: area.width - 4,
        ..area
    };

    let top = component
        .scroll_offset()
        .saturating_sub(component.y_offset());

    let mut content = component.content_owned();

    let content = match clip {
        Clipping::Both => {
            content.drain(0..usize::from(top));
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::Upper => {
            let len = content.len();
            let height = area.height;
            let offset = len - usize::from(height);
            let mut content = content;
            content.drain(0..offset);
            content
        }
        Clipping::Lower => {
            let mut content = content;
            content.drain(usize::from(area.height)..);
            content
        }
        Clipping::None => content,
    };

    let lines = content
        .iter()
        .map(|c| Line::from(c.iter().map(style_word).collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines);

    paragraph.render(area, buf);
}

fn render_horizontal_seperator(area: Rect, buf: &mut Buffer) {
    let paragraph = Paragraph::new(Line::from(vec![Span::raw(
        "\u{2014}".repeat(GENERAL_CONFIG.width.into()),
    )]));

    paragraph.render(area, buf);
}
