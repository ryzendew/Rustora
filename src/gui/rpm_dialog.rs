use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    LoadRpmInfo,
    RpmInfoLoaded(RpmInfo),
    InstallRpm,
    InstallationProgress(String),
    InstallationComplete,
    InstallationError(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct RpmInfo {
    pub name: String,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub summary: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub size: String,
}

#[derive(Debug)]
pub struct RpmDialog {
    pub rpm_path: PathBuf,
    pub rpm_info: Option<RpmInfo>,
    pub is_loading: bool,
    pub is_installing: bool,
    pub is_complete: bool,
    pub installation_progress: String,
    pub show_dialog: bool,
}

impl RpmDialog {
    pub fn new(rpm_path: PathBuf) -> Self {
        Self {
            rpm_path,
            rpm_info: None,
            is_loading: true,
            is_installing: false,
            is_complete: false,
            installation_progress: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window(rpm_path: PathBuf) -> Result<(), iced::Error> {
        let dialog = Self::new(rpm_path);
        
        let mut window_settings = iced::window::Settings::default();
        // Optimized for 720p (1280x720) and up, with tiling manager support
        // Width: ~60% of 1280px = 768px, fits comfortably in tiling layouts
        // Height: ~85% of 720px = 612px, leaves room for status bars
        window_settings.size = iced::Size::new(768.0, 612.0);
        window_settings.min_size = Some(iced::Size::new(640.0, 480.0)); // Minimum for smaller screens
        window_settings.max_size = None; // Allow full screen for larger displays
        window_settings.resizable = true;
        window_settings.decorations = true; // Keep decorations for tiling manager compatibility
        
        // Use cached InterVariable font (optimized)
        let default_font = crate::gui::fonts::get_inter_font();

        <RpmDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0), // Optimized for 720p scaling
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }

    pub fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        if !self.show_dialog {
            return Space::with_width(Length::Shrink).into();
        }

        let content =         if self.is_loading {
            container(
                column![
                    text("Loading RPM information...").size(18),
                    Space::with_height(Length::Fixed(20.0)),
                    progress_bar(0.0..=1.0, 0.5).width(Length::Fill),
                ]
                .spacing(15)
                .align_items(Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
        } else if let Some(ref info) = self.rpm_info {
            // Professional header with close button - responsive sizing
            let material_font = crate::gui::fonts::get_material_symbols_font();
            let title = container(
                row![
                    column![
                        text(format!("Install {}", info.name))
                            .size(18) // Optimized for 720p
                            .style(iced::theme::Text::Color(theme.primary())),
                        text(format!("{} {} ({})", info.version, info.release, info.arch))
                            .size(12) // Responsive text size
                            .style(iced::theme::Text::Color(iced::Color::from_rgba(0.6, 0.6, 0.6, 1.0))),
                    ]
                    .spacing(2),
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
            .padding(Padding::new(16.0)); // Optimized padding for 720p

            // Clean, organized info section - optimized for 720p with better space usage
            let info_section = container(
                column![
                    text("Package Information").size(14).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(10.0)),
                    // Use a more compact grid layout for package info
                    row![
                        // Left column
                        column![
                            row![
                                text("Name:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.name).size(11).width(Length::Fill),
                            ]
                            .spacing(8),
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Version:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.version).size(11).width(Length::Fill),
                            ]
                            .spacing(8),
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Release:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.release).size(11).width(Length::Fill),
                            ]
                            .spacing(8),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                        Space::with_width(Length::Fixed(12.0)),
                        // Right column
                        column![
                            row![
                                text("Arch:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.arch).size(11).width(Length::Fill),
                            ]
                            .spacing(8),
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Size:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.size).size(11).width(Length::Fill),
                            ]
                            .spacing(8),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(0)
                    .padding(Padding::new(12.0)),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Summary").size(13).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(4.0)),
                    text(&info.summary).size(11),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Description").size(13).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(4.0)),
                    scrollable(
                        text(&info.description).size(11)
                    )
                    .height(Length::Fixed(110.0)), // Slightly increased for better readability
                ]
                .spacing(0)
                .padding(Padding::new(16.0))
            )
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)));

            let deps_section = if !info.dependencies.is_empty() {
                let material_font = crate::gui::fonts::get_material_symbols_font();
                container(
                    column![
                        row![
                            text(crate::gui::fonts::glyphs::SETTINGS_SYMBOL).font(material_font).size(15).style(iced::theme::Text::Color(theme.primary())),
                            text(format!(" Dependencies ({}):", info.dependencies.len())).size(15).style(iced::theme::Text::Color(theme.primary()))
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center),
                        Space::with_height(Length::Fixed(8.0)),
                        scrollable(
                            column(
                                info.dependencies
                                    .iter()
                                    .map(|dep| {
                                        container(
                                            text(dep).size(11)
                                        )
                                        .padding(Padding::new(8.0))
                                        .style(iced::theme::Container::Custom(Box::new(DependencyItemStyle)))
                                        .width(Length::Fill)
                                        .into()
                                    })
                                    .collect::<Vec<_>>()
                            )
                            .spacing(5)
                        )
                        .height(Length::Fixed(130.0)), // Optimized for 720p
                    ]
                    .spacing(6)
                    .padding(Padding::new(16.0))
                )
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
            } else {
                container(Space::with_height(Length::Shrink))
            };

            let progress_section = if self.is_installing || self.is_complete {
                let progress_value = if self.is_complete { 1.0 } else { 0.7 };
                let progress_text = if self.is_complete {
                    "Installation completed successfully!".to_string()
                } else {
                    self.installation_progress.clone()
                };
                container(
                    column![
                        text("Installation Progress").size(15).style(iced::theme::Text::Color(theme.primary())),
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
                // Show only Exit button when installation is complete
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
                        if self.is_installing {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font),
                                    text(" Installing...")
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
                                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font),
                                    text(" Install")
                                ]
                                .spacing(4)
                                .align_items(Alignment::Center)
                            )
                                .on_press(Message::InstallRpm)
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
                            info_section,
                            deps_section,
                            progress_section,
                        ]
                        .spacing(12) // Optimized spacing for 720p
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
                text("Failed to load RPM information")
                    .size(18)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3)))
            )
            .width(Length::Fixed(600.0))
            .padding(Padding::new(30.0))
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
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

