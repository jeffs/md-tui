use std::fmt;

use config::{Config, Environment, File, Value, ValueKind};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazy_static::lazy_static;

/// Represents a single key binding with optional modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Create a new `KeyBinding` from a character (no modifiers)
    #[must_use] 
    pub fn from_char(c: char) -> Self {
        Self {
            key: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        }
    }

    /// Check if this binding matches a `KeyEvent`
    #[must_use] 
    pub fn matches(&self, event: &KeyEvent) -> bool {
        if self.key != event.code {
            return false;
        }
        // For comparison, mask out SHIFT when checking modifiers
        // (terminals may report Ctrl+Shift for some keys)
        let event_mods = event.modifiers & !KeyModifiers::SHIFT;
        let self_mods = self.modifiers & !KeyModifiers::SHIFT;
        event_mods == self_mods
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Add modifier prefix
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            write!(f, "C-")?;
        }
        // Format the key
        match self.key {
            KeyCode::Char(' ') => write!(f, "space"),
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Tab => write!(f, "tab"),
            KeyCode::Enter => write!(f, "enter"),
            KeyCode::Esc => write!(f, "esc"),
            KeyCode::Backspace => write!(f, "backspace"),
            _ => write!(f, "?"),
        }
    }
}

/// Format a Vec<KeyBinding> for display (shows first binding only for brevity)
#[must_use] 
pub fn format_bindings(bindings: &[KeyBinding]) -> String {
    bindings
        .first().map_or_else(|| "?".to_string(), std::string::ToString::to_string)
}

/// Parse a key string like "k", "space", "C-e" into a `KeyBinding`
fn parse_key_string(s: &str) -> Option<KeyBinding> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Check for C- prefix (control modifier, case-insensitive)
    let (has_ctrl, key_part) = if s.len() >= 2 && s[..2].eq_ignore_ascii_case("c-") {
        (true, &s[2..])
    } else {
        (false, s)
    };

    let modifiers = if has_ctrl {
        KeyModifiers::CONTROL
    } else {
        KeyModifiers::NONE
    };

    // Parse the key part
    let key = match key_part.to_lowercase().as_str() {
        "space" | " " => KeyCode::Char(' '),
        "tab" => KeyCode::Tab,
        "enter" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        _ => {
            // Single character
            let chars: Vec<char> = key_part.chars().collect();
            if chars.len() == 1 {
                KeyCode::Char(chars[0])
            } else {
                return None; // Invalid key string
            }
        }
    };

    Some(KeyBinding { key, modifiers })
}

/// Parse config value (string or array) into Vec<KeyBinding>
fn parse_bindings(value: &Value, default: char) -> Vec<KeyBinding> {
    match &value.kind {
        ValueKind::String(s) => {
            // Single string -> single-element Vec
            parse_key_string(s).map_or_else(|| vec![KeyBinding::from_char(default)], |kb| vec![kb])
        }
        ValueKind::Array(arr) => {
            // Array of strings
            let bindings: Vec<KeyBinding> = arr
                .iter()
                .filter_map(|v| {
                    if let ValueKind::String(s) = &v.kind {
                        parse_key_string(s)
                    } else {
                        None
                    }
                })
                .collect();
            if bindings.is_empty() {
                vec![KeyBinding::from_char(default)]
            } else {
                bindings
            }
        }
        _ => vec![KeyBinding::from_char(default)],
    }
}

/// Helper to get bindings from config with a default char
fn get_bindings(settings: &Config, key: &str, default: char) -> Vec<KeyBinding> {
    settings
        .get::<Value>(key).map_or_else(|_| vec![KeyBinding::from_char(default)], |v| parse_bindings(&v, default))
}

pub enum Action {
    Up,
    Down,
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,
    Search,
    SelectLink,
    SelectLinkAlt,
    SearchNext,
    SearchPrevious,
    Edit,
    Hover,
    Enter,
    Escape,
    ToTop,
    ToBottom,
    Help,
    Back,
    ToFileTree,
    Sort,
    None,
}

#[derive(Debug)]
pub struct KeyConfig {
    pub up: Vec<KeyBinding>,
    pub down: Vec<KeyBinding>,
    pub page_up: Vec<KeyBinding>,
    pub page_down: Vec<KeyBinding>,
    pub half_page_up: Vec<KeyBinding>,
    pub half_page_down: Vec<KeyBinding>,
    pub search: Vec<KeyBinding>,
    pub search_next: Vec<KeyBinding>,
    pub search_previous: Vec<KeyBinding>,
    pub select_link: Vec<KeyBinding>,
    pub select_link_alt: Vec<KeyBinding>,
    pub edit: Vec<KeyBinding>,
    pub hover: Vec<KeyBinding>,
    pub top: Vec<KeyBinding>,
    pub bottom: Vec<KeyBinding>,
    pub back: Vec<KeyBinding>,
    pub file_tree: Vec<KeyBinding>,
    pub sort: Vec<KeyBinding>,
}

