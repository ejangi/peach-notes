use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub notes_dir: PathBuf,
    pub window_width: i32,
    pub window_height: i32,
    pub is_maximized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_notes_dir = match std::env::var("HOME") {
            Ok(home) => PathBuf::from(home).join("Documents").join("Notes"),
            Err(_) => PathBuf::from("./.notes"),
        };

        Self {
            notes_dir: default_notes_dir,
            window_width: 900,
            window_height: 650,
            is_maximized: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let config_dir = match std::env::var("XDG_CONFIG_HOME") {
            Ok(path) => PathBuf::from(path).join("peach-notes"),
            Err(_) => match std::env::var("HOME") {
                Ok(home) => PathBuf::from(home).join(".config").join("peach-notes"),
                Err(_) => PathBuf::from("./.config/peach-notes"),
            },
        };
        config_dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }

        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.window_width, 900);
        assert_eq!(config.window_height, 650);
        assert!(!config.is_maximized);
    }
}
