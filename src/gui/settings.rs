use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use iced::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub tab_visibility: HashMap<String, bool>,
    pub font_size: f32,
    pub font_family: String,
    pub font_color: ColorData,
    pub background_color: ColorData,
    pub text_color: ColorData,
    pub primary_color: ColorData,
    pub secondary_text_color: ColorData,
    pub scaling: f32,
    pub border_radius: f32,

    pub font_size_buttons: f32,
    pub font_size_titles: f32,
    pub font_size_body: f32,
    pub font_size_inputs: f32,
    pub font_size_tabs: f32,
    pub font_size_icons: f32,
    pub font_size_package_names: f32,
    pub font_size_package_details: f32,

    pub scale_buttons: f32,
    pub scale_titles: f32,
    pub scale_body: f32,
    pub scale_inputs: f32,
    pub scale_tabs: f32,
    pub scale_icons: f32,
    pub scale_package_cards: f32,
    
    #[serde(default = "default_true")]
    pub show_cfhdb_profiles: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for ColorData {
    fn from(color: Color) -> Self {
        ColorData {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

impl From<ColorData> for Color {
    fn from(data: ColorData) -> Self {
        Color::from_rgba(data.r, data.g, data.b, data.a)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    pub name: String,
    pub settings: AppSettings,
}

impl AppSettings {
    pub fn load() -> Self {
        let settings_path = Self::settings_path();
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return settings;
            }
        }

        AppSettings {
            tab_visibility: HashMap::new(),
            font_size: 14.0,
            font_family: "Inter Variable".to_string(),
            font_color: ColorData::from(Color::from_rgb(0.95, 0.95, 0.95)),
            background_color: ColorData::from(Color::from_rgb(0.12, 0.12, 0.12)),
            text_color: ColorData::from(Color::from_rgb(0.95, 0.95, 0.95)),
            primary_color: ColorData::from(Color::from_rgb(0.2, 0.6, 0.9)),
            secondary_text_color: ColorData::from(Color::from_rgb(0.7, 0.7, 0.7)),
            scaling: 1.0,
            border_radius: 16.0,

            font_size_buttons: 14.0,
            font_size_titles: 18.0,
            font_size_body: 14.0,
            font_size_inputs: 14.0,
            font_size_tabs: 13.0,
            font_size_icons: 16.0,
            font_size_package_names: 17.0,
            font_size_package_details: 13.0,

            scale_buttons: 1.0,
            scale_titles: 1.0,
            scale_body: 1.0,
            scale_inputs: 1.0,
            scale_tabs: 1.0,
            scale_icons: 1.0,
            scale_package_cards: 1.0,
            show_cfhdb_profiles: true,
        }
    }

    pub fn save(&self) {
        let settings_path = Self::settings_path();
        if let Some(parent) = settings_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&settings_path, json);
        }
    }

    pub fn settings_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("rustora").join("settings.json")
        } else {
            PathBuf::from(".config").join("rustora").join("settings.json")
        }
    }

    pub fn themes_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("rustora").join("themes")
        } else {
            PathBuf::from(".config").join("rustora").join("themes")
        }
    }

    pub fn is_tab_visible(&self, tab_name: &str) -> bool {
        self.tab_visibility.get(tab_name).copied().unwrap_or(true)
    }
}

impl CustomTheme {
    pub fn save(name: &str, settings: &AppSettings) {
        let themes_dir = AppSettings::themes_path();
        let _ = fs::create_dir_all(&themes_dir);

        let theme = CustomTheme {
            name: name.to_string(),
            settings: settings.clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&theme) {
            let theme_path = themes_dir.join(format!("{}.json", name));
            let _ = fs::write(&theme_path, json);
        }
    }

    pub fn load(name: &str) -> Option<AppSettings> {
        let themes_dir = AppSettings::themes_path();
        let theme_path = themes_dir.join(format!("{}.json", name));

        if let Ok(content) = fs::read_to_string(&theme_path) {
            if let Ok(theme) = serde_json::from_str::<CustomTheme>(&content) {
                return Some(theme.settings);
            }
        }
        None
    }

    pub fn list() -> Vec<String> {
        let themes_dir = AppSettings::themes_path();
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn delete(name: &str) {
        let themes_dir = AppSettings::themes_path();
        let theme_path = themes_dir.join(format!("{}.json", name));
        let _ = fs::remove_file(&theme_path);
    }
}
