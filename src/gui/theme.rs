use iced::theme::Theme as IcedTheme;
use iced::Color;

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

    pub fn background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.94, 0.94, 0.96), // Soft warm gray instead of bright white
            Theme::Dark => Color::from_rgb(0.12, 0.12, 0.12),
        }
    }

    #[allow(dead_code)]
    pub fn surface(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.97, 0.97, 0.98), // Slightly lighter than background
            Theme::Dark => Color::from_rgb(0.18, 0.18, 0.18),
        }
    }

    pub fn text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.05, 0.05, 0.05), // Near black for better readability
            Theme::Dark => Color::from_rgb(0.95, 0.95, 0.95),
        }
    }

    pub fn secondary_text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.35, 0.35, 0.4), // Muted dark gray for secondary text
            Theme::Dark => Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    pub fn primary(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.15, 0.45, 0.65), // Calmer, softer blue
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


