use std::{collections::HashMap, path::PathBuf};

use color_eyre::eyre::Result;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use indexmap::IndexMap;
use itertools::Itertools;
use ratatui::style::{Color, Modifier, Style};
use serde::{de::Deserializer, Deserialize};

use crate::{action::Action, mode::Mode};

const CONFIG: &str = include_str!("../.config/config.json5");

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub _data_dir: PathBuf,
    #[serde(default)]
    pub _config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub styles: Styles,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let default_config: Config = json5::from_str(CONFIG).unwrap();
        let data_dir = crate::utils::get_data_dir();
        let config_dir = crate::utils::get_config_dir();
        let mut builder = config::Config::builder()
            .set_default("_data_dir", data_dir.to_str().unwrap())?
            .set_default("_config_dir", config_dir.to_str().unwrap())?;

        let config_files = [
            ("config.json5", config::FileFormat::Json5),
            ("config.json", config::FileFormat::Json),
            ("config.yaml", config::FileFormat::Yaml),
            ("config.toml", config::FileFormat::Toml),
            ("config.ini", config::FileFormat::Ini),
        ];
        let mut found_config = false;
        for (file, format) in &config_files {
            builder = builder.add_source(
                config::File::from(config_dir.join(file))
                    .format(*format)
                    .required(false),
            );
            if config_dir.join(file).exists() {
                found_config = true
            }
        }
        if !found_config {
            log::debug!("No configuration file found. Using default config.");
        }

        Ok(default_config)
    }
}

#[derive(Clone, Debug, Default)]
pub struct KeyBindings {
    pub bindings: HashMap<Mode, HashMap<Vec<KeyEvent>, Action>>,
    pub display_names: HashMap<Mode, Vec<String>>,
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let parsed_map = IndexMap::<Mode, ModeValue>::deserialize(deserializer)?;
        let modes: IndexMap<Mode, Vec<ModeValue>> = parsed_map
            .keys()
            .map(|mode| (mode.clone(), collect_modes(&parsed_map, mode)))
            .collect();
        let mut bindings: HashMap<Mode, HashMap<Vec<KeyEvent>, Action>> = HashMap::new();
        let mut events_to_display_names: HashMap<Mode, IndexMap<Vec<KeyEvent>, String>> = HashMap::new();
        for (mode, mode_values) in modes {
            let mut events_to_action: HashMap<Vec<KeyEvent>, Action> = HashMap::new();
            let mut events_to_display_name: IndexMap<Vec<KeyEvent>, String> = IndexMap::new();

            let groups: IndexMap<String, GroupValue> = mode_values
                .iter()
                .flat_map(|mode_value| mode_value.clone().display_groups)
                .flatten()
                .collect();
            let mut display_groups: IndexMap<String, Vec<KeyEvent>> = groups
                .keys()
                .into_iter()
                .map(|key| (key.clone(), Vec::new()))
                .collect();

            for mode_value in mode_values {
                for (key, key_binding) in mode_value.key_bindings {
                    let events = parse_key_sequence(&key).unwrap();
                    if let Some(group) = key_binding.display_group {
                        display_groups
                            .get_mut(&group)
                            .unwrap()
                            .append(&mut events.clone());
                    } else if let Some(display_name) = key_binding.display_name {
                        events_to_display_name.insert(events.clone(), display_name);
                    }

                    events_to_action.insert(events, key_binding.action.clone());
                }
            }

            for (group_name, key_events) in display_groups {
                events_to_display_name.insert(
                    key_events,
                    groups.get(&group_name).unwrap().display_name.clone(),
                );
            }

            bindings.insert(mode, events_to_action);
            events_to_display_names.insert(mode, events_to_display_name);
        }

        Ok(KeyBindings {
            bindings,
            display_names: to_display_names(events_to_display_names),
        })
    }
}

