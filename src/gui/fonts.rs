use std::path::PathBuf;
use std::fs;
use tokio::process::Command as TokioCommand;
use once_cell::sync::Lazy;

const INTER_FONT_URL: &str = "https://github.com/rsms/inter/releases/download/v4.0/Inter-4.0.zip";
const FIRA_CODE_URL: &str = "https://github.com/tonsky/FiraCode/releases/download/6.2/Fira_Code_v6.2.zip";
const MATERIAL_SYMBOLS_URL: &str = "https://github.com/google/material-design-icons/raw/master/variablefont/MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf";
const FONT_DIR: &str = ".local/share/fonts";

// Cache fonts to avoid reloading on every render
static INTER_FONT_CACHE: Lazy<iced::Font> = Lazy::new(|| {
    load_inter_font().unwrap_or_else(|_| iced::Font::DEFAULT)
});

static MATERIAL_SYMBOLS_FONT_CACHE: Lazy<iced::Font> = Lazy::new(|| {
    load_material_symbols_font().unwrap_or_else(|_| iced::Font::DEFAULT)
});

pub async fn ensure_fonts() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    let font_dir = PathBuf::from(&home).join(FONT_DIR);

    // Create font directory if it doesn't exist
    fs::create_dir_all(&font_dir).map_err(|e| format!("Failed to create font directory: {}", e))?;

    // Install InterVariable font
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

    // Install Fira Code font
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

    // Install Material Symbols font
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
    use iced::widget::text;
    
    // Material Symbols Unicode codepoints (Private Use Area)
    // These are the actual Unicode characters that Material Symbols uses
    // Note: These constants are used indirectly via the string constants below
    #[allow(dead_code)]
    const SEARCH_CODEPOINT: char = '\u{E8B6}';      // Material Symbols: search
    #[allow(dead_code)]
    const APPS_CODEPOINT: char = '\u{E5C3}';        // Material Symbols: apps
    #[allow(dead_code)]
    const REFRESH_CODEPOINT: char = '\u{E5D5}';     // Material Symbols: refresh
    #[allow(dead_code)]
    const DOWNLOAD_CODEPOINT: char = '\u{E2C4}';    // Material Symbols: download
    #[allow(dead_code)]
    const SETTINGS_CODEPOINT: char = '\u{E8B8}';    // Material Symbols: settings
    #[allow(dead_code)]
    const LIGHT_MODE_CODEPOINT: char = '\u{E518}';  // Material Symbols: light_mode
    #[allow(dead_code)]
    const CLOSE_CODEPOINT: char = '\u{E5CD}';       // Material Symbols: close
    #[allow(dead_code)]
    const CANCEL_CODEPOINT: char = '\u{E5C9}';      // Material Symbols: cancel
    #[allow(dead_code)]
    const EXIT_CODEPOINT: char = '\u{E879}';       // Material Symbols: exit_to_app
    #[allow(dead_code)]
    const DELETE_CODEPOINT: char = '\u{E872}';      // Material Symbols: delete
    
    // Get cached Material Symbols font (optimized - no reload on every call)
    pub fn material_font() -> iced::Font {
        super::get_material_symbols_font()
    }
    
    // Helper function to create text with Material Symbols font applied
    // Optimized: uses string constants instead of char.to_string() to avoid allocation
    #[allow(dead_code)]
    pub fn icon_text(icon_char: char) -> iced::widget::Text<'static> {
        let icon_str = match icon_char {
            SEARCH_CODEPOINT => SEARCH_SYMBOL,
            APPS_CODEPOINT => INSTALLED_SYMBOL,
            REFRESH_CODEPOINT => REFRESH_SYMBOL,
            DOWNLOAD_CODEPOINT => DOWNLOAD_SYMBOL,
            SETTINGS_CODEPOINT => SETTINGS_SYMBOL,
            LIGHT_MODE_CODEPOINT => THEME_SYMBOL,
            CLOSE_CODEPOINT => CLOSE_SYMBOL,
            CANCEL_CODEPOINT => CANCEL_SYMBOL,
            EXIT_CODEPOINT => EXIT_SYMBOL,
            DELETE_CODEPOINT => DELETE_SYMBOL,
            _ => SEARCH_SYMBOL, // fallback
        };
        text(icon_str).font(material_font())
    }
    
    // Convenience functions for each icon
    // Note: These are kept for potential future use, but currently we use string constants directly
    #[allow(dead_code)]
    pub fn search() -> iced::widget::Text<'static> { icon_text(SEARCH_CODEPOINT) }
    #[allow(dead_code)]
    pub fn installed() -> iced::widget::Text<'static> { icon_text(APPS_CODEPOINT) }
    #[allow(dead_code)]
    pub fn refresh() -> iced::widget::Text<'static> { icon_text(REFRESH_CODEPOINT) }
    #[allow(dead_code)]
    pub fn download() -> iced::widget::Text<'static> { icon_text(DOWNLOAD_CODEPOINT) }
    #[allow(dead_code)]
    pub fn settings() -> iced::widget::Text<'static> { icon_text(SETTINGS_CODEPOINT) }
    #[allow(dead_code)]
    pub fn theme() -> iced::widget::Text<'static> { icon_text(LIGHT_MODE_CODEPOINT) }
    #[allow(dead_code)]
    pub fn close() -> iced::widget::Text<'static> { icon_text(CLOSE_CODEPOINT) }
    #[allow(dead_code)]
    pub fn cancel() -> iced::widget::Text<'static> { icon_text(CANCEL_CODEPOINT) }
    #[allow(dead_code)]
    pub fn exit() -> iced::widget::Text<'static> { icon_text(EXIT_CODEPOINT) }
    #[allow(dead_code)]
    pub fn delete() -> iced::widget::Text<'static> { icon_text(DELETE_CODEPOINT) }
    
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
}

