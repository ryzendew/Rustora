use iced::widget::{button, column, container, row, text, Space, text_input, slider, pick_list, scrollable};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::Color;
use crate::gui::settings::{AppSettings, CustomTheme, ColorData};

#[derive(Debug, Clone)]
pub enum Message {
    CategorySelected(SettingsCategory),
    ToggleTabVisibility(String),
    FontSizeChanged(f32),
    FontFamilyChanged(String),
    BackgroundColorRChanged(u8),
    BackgroundColorGChanged(u8),
    BackgroundColorBChanged(u8),
    TextColorRChanged(u8),
    TextColorGChanged(u8),
    TextColorBChanged(u8),
    PrimaryColorRChanged(u8),
    PrimaryColorGChanged(u8),
    PrimaryColorBChanged(u8),
    SecondaryTextColorRChanged(u8),
    SecondaryTextColorGChanged(u8),
    SecondaryTextColorBChanged(u8),
    ScalingChanged(f32),
    BorderRadiusChanged(f32),
    // Individual font size changes
    FontSizeButtonsChanged(f32),
    FontSizeTitlesChanged(f32),
    FontSizeBodyChanged(f32),
    FontSizeInputsChanged(f32),
    FontSizeTabsChanged(f32),
    FontSizeIconsChanged(f32),
    FontSizePackageNamesChanged(f32),
    FontSizePackageDetailsChanged(f32),
    // Individual UI scale changes
    ScaleButtonsChanged(f32),
    ScaleTitlesChanged(f32),
    ScaleBodyChanged(f32),
    ScaleInputsChanged(f32),
    ScaleTabsChanged(f32),
    ScaleIconsChanged(f32),
    ScalePackageCardsChanged(f32),
    SaveTheme,
    LoadTheme(String),
    DeleteTheme(String),
    ThemeNameChanged(String),
    SaveSettings,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Appearance,
    Fonts,
    UIScale,
    Tabs,
}

#[derive(Debug)]
pub struct SettingsDialog {
    current_category: SettingsCategory,
    settings: AppSettings,
    available_tabs: Vec<String>,
    theme_name: String,
    saved_themes: Vec<String>,
}

impl SettingsDialog {
    pub fn new() -> Self {
        let mut settings = AppSettings::load();
        let available_tabs = vec![
            "Search".to_string(),
            "Installed".to_string(),
            "Updates".to_string(),
            "Flatpak".to_string(),
            "Maintenance".to_string(),
            "Repositories".to_string(),
            "Kernel".to_string(),
            "Device".to_string(),
            "FPM".to_string(),
        ];

        // Initialize tab visibility if not set
        for tab in &available_tabs {
            settings.tab_visibility.entry(tab.clone()).or_insert(true);
        }

        Self {
            current_category: SettingsCategory::General,
            settings,
            available_tabs,
            theme_name: String::new(),
            saved_themes: CustomTheme::list(),
        }
    }

