use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_BACKGROUND_COLOR: &str = "transparent";
const DEFAULT_FONT_COLOR: &str = "#e9ecf1";
const DEFAULT_WINDOW_WIDTH: u32 = 480;
const DEFAULT_WINDOW_HEIGHT: u32 = 300;

#[derive(Clone, Debug, Serialize)]
pub struct AppConfig {
    pub appearance: AppearanceConfig,
    pub window: WindowConfig,
}

#[derive(Clone, Debug, Serialize)]
pub struct AppearanceConfig {
    pub background_color: String,
    pub font_color: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    appearance: Option<RawAppearance>,
    window: Option<RawWindow>,
}

#[derive(Debug, Deserialize)]
struct RawAppearance {
    background_color: Option<String>,
    font_color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawWindow {
    width: Option<u32>,
    height: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            background_color: DEFAULT_BACKGROUND_COLOR.to_string(),
            font_color: DEFAULT_FONT_COLOR.to_string(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    BaseDirs::new().map(|dirs| dirs.config_dir().join("search").join("search.config"))
}

fn merge_with_defaults(raw: RawConfig) -> AppConfig {
    let mut cfg = AppConfig::default();

    if let Some(appearance) = raw.appearance {
        if let Some(background_color) = appearance.background_color {
            cfg.appearance.background_color = background_color;
        }

        if let Some(font_color) = appearance.font_color {
            cfg.appearance.font_color = font_color;
        }
    }

    if let Some(window) = raw.window {
        if let Some(width) = window.width {
            cfg.window.width = width.max(1);
        }

        if let Some(height) = window.height {
            cfg.window.height = height.max(1);
        }
    }

    cfg
}

pub fn load() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig::default();
    };

    let content = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return AppConfig::default(),
    };

    let raw: RawConfig = match toml::from_str(&content) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to parse config at {:?}: {err}", path);
            return AppConfig::default();
        }
    };

    merge_with_defaults(raw)
}
