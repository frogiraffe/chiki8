use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub emulation: EmulationConfig,
    #[serde(default)]
    pub keymap: KeymapConfig,
}

#[derive(Debug, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_scale")]
    pub scale: u32,
    #[serde(default = "default_background")]
    pub background: [u8; 3],
    #[serde(default = "default_foreground")]
    pub foreground: [u8; 3],
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            scale: default_scale(),
            background: default_background(),
            foreground: default_foreground(),
        }
    }
}

fn default_scale() -> u32 {
    15
}
fn default_background() -> [u8; 3] {
    [0, 0, 0]
}
fn default_foreground() -> [u8; 3] {
    [255, 255, 255]
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_volume")]
    pub volume: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            volume: default_volume(),
        }
    }
}

fn default_volume() -> u32 {
    25
}

#[derive(Debug, Deserialize)]
pub struct EmulationConfig {
    #[serde(default = "default_speed")]
    pub speed: u32,
}

impl Default for EmulationConfig {
    fn default() -> Self {
        Self {
            speed: default_speed(),
        }
    }
}

fn default_speed() -> u32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct KeymapConfig {
    #[serde(default = "default_keymap")]
    pub keys: HashMap<String, u8>,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            keys: default_keymap(),
        }
    }
}

fn default_keymap() -> HashMap<String, u8> {
    let mut map = HashMap::new();
    map.insert("1".to_string(), 0x1);
    map.insert("2".to_string(), 0x2);
    map.insert("3".to_string(), 0x3);
    map.insert("4".to_string(), 0xC);
    map.insert("Q".to_string(), 0x4);
    map.insert("W".to_string(), 0x5);
    map.insert("E".to_string(), 0x6);
    map.insert("R".to_string(), 0xD);
    map.insert("A".to_string(), 0x7);
    map.insert("S".to_string(), 0x8);
    map.insert("D".to_string(), 0x9);
    map.insert("F".to_string(), 0xE);
    map.insert("Z".to_string(), 0xA);
    map.insert("X".to_string(), 0x0);
    map.insert("C".to_string(), 0xB);
    map.insert("V".to_string(), 0xF);
    map
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Could not read config file: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("Config parse error: {}", e))
    }

    pub fn get_keycode(&self, key_name: &str) -> Option<usize> {
        self.keymap.keys.get(key_name).map(|&v| v as usize)
    }
}

/// Create example config file
pub fn create_example_config(path: &Path) -> std::io::Result<()> {
    let example = r#"# Chiki8 CHIP-8 Emulator Configuration File

[display]
scale = 15
background = [0, 0, 0]       # Black background
foreground = [255, 255, 255] # White foreground

[audio]
volume = 25  # 0-100 range

[emulation]
speed = 10   # Ticks per frame

[keymap]
# CHIP-8 keys -> Keyboard keys
# Format: "KEYBOARD_KEY" = CHIP8_VALUE (hex)
[keymap.keys]
"1" = 0x1
"2" = 0x2
"3" = 0x3
"4" = 0xC
"Q" = 0x4
"W" = 0x5
"E" = 0x6
"R" = 0xD
"A" = 0x7
"S" = 0x8
"D" = 0x9
"F" = 0xE
"Z" = 0xA
"X" = 0x0
"C" = 0xB
"V" = 0xF
"#;
    fs::write(path, example)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invalid(text: &str, field: &str) {
        let error = Config::parse("test.toml", text).unwrap_err();
        assert!(error.contains(field), "{error:?} did not name {field:?}");
    }

    #[test]
    fn config_validation_accepts_defaults() {
        Config::parse("test.toml", "").unwrap();
    }

    #[test]
    fn config_validation_rejects_unknown_and_invalid_fields() {
        invalid("unknown = 1", "unknown");
        invalid("[display]\nunknown = 1", "unknown");
        invalid("[display]\nscale = 0", "display.scale");
        invalid("[display]\nfilter = 'blur'", "display.filter");
        invalid("[display]\nbackground = [0, 1]", "display.background");
        invalid("[audio]\nvolume = 101", "audio.volume");
        invalid("[emulation]\nspeed = 0", "emulation.speed");
        invalid("[emulation]\nrefresh = 59", "emulation.refresh");
        invalid("[emulation]\nprofile = 'SuperChip-1.1'", "emulation.profile");
        invalid("[keymap.keys]\nUnknownKey = 1", "keymap.keys.UnknownKey");
        invalid("[keymap.keys]\nQ = 16", "keymap.keys.Q");
        invalid("[keymap]\nkeys = {}", "keymap.keys");
    }

    #[test]
    fn strict_color_parser_names_cli_field() {
        assert_eq!(parse_color("CLI background", "1,2,3"), Ok([1, 2, 3]));
        assert!(parse_color("CLI background", "1,2").unwrap_err().contains("CLI background"));
        assert!(parse_color("CLI background", "1,2,999").unwrap_err().contains("CLI background"));
    }
}
