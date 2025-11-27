use iced::widget::{button, checkbox, column, container, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme, Color};
use crate::gui::dialog_design::DialogDesign;
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
        window_settings.min_size = Some(iced::Size::new(400.0, 350.0));
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
        let settings = crate::gui::settings::AppSettings::load();
        let title_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_size = (settings.font_size_body * settings.scale_body).round();
        let button_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let header = container(
            row![
                text(crate::gui::fonts::glyphs::SETTINGS_SYMBOL)
                    .font(material_font)
                    .size(title_size * 1.2)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_width(DialogDesign::space_small()),
                text("Update Settings")
                    .size(title_size)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_width(Length::Fill),
            ]
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(DialogDesign::pad_medium());

        let options = container(
            column![
                container(
                    column![
                        checkbox("Allow erasing installed packages", self.settings.allowerasing)
                            .on_toggle(|_| Message::ToggleAllowerasing)
                            .text_size(body_size),
                        Space::with_height(DialogDesign::space_tiny()),
                        text("Allow removing installed packages to resolve conflicts")
                            .size(body_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_small())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
                Space::with_height(DialogDesign::space_small()),
                container(
                    column![
                        checkbox("Skip unavailable packages", self.settings.skip_unavailable)
                            .on_toggle(|_| Message::ToggleSkipUnavailable)
                            .text_size(body_size),
                        Space::with_height(DialogDesign::space_tiny()),
                        text("Allow skipping packages that are not available")
                            .size(body_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_small())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
                Space::with_height(DialogDesign::space_small()),
                container(
                    column![
                        checkbox("Allow downgrade", self.settings.allow_downgrade)
                            .on_toggle(|_| Message::ToggleAllowDowngrade)
                            .text_size(body_size),
                        Space::with_height(DialogDesign::space_tiny()),
                        text("Allow downgrade of dependencies to resolve conflicts")
                            .size(body_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_small())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
                Space::with_height(DialogDesign::space_small()),
                container(
                    column![
                        checkbox("Security updates only", self.settings.security_only)
                            .on_toggle(|_| Message::ToggleSecurityOnly)
                            .text_size(body_size),
                        Space::with_height(DialogDesign::space_tiny()),
                        text("Only install updates from security advisories")
                            .size(body_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_small())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
                Space::with_height(DialogDesign::space_small()),
                container(
                    column![
                        checkbox("Bugfix updates only", self.settings.bugfix_only)
                            .on_toggle(|_| Message::ToggleBugfixOnly)
                            .text_size(body_size),
                        Space::with_height(DialogDesign::space_tiny()),
                        text("Only install updates from bugfix advisories")
                            .size(body_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_small())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
            ]
            .spacing(0)
            .padding(DialogDesign::pad_medium())
        )
        .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)));

        let buttons = row![
            button(
                row![
                    text(crate::gui::fonts::glyphs::CANCEL_SYMBOL).font(material_font).size(button_size * 1.1),
                    text(" Cancel").size(button_size)
                ]
                .spacing(DialogDesign::SPACE_TINY)
                .align_items(Alignment::Center)
            )
            .on_press(Message::Cancel)
            .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: false })))
            .padding(DialogDesign::pad_small()),
            Space::with_width(Length::Fill),
            button(
                row![
                    text(crate::gui::fonts::glyphs::CHECK_SYMBOL).font(material_font).size(button_size * 1.1),
                    text(" Save").size(button_size)
                ]
                .spacing(DialogDesign::SPACE_TINY)
                .align_items(Alignment::Center)
            )
            .on_press(Message::Save)
            .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
            .padding(DialogDesign::pad_small()),
        ]
        .spacing(DialogDesign::SPACE_SMALL);

        container(
            column![
                header,
                container(Space::with_height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                scrollable(
                    column![
                        options,
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_medium())
                )
                .height(Length::Fill),
                container(Space::with_height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                container(buttons)
                    .width(Length::Fill)
                    .padding(DialogDesign::pad_medium()),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(WindowContainerStyle {
            background: theme.background(),
        })))
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

struct CleanContainerStyle;

impl iced::widget::container::StyleSheet for CleanContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(
                palette.background.r * 0.98,
                palette.background.g * 0.98,
                palette.background.b * 0.98,
                1.0,
            ))),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.2),
            },
            ..Default::default()
        }
    }
}

struct DividerStyle;

impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.2))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct CleanButtonStyle {
    is_primary: bool,
}

impl ButtonStyleSheet for CleanButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                palette.primary
            } else {
                Color::from_rgba(0.4, 0.4, 0.4, 0.2)
            })),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: if self.is_primary {
                    palette.primary
                } else {
                    Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                },
            },
            text_color: if self.is_primary { Color::WHITE } else { palette.text },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        let palette = style.palette();
        if self.is_primary {
            appearance.background = Some(iced::Background::Color(
                Color::from_rgba(palette.primary.r * 0.85, palette.primary.g * 0.85, palette.primary.b * 0.85, 1.0)
            ));
        } else {
            appearance.background = Some(iced::Background::Color(Color::from_rgba(0.4, 0.4, 0.4, 0.3)));
        }
        appearance
    }
}

struct WindowContainerStyle {
    background: iced::Color,
}

impl iced::widget::container::StyleSheet for WindowContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.background)),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}