fn to_display_names(
    map: HashMap<Mode, IndexMap<Vec<KeyEvent>, String>>,
) -> HashMap<Mode, Vec<String>> {
    let mut display_names_map: HashMap<Mode, Vec<String>> = HashMap::new();
    for (mode, map) in map {
        let mut key_hints: Vec<String> = Vec::new();
        for (key_events, display_name) in map {
            let keys = key_events
                .iter()
                .map(|event| {
                    let prefix = if event.modifiers == KeyModifiers::SHIFT {
                        Some("⇧")
                    } else if event.modifiers == KeyModifiers::CONTROL {
                        if cfg!(target_os = "macos") {
                            Some("⌃")
                        } else {
                            Some("Ctrl-")
                        }
                    } else if event.modifiers == KeyModifiers::ALT {
                        if cfg!(target_os = "macos") {
                            Some("⌥")
                        } else {
                            Some("Alt-")
                        }
                    } else {
                        None
                    };

                    let symbol = match event.code {
                        KeyCode::Backspace => "⌫".to_string(),
                        KeyCode::Enter => "⏎".to_string(),
                        KeyCode::Left => "←".to_string(),
                        KeyCode::Right => "→".to_string(),
                        KeyCode::Up => "↑".to_string(),
                        KeyCode::Down => "↓".to_string(),
                        KeyCode::Home => "⌂".to_string(),
                        KeyCode::End => "⇲".to_string(),
                        KeyCode::PageUp => "PgUp".to_string(),
                        KeyCode::PageDown => "PgDown".to_string(),
                        KeyCode::Tab => "↹".to_string(),
                        KeyCode::BackTab => "⇧↹".to_string(),
                        KeyCode::Delete => "⌫".to_string(),
                        KeyCode::Insert => "Insert".to_string(),
                        KeyCode::F(num) => format!("F{num}").to_string(),
                        KeyCode::Char(c) => c.to_string(),
                        KeyCode::Null => "Null".to_string(),
                        KeyCode::Esc => "Esc".to_string(),
                        KeyCode::CapsLock => "⇪".to_string(),
                        KeyCode::ScrollLock => "⤓".to_string(),
                        KeyCode::NumLock => "NumLock".to_string(),
                        KeyCode::PrintScreen => "PrtSc".to_string(),
                        KeyCode::Pause => "Pause".to_string(),
                        KeyCode::Menu => "Menu".to_string(),
                        KeyCode::KeypadBegin => "￣".to_string(),
                        KeyCode::Media(_) => "?".to_string(),
                        KeyCode::Modifier(_) => "?".to_string(),
                    };

                    match prefix {
                        None => symbol,
                        Some(prefix) => format!("{prefix}{symbol}"),
                    }
                })
                .join("");
            key_hints.push(format!("{} [{}]", display_name, keys))
        }
        display_names_map.insert(mode, key_hints);
    }

    display_names_map
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModeValue {
    #[serde(rename = "displayGroups")]
    display_groups: Option<IndexMap<String, GroupValue>>,
    extends: Option<Vec<Mode>>,
    #[serde(flatten)]
    key_bindings: IndexMap<String, KeyBinding>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeyBinding {
    action: Action,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "displayGroup")]
    display_group: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupValue {
    #[serde(rename = "displayName")]
    display_name: String,
}

fn collect_modes(modes: &IndexMap<Mode, ModeValue>, mode: &Mode) -> Vec<ModeValue> {
    let mode_value = modes
        .get(mode)
        .expect(&*format!("Mode {:?} not found", mode));
    let mut mode_values = Vec::new();
    if let Some(extended_modes) = &mode_value.extends {
        for extended_mode in extended_modes {
            mode_values.append(collect_modes(modes, extended_mode).as_mut());
        }
    }
    mode_values.push(mode_value.clone());
    mode_values
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

pub fn key_event_to_string(key_event: &KeyEvent) -> String {
    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "backspace",
        KeyCode::Enter => "enter",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::Tab => "tab",
        KeyCode::BackTab => "backtab",
        KeyCode::Delete => "delete",
        KeyCode::Insert => "insert",
        KeyCode::F(c) => {
            char = format!("f({c})");
            &char
        }
        KeyCode::Char(c) if c == ' ' => "space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("ctrl");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("shift");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("alt");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
    if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
        return Err(format!("Unable to parse `{}`", raw));
    }
    let raw = if !raw.contains("><") {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        let raw = raw.strip_prefix('>').unwrap_or(raw);
        raw
    } else {
        raw
    };
    let sequences = raw
        .split("><")
        .map(|seq| {
            if let Some(s) = seq.strip_prefix('<') {
                s
            } else if let Some(s) = seq.strip_suffix('>') {
                s
            } else {
                seq
            }
        })
        .collect::<Vec<_>>();

    sequences.into_iter().map(parse_key_event).collect()
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styles(pub HashMap<Mode, HashMap<String, Style>>);

impl<'de> Deserialize<'de> for Styles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<Mode, HashMap<String, String>>::deserialize(deserializer)?;

        let styles = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(str, style)| (str, parse_style(&style)))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(Styles(styles))
    }
}

pub fn parse_style(line: &str) -> Style {
    let (foreground, background) =
        line.split_at(line.to_lowercase().find("on ").unwrap_or(line.len()));
    let foreground = process_color_string(foreground);
    let background = process_color_string(&background.replace("on ", ""));

    let mut style = Style::default();
    if let Some(fg) = parse_color(&foreground.0) {
        style = style.fg(fg);
    }
    if let Some(bg) = parse_color(&background.0) {
        style = style.bg(bg);
    }
    style = style.add_modifier(foreground.1 | background.1);
    style
}

