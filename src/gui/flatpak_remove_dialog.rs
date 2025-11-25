use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use std::path::PathBuf;
use tokio::fs;

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
        window_settings.size = iced::Size::new(750.0, 800.0);
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
        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.2).round();
        let _body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let _button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let _icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();

        let content = if self.is_loading {
            container(
                column![
                    text("Loading Flatpak information...").size(title_font_size),
                    Space::with_height(Length::Fixed(20.0)),
                    progress_bar(0.0..=1.0, 0.5).width(Length::Fill),
                ]
                .spacing(15)
                .align_items(Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fixed(600.0))
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
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
            let title = container(
                row![
                    text(&title_text)
                        .size(18)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(Length::Fill),
                    button(
                        text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(18)
                    )
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
                    .padding(Padding::new(6.0)),
                ]
                .align_items(Alignment::Center)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .padding(Padding::new(16.0));

            let packages_section = container(
                column![
                    text("Packages to Remove").size(14).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(10.0)),
                    scrollable(
                        column(
                            infos
                                .iter()
                                .map(|info| {
                                    container(
                                        column![
                                            text(&info.name).size(13).style(iced::theme::Text::Color(theme.primary())),
                                            text(&info.application_id).size(11),
                                            row![
                                                text("Version:").size(10).width(Length::Fixed(60.0)),
                                                text(&info.version).size(10),
                                                Space::with_width(Length::Fill),
                                                text("Size:").size(10).width(Length::Fixed(50.0)),
                                                text(&info.size).size(10),
                                            ]
                                            .spacing(8),
                                        ]
                                        .spacing(4)
                                    )
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(PackageItemStyle)))
                                    .width(Length::Fill)
                                    .into()
                                })
                                .collect::<Vec<_>>()
                        )
                        .spacing(8)
                    )
                    .height(Length::Fixed(300.0)),
                ]
                .spacing(0)
                .padding(Padding::new(16.0))
            )
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)));

            let progress_section = if self.is_removing || self.is_complete {
                let progress_value = if self.is_complete { 1.0 } else { 0.7 };
                let progress_text = if self.is_complete {
                    "Removal completed successfully!".to_string()
                } else {
                    self.removal_progress.clone()
                };
                container(
                    column![
                        text("Removal Progress").size(15).style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        progress_bar(0.0..=1.0, progress_value).width(Length::Fill),
                        Space::with_height(Length::Fixed(5.0)),
                        text(&progress_text).size(12)
                            .style(iced::theme::Text::Color(if self.is_complete {
                                iced::Color::from_rgb(0.0, 0.8, 0.0)
                            } else {
                                theme.text()
                            })),
                    ]
                    .spacing(6)
                    .padding(Padding::new(16.0))
                )
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
            } else {
                container(Space::with_height(Length::Shrink))
            };

            let material_font = crate::gui::fonts::get_material_symbols_font();

            let buttons = if self.is_complete {
                row![
                    Space::with_width(Length::Fill),
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::EXIT_SYMBOL).font(material_font),
                            text(" Exit")
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                    })))
                    .padding(Padding::new(12.0)),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            } else {
                row![
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::CANCEL_SYMBOL).font(material_font),
                            text(" Cancel")
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: false,
                    })))
                    .padding(Padding::new(12.0)),
                    Space::with_width(Length::Fill),
                    {
                        if self.is_removing {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font),
                                    text(" Removing...")
                                ]
                                .spacing(4)
                                .align_items(Alignment::Center)
                            )
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                            })))
                            .padding(Padding::new(12.0))
                        } else {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font),
                                    text(" Remove")
                                ]
                                .spacing(4)
                                .align_items(Alignment::Center)
                            )
                            .on_press(Message::RemoveFlatpaks)
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                            })))
                            .padding(Padding::new(12.0))
                        }
                    },
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            };

            container(
                column![
                    scrollable(
                        column![
                            title,
                            packages_section,
                            progress_section,
                        ]
                        .spacing(12)
                        .padding(Padding::new(0.0))
                    )
                    .height(Length::Fill),
                    container(buttons)
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(ButtonBarStyle))),
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
                text("Failed to load Flatpak information")
                    .size(18)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3)))
            )
            .width(Length::Fixed(600.0))
            .padding(Padding::new(30.0))
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
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

struct DialogContainerStyle;

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.97,
                palette.background.g * 0.97,
                palette.background.b * 0.97,
                1.0,
            ))),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct InfoContainerStyle;

impl iced::widget::container::StyleSheet for InfoContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.97,
                palette.background.g * 0.97,
                palette.background.b * 0.97,
                1.0,
            ))),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
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
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.primary.r * 0.1,
                palette.primary.g * 0.1,
                palette.primary.b * 0.1,
                0.3,
            ))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}