    pub fn run_separate_window() -> Result<(), iced::Error> {
        let dialog = Self::new();

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(1000.0, 700.0);
        window_settings.min_size = Some(iced::Size::new(800.0, 600.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <SettingsDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }

}

impl SettingsDialog {
    fn view_general(&self) -> Element<'_, Message> {
        column![
            container(
                column![
                    text("Border Radius")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        slider(0.0..=32.0, self.settings.border_radius, Message::BorderRadiusChanged)
                            .width(Length::Fill),
                        text(format!("{:.0}px", self.settings.border_radius))
                            .size(14)
                            .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                            .width(Length::Fixed(50.0)),
                    ]
                    .spacing(12)
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(8.0)),
                    text("Controls the roundness of buttons, containers, and other UI elements")
                        .size(12)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(8)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
        ]
        .spacing(16)
        .into()
    }

    fn view_appearance(&self) -> Element<'_, Message> {
        const FONT_FAMILIES: &[&str] = &["Inter Variable", "Fira Code", "DejaVu Sans", "Liberation Sans"];
        let selected_font = FONT_FAMILIES.iter().find(|&f| *f == self.settings.font_family.as_str());

        column![
            container(
                column![
                    text("Font Family")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        text("Font Family:")
                            .size(14)
                            .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                            .width(Length::Fixed(120.0)),
                        pick_list(
                            FONT_FAMILIES,
                            selected_font,
                            |s| Message::FontFamilyChanged(s.to_string())
                        )
                        .width(Length::Fill),
                    ]
                    .spacing(12)
                    .align_items(Alignment::Center),
                ]
                .spacing(8)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
            container(
                column![
                    text("Color Settings")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(12.0)),
                    self.color_picker_row("Background", &self.settings.background_color,
                        Message::BackgroundColorRChanged, Message::BackgroundColorGChanged, Message::BackgroundColorBChanged),
                    Space::with_height(Length::Fixed(12.0)),
                    self.color_picker_row("Text", &self.settings.text_color,
                        Message::TextColorRChanged, Message::TextColorGChanged, Message::TextColorBChanged),
                    Space::with_height(Length::Fixed(12.0)),
                    self.color_picker_row("Primary", &self.settings.primary_color,
                        Message::PrimaryColorRChanged, Message::PrimaryColorGChanged, Message::PrimaryColorBChanged),
                    Space::with_height(Length::Fixed(12.0)),
                    self.color_picker_row("Secondary Text", &self.settings.secondary_text_color,
                        Message::SecondaryTextColorRChanged, Message::SecondaryTextColorGChanged, Message::SecondaryTextColorBChanged),
                ]
                .spacing(8)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
            container(
                column![
                    text("Theme Management")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        text_input("Theme name", &self.theme_name)
                            .on_input(Message::ThemeNameChanged)
                            .width(Length::Fill),
                        Space::with_width(Length::Fixed(12.0)),
                        button(
                            text("Save")
                                .size(14)
                        )
                        .on_press(Message::SaveTheme)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
                            radius: self.settings.border_radius,
                        })))
                        .padding(Padding::from([8.0, 16.0]))
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(12.0)),
                    if self.saved_themes.is_empty() {
                        Element::from(
                            text("No saved themes")
                                .size(14)
                                .style(iced::theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.5)))
                        )
                    } else {
                        column(
                            self.saved_themes
                                .iter()
                                .map(|theme_name| {
                                    row![
                                        text(theme_name)
                                            .size(14)
                                            .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95)))
                                            .width(Length::Fill),
                                        button(
                                            text("Load")
                                                .size(12)
                                        )
                                        .on_press(Message::LoadTheme(theme_name.clone()))
                                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                    is_primary: false,
                                    radius: self.settings.border_radius,
                                })))
                                        .padding(Padding::from([6.0, 12.0])),
                                        Space::with_width(Length::Fixed(8.0)),
                                        button(
                                            text("Delete")
                                                .size(12)
                                        )
                                        .on_press(Message::DeleteTheme(theme_name.clone()))
                                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                    is_primary: false,
                                    radius: self.settings.border_radius,
                                })))
                                        .padding(Padding::from([6.0, 12.0])),
                                    ]
                                    .spacing(8)
                                    .align_items(Alignment::Center)
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(8)
                        .into()
                    },
                ]
                .spacing(8)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
        ]
        .spacing(16)
        .into()
    }

    fn color_picker_row(&self, label: &str, color: &ColorData,
        on_r: fn(u8) -> Message, on_g: fn(u8) -> Message, on_b: fn(u8) -> Message) -> Element<'_, Message> {
        let preview_color = Color::from_rgba(color.r, color.g, color.b, color.a);
        let r_val = (color.r * 255.0).round() as u8;
        let g_val = (color.g * 255.0).round() as u8;
        let b_val = (color.b * 255.0).round() as u8;

        column![
            row![
                text(format!("{}:", label))
                    .size(14)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(120.0)),
                container(
                    Space::with_width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0))
                )
                .style(iced::theme::Container::Custom(Box::new(ColorPreviewStyle { color: preview_color }))),
                Space::with_width(Length::Fixed(12.0)),
                text(format!("RGB({}, {}, {})", r_val, g_val, b_val))
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(120.0)),
            ]
            .spacing(8)
            .align_items(Alignment::Center),
            Space::with_height(Length::Fixed(8.0)),
            row![
                text("R:")
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(30.0)),
                slider(0u8..=255u8, r_val, on_r)
                    .width(Length::Fill),
                text(format!("{}", r_val))
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(40.0)),
            ]
            .spacing(8)
            .align_items(Alignment::Center),
            row![
                text("G:")
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(30.0)),
                slider(0u8..=255u8, g_val, on_g)
                    .width(Length::Fill),
                text(format!("{}", g_val))
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(40.0)),
            ]
            .spacing(8)
            .align_items(Alignment::Center),
            row![
                text("B:")
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(30.0)),
                slider(0u8..=255u8, b_val, on_b)
                    .width(Length::Fill),
                text(format!("{}", b_val))
                    .size(12)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Fixed(40.0)),
            ]
            .spacing(8)
            .align_items(Alignment::Center),
        ]
        .spacing(4)
        .into()
    }

    fn view_tabs(&self) -> Element<'_, Message> {
        column![
            container(
                column(
                    self.available_tabs
                        .iter()
                        .map(|tab| {
                            let is_visible = self.settings.tab_visibility.get(tab).copied().unwrap_or(true);
                            row![
                                text(tab)
                                    .size(14)
                                    .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95)))
                                    .width(Length::Fill),
                                button(
                                    text(if is_visible { "Hide" } else { "Show" })
                                        .size(12)
                                )
                                .on_press(Message::ToggleTabVisibility(tab.clone()))
                                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                    is_primary: is_visible,
                                    radius: self.settings.border_radius,
                                })))
                                .padding(Padding::from([6.0, 12.0]))
                            ]
                            .spacing(12)
                            .align_items(Alignment::Center)
                            .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(12)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
        ]
        .spacing(16)
        .into()
    }

    fn view_fonts(&self) -> Element<'_, Message> {
        column![
            // Universal font size slider at top
            container(
                column![
                    text("Universal Font Size")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        slider(10.0..=24.0, self.settings.font_size, Message::FontSizeChanged)
                            .width(Length::Fill),
                        text(format!("{:.0}px", self.settings.font_size))
                            .size(14)
                            .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                            .width(Length::Fixed(50.0)),
                    ]
                    .spacing(12)
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(8.0)),
                    text("Changes all font sizes proportionally")
                        .size(12)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(8)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
            // Individual font size sliders
            container(
                column![
                    text("Individual Font Sizes")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(12.0)),
                    self.font_size_slider("Buttons", self.settings.font_size_buttons, Message::FontSizeButtonsChanged),
                    self.font_size_slider("Titles", self.settings.font_size_titles, Message::FontSizeTitlesChanged),
                    self.font_size_slider("Body Text", self.settings.font_size_body, Message::FontSizeBodyChanged),
                    self.font_size_slider("Input Fields", self.settings.font_size_inputs, Message::FontSizeInputsChanged),
                    self.font_size_slider("Tab Buttons", self.settings.font_size_tabs, Message::FontSizeTabsChanged),
                    self.font_size_slider("Icons", self.settings.font_size_icons, Message::FontSizeIconsChanged),
                    self.font_size_slider("Package Names", self.settings.font_size_package_names, Message::FontSizePackageNamesChanged),
                    self.font_size_slider("Package Details", self.settings.font_size_package_details, Message::FontSizePackageDetailsChanged),
                ]
                .spacing(12)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
        ]
        .spacing(16)
        .into()
    }

    fn view_ui_scale(&self) -> Element<'_, Message> {
        column![
            // Universal UI scale slider at top
            container(
                column![
                    text("Universal UI Scale")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        slider(0.5..=4.0, self.settings.scaling, Message::ScalingChanged)
                            .step(0.1)
                            .width(Length::Fill),
                        text(format!("{:.1}x", self.settings.scaling))
                            .size(14)
                            .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                            .width(Length::Fixed(50.0)),
                    ]
                    .spacing(12)
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(8.0)),
                    text("Changes all UI elements proportionally")
                        .size(12)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(8)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
            // Individual UI scale sliders
            container(
                column![
                    text("Individual UI Scales")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(12.0)),
                    self.ui_scale_slider("Buttons", self.settings.scale_buttons, Message::ScaleButtonsChanged),
                    self.ui_scale_slider("Titles", self.settings.scale_titles, Message::ScaleTitlesChanged),
                    self.ui_scale_slider("Body Text", self.settings.scale_body, Message::ScaleBodyChanged),
                    self.ui_scale_slider("Input Fields", self.settings.scale_inputs, Message::ScaleInputsChanged),
                    self.ui_scale_slider("Tab Buttons", self.settings.scale_tabs, Message::ScaleTabsChanged),
                    self.ui_scale_slider("Icons", self.settings.scale_icons, Message::ScaleIconsChanged),
                    self.ui_scale_slider("Package Cards", self.settings.scale_package_cards, Message::ScalePackageCardsChanged),
                ]
                .spacing(12)
            )
            .padding(Padding::from([16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(SectionContainerStyle {
                radius: self.settings.border_radius,
            }))),
        ]
        .spacing(16)
        .into()
    }

    fn font_size_slider(&self, label: &str, value: f32, on_change: fn(f32) -> Message) -> Element<'_, Message> {
        row![
            text(format!("{}:", label))
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                .width(Length::Fixed(150.0)),
            slider(8.0..=28.0, value, on_change)
                .width(Length::Fill),
            text(format!("{:.0}px", value))
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                .width(Length::Fixed(50.0)),
        ]
        .spacing(12)
        .align_items(Alignment::Center)
        .into()
    }

    fn ui_scale_slider(&self, label: &str, value: f32, on_change: fn(f32) -> Message) -> Element<'_, Message> {
        row![
            text(format!("{}:", label))
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                .width(Length::Fixed(150.0)),
            slider(0.5..=4.0, value, on_change)
                .step(0.1)
                .width(Length::Fill),
            text(format!("{:.1}x", value))
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                .width(Length::Fixed(50.0)),
        ]
        .spacing(12)
        .align_items(Alignment::Center)
        .into()
    }

}

