use std::{fmt, sync::LazyLock};

use config::{Config, Environment, File, Value, ValueKind};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
        // Mask out SHIFT when checking modifiers (terminals may
        // report Ctrl+Shift for some keys)
        let event_mods = event.modifiers & !KeyModifiers::SHIFT;
        let self_mods = self.modifiers & !KeyModifiers::SHIFT;
        event_mods == self_mods
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            write!(f, "C-")?;
        }
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

/// Format a slice of bindings for display (shows first only)
#[must_use]
pub fn format_bindings(bindings: &[KeyBinding]) -> String {
    bindings
        .first()
        .map_or_else(|| "?".to_string(), ToString::to_string)
}

/// Parse a key string like "k", "space", "C-e" into a KeyBinding
fn parse_key_string(s: &str) -> Option<KeyBinding> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (has_ctrl, key_part) =
        if s.len() >= 2 && s[..2].eq_ignore_ascii_case("c-") {
            (true, &s[2..])
        } else {
            (false, s)
        };

    let modifiers = if has_ctrl {
        KeyModifiers::CONTROL
    } else {
        KeyModifiers::NONE
    };

    let key = match key_part.to_lowercase().as_str() {
        "space" | " " => KeyCode::Char(' '),
        "tab" => KeyCode::Tab,
        "enter" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        _ => {
            let chars: Vec<char> = key_part.chars().collect();
            if chars.len() == 1 {
                KeyCode::Char(chars[0])
            } else {
                return None;
            }
        }
    };

    Some(KeyBinding { key, modifiers })
}

/// Parse config value (string or array) into Vec<KeyBinding>
fn parse_bindings(value: &Value, default: char) -> Vec<KeyBinding> {
    match &value.kind {
        ValueKind::String(s) => parse_key_string(s)
            .map_or_else(
                || vec![KeyBinding::from_char(default)],
                |kb| vec![kb],
            ),
        ValueKind::Array(arr) => {
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

/// Get bindings from config with a default char
fn get_bindings(
    settings: &Config,
    key: &str,
    default: char,
) -> Vec<KeyBinding> {
    settings.get::<Value>(key).map_or_else(
        |_| vec![KeyBinding::from_char(default)],
        |v| parse_bindings(&v, default),
    )
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

fn matches_any(bindings: &[KeyBinding], event: &KeyEvent) -> bool {
    bindings.iter().any(|b| b.matches(event))
}

#[must_use]
pub fn key_to_action(event: &KeyEvent) -> Action {
    // Hardcoded keys (arrow keys, etc.)
    match event.code {
        KeyCode::Up => return Action::Up,
        KeyCode::Down => return Action::Down,
        KeyCode::PageUp | KeyCode::Left => return Action::PageUp,
        KeyCode::PageDown | KeyCode::Right => return Action::PageDown,
        KeyCode::Enter => return Action::Enter,
        KeyCode::Esc => return Action::Escape,
        _ => {}
    }

    // Configurable bindings
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
    if event.code == KeyCode::Char('/')
        && event.modifiers == KeyModifiers::NONE
    {
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
    if event.code == KeyCode::Char('?')
        && event.modifiers == KeyModifiers::NONE
    {
        return Action::Help;
    }

    Action::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // -- parse_key_string tests --

    #[test]
    fn parse_key_string_char() {
        let kb = parse_key_string("k").unwrap();
        assert_eq!(kb.key, KeyCode::Char('k'));
        assert_eq!(kb.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn parse_key_string_space() {
        let kb = parse_key_string("space").unwrap();
        assert_eq!(kb.key, KeyCode::Char(' '));
        assert_eq!(kb.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn parse_key_string_ctrl() {
        let kb = parse_key_string("C-e").unwrap();
        assert_eq!(kb.key, KeyCode::Char('e'));
        assert_eq!(kb.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn parse_key_string_tab() {
        let kb = parse_key_string("tab").unwrap();
        assert_eq!(kb.key, KeyCode::Tab);
        assert_eq!(kb.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn parse_key_string_enter() {
        let kb = parse_key_string("enter").unwrap();
        assert_eq!(kb.key, KeyCode::Enter);
        assert_eq!(kb.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn parse_key_string_esc() {
        let kb = parse_key_string("esc").unwrap();
        assert_eq!(kb.key, KeyCode::Esc);
        assert_eq!(kb.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn parse_key_string_empty() {
        assert!(parse_key_string("").is_none());
    }

    #[test]
    fn parse_key_string_invalid() {
        // Multi-char non-keyword string is not a valid key
        assert!(parse_key_string("xyz").is_none());
    }

    // -- KeyBinding::matches tests --

    #[test]
    fn keybinding_matches_exact() {
        let kb = KeyBinding::from_char('k');
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(kb.matches(&event));
    }

    #[test]
    fn keybinding_matches_ignores_shift() {
        // A Ctrl binding should match even when the terminal also reports SHIFT
        let kb = KeyBinding {
            key: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
        };
        let event = KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(kb.matches(&event));
    }

    #[test]
    fn keybinding_no_false_positive() {
        let kb = KeyBinding::from_char('k');
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!kb.matches(&event));
    }

    // -- Display tests --

    #[test]
    fn keybinding_display() {
        assert_eq!(
            KeyBinding {
                key: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
            }
            .to_string(),
            "C-e"
        );
        assert_eq!(
            KeyBinding::from_char(' ').to_string(),
            "space"
        );
        assert_eq!(
            KeyBinding::from_char('k').to_string(),
            "k"
        );
    }

    // -- key_to_action tests --

    #[test]
    fn key_to_action_arrow_keys() {
        // Arrow keys are hardcoded — not affected by config
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(matches!(key_to_action(&up), Action::Up));

        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(matches!(key_to_action(&down), Action::Down));

        let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert!(matches!(key_to_action(&left), Action::PageUp));

        let right = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(matches!(key_to_action(&right), Action::PageDown));

        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(key_to_action(&enter), Action::Enter));

        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(key_to_action(&esc), Action::Escape));
    }

    #[test]
    fn key_to_action_configurable() {
        // Assumption: KEY_CONFIG uses default bindings (no user config override).
        // Default vim bindings: j→Down, k→Up, f→Search, s→SelectLink, etc.
        let j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&j), Action::Down));

        let k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&k), Action::Up));

        let f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&f), Action::Search));

        let s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&s), Action::SelectLink));

        let e = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&e), Action::Edit));

        let g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&g), Action::ToTop));

        // Unbound key should return None
        let z = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);
        assert!(matches!(key_to_action(&z), Action::None));
    }
}

pub static KEY_CONFIG: LazyLock<KeyConfig> = LazyLock::new(|| {
    let config_dir = dirs::home_dir().unwrap();
    let config_file = config_dir
        .join(".config")
        .join("mdt")
        .join("config.toml");
    let settings = Config::builder()
        .add_source(
            File::with_name(config_file.to_str().unwrap())
                .required(false),
        )
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
});
