use std::sync::LazyLock;

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug)]
pub struct GeneralConfig {
    pub width: u16,
    pub gitignore: bool,
    pub centering: Centering,
    pub help_menu: bool,
    pub emoji_check_marks: bool,
    pub flavor: Flavor,
    pub search_style: SearchStyle,
}

#[derive(Debug, Deserialize)]
pub enum Centering {
    Left,
    Center,
    Right,
}

/// Markdown flavor for parsing behavior.
#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// Standard CommonMark behavior: newlines within paragraphs
    /// become spaces.
    #[default]
    #[serde(alias = "commonmark")]
    CommonMark,
    /// Claude-style Markdown: newlines within paragraphs become
    /// line breaks.
    #[serde(alias = "claude")]
    Claude,
}

/// Search style for in-document search.
#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub enum SearchStyle {
    /// Whole-word matching only.
    #[serde(alias = "word")]
    Word,
    /// Flexible matching: substring for single words, phrase
    /// matching for multi-word.
    #[default]
    #[serde(alias = "flex")]
    Flex,
    /// Fuzzy matching using Damerau-Levenshtein distance.
    #[serde(alias = "fuzz")]
    Fuzz,
}

pub static GENERAL_CONFIG: LazyLock<GeneralConfig> = LazyLock::new(|| {
    let config_dir = dirs::home_dir().unwrap();
    let config_file = config_dir.join(".config").join("mdt").join("config.toml");
    let settings = Config::builder()
        .add_source(File::with_name(config_file.to_str().unwrap()).required(false))
        .add_source(Environment::with_prefix("MDT").separator("_"))
        .build()
        .unwrap();

    let width = settings.get::<u16>("width").unwrap_or(100);
    GeneralConfig {
        // width = 0 means "use full terminal width"
        width: if width == 0 { u16::MAX } else { width },
        gitignore: settings.get::<bool>("gitignore").unwrap_or(false),
        centering: settings
            .get::<Centering>("alignment")
            .unwrap_or(Centering::Left),
        help_menu: settings.get::<bool>("help_menu").unwrap_or(true),
        emoji_check_marks: settings.get::<bool>("emoji_check_marks").unwrap_or(true),
        flavor: settings.get::<Flavor>("flavor").unwrap_or_default(),
        search_style: settings.get::<SearchStyle>("search_style").unwrap_or_default(),
    }
});