impl Application for SettingsDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        (flags, Command::none())
    }

    fn title(&self) -> String {
        "Settings".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CategorySelected(category) => {
                self.current_category = category;
                Command::none()
            }
            Message::ToggleTabVisibility(tab_name) => {
                if let Some(visible) = self.settings.tab_visibility.get_mut(&tab_name) {
                    *visible = !*visible;
                    self.settings.save();
                }
                Command::none()
            }
            Message::FontSizeChanged(size) => {
                // Update universal font size and all individual font sizes proportionally
                let ratio = if self.settings.font_size > 0.0 {
                    size / self.settings.font_size
                } else {
                    1.0
                };
                self.settings.font_size = size;
                self.settings.font_size_buttons *= ratio;
                self.settings.font_size_titles *= ratio;
                self.settings.font_size_body *= ratio;
                self.settings.font_size_inputs *= ratio;
                self.settings.font_size_tabs *= ratio;
                self.settings.font_size_icons *= ratio;
                self.settings.font_size_package_names *= ratio;
                self.settings.font_size_package_details *= ratio;
                self.settings.save();
                Command::none()
            }
            Message::FontFamilyChanged(family) => {
                self.settings.font_family = family;
                self.settings.save();
                Command::none()
            }
            Message::BackgroundColorRChanged(r) => {
                self.settings.background_color.r = r as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::BackgroundColorGChanged(g) => {
                self.settings.background_color.g = g as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::BackgroundColorBChanged(b) => {
                self.settings.background_color.b = b as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::TextColorRChanged(r) => {
                self.settings.text_color.r = r as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::TextColorGChanged(g) => {
                self.settings.text_color.g = g as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::TextColorBChanged(b) => {
                self.settings.text_color.b = b as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::PrimaryColorRChanged(r) => {
                self.settings.primary_color.r = r as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::PrimaryColorGChanged(g) => {
                self.settings.primary_color.g = g as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::PrimaryColorBChanged(b) => {
                self.settings.primary_color.b = b as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::SecondaryTextColorRChanged(r) => {
                self.settings.secondary_text_color.r = r as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::SecondaryTextColorGChanged(g) => {
                self.settings.secondary_text_color.g = g as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::SecondaryTextColorBChanged(b) => {
                self.settings.secondary_text_color.b = b as f32 / 255.0;
                self.settings.save();
                Command::none()
            }
            Message::ScalingChanged(scale) => {
                // Update universal scale and all individual scales proportionally
                let ratio = if self.settings.scaling > 0.0 {
                    scale / self.settings.scaling
                } else {
                    1.0
                };
                self.settings.scaling = scale;
                self.settings.scale_buttons *= ratio;
                self.settings.scale_titles *= ratio;
                self.settings.scale_body *= ratio;
                self.settings.scale_inputs *= ratio;
                self.settings.scale_tabs *= ratio;
                self.settings.scale_icons *= ratio;
                self.settings.scale_package_cards *= ratio;
                self.settings.save();
                Command::none()
            }
            Message::BorderRadiusChanged(radius) => {
                self.settings.border_radius = radius;
                self.settings.save();
                Command::none()
            }
            // Individual font size handlers
            Message::FontSizeButtonsChanged(size) => {
                self.settings.font_size_buttons = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizeTitlesChanged(size) => {
                self.settings.font_size_titles = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizeBodyChanged(size) => {
                self.settings.font_size_body = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizeInputsChanged(size) => {
                self.settings.font_size_inputs = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizeTabsChanged(size) => {
                self.settings.font_size_tabs = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizeIconsChanged(size) => {
                self.settings.font_size_icons = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizePackageNamesChanged(size) => {
                self.settings.font_size_package_names = size;
                self.settings.save();
                Command::none()
            }
            Message::FontSizePackageDetailsChanged(size) => {
                self.settings.font_size_package_details = size;
                self.settings.save();
                Command::none()
            }
            // Individual UI scale handlers
            Message::ScaleButtonsChanged(scale) => {
                self.settings.scale_buttons = scale;
                self.settings.save();
                Command::none()
            }
            Message::ScaleTitlesChanged(scale) => {
                self.settings.scale_titles = scale;
                self.settings.save();
                Command::none()
            }
            Message::ScaleBodyChanged(scale) => {
                self.settings.scale_body = scale;
                self.settings.save();
                Command::none()
            }
            Message::ScaleInputsChanged(scale) => {
                self.settings.scale_inputs = scale;
                self.settings.save();
                Command::none()
            }
            Message::ScaleTabsChanged(scale) => {
                self.settings.scale_tabs = scale;
                self.settings.save();
                Command::none()
            }
            Message::ScaleIconsChanged(scale) => {
                self.settings.scale_icons = scale;
                self.settings.save();
                Command::none()
            }
            Message::ScalePackageCardsChanged(scale) => {
                self.settings.scale_package_cards = scale;
                self.settings.save();
                Command::none()
            }
            Message::SaveSettings => {
                self.settings.save();
                Command::none()
            }
            Message::SaveTheme => {
                if !self.theme_name.is_empty() {
                    CustomTheme::save(&self.theme_name, &self.settings);
                    self.saved_themes = CustomTheme::list();
                }
                Command::none()
            }
            Message::LoadTheme(name) => {
                if let Some(loaded_settings) = CustomTheme::load(&name) {
                    self.settings = loaded_settings;
                    self.settings.save();
                }
                Command::none()
            }
            Message::DeleteTheme(name) => {
                CustomTheme::delete(&name);
                self.saved_themes = CustomTheme::list();
                Command::none()
            }
            Message::ThemeNameChanged(name) => {
                self.theme_name = name;
                Command::none()
            }
            Message::Close => {
                iced::window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        let material_font = glyphs::material_font();

        // Left navigation pane
        let nav_items = vec![
            (SettingsCategory::General, "General"),
            (SettingsCategory::Appearance, "Appearance"),
            (SettingsCategory::Fonts, "Fonts"),
            (SettingsCategory::UIScale, "UI Scale"),
            (SettingsCategory::Tabs, "Tabs"),
        ];

        let nav_pane = container(
            column(
                nav_items
                    .into_iter()
                    .map(|(cat, label)| {
                        let is_selected = self.current_category == cat;
                        button(
                            text(label)
                                .size(14)
                                .style(iced::theme::Text::Color(if is_selected {
                                    Color::from_rgb(0.2, 0.6, 0.9)
                                } else {
                                    Color::from_rgb(0.7, 0.7, 0.7)
                                }))
                        )
                        .on_press(Message::CategorySelected(cat))
                        .style(iced::theme::Button::Custom(Box::new(NavButtonStyle {
                            is_selected,
                        })))
                        .width(Length::Fill)
                        .padding(Padding::from([12.0, 16.0]))
                        .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(4)
            .width(Length::Fill)
        )
        .width(Length::Fixed(200.0))
        .padding(Padding::from([20.0, 0.0]))
        .style(iced::theme::Container::Custom(Box::new(NavPaneStyle)));

        // Right content pane
        let content = match self.current_category {
            SettingsCategory::General => self.view_general(),
            SettingsCategory::Appearance => self.view_appearance(),
            SettingsCategory::Fonts => self.view_fonts(),
            SettingsCategory::UIScale => self.view_ui_scale(),
            SettingsCategory::Tabs => self.view_tabs(),
        };

        let content_pane = container(
            scrollable(
                column![
                    text(match self.current_category {
                        SettingsCategory::General => "General",
                        SettingsCategory::Appearance => "Appearance",
                        SettingsCategory::Fonts => "Fonts",
                        SettingsCategory::UIScale => "UI Scale",
                        SettingsCategory::Tabs => "Tabs",
                    })
                    .size(24)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    Space::with_height(Length::Fixed(20.0)),
                    content,
                ]
                .spacing(16)
            )
        )
        .width(Length::Fill)
        .padding(Padding::from([20.0, 30.0]))
        .style(iced::theme::Container::Custom(Box::new(ContentPaneStyle)));

        // Top bar with close button
        let top_bar = container(
            row![
                text("Settings")
                    .size(20)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                Space::with_width(Length::Fill),
                button(
                    text(glyphs::CLOSE_SYMBOL)
                        .font(material_font)
                        .size(20)
                )
                .on_press(Message::Close)
                .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
                .padding(Padding::from([8.0, 8.0]))
            ]
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(Padding::from([16.0, 20.0]))
        .style(iced::theme::Container::Custom(Box::new(TopBarStyle)));

        // Bottom bar with save and close buttons
        let bottom_bar = container(
            row![
                Space::with_width(Length::Fill),
                button(
                    text("Save")
                        .size(14)
                )
                .on_press(Message::SaveSettings)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: self.settings.border_radius,
                })))
                .padding(Padding::from([12.0, 24.0])),
                Space::with_width(Length::Fixed(12.0)),
                button(
                    text("Close")
                        .size(14)
                )
                .on_press(Message::Close)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: self.settings.border_radius,
                })))
                .padding(Padding::from([12.0, 24.0]))
            ]
        )
        .width(Length::Fill)
        .padding(Padding::from([16.0, 20.0]))
        .style(iced::theme::Container::Custom(Box::new(BottomBarStyle)));

        container(
            column![
                top_bar,
                row![
                    nav_pane,
                    content_pane,
                ]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill),
                bottom_bar,
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(MainContainerStyle)))
        .into()
    }

    fn theme(&self) -> IcedTheme {
        IcedTheme::Dark
    }
}

// Style implementations
struct MainContainerStyle;
impl iced::widget::container::StyleSheet for MainContainerStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
            ..Default::default()
        }
    }
}

struct NavPaneStyle;
impl iced::widget::container::StyleSheet for NavPaneStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ContentPaneStyle;
impl iced::widget::container::StyleSheet for ContentPaneStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.12))),
            ..Default::default()
        }
    }
}

struct TopBarStyle;
impl iced::widget::container::StyleSheet for TopBarStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct BottomBarStyle;
impl iced::widget::container::StyleSheet for BottomBarStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct SectionContainerStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for SectionContainerStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.18, 0.18, 0.18))),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            },
            ..Default::default()
        }
    }
}