fn process_color_string(color_str: &str) -> (String, Modifier) {
    let color = color_str
        .replace("grey", "gray")
        .replace("bright ", "")
        .replace("bold ", "")
        .replace("underline ", "")
        .replace("inverse ", "");

    let mut modifiers = Modifier::empty();
    if color_str.contains("underline") {
        modifiers |= Modifier::UNDERLINED;
    }
    if color_str.contains("bold") {
        modifiers |= Modifier::BOLD;
    }
    if color_str.contains("inverse") {
        modifiers |= Modifier::REVERSED;
    }

    (color, modifiers)
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim_start();
    let s = s.trim_end();
    if s.contains("bright color") {
        let s = s.trim_start_matches("bright ");
        let c = s
            .trim_start_matches("color")
            .parse::<u8>()
            .unwrap_or_default();
        Some(Color::Indexed(c.wrapping_shl(8)))
    } else if s.contains("color") {
        let c = s
            .trim_start_matches("color")
            .parse::<u8>()
            .unwrap_or_default();
        Some(Color::Indexed(c))
    } else if s.contains("gray") {
        let c = 232
            + s.trim_start_matches("gray")
            .parse::<u8>()
            .unwrap_or_default();
        Some(Color::Indexed(c))
    } else if s.contains("rgb") {
        let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
        let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
        let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
        let c = 16 + red * 36 + green * 6 + blue;
        Some(Color::Indexed(c))
    } else if s == "bold black" {
        Some(Color::Indexed(8))
    } else if s == "bold red" {
        Some(Color::Indexed(9))
    } else if s == "bold green" {
        Some(Color::Indexed(10))
    } else if s == "bold yellow" {
        Some(Color::Indexed(11))
    } else if s == "bold blue" {
        Some(Color::Indexed(12))
    } else if s == "bold magenta" {
        Some(Color::Indexed(13))
    } else if s == "bold cyan" {
        Some(Color::Indexed(14))
    } else if s == "bold white" {
        Some(Color::Indexed(15))
    } else if s == "black" {
        Some(Color::Indexed(0))
    } else if s == "red" {
        Some(Color::Indexed(1))
    } else if s == "green" {
        Some(Color::Indexed(2))
    } else if s == "yellow" {
        Some(Color::Indexed(3))
    } else if s == "blue" {
        Some(Color::Indexed(4))
    } else if s == "magenta" {
        Some(Color::Indexed(5))
    } else if s == "cyan" {
        Some(Color::Indexed(6))
    } else if s == "white" {
        Some(Color::Indexed(7))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_style_default() {
        let style = parse_style("");
        assert_eq!(style, Style::default());
    }

    #[test]
    fn test_parse_style_foreground() {
        let style = parse_style("red");
        assert_eq!(style.fg, Some(Color::Indexed(1)));
    }

    #[test]
    fn test_parse_style_background() {
        let style = parse_style("on blue");
        assert_eq!(style.bg, Some(Color::Indexed(4)));
    }

    #[test]
    fn test_parse_style_modifiers() {
        let style = parse_style("underline red on blue");
        assert_eq!(style.fg, Some(Color::Indexed(1)));
        assert_eq!(style.bg, Some(Color::Indexed(4)));
    }

    #[test]
    fn test_process_color_string() {
        let (color, modifiers) = process_color_string("underline bold inverse gray");
        assert_eq!(color, "gray");
        assert!(modifiers.contains(Modifier::UNDERLINED));
        assert!(modifiers.contains(Modifier::BOLD));
        assert!(modifiers.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_parse_color_rgb() {
        let color = parse_color("rgb123");
        let expected = 16 + 1 * 36 + 2 * 6 + 3;
        assert_eq!(color, Some(Color::Indexed(expected)));
    }

    #[test]
    fn test_parse_color_unknown() {
        let color = parse_color("unknown");
        assert_eq!(color, None);
    }

    #[test]
    fn test_config() -> Result<()> {
        let c = Config::new()?;
        assert_eq!(
            c.keybindings
                .bindings
                .get(&Mode::Home)
                .unwrap()
                .get(&parse_key_sequence("<q>").unwrap_or_default())
                .unwrap(),
            &Action::Quit
        );
        assert_eq!(c.keybindings.display_names
                       .get(&Mode::Home)
                       .unwrap()
                       .get(0).unwrap(),
                   "Quit [q]");
        Ok(())
    }

    #[test]
    fn test_simple_keys() {
        assert_eq!(
            parse_key_event("a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );

        assert_eq!(
            parse_key_event("enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())
        );

        assert_eq!(
            parse_key_event("esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())
        );
    }

    #[test]
    fn test_with_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            parse_key_event("alt-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );

        assert_eq!(
            parse_key_event("shift-esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_multiple_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-alt-a").unwrap(),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            )
        );

        assert_eq!(
            parse_key_event("ctrl-shift-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_reverse_multiple_modifiers() {
        assert_eq!(
            key_event_to_string(&KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            )),
            "ctrl-alt-a".to_string()
        );
    }

    #[test]
    fn test_invalid_keys() {
        assert!(parse_key_event("invalid-key").is_err());
        assert!(parse_key_event("ctrl-invalid-key").is_err());
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(
            parse_key_event("CTRL-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            parse_key_event("AlT-eNtEr").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );
    }
}
