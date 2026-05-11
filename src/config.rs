use serde::{Deserialize, Deserializer};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub prompt: String,
    pub hide_prompt: bool,
    pub placeholder: String,
    pub message: String,
    pub minimal_lines: bool,
    pub lines: usize,
    pub width: u32,
    pub window_x: i32,
    pub window_y: i32,
    pub font: String,
    pub icon_theme: String,
    pub icons_enabled: bool,
    pub image_size_ratio: f32,
    pub horizontal_pad: u32,
    pub vertical_pad: u32,
    pub inner_pad: u32,
    pub line_height: Option<u32>,
    pub show_paths: bool,
    pub hide_before_typing: bool,
    pub auto_select: bool,
    pub match_counter: bool,
    pub password_character: String,
    #[serde(deserialize_with = "deserialize_fields")]
    pub fields: Vec<MatchField>,
    pub match_mode: MatchMode,
    #[serde(alias = "terminal")]
    pub terminal_command: String,
    pub colors: Colors,
    pub border: BorderConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Colors {
    #[serde(deserialize_with = "deserialize_hex_color")]
    pub background: u32,
    #[serde(deserialize_with = "deserialize_hex_color")]
    pub text: u32,
    #[serde(rename = "message", alias = "message", deserialize_with = "deserialize_hex_color")]
    pub message: u32,
    #[serde(rename = "prompt", alias = "prompt", deserialize_with = "deserialize_hex_color")]
    pub prompt: u32,
    #[serde(rename = "placeholder", alias = "placeholder", deserialize_with = "deserialize_hex_color")]
    pub placeholder: u32,
    #[serde(rename = "input", alias = "input", deserialize_with = "deserialize_hex_color")]
    pub input: u32,
    #[serde(rename = "match", deserialize_with = "deserialize_hex_color")]
    pub matched_text: u32,
    #[serde(rename = "selection", alias = "selection_background", deserialize_with = "deserialize_hex_color")]
    pub selection_background: u32,
    #[serde(rename = "selection-text", alias = "selection_text", deserialize_with = "deserialize_hex_color")]
    pub selection_text: u32,
    #[serde(rename = "selection-match", alias = "selection_match", deserialize_with = "deserialize_hex_color")]
    pub selection_match: u32,
    #[serde(rename = "counter", alias = "counter", deserialize_with = "deserialize_hex_color")]
    pub counter: u32,
    #[serde(deserialize_with = "deserialize_hex_color")]
    pub border: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct BorderConfig {
    pub width: u32,
    pub radius: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchField {
    Filename,
    Name,
    Generic,
    Exec,
    Keywords,
    Categories,
    Comment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchMode {
    Exact,
    Fzf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt: "> ".to_string(),
            hide_prompt: false,
            placeholder: String::new(),
            message: String::new(),
            minimal_lines: false,
            lines: 15,
            width: 30,
            window_x: -1,
            window_y: -1,
            font: "monospace".to_string(),
            icon_theme: "default".to_string(),
            icons_enabled: true,
            image_size_ratio: 0.5,
            horizontal_pad: 40,
            vertical_pad: 8,
            inner_pad: 0,
            line_height: None,
            show_paths: false,
            hide_before_typing: false,
            auto_select: false,
            match_counter: false,
            password_character: "*".to_string(),
            fields: vec![MatchField::Filename, MatchField::Name, MatchField::Generic],
            match_mode: MatchMode::Fzf,
            terminal_command: "xterm -e {cmd}".to_string(),
            colors: Colors::default(),
            border: BorderConfig::default(),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: 0x00fdf6e3,
            text: 0x00657b83,
            message: 0x00657b83,
            prompt: 0x00586e75,
            placeholder: 0x0093a1a1,
            input: 0x00657b83,
            matched_text: 0x00cb4b16,
            selection_background: 0x00eee8d5,
            selection_text: 0x00586e75,
            selection_match: 0x00cb4b16,
            counter: 0x0093a1a1,
            border: 0x00002b36,
        }
    }
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self { width: 1, radius: 10 }
    }
}

