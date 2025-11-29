use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Border, Theme as IcedTheme, Color};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use futures::future;
use crate::gui::dialog_design::DialogDesign;

#[derive(Debug, Clone)]
pub enum Message {
    LoadPackageInfo,
    PackageInfoLoaded(Vec<PackageInfo>),
    InstallPackages,
    InstallationProgress(String),
    InstallationComplete,
    InstallationError(String),
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
pub struct InstallDialog {
    pub package_names: Vec<String>,
    pub package_info: Vec<PackageInfo>,
    pub is_loading: bool,
    pub is_installing: bool,
    pub is_complete: bool,
    pub installation_progress: String,
    pub show_dialog: bool,
}

impl InstallDialog {
    pub fn new(package_names: Vec<String>) -> Self {
        Self {
            package_names,
            package_info: Vec::new(),
            is_loading: true,
            is_installing: false,
            is_complete: false,
            installation_progress: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window(package_names: Vec<String>) -> Result<(), iced::Error> {
        let dialog = Self::new(package_names);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(600.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(480.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <InstallDialog as Application>::run(iced::Settings {
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
                    text("Loading package information...")
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
        } else if !self.package_info.is_empty() {
            let material_font = crate::gui::fonts::get_material_symbols_font();
            let title_text = if self.package_info.len() == 1 {
                format!("Install {}", self.package_info[0].name)
            } else {
                format!("Install {} Packages", self.package_info.len())
            };

            let header = container(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL)
                        .font(material_font)
                        .size(title_size * 1.2)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(DialogDesign::space_small()),
                    text(&title_text)
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(Length::Fill),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding(DialogDesign::pad_medium());

            let packages_content = if self.package_info.len() == 1 {
                let info = &self.package_info[0];
                let label_w = 100.0;
                container(
                    column![
                        text("Package Details")
                            .size(body_size * 1.1)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(DialogDesign::space_small()),
                        row![
                            text("Name:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                            text(&info.name).size(body_size).width(Length::Fill),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL),
                        Space::with_height(DialogDesign::space_tiny()),
                        row![
                            text("Version:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                            text(&info.version).size(body_size).width(Length::Fill),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL),
                        Space::with_height(DialogDesign::space_tiny()),
                        row![
                            text("Release:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                            text(&info.release).size(body_size).width(Length::Fill),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL),
                        Space::with_height(DialogDesign::space_tiny()),
                        row![
                            text("Arch:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                            text(&info.arch).size(body_size).width(Length::Fill),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL),
                        Space::with_height(DialogDesign::space_tiny()),
                        row![
                            text("Size:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                            text(&info.size).size(body_size).width(Length::Fill),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL),
                        Space::with_height(DialogDesign::space_medium()),
                        container(Space::with_height(Length::Fixed(1.0)))
                            .width(Length::Fill)
                            .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                        Space::with_height(DialogDesign::space_medium()),
                        text("Summary")
                            .size(body_size * 1.05)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(DialogDesign::space_tiny()),
                        text(&info.summary)
                            .size(body_size * 0.95)
                            .shaping(iced::widget::text::Shaping::Advanced),
                        Space::with_height(DialogDesign::space_medium()),
                        text("Description")
                            .size(body_size * 1.05)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(DialogDesign::space_tiny()),
                        scrollable(
                            text(&info.description)
                                .size(body_size * 0.95)
                                .shaping(iced::widget::text::Shaping::Advanced)
                        )
                        .height(Length::Fixed(150.0)),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_medium())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
            } else {
                container(
                    column![
                        text(format!("{} Packages", self.package_info.len()))
                            .size(body_size * 1.1)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(DialogDesign::space_small()),
                        scrollable(
                            column(
                                self.package_info
                                    .iter()
                                    .map(|info| {
                                        container(
                                            column![
                                                row![
                                                    text(&info.name)
                                                        .size(body_size)
                                                        .style(iced::theme::Text::Color(theme.primary())),
                                                    Space::with_width(Length::Fill),
                                                    text(format!("{} {}", info.version, info.release))
                                                        .size(body_size * 0.9)
                                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                                ]
                                                .spacing(DialogDesign::SPACE_SMALL),
                                                Space::with_height(DialogDesign::space_tiny()),
                                                text(&info.summary)
                                                    .size(body_size * 0.9)
                                                    .shaping(iced::widget::text::Shaping::Advanced),
                                                Space::with_height(DialogDesign::space_tiny()),
                                                row![
                                                    text(format!("Arch: {}", info.arch))
                                                        .size(body_size * 0.85)
                                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                                    Space::with_width(Length::Fill),
                                                    text(format!("Size: {}", info.size))
                                                        .size(body_size * 0.85)
                                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                                ],
                                            ]
                                            .spacing(0)
                                            .padding(DialogDesign::pad_small())
                                        )
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
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
            };

            let progress = if self.is_installing || self.is_complete {
                let value = if self.is_complete { 1.0 } else { 0.7 };
                let progress_text = if self.is_complete {
                    "Installation completed successfully!".to_string()
                } else {
                    self.installation_progress.clone()
                };
                container(
                    column![
                        text("Progress")
                            .size(body_size * 1.05)
                            .style(iced::theme::Text::Color(theme.primary())),
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

            let buttons = {
                let material_font = crate::gui::fonts::get_material_symbols_font();
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
                            if self.is_installing {
                                button(
                                    row![
                                        text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(button_size * 1.1),
                                        text(" Installing...").size(button_size)
                                    ]
                                    .spacing(DialogDesign::SPACE_TINY)
                                    .align_items(Alignment::Center)
                                )
                                .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
                                .padding(DialogDesign::pad_small())
                            } else {
                                button(
                                    row![
                                        text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(button_size * 1.1),
                                        text(" Install").size(button_size)
                                    ]
                                    .spacing(DialogDesign::SPACE_TINY)
                                    .align_items(Alignment::Center)
                                )
                                .on_press(Message::InstallPackages)
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
                            packages_content,
                            progress,
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
                    text("Failed to load package information")
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

impl Application for InstallDialog {
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
        if !self.package_info.is_empty() {
            if self.package_info.len() == 1 {
                format!("Install {} - Rustora", self.package_info[0].name)
            } else {
                format!("Install {} Packages - Rustora", self.package_info.len())
            }
        } else {
            "Install Package - Rustora".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPackageInfo => {
                self.is_loading = true;
                let package_names = self.package_names.clone();
                iced::Command::perform(load_package_info(package_names), |result| {
                    match result {
                        Ok(infos) => Message::PackageInfoLoaded(infos),
                        Err(e) => Message::InstallationError(e),
                    }
                })
            }
            Message::PackageInfoLoaded(infos) => {
                self.is_loading = false;
                self.package_info = infos;
                iced::Command::none()
            }
            Message::InstallPackages => {
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                let package_names = self.package_names.clone();
                iced::Command::perform(install_packages(package_names), |result| {
                    match result {
                        Ok(progress) => Message::InstallationProgress(progress),
                        Err(e) => Message::InstallationError(e.to_string()),
                    }
                })
            }
            Message::InstallationProgress(progress) => {
                let progress_clone = progress.clone();
                self.installation_progress = progress;
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
                self.is_installing = false;
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

async fn load_package_info(package_names: Vec<String>) -> Result<Vec<PackageInfo>, String> {
    if package_names.is_empty() {
        return Err("No packages specified".to_string());
    }

    let mut futures = Vec::new();
    for package_name in &package_names {
        let name = package_name.clone();
        futures.push(async move {
            load_single_package_info(name).await
        });
    }

    let results: Vec<Result<PackageInfo, String>> = future::join_all(futures).await;

    let mut infos = Vec::new();
    for result in results {
        match result {
            Ok(info) => infos.push(info),
            Err(_) => {}
        }
    }

    if infos.is_empty() {
        return Err("Failed to load information for any packages".to_string());
    }

    Ok(infos)
}

async fn load_single_package_info(package_name: String) -> Result<PackageInfo, String> {
    let output = TokioCommand::new("dnf")
        .args(["info", &package_name])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf: {}", e))?;

    if !output.status.success() {
        return Err(format!("Failed to read package info for {}: {}",
            package_name, String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = PackageInfo {
        name: package_name.clone(),
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
        if line.starts_with("Name") && line.contains(':') {
            info.name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Version") && line.contains(':') {
            info.version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Release") && line.contains(':') {
            info.release = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Architecture") && line.contains(':') {
            info.arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Summary") && line.contains(':') {
            info.summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if (line.starts_with("Installed size") || line.starts_with("Download size") || line.starts_with("Size")) && line.contains(':') {
            let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if line.starts_with("Installed size") || info.size.is_empty() {
                if let Ok(size_bytes) = parse_size(size_str) {
                    info.size = format_size(size_bytes);
                } else {
                    info.size = size_str.to_string();
                }
            }
            in_description = false;
        } else if line.starts_with("Description") && line.contains(':') {
            in_description = true;
            let desc = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !desc.is_empty() {
                description_lines.push(desc.to_string());
            }
        } else if in_description {
            if line.is_empty() {
            } else if line.starts_with("               :") {
                let desc_cont = line.trim_start_matches("               :").trim();
                if !desc_cont.is_empty() {
                    description_lines.push(desc_cont.to_string());
                }
            } else if line.contains(':') {
                let field_name = line.split(':').next().unwrap_or("").trim();
                let known_fields = ["URL", "License", "Vendor", "Source", "Repository", "Epoch"];
                if known_fields.iter().any(|&f| field_name.starts_with(f)) ||
                   (field_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) &&
                    field_name.len() < 20 &&
                    !field_name.eq_ignore_ascii_case("description")) {
                    in_description = false;
                } else {
                    description_lines.push(line.to_string());
                }
            } else {
                description_lines.push(line.to_string());
            }
        }
    }

    info.description = description_lines.join(" ").trim().to_string();
    if info.description.is_empty() {
        info.description = info.summary.clone();
    }

    if info.name.is_empty() {
        info.name = package_name;
    }

    Ok(info)
}

fn parse_size(size_str: &str) -> Result<u64, ()> {
    let size_str = size_str.trim();
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(());
    }

    let number: f64 = parts[0].parse().map_err(|_| ())?;
    let unit = if parts.len() > 1 {
        parts[1].to_lowercase()
    } else {
        "b".to_string()
    };

    let multiplier = match unit.as_str() {
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    Ok((number * multiplier) as u64)
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

async fn install_packages(package_names: Vec<String>) -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf")
        .arg("install")
        .arg("-y")
        .arg("--assumeyes");
    for name in &package_names {
        cmd.arg(name);
    }
    let output = cmd
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