impl Application for RpmDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::LoadRpmInfo);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        if let Some(ref info) = self.rpm_info {
            format!("Install {} - FedoraForge", info.name)
        } else {
            "Install RPM Package - FedoraForge".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadRpmInfo => {
                self.is_loading = true;
                let path = self.rpm_path.clone();
                iced::Command::perform(load_rpm_info(path), |result| {
                    match result {
                        Ok(info) => Message::RpmInfoLoaded(info),
                        Err(e) => Message::InstallationError(e.to_string()),
                    }
                })
            }
            Message::RpmInfoLoaded(info) => {
                self.is_loading = false;
                self.rpm_info = Some(info);
                iced::Command::none()
            }
            Message::InstallRpm => {
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                let path = self.rpm_path.clone();
                iced::Command::perform(install_rpm(path), |result| {
                    match result {
                        Ok(progress) => Message::InstallationProgress(progress),
                        Err(e) => Message::InstallationError(e.to_string()),
                    }
                })
            }
            Message::InstallationProgress(progress) => {
                let progress_clone = progress.clone();
                self.installation_progress = progress.clone();
                // Check if installation is complete
                if progress_clone.contains("Complete") || 
                   progress_clone.contains("Installed") || 
                   progress_clone.contains("complete") ||
                   progress_clone.to_lowercase().contains("success") {
                    iced::Command::perform(async {}, |_| Message::InstallationComplete)
                } else {
                    iced::Command::none()
                }
            }
            Message::InstallationComplete => {
                self.is_installing = false;
                self.is_complete = true;
                self.installation_progress = "Installation completed successfully!".to_string();
                iced::Command::none()
            }
            Message::InstallationError(_msg) => {
                // Installation error occurred
                self.is_installing = false;
                iced::Command::none()
            }
            Message::Cancel => {
                // Fully close the window and exit cleanly
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

async fn load_rpm_info(rpm_path: PathBuf) -> Result<RpmInfo, String> {
    let path_str = rpm_path.to_string_lossy().to_string();
    
    // Run both RPM queries in parallel for faster loading
    let (info_output, deps_output) = tokio::join!(
        TokioCommand::new("rpm")
            .args(["-qip", &path_str])
            .output(),
        TokioCommand::new("rpm")
            .args(["-qpR", &path_str])
            .output()
    );

    let output = info_output.map_err(|e| format!("Failed to execute rpm: {}", e))?;

    if !output.status.success() {
        return Err(format!("Failed to read RPM info: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = RpmInfo {
        name: String::new(),
        version: String::new(),
        release: String::new(),
        arch: String::new(),
        summary: String::new(),
        description: String::new(),
        dependencies: Vec::new(),
        size: String::new(),
    };

    // Optimized parsing: use pattern matching and early exit when all fields found
    let mut found_fields = 0u8;
    const ALL_FIELDS: u8 = 0b01111111; // 7 fields (name, version, release, arch, summary, size, description)
    let mut in_description = false;
    let mut description_lines = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if found_fields == ALL_FIELDS && !in_description {
            break; // Early exit when all fields are found
        }
        
        if line.starts_with("Name        :") && (found_fields & 0b00000001) == 0 {
            info.name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_fields |= 0b00000001;
            in_description = false;
        } else if line.starts_with("Version     :") && (found_fields & 0b00000010) == 0 {
            info.version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_fields |= 0b00000010;
            in_description = false;
        } else if line.starts_with("Release     :") && (found_fields & 0b00000100) == 0 {
            info.release = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_fields |= 0b00000100;
            in_description = false;
        } else if line.starts_with("Architecture:") && (found_fields & 0b00001000) == 0 {
            info.arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_fields |= 0b00001000;
            in_description = false;
        } else if line.starts_with("Summary     :") && (found_fields & 0b00010000) == 0 {
            info.summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_fields |= 0b00010000;
            in_description = false;
        } else if line.starts_with("Size        :") && (found_fields & 0b00100000) == 0 {
            let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if let Ok(size_bytes) = size_str.parse::<u64>() {
                info.size = format_size(size_bytes);
            }
            found_fields |= 0b00100000;
            in_description = false;
        } else if line.starts_with("Description :") && (found_fields & 0b01000000) == 0 {
            let desc_start = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !desc_start.is_empty() {
                description_lines.push(desc_start.to_string());
            }
            found_fields |= 0b01000000;
            in_description = true;
        } else if in_description {
            // Continue reading multi-line description
            if line.is_empty() || (line.contains(':') && !line.starts_with(' ')) {
                in_description = false;
            } else if !line.is_empty() {
                description_lines.push(line.to_string());
            }
        }
    }
    
    if !description_lines.is_empty() {
        info.description = description_lines.join("\n");
    }

    // Process dependencies from parallel query
    if let Ok(deps_result) = deps_output {
        if deps_result.status.success() {
            let deps_stdout = String::from_utf8_lossy(&deps_result.stdout);
            info.dependencies = deps_stdout
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
        }
    }

    Ok(info)
}

async fn install_rpm(rpm_path: PathBuf) -> Result<String, String> {
    let path_str = rpm_path.to_string_lossy().to_string();

    // Use pkexec (polkit) instead of sudo for better security
    let output = TokioCommand::new("pkexec")
        .args(["dnf", "install", "-y", "--assumeyes", &path_str])
        .output()
        .await
        .map_err(|e| format!("Failed to execute installation: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Installation failed: {}\n{}", stderr, stdout));
    }

    Ok("Installation Complete!".to_string())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
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

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
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

struct DependencyItemStyle;

impl iced::widget::container::StyleSheet for DependencyItemStyle {
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

