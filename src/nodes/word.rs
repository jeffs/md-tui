use ratatui::style::Color;

use crate::parser::MdParseEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaData {
    UList,
    OList,
    PLanguage,
    Other,
    ColumnsCount,
    Important,
    Note,
    Tip,
    Warning,
    Caution,
    HeadingLevel(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordType {
    Bold,
    BoldItalic,
    Code,
    CodeBlock(Color),
    Footnote,
    FootnoteData,
    FootnoteInline,
    Italic,
    Link,
    LinkData,
    ListMarker,
    MetaInfo(MetaData),
    Normal,
    Selected,
    Strikethrough,
    White,
}

impl From<MdParseEnum> for WordType {
    fn from(value: MdParseEnum) -> Self {
        match value {
            MdParseEnum::PLanguage
            | MdParseEnum::BlockSeparator
            | MdParseEnum::TaskOpen
            | MdParseEnum::TaskClosed
            | MdParseEnum::Indent
            | MdParseEnum::HorizontalSeparator => WordType::MetaInfo(MetaData::Other),
            MdParseEnum::FootnoteRef => WordType::FootnoteInline,
            MdParseEnum::Code => WordType::Code,
            MdParseEnum::Bold => WordType::Bold,
            MdParseEnum::Italic => WordType::Italic,
            MdParseEnum::Strikethrough => WordType::Strikethrough,
            MdParseEnum::Link | MdParseEnum::WikiLink | MdParseEnum::InlineLink => WordType::Link,
            MdParseEnum::BoldItalic => WordType::BoldItalic,
            MdParseEnum::Digit => WordType::ListMarker,
            MdParseEnum::Paragraph
            | MdParseEnum::AltText
            | MdParseEnum::Quote
            | MdParseEnum::Sentence
            | MdParseEnum::Word => WordType::Normal,
            MdParseEnum::LinkData => WordType::LinkData,
            MdParseEnum::Imortant => WordType::MetaInfo(MetaData::Important),
            MdParseEnum::Note => WordType::MetaInfo(MetaData::Note),
            MdParseEnum::Tip => WordType::MetaInfo(MetaData::Tip),
            MdParseEnum::Warning => WordType::MetaInfo(MetaData::Warning),
            MdParseEnum::Caution => WordType::MetaInfo(MetaData::Caution),
            MdParseEnum::Heading
            | MdParseEnum::BoldItalicStr
            | MdParseEnum::BoldStr
            | MdParseEnum::CodeBlock
            | MdParseEnum::CodeStr
            | MdParseEnum::Image
            | MdParseEnum::ItalicStr
            | MdParseEnum::ListContainer
            | MdParseEnum::OrderedList
            | MdParseEnum::StrikethroughStr
            | MdParseEnum::Footnote
            | MdParseEnum::Table
            | MdParseEnum::TableCell
            | MdParseEnum::Task
            | MdParseEnum::UnorderedList
            | MdParseEnum::TableSeparator => {
                unreachable!("Edit this or pest file to fix for value: {:?}", value)
            }
            MdParseEnum::CodeBlockStr | MdParseEnum::CodeBlockStrSpaceIndented => {
                WordType::CodeBlock(Color::Reset)
            } // MdParseEnum::FootnoteRef => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    content: String,
    word_type: WordType,
    previous_type: Option<WordType>,
}

impl Word {
    #[must_use]
    pub fn new(content: String, word_type: WordType) -> Self {
        Self {
            word_type,
            previous_type: None,
            content,
        }
    }

    #[must_use]
    pub fn previous_type(&self) -> WordType {
        self.previous_type.unwrap_or(self.word_type)
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    #[must_use]
    pub fn kind(&self) -> WordType {
        self.word_type
    }

    pub fn set_kind(&mut self, kind: WordType) {
        self.previous_type = Some(self.word_type);
        self.word_type = kind;
    }

    pub fn clear_kind(&mut self) {
        self.word_type = self.previous_type.unwrap_or(self.word_type);
        self.previous_type = None;
    }

    #[must_use]
    pub fn is_renderable(&self) -> bool {
        !matches!(
            self.kind(),
            WordType::MetaInfo(_) | WordType::LinkData | WordType::FootnoteData
        )
    }

    pub fn split_off(&mut self, at: usize) -> Word {
        Word {
            content: self.content.split_off(at),
            word_type: self.word_type,
            previous_type: self.previous_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn word_new_sets_type() {
        let w = Word::new("hello".into(), WordType::Bold);
        assert_eq!(w.content(), "hello");
        assert_eq!(w.kind(), WordType::Bold);
        assert_eq!(w.previous_type(), WordType::Bold); // no previous → falls back to current
    }

    #[test]
    fn word_set_kind_preserves_previous() {
        let mut w = Word::new("x".into(), WordType::Normal);
        w.set_kind(WordType::Selected);
        assert_eq!(w.kind(), WordType::Selected);
        assert_eq!(w.previous_type(), WordType::Normal);

        w.clear_kind();
        assert_eq!(w.kind(), WordType::Normal);
        assert!(w.previous_type.is_none());
    }

    #[test]
    fn word_clear_kind_without_previous() {
        let mut w = Word::new("x".into(), WordType::Italic);
        // No set_kind called, so previous_type is None
        w.clear_kind();
        assert_eq!(w.kind(), WordType::Italic); // unchanged
        assert!(w.previous_type.is_none());
    }

    #[test]
    fn word_is_renderable() {
        // Renderable variants
        let renderable = [
            WordType::Bold,
            WordType::BoldItalic,
            WordType::Code,
            WordType::CodeBlock(Color::Reset),
            WordType::Footnote,
            WordType::FootnoteInline,
            WordType::Italic,
            WordType::Link,
            WordType::ListMarker,
            WordType::Normal,
            WordType::Selected,
            WordType::Strikethrough,
            WordType::White,
        ];
        for wt in renderable {
            let w = Word::new(String::new(), wt);
            assert!(w.is_renderable(), "{wt:?} should be renderable");
        }

        // Non-renderable variants
        let non_renderable = [
            WordType::MetaInfo(MetaData::Other),
            WordType::MetaInfo(MetaData::UList),
            WordType::MetaInfo(MetaData::OList),
            WordType::MetaInfo(MetaData::PLanguage),
            WordType::MetaInfo(MetaData::ColumnsCount),
            WordType::MetaInfo(MetaData::Important),
            WordType::MetaInfo(MetaData::Note),
            WordType::MetaInfo(MetaData::Tip),
            WordType::MetaInfo(MetaData::Warning),
            WordType::MetaInfo(MetaData::Caution),
            WordType::MetaInfo(MetaData::HeadingLevel(1)),
            WordType::LinkData,
            WordType::FootnoteData,
        ];
        for wt in non_renderable {
            let w = Word::new(String::new(), wt);
            assert!(!w.is_renderable(), "{wt:?} should NOT be renderable");
        }
    }

    #[test]
    fn word_split_off() {
        let mut w = Word::new("hello world".into(), WordType::Bold);
        w.set_kind(WordType::Selected);
        let right = w.split_off(5);

        assert_eq!(w.content(), "hello");
        assert_eq!(w.kind(), WordType::Selected);
        assert_eq!(w.previous_type(), WordType::Bold);

        assert_eq!(right.content(), " world");
        assert_eq!(right.kind(), WordType::Selected);
        assert_eq!(right.previous_type(), WordType::Bold);
    }

    #[test]
    fn wordtype_from_mdparseenum() {
        // Exhaustive mapping of every non-container variant
        let cases: &[(MdParseEnum, WordType)] = &[
            (MdParseEnum::PLanguage, WordType::MetaInfo(MetaData::Other)),
            (MdParseEnum::BlockSeparator, WordType::MetaInfo(MetaData::Other)),
            (MdParseEnum::TaskOpen, WordType::MetaInfo(MetaData::Other)),
            (MdParseEnum::TaskClosed, WordType::MetaInfo(MetaData::Other)),
            (MdParseEnum::Indent, WordType::MetaInfo(MetaData::Other)),
            (MdParseEnum::HorizontalSeparator, WordType::MetaInfo(MetaData::Other)),
            (MdParseEnum::FootnoteRef, WordType::FootnoteInline),
            (MdParseEnum::Code, WordType::Code),
            (MdParseEnum::Bold, WordType::Bold),
            (MdParseEnum::Italic, WordType::Italic),
            (MdParseEnum::Strikethrough, WordType::Strikethrough),
            (MdParseEnum::Link, WordType::Link),
            (MdParseEnum::WikiLink, WordType::Link),
            (MdParseEnum::InlineLink, WordType::Link),
            (MdParseEnum::BoldItalic, WordType::BoldItalic),
            (MdParseEnum::Digit, WordType::ListMarker),
            (MdParseEnum::Paragraph, WordType::Normal),
            (MdParseEnum::AltText, WordType::Normal),
            (MdParseEnum::Quote, WordType::Normal),
            (MdParseEnum::Sentence, WordType::Normal),
            (MdParseEnum::Word, WordType::Normal),
            (MdParseEnum::LinkData, WordType::LinkData),
            (MdParseEnum::Imortant, WordType::MetaInfo(MetaData::Important)),
            (MdParseEnum::Note, WordType::MetaInfo(MetaData::Note)),
            (MdParseEnum::Tip, WordType::MetaInfo(MetaData::Tip)),
            (MdParseEnum::Warning, WordType::MetaInfo(MetaData::Warning)),
            (MdParseEnum::Caution, WordType::MetaInfo(MetaData::Caution)),
            (MdParseEnum::CodeBlockStr, WordType::CodeBlock(Color::Reset)),
            (MdParseEnum::CodeBlockStrSpaceIndented, WordType::CodeBlock(Color::Reset)),
        ];

        for &(input, expected) in cases {
            let actual: WordType = input.into();
            assert_eq!(actual, expected, "MdParseEnum::{input:?} should map to {expected:?}");
        }
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn wordtype_from_container_panics() {
        // Container variants should panic via unreachable!
        let _: WordType = MdParseEnum::Heading.into();
    }
}
