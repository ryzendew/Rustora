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
            Theme::Light => Color::from_rgb(0.98, 0.98, 0.98),
            Theme::Dark => Color::from_rgb(0.12, 0.12, 0.12),
        }
    }

    #[allow(dead_code)]
    pub fn surface(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(1.0, 1.0, 1.0),
            Theme::Dark => Color::from_rgb(0.18, 0.18, 0.18),
        }
    }

    pub fn text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.1, 0.1, 0.1),
            Theme::Dark => Color::from_rgb(0.95, 0.95, 0.95),
        }
    }

    pub fn primary(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.0, 0.48, 0.78),
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


