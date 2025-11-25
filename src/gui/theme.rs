use iced::theme::Theme as IcedTheme;
use iced::Color;
use crate::gui::settings::AppSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn iced_theme(&self) -> IcedTheme {
        match self {
            Theme::Light => IcedTheme::Light,
            Theme::Dark => IcedTheme::Dark,
        }
    }

    fn is_color_appropriate_for_theme(&self, color: &Color) -> bool {
        let is_dark = color.r < 0.5;
        match self {
            Theme::Light => !is_dark,
            Theme::Dark => is_dark,
        }
    }

    pub fn background(&self) -> Color {
        self.background_with_settings(None)
    }

    pub fn background_with_settings(&self, settings: Option<&AppSettings>) -> Color {
        if let Some(settings) = settings {
            let custom_bg = Color::from(settings.background_color.clone());
            if self.is_color_appropriate_for_theme(&custom_bg) {
                return custom_bg;
            }
        }
        match self {
            Theme::Light => Color::from_rgb(0.94, 0.94, 0.96),
            Theme::Dark => Color::from_rgb(0.12, 0.12, 0.12),
        }
    }

    #[allow(dead_code)]
    pub fn surface(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.97, 0.97, 0.98),
            Theme::Dark => Color::from_rgb(0.18, 0.18, 0.18),
        }
    }

    pub fn text(&self) -> Color {
        self.text_with_settings(None)
    }

    pub fn text_with_settings(&self, settings: Option<&AppSettings>) -> Color {
        if let Some(settings) = settings {
            let custom_text = Color::from(settings.text_color.clone());

            let bg = self.background_with_settings(Some(settings));
            let text_is_dark = custom_text.r < 0.5;
            let bg_is_dark = bg.r < 0.5;

            if text_is_dark != bg_is_dark {
                return custom_text;
            }
        }
        match self {
            Theme::Light => Color::from_rgb(0.05, 0.05, 0.05),
            Theme::Dark => Color::from_rgb(0.95, 0.95, 0.95),
        }
    }

    pub fn secondary_text(&self) -> Color {
        self.secondary_text_with_settings(None)
    }

    pub fn secondary_text_with_settings(&self, settings: Option<&AppSettings>) -> Color {
        if let Some(settings) = settings {
            let custom_secondary = Color::from(settings.secondary_text_color.clone());

            let bg = self.background_with_settings(Some(settings));
            let text_is_dark = custom_secondary.r < 0.5;
            let bg_is_dark = bg.r < 0.5;

            if text_is_dark != bg_is_dark {
                return custom_secondary;
            }
        }
        match self {
            Theme::Light => Color::from_rgb(0.35, 0.35, 0.4),
            Theme::Dark => Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    pub fn primary(&self) -> Color {
        self.primary_with_settings(None)
    }

    pub fn primary_with_settings(&self, settings: Option<&AppSettings>) -> Color {
        if let Some(settings) = settings {
            let custom_primary = Color::from(settings.primary_color.clone());

            let saturation = ((custom_primary.r - custom_primary.g).abs() +
                             (custom_primary.g - custom_primary.b).abs() +
                             (custom_primary.b - custom_primary.r).abs()) / 3.0;
            if saturation > 0.1 {
                return custom_primary;
            }
        }
        match self {
            Theme::Light => Color::from_rgb(0.15, 0.45, 0.65),
            Theme::Dark => Color::from_rgb(0.2, 0.6, 0.9),
        }
    }

    #[allow(dead_code)]
    pub fn accent(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.0, 0.6, 0.4),
            Theme::Dark => Color::from_rgb(0.3, 0.8, 0.6),
        }
    }

    #[allow(dead_code)]
    pub fn danger(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.9, 0.2, 0.2),
            Theme::Dark => Color::from_rgb(1.0, 0.3, 0.3),
        }
    }
}
