//! Configuration options specific to this fork.
//!
//! These live apart from [`crate::util::general`] so that the
//! upstream `GeneralConfig` stays untouched.

use std::sync::LazyLock;

use config::{Config, Environment, File};
use serde::Deserialize;

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

#[derive(Debug)]
pub struct CustomConfig {
    pub emoji_check_marks: bool,
    pub flavor: Flavor,
    pub search_style: SearchStyle,
}

pub static CUSTOM_CONFIG: LazyLock<CustomConfig> = LazyLock::new(|| {
    let config_dir = dirs::home_dir().unwrap();
    let config_file = config_dir.join(".config").join("mdt").join("config.toml");
    let settings = Config::builder()
        .add_source(File::with_name(config_file.to_str().unwrap()).required(false))
        .add_source(Environment::with_prefix("MDT").separator("_"))
        .build()
        .unwrap_or_default();

    CustomConfig {
        emoji_check_marks: settings.get::<bool>("emoji_check_marks").unwrap_or(true),
        flavor: settings.get::<Flavor>("flavor").unwrap_or_default(),
        search_style: settings
            .get::<SearchStyle>("search_style")
            .unwrap_or_default(),
    }
});