struct NavButtonStyle {
    is_selected: bool,
}
impl ButtonStyleSheet for NavButtonStyle {
    type Style = IcedTheme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_selected {
                Color::from_rgba(0.2, 0.6, 0.9, 0.2)
            } else {
                Color::TRANSPARENT
            })),
            border: Border {
                radius: 6.0.into(),
                width: if self.is_selected { 1.0 } else { 0.0 },
                color: if self.is_selected {
                    Color::from_rgb(0.2, 0.6, 0.9)
                } else {
                    Color::TRANSPARENT
                },
            },
            ..Default::default()
        }
    }
}

struct CloseButtonStyle;
impl ButtonStyleSheet for CloseButtonStyle {
    type Style = IcedTheme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
            ..Default::default()
        }
    }
}

struct RoundedButtonStyle {
    is_primary: bool,
    radius: f32,
}
impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = IcedTheme;
    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                Color::from_rgb(0.2, 0.6, 0.9)
            } else {
                Color::from_rgb(0.3, 0.3, 0.3)
            })),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: if self.is_primary {
                Color::from_rgb(1.0, 1.0, 1.0)
            } else {
                Color::from_rgb(0.9, 0.9, 0.9)
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                Color::from_rgb(0.25, 0.65, 0.95)
            } else {
                Color::from_rgb(0.4, 0.4, 0.4)
            })),
            ..Default::default()
        }
    }
}

struct ColorPreviewStyle {
    color: Color,
}
impl iced::widget::container::StyleSheet for ColorPreviewStyle {
    type Style = IcedTheme;
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.color)),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.5, 0.5, 0.5),
            },
            ..Default::default()
        }
    }
}

use iced::window;

