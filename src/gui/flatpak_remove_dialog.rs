use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use std::path::PathBuf;
use tokio::fs;
use crate::gui::dialog_design::DialogDesign;
use iced::Color;

#[derive(Debug, Clone)]
pub enum Message {
    LoadFlatpakInfo,
    FlatpakInfoLoaded(Vec<FlatpakInfo>),
    RemoveFlatpaks,
    RemovalProgress(String),
    RemovalComplete,
    RemovalError(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct FlatpakInfo {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub size: String,
}

#[derive(Debug)]
pub struct FlatpakRemoveDialog {
    pub application_ids: Vec<String>,
    pub flatpak_info: Option<Vec<FlatpakInfo>>,
    pub is_loading: bool,
    pub is_removing: bool,
    pub is_complete: bool,
    pub removal_progress: String,
    pub show_dialog: bool,
}

impl FlatpakRemoveDialog {
    pub fn new(application_ids: Vec<String>) -> Self {
        Self {
            application_ids,
            flatpak_info: None,
            is_loading: true,
            is_removing: false,
            is_complete: false,
            removal_progress: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window(application_ids: Vec<String>) -> Result<(), iced::Error> {
        let dialog = Self::new(application_ids);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(600.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(480.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <FlatpakRemoveDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(20.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }

    pub fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        if !self.show_dialog {
            return Space::with_width(Length::Shrink).into();
        }

        let settings = crate::gui::settings::AppSettings::load();
        let title_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_size = (settings.font_size_body * settings.scale_body).round();
        let button_size = (settings.font_size_buttons * settings.scale_buttons).round();

        let content = if self.is_loading {
            container(
                column![
                    text("Loading Flatpak information...")
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(DialogDesign::space_medium()),
                    progress_bar(0.0..=1.0, 0.5)
                        .width(Length::Fill)
                        .height(Length::Fixed(DialogDesign::PROGRESS_HEIGHT)),
                ]
                .spacing(DialogDesign::SPACE_SMALL)
                .align_items(Alignment::Center)
                .padding(DialogDesign::pad_large())
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
        } else if let Some(ref infos) = self.flatpak_info {
            let title_text = if self.application_ids.len() == 1 {
                if let Some(ref info) = infos.first() {
                    format!("Remove {}", info.name)
                } else {
                    format!("Remove {}", self.application_ids[0])
                }
            } else {
                format!("Remove {} Flatpaks", self.application_ids.len())
            };

            let material_font = crate::gui::fonts::get_material_symbols_font();
            let header = container(
                row![
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL)
                        .font(material_font)
                        .size(title_size * 1.2)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_width(DialogDesign::space_small()),
                    text(&title_text)
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_width(Length::Fill),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding(DialogDesign::pad_medium());

            let packages_section = {
                let material_font = crate::gui::fonts::get_material_symbols_font();
                container(
                    column![
                        row![
                            text(crate::gui::fonts::glyphs::DELETE_SYMBOL)
                                .font(material_font)
                                .size(body_size * 1.1)
                                .style(iced::theme::Text::Color(theme.danger())),
                            Space::with_width(DialogDesign::space_small()),
                            text("Packages to Remove")
                                .size(body_size * 1.05)
                                .style(iced::theme::Text::Color(theme.danger())),
                        ]
                        .spacing(DialogDesign::SPACE_TINY)
                        .align_items(Alignment::Center),
                        Space::with_height(DialogDesign::space_small()),
                        scrollable(
                            column(
                                infos
                                    .iter()
                                    .map(|info| {
                                        container(
                                            column![
                                                text(&info.name)
                                                    .size(body_size)
                                                    .style(iced::theme::Text::Color(theme.primary())),
                                                text(&info.application_id)
                                                    .size(body_size * 0.85)
                                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                                Space::with_height(DialogDesign::space_tiny()),
                                                row![
                                                    text("Version:")
                                                        .size(body_size * 0.9)
                                                        .width(Length::Fixed(65.0))
                                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                                    text(&info.version)
                                                        .size(body_size * 0.9),
                                                    Space::with_width(Length::Fill),
                                                    text("Size:")
                                                        .size(body_size * 0.9)
                                                        .width(Length::Fixed(50.0))
                                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                                    text(&info.size)
                                                        .size(body_size * 0.9),
                                                ]
                                                .spacing(DialogDesign::SPACE_SMALL),
                                            ]
                                            .spacing(0)
                                        )
                                        .padding(DialogDesign::pad_small())
                                        .style(iced::theme::Container::Custom(Box::new(PackageItemStyle)))
                                        .width(Length::Fill)
                                        .into()
                                    })
                                    .collect::<Vec<_>>()
                            )
                            .spacing(DialogDesign::SPACE_SMALL)
                        )
                        .height(Length::Fill),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_medium())
                )
                .style(iced::theme::Container::Custom(Box::new(RemoveWarningContainerStyle)))
            };

            let progress_section = if self.is_removing || self.is_complete {
                let value = if self.is_complete { 1.0 } else { 0.7 };
                let progress_text = if self.is_complete {
                    "Removal completed successfully!".to_string()
                } else {
                    self.removal_progress.clone()
                };
                let material_font = crate::gui::fonts::get_material_symbols_font();
                container(
                    column![
                        row![
                            text(crate::gui::fonts::glyphs::DELETE_SYMBOL)
                                .font(material_font)
                                .size(body_size * 1.1)
                                .style(iced::theme::Text::Color(theme.danger())),
                            Space::with_width(DialogDesign::space_small()),
                            text("Progress")
                                .size(body_size * 1.05)
                                .style(iced::theme::Text::Color(theme.danger())),
                        ]
                        .spacing(DialogDesign::SPACE_TINY)
                        .align_items(Alignment::Center),
                        Space::with_height(DialogDesign::space_small()),
                        progress_bar(0.0..=1.0, value)
                            .width(Length::Fill)
                            .height(Length::Fixed(DialogDesign::PROGRESS_HEIGHT)),
                        Space::with_height(DialogDesign::space_tiny()),
                        text(&progress_text)
                            .size(body_size * 0.95)
                            .style(iced::theme::Text::Color(if self.is_complete {
                                Color::from_rgb(0.0, 0.8, 0.0)
                            } else {
                                theme.text()
                            })),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_medium())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
            } else {
                container(Space::with_height(Length::Shrink))
            };

            let material_font = crate::gui::fonts::get_material_symbols_font();

            let buttons = {
                if self.is_complete {
                    row![
                        Space::with_width(Length::Fill),
                        button(
                            row![
                                text(crate::gui::fonts::glyphs::EXIT_SYMBOL).font(material_font).size(button_size * 1.1),
                                text(" Close").size(button_size)
                            ]
                            .spacing(DialogDesign::SPACE_TINY)
                            .align_items(Alignment::Center)
                        )
                        .on_press(Message::Cancel)
                        .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
                        .padding(DialogDesign::pad_small()),
                    ]
                    .spacing(DialogDesign::SPACE_SMALL)
                } else {
                    row![
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
                        {
                            if self.is_removing {
                                button(
                                    row![
                                        text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font).size(button_size * 1.1),
                                        text(" Removing...").size(button_size)
                                    ]
                                    .spacing(DialogDesign::SPACE_TINY)
                                    .align_items(Alignment::Center)
                                )
                                .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
                                .padding(DialogDesign::pad_small())
                            } else {
                                button(
                                    row![
                                        text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font).size(button_size * 1.1),
                                        text(" Remove").size(button_size)
                                    ]
                                    .spacing(DialogDesign::SPACE_TINY)
                                    .align_items(Alignment::Center)
                                )
                                .on_press(Message::RemoveFlatpaks)
                                .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
                                .padding(DialogDesign::pad_small())
                            }
                        },
                    ]
                    .spacing(DialogDesign::SPACE_SMALL)
                }
            };

            container(
                column![
                    header,
                    container(Space::with_height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                    scrollable(
                        column![
                            packages_section,
                            progress_section,
                        ]
                        .spacing(DialogDesign::SPACE_MEDIUM)
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
        } else {
            container(
                column![
                    text("Failed to load Flatpak information")
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.danger())),
                ]
                .spacing(DialogDesign::SPACE_SMALL)
                .align_items(Alignment::Center)
                .padding(DialogDesign::pad_large())
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(WindowContainerStyle {
                background: theme.background(),
            })))
            .into()
    }
}

impl Application for FlatpakRemoveDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::LoadFlatpakInfo);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        if self.application_ids.len() == 1 {
            format!("Remove Flatpak - Rustora")
        } else {
            format!("Remove {} Flatpaks - Rustora", self.application_ids.len())
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadFlatpakInfo => {
                self.is_loading = true;
                let app_ids = self.application_ids.clone();
                iced::Command::perform(load_flatpak_infos(app_ids), |result| {
                    match result {
                        Ok(infos) => Message::FlatpakInfoLoaded(infos),
                        Err(e) => Message::RemovalError(e.to_string()),
                    }
                })
            }
            Message::FlatpakInfoLoaded(infos) => {
                self.is_loading = false;
                self.flatpak_info = Some(infos);
                iced::Command::none()
            }
            Message::RemoveFlatpaks => {
                self.is_removing = true;
                self.removal_progress = "Preparing removal...".to_string();
                let app_ids = self.application_ids.clone();
                iced::Command::perform(remove_flatpaks(app_ids), |result| {
                    match result {
                        Ok(progress) => Message::RemovalProgress(progress),
                        Err(e) => Message::RemovalError(e.to_string()),
                    }
                })
            }
            Message::RemovalProgress(progress) => {
                let progress_clone = progress.clone();
                self.removal_progress = progress.clone();
                if progress_clone.contains("Complete") ||
                   progress_clone.contains("Removed") ||
                   progress_clone.contains("complete") ||
                   progress_clone.to_lowercase().contains("success") {
                    iced::Command::perform(async {}, |_| Message::RemovalComplete)
                } else {
                    iced::Command::none()
                }
            }
            Message::RemovalComplete => {
                self.is_removing = false;
                self.is_complete = true;
                self.removal_progress = "Removal completed successfully!".to_string();
                iced::Command::none()
            }
            Message::RemovalError(_msg) => {
                self.is_removing = false;
                iced::Command::none()
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

async fn load_flatpak_infos(application_ids: Vec<String>) -> Result<Vec<FlatpakInfo>, String> {
    let mut infos = Vec::new();

    for app_id in application_ids {
        let output = TokioCommand::new("flatpak")
            .args(["info", &app_id])
            .output()
            .await
            .map_err(|e| format!("Failed to execute flatpak info: {}", e))?;

        if !output.status.success() {
            // If info fails, create a basic entry
            infos.push(FlatpakInfo {
                name: app_id.clone(),
                application_id: app_id,
                version: "Unknown".to_string(),
                size: "Unknown".to_string(),
            });
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut name = app_id.clone();
        let mut version = String::new();
        let mut size = String::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("Name:") {
                name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Version:") {
                version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Installed size:") {
                size = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            }
        }

        if name.is_empty() {
            name = app_id.clone();
        }

        infos.push(FlatpakInfo {
            name,
            application_id: app_id,
            version: if version.is_empty() { "Unknown".to_string() } else { version },
            size: if size.is_empty() { "Unknown".to_string() } else { size },
        });
    }

    Ok(infos)
}

async fn write_flatpak_remove_log(application_ids: &[String], output: &str, success: bool) {
    if let Ok(home) = std::env::var("HOME") {
        let log_dir = PathBuf::from(&home).join(".rustora");
        let _ = fs::create_dir_all(&log_dir).await;

        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_file = log_dir.join(format!("flatpak_remove_{}.log", timestamp));

        let mut log_content = String::new();
        log_content.push_str("=== Flatpak Remove Log ===\n");
        log_content.push_str(&format!("Timestamp: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        log_content.push_str(&format!("Application IDs: {}\n", application_ids.join(", ")));
        log_content.push_str(&format!("Status: {}\n", if success { "SUCCESS" } else { "FAILED" }));
        log_content.push_str("\n--- Command Output ---\n");
        log_content.push_str(output);
        log_content.push_str("\n--- End of Log ---\n");

        let _ = fs::write(&log_file, log_content).await;
    }
}

async fn remove_flatpaks(application_ids: Vec<String>) -> Result<String, String> {
    let command_str = format!("flatpak uninstall -y --noninteractive {}", application_ids.join(" "));

    let output = TokioCommand::new("flatpak")
        .args(["uninstall", "-y", "--noninteractive"])
        .args(&application_ids)
        .output()
        .await
        .map_err(|e| format!("Failed to execute flatpak uninstall: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut combined_output = String::new();
    combined_output.push_str(&format!("Command: {}\n", command_str));
    combined_output.push_str("--- Output ---\n");
    if !stdout.is_empty() {
        combined_output.push_str("STDOUT:\n");
        combined_output.push_str(&stdout);
        combined_output.push('\n');
    }
    if !stderr.is_empty() {
        combined_output.push_str("STDERR:\n");
        combined_output.push_str(&stderr);
        combined_output.push('\n');
    }

    let success = output.status.success();

    // Write log file
    write_flatpak_remove_log(&application_ids, &combined_output, success).await;

    if !success {
        return Err(format!("Removal failed: {}\n{}", stderr, stdout));
    }

    Ok("Removal Complete!".to_string())
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

struct CloseButtonStyle;

impl ButtonStyleSheet for CloseButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        ButtonAppearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            text_color: palette.text,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(_style);
        appearance.background = Some(iced::Background::Color(iced::Color::from_rgb(0.9, 0.2, 0.2)));
        appearance.text_color = iced::Color::WHITE;
        appearance
    }
}

struct ButtonBarStyle;

impl iced::widget::container::StyleSheet for ButtonBarStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.98,
                palette.background.g * 0.98,
                palette.background.b * 0.98,
                1.0,
            ))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
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
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
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

struct RemoveWarningContainerStyle;

impl iced::widget::container::StyleSheet for RemoveWarningContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                1.0, 0.3, 0.3, 0.08,
            ))),
            border: Border {
                radius: 16.0.into(),
                width: 1.5,
                color: iced::Color::from_rgba(1.0, 0.3, 0.3, 0.4),
            },
            ..Default::default()
        }
    }
}

struct PackageItemStyle;

impl iced::widget::container::StyleSheet for PackageItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(
                palette.background.r * 0.96,
                palette.background.g * 0.96,
                palette.background.b * 0.96,
                1.0,
            ))),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.15),
            },
            ..Default::default()
        }
    }
}
