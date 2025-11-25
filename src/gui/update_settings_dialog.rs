use iced::widget::{button, checkbox, column, container, row, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub allowerasing: bool,
    pub skip_unavailable: bool,
    pub allow_downgrade: bool,
    pub security_only: bool,
    pub bugfix_only: bool,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            allowerasing: false,
            skip_unavailable: false,
            allow_downgrade: false,
            security_only: false,
            bugfix_only: false,
        }
    }
}

impl UpdateSettings {
    pub fn to_dnf_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.allowerasing {
            args.push("--allowerasing".to_string());
        }
        if self.skip_unavailable {
            args.push("--skip-unavailable".to_string());
        }
        if self.allow_downgrade {
            args.push("--allow-downgrade".to_string());
        }
        if self.security_only {
            args.push("--security".to_string());
        }
        if self.bugfix_only {
            args.push("--bugfix".to_string());
        }
        args
    }

    pub fn load() -> Self {
        if let Ok(home) = std::env::var("HOME") {
            let settings_path = PathBuf::from(&home).join(".rustora").join("update_settings.json");
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(settings) = serde_json::from_str::<UpdateSettings>(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        if let Ok(home) = std::env::var("HOME") {
            let settings_dir = PathBuf::from(&home).join(".rustora");
            if let Err(e) = std::fs::create_dir_all(&settings_dir) {
                return Err(format!("Failed to create settings directory: {}", e));
            }
            let settings_path = settings_dir.join("update_settings.json");
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize settings: {}", e))?;
            std::fs::write(&settings_path, json)
                .map_err(|e| format!("Failed to write settings: {}", e))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleAllowerasing,
    ToggleSkipUnavailable,
    ToggleAllowDowngrade,
    ToggleSecurityOnly,
    ToggleBugfixOnly,
    Save,
    Cancel,
}

#[derive(Debug)]
pub struct UpdateSettingsDialog {
    settings: UpdateSettings,
}

impl UpdateSettingsDialog {
    pub fn new() -> Self {
        Self {
            settings: UpdateSettings::load(),
        }
    }

    pub fn run_separate_window() -> Result<(), iced::Error> {
        let dialog = Self::new();

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(500.0, 400.0);
        window_settings.min_size = Some(iced::Size::new(450.0, 350.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <UpdateSettingsDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }

    pub fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let title = container(
            text("Update Settings").size(20).style(iced::theme::Text::Color(theme.primary()))
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0));

        let options = column![
            checkbox("Allow erasing installed packages", self.settings.allowerasing)
                .on_toggle(|_| Message::ToggleAllowerasing)
                .text_size(14),
            Space::with_height(Length::Fixed(8.0)),
            text("Allow removing installed packages to resolve conflicts")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0))),
            Space::with_height(Length::Fixed(16.0)),
            checkbox("Skip unavailable packages", self.settings.skip_unavailable)
                .on_toggle(|_| Message::ToggleSkipUnavailable)
                .text_size(14),
            Space::with_height(Length::Fixed(8.0)),
            text("Allow skipping packages that are not available")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0))),
            Space::with_height(Length::Fixed(16.0)),
            checkbox("Allow downgrade", self.settings.allow_downgrade)
                .on_toggle(|_| Message::ToggleAllowDowngrade)
                .text_size(14),
            Space::with_height(Length::Fixed(8.0)),
            text("Allow downgrade of dependencies to resolve conflicts")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0))),
            Space::with_height(Length::Fixed(16.0)),
            checkbox("Security updates only", self.settings.security_only)
                .on_toggle(|_| Message::ToggleSecurityOnly)
                .text_size(14),
            Space::with_height(Length::Fixed(8.0)),
            text("Only install updates from security advisories")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0))),
            Space::with_height(Length::Fixed(16.0)),
            checkbox("Bugfix updates only", self.settings.bugfix_only)
                .on_toggle(|_| Message::ToggleBugfixOnly)
                .text_size(14),
            Space::with_height(Length::Fixed(8.0)),
            text("Only install updates from bugfix advisories")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0))),
        ]
        .spacing(4)
        .padding(Padding::new(20.0));

        let save_button = button(
            row![
                text("[OK]"),
                text(" Save")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::Save)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
        })))
        .padding(Padding::new(12.0));

        let cancel_button = button(
            row![
                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font),
                text(" Cancel")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::Cancel)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(12.0));

        let buttons = row![
            Space::with_width(Length::Fill),
            cancel_button,
            Space::with_width(Length::Fixed(10.0)),
            save_button,
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .padding(Padding::new(20.0));

        container(
            column![
                title,
                options,
                buttons,
            ]
            .spacing(10)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        .into()
    }
}

impl Application for UpdateSettingsDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        (flags, Command::none())
    }

    fn title(&self) -> String {
        "Update Settings - Rustora".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ToggleAllowerasing => {
                self.settings.allowerasing = !self.settings.allowerasing;
                Command::none()
            }
            Message::ToggleSkipUnavailable => {
                self.settings.skip_unavailable = !self.settings.skip_unavailable;
                Command::none()
            }
            Message::ToggleAllowDowngrade => {
                self.settings.allow_downgrade = !self.settings.allow_downgrade;
                Command::none()
            }
            Message::ToggleSecurityOnly => {
                self.settings.security_only = !self.settings.security_only;
                if self.settings.security_only {
                    self.settings.bugfix_only = false;
                }
                Command::none()
            }
            Message::ToggleBugfixOnly => {
                self.settings.bugfix_only = !self.settings.bugfix_only;
                if self.settings.bugfix_only {
                    self.settings.security_only = false;
                }
                Command::none()
            }
            Message::Save => {
                if let Err(_e) = self.settings.save() {
                }
                iced::window::close(window::Id::MAIN)
            }
            Message::Cancel => {
                iced::window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = &crate::gui::Theme::Dark;
        self.view_impl(theme)
    }

    fn theme(&self) -> IcedTheme {
        crate::gui::Theme::Dark.iced_theme()
    }
}

struct DialogContainerStyle;

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}

struct RoundedButtonStyle {
    is_primary: bool,
}

impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                palette.primary
            } else {
                iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1)
            })),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: if self.is_primary {
                    palette.primary
                } else {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                },
            },
            text_color: palette.text,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        let palette = style.palette();
        appearance.background = Some(iced::Background::Color(if self.is_primary {
            iced::Color::from_rgba(palette.primary.r * 0.9, palette.primary.g * 0.9, palette.primary.b * 0.9, 1.0)
        } else {
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15)
        }));
        appearance
    }
}
