use std::path::PathBuf;
use std::fs;
use tokio::process::Command as TokioCommand;
use once_cell::sync::Lazy;

const INTER_FONT_URL: &str = "https://github.com/rsms/inter/releases/download/v4.0/Inter-4.0.zip";
const FIRA_CODE_URL: &str = "https://github.com/tonsky/FiraCode/releases/download/6.2/Fira_Code_v6.2.zip";
const MATERIAL_SYMBOLS_URL: &str = "https://github.com/google/material-design-icons/raw/master/variablefont/MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf";
const FONT_DIR: &str = ".local/share/fonts";

static INTER_FONT_CACHE: Lazy<iced::Font> = Lazy::new(|| {
    load_inter_font().unwrap_or(iced::Font::DEFAULT)
});

static MATERIAL_SYMBOLS_FONT_CACHE: Lazy<iced::Font> = Lazy::new(|| {
    load_material_symbols_font().unwrap_or(iced::Font::DEFAULT)
});

pub fn fonts_exist() -> bool {
    if let Ok(home) = std::env::var("HOME") {
        let font_dir = PathBuf::from(&home).join(FONT_DIR);
        let inter_variable = font_dir.join("InterVariable.ttf");
        let _inter_variable_italic = font_dir.join("InterVariable-Italic.ttf");
        let material_symbols = font_dir.join("MaterialSymbolsRounded.ttf");

        inter_variable.exists() && material_symbols.exists()
    } else {
        false
    }
}

pub async fn ensure_fonts() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    let font_dir = PathBuf::from(&home).join(FONT_DIR);

    fs::create_dir_all(&font_dir).map_err(|e| format!("Failed to create font directory: {}", e))?;

    let inter_variable = font_dir.join("InterVariable.ttf");
    let inter_variable_italic = font_dir.join("InterVariable-Italic.ttf");
    if !inter_variable.exists() || !inter_variable_italic.exists() {
        let zip_path = "/tmp/Inter.zip";
        let output = TokioCommand::new("curl")
            .args(["-L", INTER_FONT_URL, "-o", zip_path])
            .output()
            .await
            .map_err(|e| format!("Failed to download InterVariable font: {}", e))?;

        if output.status.success() {
            let extract_output = TokioCommand::new("unzip")
                .args(["-j", "-o", zip_path, "InterVariable.ttf", "InterVariable-Italic.ttf", "-d"])
                .arg(&font_dir)
                .output()
                .await;

            if let Ok(extract) = extract_output {
                if !extract.status.success() {
                    let _ = fs::remove_file(zip_path);
                    return Err("Failed to extract InterVariable font".to_string());
                }
            }
            let _ = fs::remove_file(zip_path);
        }
    }

    let fira_code_regular = font_dir.join("FiraCode-Regular.ttf");
    if !fira_code_regular.exists() {
        let zip_path = "/tmp/FiraCode.zip";
        let output = TokioCommand::new("curl")
            .args(["-L", FIRA_CODE_URL, "-o", zip_path])
            .output()
            .await
            .map_err(|e| format!("Failed to download Fira Code font: {}", e))?;

        if output.status.success() {
            let extract_output = TokioCommand::new("unzip")
                .args(["-j", "-o", zip_path, "ttf/*.ttf", "-d"])
                .arg(&font_dir)
                .output()
                .await;

            if let Ok(extract) = extract_output {
                if !extract.status.success() {
                    let _ = fs::remove_file(zip_path);
                    return Err("Failed to extract Fira Code font".to_string());
                }
            }
            let _ = fs::remove_file(zip_path);
        }
    }

    let material_symbols = font_dir.join("MaterialSymbolsRounded.ttf");
    if !material_symbols.exists() {
        let output = TokioCommand::new("curl")
            .args(["-L", MATERIAL_SYMBOLS_URL, "-o"])
            .arg(&material_symbols)
            .output()
            .await
            .map_err(|e| format!("Failed to download Material Symbols font: {}", e))?;

        if !output.status.success() {
            return Err("Failed to download Material Symbols font".to_string());
        }
    }

    // Update font cache (non-blocking, ignore errors)
    let _ = TokioCommand::new("fc-cache")
        .args(["-f"])
        .output()
        .await;

    Ok(())
}

// Get cached Inter font (optimized - no reload on every call)
pub fn get_inter_font() -> iced::Font {
    *INTER_FONT_CACHE
}

fn load_inter_font() -> Result<iced::Font, String> {
    // Try different font name variations
    // After installation and fc-cache, the font should be available by name
    let font_names = [
        "Inter Variable",
        "InterVariable",
        "Inter-Variable",
    ];

    for name in &font_names {
        let font = iced::Font::with_name(name);
        // Check if font was successfully loaded by verifying it's not the default
        if font != iced::Font::DEFAULT {
            return Ok(font);
        }
    }

    // Fallback to Fira Code
    let fira_names = [
        "Fira Code",
        "FiraCode",
    ];

    for name in &fira_names {
        let font = iced::Font::with_name(name);
        if font != iced::Font::DEFAULT {
            return Ok(font);
        }
    }

    Err("InterVariable or Fira Code font not found in system font cache".to_string())
}

// Get cached Material Symbols font (optimized - no reload on every call)
pub fn get_material_symbols_font() -> iced::Font {
    *MATERIAL_SYMBOLS_FONT_CACHE
}

fn load_material_symbols_font() -> Result<iced::Font, String> {
    let font_names = [
        "Material Symbols Rounded",
        "MaterialSymbolsRounded",
    ];

    for name in &font_names {
        let font = iced::Font::with_name(name);
        if font != iced::Font::DEFAULT {
            return Ok(font);
        }
    }

    Err("Material Symbols font not found".to_string())
}

// Helper function to get glyph characters
// Using Material Symbols Unicode codepoints (Iced doesn't support ligatures like Qt)
pub mod glyphs {
    // Material Symbols Unicode codepoints (Private Use Area)
    // Get cached Material Symbols font (optimized - no reload on every call)
    pub fn material_font() -> iced::Font {
        super::get_material_symbols_font()
    }

    // String constants for use in format strings (using Unicode characters)
    pub const SEARCH_SYMBOL: &str = "\u{E8B6}";
    pub const INSTALLED_SYMBOL: &str = "\u{E5C3}";
    pub const REFRESH_SYMBOL: &str = "\u{E5D5}";
    pub const DOWNLOAD_SYMBOL: &str = "\u{E2C4}";
    pub const SETTINGS_SYMBOL: &str = "\u{E8B8}";
    pub const THEME_SYMBOL: &str = "\u{E518}";
    pub const CLOSE_SYMBOL: &str = "\u{E5CD}";
    pub const CANCEL_SYMBOL: &str = "\u{E5C9}";
    pub const EXIT_SYMBOL: &str = "\u{E879}";
    pub const DELETE_SYMBOL: &str = "\u{E872}";
    pub const CHECK_SYMBOL: &str = "\u{E5CA}";
    pub const ADD_SYMBOL: &str = "\u{E145}";
    pub const SYNC_SYMBOL: &str = "\u{E5D5}"; // sync/refresh
    pub const FOLDER_SYMBOL: &str = "\u{E2C7}"; // folder
    pub const INFO_SYMBOL: &str = "\u{E88E}"; // info
    pub const COPY_SYMBOL: &str = "\u{E14D}"; // content_copy
}