impl Config {
    pub fn load(explicit_path: Option<&Path>) -> Self {
        let Some(path) = explicit_path.map(PathBuf::from).or_else(default_config_path) else {
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(&path) else {
            return Self::default();
        };

        toml::from_str::<Config>(&contents).unwrap_or_default()
    }
}

fn deserialize_fields<'de, D>(deserializer: D) -> Result<Vec<MatchField>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    raw.split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(|field| match field {
            "filename" => Ok(MatchField::Filename),
            "name" => Ok(MatchField::Name),
            "generic" => Ok(MatchField::Generic),
            "exec" => Ok(MatchField::Exec),
            "keywords" => Ok(MatchField::Keywords),
            "categories" => Ok(MatchField::Categories),
            "comment" => Ok(MatchField::Comment),
            other => Err(serde::de::Error::custom(format!("unknown field '{other}'"))),
        })
        .collect()
}

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    let hex = raw.trim().trim_start_matches('#');
    parse_hex_color(hex).map_err(serde::de::Error::custom)
}

fn parse_hex_color(hex: &str) -> Result<u32, String> {
    match hex.len() {
        6 => u32::from_str_radix(hex, 16).map_err(|err| err.to_string()),
        8 => u32::from_str_radix(hex, 16)
            .map(|rgba| rgba >> 8)
            .map_err(|err| err.to_string()),
        _ => Err(format!(
            "expected 6 or 8 hex digits for color, got {}",
            hex.len()
        )),
    }
}

fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("xuzzel").join("xuzzel.toml"))
}

#[cfg(test)]
mod tests {
    use super::{parse_hex_color, Config, MatchField, MatchMode};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn config_loads_toml_values() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("xuzzel-config-{stamp}.toml"));
        let contents = r##"
prompt = "apps"
lines = 12
show_paths = true
hide_before_typing = true
auto_select = true
match_counter = true
minimal_lines = true
icon_theme = "Adwaita"
icons_enabled = true
image_size_ratio = 0.6
horizontal_pad = 12
vertical_pad = 6
inner_pad = 2
line_height = 20
fields = "name,exec,keywords"
match_mode = "exact"
terminal_command = "kitty -e"
placeholder = "type"
message = "hello"
password_character = "#"

[colors]
background = "#112233"
"##;
        fs::write(&path, contents).unwrap();

        let config = Config::load(Some(&path));
        let _ = fs::remove_file(&path);

        assert_eq!(config.prompt, "apps");
        assert_eq!(config.lines, 12);
        assert!(config.show_paths);
        assert!(config.hide_before_typing);
        assert!(config.auto_select);
        assert!(config.match_counter);
        assert!(config.minimal_lines);
        assert_eq!(config.icon_theme, "Adwaita");
        assert!(config.icons_enabled);
        assert!((config.image_size_ratio - 0.6).abs() < f32::EPSILON);
        assert_eq!(config.horizontal_pad, 12);
        assert_eq!(config.vertical_pad, 6);
        assert_eq!(config.inner_pad, 2);
        assert_eq!(config.line_height, Some(20));
        assert_eq!(config.password_character, "#");
        assert_eq!(config.fields, vec![MatchField::Name, MatchField::Exec, MatchField::Keywords]);
        assert_eq!(config.match_mode, MatchMode::Exact);
        assert_eq!(config.terminal_command, "kitty -e");
        assert_eq!(config.placeholder, "type");
        assert_eq!(config.message, "hello");
        assert_eq!(config.colors.background, 0x112233);
    }

    #[test]
    fn color_parser_supports_rgb_and_rgba() {
        assert_eq!(parse_hex_color("112233").unwrap(), 0x00112233);
        assert_eq!(parse_hex_color("11223344").unwrap(), 0x00112233);
    }

    #[test]
    fn color_parser_rejects_invalid_lengths() {
        assert!(parse_hex_color("12345").is_err());
    }
}