/// Helper to check if any binding in a list matches the event
fn matches_any(bindings: &[KeyBinding], event: &KeyEvent) -> bool {
    bindings.iter().any(|b| b.matches(event))
}

#[must_use] 
pub fn key_to_action(event: &KeyEvent) -> Action {
    // Check for hardcoded keys first (arrow keys, etc.)
    match event.code {
        KeyCode::Up => return Action::Up,
        KeyCode::Down => return Action::Down,
        KeyCode::PageUp => return Action::PageUp,
        KeyCode::PageDown => return Action::PageDown,
        KeyCode::Right => return Action::PageDown,
        KeyCode::Left => return Action::PageUp,
        KeyCode::Enter => return Action::Enter,
        KeyCode::Esc => return Action::Escape,
        _ => {}
    }

    // Check configurable bindings
    if matches_any(&KEY_CONFIG.up, event) {
        return Action::Up;
    }
    if matches_any(&KEY_CONFIG.down, event) {
        return Action::Down;
    }
    if matches_any(&KEY_CONFIG.page_up, event) {
        return Action::PageUp;
    }
    if matches_any(&KEY_CONFIG.page_down, event) {
        return Action::PageDown;
    }
    if matches_any(&KEY_CONFIG.half_page_up, event) {
        return Action::HalfPageUp;
    }
    if matches_any(&KEY_CONFIG.half_page_down, event) {
        return Action::HalfPageDown;
    }
    if matches_any(&KEY_CONFIG.search, event) {
        return Action::Search;
    }
    // Also check hardcoded '/' for search
    if event.code == KeyCode::Char('/') && event.modifiers == KeyModifiers::NONE {
        return Action::Search;
    }
    if matches_any(&KEY_CONFIG.select_link, event) {
        return Action::SelectLink;
    }
    if matches_any(&KEY_CONFIG.select_link_alt, event) {
        return Action::SelectLinkAlt;
    }
    if matches_any(&KEY_CONFIG.search_next, event) {
        return Action::SearchNext;
    }
    if matches_any(&KEY_CONFIG.search_previous, event) {
        return Action::SearchPrevious;
    }
    if matches_any(&KEY_CONFIG.edit, event) {
        return Action::Edit;
    }
    if matches_any(&KEY_CONFIG.hover, event) {
        return Action::Hover;
    }
    if matches_any(&KEY_CONFIG.top, event) {
        return Action::ToTop;
    }
    if matches_any(&KEY_CONFIG.bottom, event) {
        return Action::ToBottom;
    }
    if matches_any(&KEY_CONFIG.back, event) {
        return Action::Back;
    }
    if matches_any(&KEY_CONFIG.file_tree, event) {
        return Action::ToFileTree;
    }
    if matches_any(&KEY_CONFIG.sort, event) {
        return Action::Sort;
    }
    // Hardcoded help key
    if event.code == KeyCode::Char('?') && event.modifiers == KeyModifiers::NONE {
        return Action::Help;
    }

    Action::None
}

lazy_static! {
    pub static ref KEY_CONFIG: KeyConfig = {
        let config_dir = dirs::home_dir().unwrap();
        let config_file = config_dir.join(".config").join("mdt").join("config.toml");
        let settings = Config::builder()
            .add_source(File::with_name(config_file.to_str().unwrap()).required(false))
            .add_source(Environment::with_prefix("MDT").separator("_"))
            .build()
            .unwrap();

        KeyConfig {
            up: get_bindings(&settings, "up", 'k'),
            down: get_bindings(&settings, "down", 'j'),
            page_up: get_bindings(&settings, "page_up", 'u'),
            page_down: get_bindings(&settings, "page_down", 'd'),
            half_page_up: get_bindings(&settings, "half_page_up", 'h'),
            half_page_down: get_bindings(&settings, "half_page_down", 'l'),
            search: get_bindings(&settings, "search", 'f'),
            select_link: get_bindings(&settings, "select_link", 's'),
            select_link_alt: get_bindings(&settings, "select_link_alt", 'S'),
            search_next: get_bindings(&settings, "search_next", 'n'),
            search_previous: get_bindings(&settings, "search_previous", 'N'),
            edit: get_bindings(&settings, "edit", 'e'),
            hover: get_bindings(&settings, "hover", 'K'),
            top: get_bindings(&settings, "top", 'g'),
            bottom: get_bindings(&settings, "bottom", 'G'),
            back: get_bindings(&settings, "back", 'b'),
            file_tree: get_bindings(&settings, "file_tree", 't'),
            sort: get_bindings(&settings, "sort", 'o'),
        }
    };
}
