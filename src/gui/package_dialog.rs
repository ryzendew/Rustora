use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone)]
pub enum Message {
    LoadPackageInfo,
    PackageInfoLoaded(PackageInfo),
    RemovePackages,
    RemovalProgress(String),
    RemovalComplete,
    RemovalError(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub summary: String,
    pub description: String,
    pub size: String,
}

#[derive(Debug)]
pub struct PackageDialog {
    pub package_names: Vec<String>,
    pub package_info: Option<PackageInfo>,
    pub is_loading: bool,
    pub is_removing: bool,
    pub is_complete: bool,
    pub removal_progress: String,
    pub show_dialog: bool,
}

impl PackageDialog {
    pub fn new(package_names: Vec<String>) -> Self {
        Self {
            package_names,
            package_info: None,
            is_loading: true,
            is_removing: false,
            is_complete: false,
            removal_progress: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window(package_names: Vec<String>) -> Result<(), iced::Error> {
        let dialog = Self::new(package_names);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(750.0, 800.0);
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <PackageDialog as Application>::run(iced::Settings {
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

        let content = if self.is_loading {
            container(
                column![
                    text("Loading package information...").size(18),
                    Space::with_height(Length::Fixed(20.0)),
                    progress_bar(0.0..=1.0, 0.5).width(Length::Fill),
                ]
                .spacing(15)
                .align_items(Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fixed(600.0))
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if let Some(ref info) = self.package_info {
            let title_text = if self.package_names.len() == 1 {
                format!("Remove {}", info.name)
            } else {
                format!("Remove {} Packages", self.package_names.len())
            };

            let title = container(
                    text(&title_text)
                        .size(20)
                        .style(iced::theme::Text::Color(theme.primary()))
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0));

            let info_section = container(
                column![
                    text("Package Information").size(16).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(10.0)),
                    row![
                        text("Name:").width(Length::Fixed(120.0)),
                        text(&info.name).width(Length::Fill).shaping(iced::widget::text::Shaping::Advanced),
                    ]
                    .width(Length::Fill)
                    .spacing(10),
                    Space::with_height(Length::Fixed(5.0)),
                    row![
                        text("Version:").width(Length::Fixed(120.0)),
                        text(&info.version).width(Length::Fill).shaping(iced::widget::text::Shaping::Advanced),
                    ]
                    .width(Length::Fill)
                    .spacing(10),
                    Space::with_height(Length::Fixed(5.0)),
                    row![
                        text("Release:").width(Length::Fixed(120.0)),
                        text(&info.release).width(Length::Fill).shaping(iced::widget::text::Shaping::Advanced),
                    ]
                    .width(Length::Fill)
                    .spacing(10),
                    Space::with_height(Length::Fixed(5.0)),
                    row![
                        text("Architecture:").width(Length::Fixed(120.0)),
                        text(&info.arch).width(Length::Fill).shaping(iced::widget::text::Shaping::Advanced),
                    ]
                    .width(Length::Fill)
                    .spacing(10),
                    Space::with_height(Length::Fixed(5.0)),
                    row![
                        text("Size:").width(Length::Fixed(120.0)),
                        text(&info.size).width(Length::Fill).shaping(iced::widget::text::Shaping::Advanced),
                    ]
                    .width(Length::Fill)
                    .spacing(10),
                    Space::with_height(Length::Fixed(15.0)),
                    text("Summary:").size(14).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(5.0)),
                    text(&info.summary).size(12).shaping(iced::widget::text::Shaping::Advanced),
                    Space::with_height(Length::Fixed(15.0)),
                    text("Description:").size(14).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(5.0)),
                    scrollable(
                        text(&info.description).size(12).shaping(iced::widget::text::Shaping::Advanced)
                    )
                    .height(Length::Fixed(100.0)),
                ]
                .spacing(0)
                .padding(Padding::new(20.0))
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
                        text("Removal Progress").size(16).style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(10.0)),
                        progress_bar(0.0..=1.0, progress_value).width(Length::Fill),
                        Space::with_height(Length::Fixed(5.0)),
                        text(&progress_text).size(12)
                            .style(iced::theme::Text::Color(if self.is_complete {
                                iced::Color::from_rgb(0.0, 0.8, 0.0)
                            } else {
                                theme.text()
                            })),
                    ]
                    .spacing(8)
                    .padding(Padding::new(20.0))
                )
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
            } else {
                container(Space::with_height(Length::Shrink))
            };

            let buttons = if self.is_complete {
                row![
                    Space::with_width(Length::Fill),
                    {
                        let material_font = crate::gui::fonts::get_material_symbols_font();
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
                            .padding(Padding::new(14.0))
                    },
                ]
                .spacing(10)
                .align_items(Alignment::Center)
                .padding(Padding::new(20.0))
            } else {
                let material_font = crate::gui::fonts::get_material_symbols_font();
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
                        .padding(Padding::new(14.0)),
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
                                .padding(Padding::new(14.0))
                        } else {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font),
                                    text(" Remove")
                                ]
                                .spacing(4)
                                .align_items(Alignment::Center)
                            )
                                .on_press(Message::RemovePackages)
                                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                    is_primary: true,
                                })))
                                .padding(Padding::new(14.0))
                        }
                    },
                ]
                .spacing(10)
                .align_items(Alignment::Center)
                .padding(Padding::new(20.0))
            };

            container(
                column![
                    scrollable(
                        column![
                            title,
                            info_section,
                            progress_section,
                        ]
                        .spacing(15)
                        .padding(Padding::new(20.0))
                    )
                    .height(Length::Fill),
                    container(buttons)
                        .width(Length::Fill)
                        .padding(Padding::new(20.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle))),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else {
            container(
                text("Failed to load package information")
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

impl Application for PackageDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::LoadPackageInfo);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        if let Some(ref info) = self.package_info {
            if self.package_names.len() == 1 {
                format!("Remove {} - Rustora", info.name)
            } else {
                format!("Remove {} Packages - Rustora", self.package_names.len())
            }
        } else {
            "Remove Package - Rustora".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPackageInfo => {
                self.is_loading = true;
                let package_names = self.package_names.clone();
                iced::Command::perform(load_package_info(package_names), |result| {
                    match result {
                        Ok(info) => Message::PackageInfoLoaded(info),
                        Err(e) => Message::RemovalError(e),
                    }
                })
            }
            Message::PackageInfoLoaded(info) => {
                self.is_loading = false;
                self.package_info = Some(info);
                iced::Command::none()
            }
            Message::RemovePackages => {
                self.is_removing = true;
                self.removal_progress = "Preparing removal...".to_string();
                let package_names = self.package_names.clone();
                iced::Command::perform(remove_packages(package_names), |result| {
                    match result {
                        Ok(progress) => Message::RemovalProgress(progress),
                        Err(e) => Message::RemovalError(e),
                    }
                })
            }
            Message::RemovalProgress(progress) => {
                let progress_clone = progress.clone();
                self.removal_progress = progress;
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
                self.show_dialog = false;
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

async fn load_package_info(package_names: Vec<String>) -> Result<PackageInfo, String> {
    let package_name = package_names.first().ok_or("No packages specified")?;

    // Use rpm -qi for installed packages to get complete information
    let output = TokioCommand::new("rpm")
        .args(["-qi", package_name])
        .output()
        .await
        .map_err(|e| format!("Failed to execute rpm: {}", e))?;

    if !output.status.success() {
        return Err(format!("Failed to read package info: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = PackageInfo {
        name: String::new(),
        version: String::new(),
        release: String::new(),
        arch: String::new(),
        summary: String::new(),
        description: String::new(),
        size: String::new(),
    };

    let mut description_lines = Vec::new();
    let mut in_description = false;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name        :") {
            info.name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Version     :") {
            info.version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Release     :") {
            info.release = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Architecture:") {
            info.arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Summary     :") {
            info.summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Size        :") {
            let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            let size_bytes: u64 = size_str.parse().unwrap_or(0);
            info.size = format_size(size_bytes);
        } else if line.starts_with("Description :") {
            in_description = true;
            let desc = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !desc.is_empty() {
                description_lines.push(desc.to_string());
            }
        } else if in_description {
            // Continue collecting description lines until we hit another field
            if line.is_empty() {
                // Empty line might be end of description, but continue if next line is not a field
            } else if line.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && line.ends_with(':') && line.len() < 20 {
                // Likely a new field (short uppercase label ending with colon)
                in_description = false;
            } else {
                description_lines.push(line.to_string());
            }
        }
    }

    // Combine all description lines
    info.description = description_lines.join(" ").trim().to_string();
    if info.description.is_empty() {
        info.description = info.summary.clone();
    }

    Ok(info)
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

async fn remove_packages(package_names: Vec<String>) -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf")
        .arg("remove")
        .arg("-y")
        .arg("--assumeyes");
    for name in &package_names {
        cmd.arg(name);
    }
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute removal: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
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

struct DialogContainerStyle;

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: 20.0.into(),
                width: 2.0,
                color: palette.primary,
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
                palette.background.r * 0.95,
                palette.background.g * 0.95,
                palette.background.b * 0.95,
                1.0,
            ))),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}
