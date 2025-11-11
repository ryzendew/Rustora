use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use futures::future;

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
        window_settings.size = iced::Size::new(750.0, 800.0);
        window_settings.resizable = true;
        window_settings.decorations = true;
        
        // Use cached InterVariable font (optimized)
        let default_font = crate::gui::fonts::get_inter_font();

        <InstallDialog as Application>::run(iced::Settings {
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
        } else if !self.package_info.is_empty() {
            let title_text = if self.package_info.len() == 1 {
                format!("Install {}", self.package_info[0].name)
            } else {
                format!("Install {} Packages", self.package_info.len())
            };
            
            let title = container(
                    text(&title_text)
                        .size(20)
                        .style(iced::theme::Text::Color(theme.primary()))
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0));

            // Show list of all packages with their info
            let packages_list = if self.package_info.len() == 1 {
                // Single package - show detailed info
                let info = &self.package_info[0];
                container(
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
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
            } else {
                // Multiple packages - show list with summary info
                container(
                    column![
                        text(format!("Packages to Install ({})", self.package_info.len()))
                            .size(16)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(10.0)),
                        scrollable(
                            column(
                                self.package_info
                                    .iter()
                                    .map(|info| {
                                        container(
                                            column![
                                                row![
                                                    text(&info.name)
                                                        .size(14)
                                                        .style(iced::theme::Text::Color(theme.primary())),
                                                    Space::with_width(Length::Fill),
                                                    text(format!("{} {}", info.version, info.release))
                                                        .size(12),
                                                ]
                                                .width(Length::Fill)
                                                .spacing(10),
                                                Space::with_height(Length::Fixed(5.0)),
                                                text(&info.summary)
                                                    .size(12)
                                                    .shaping(iced::widget::text::Shaping::Advanced),
                                                Space::with_height(Length::Fixed(5.0)),
                                                row![
                                                    text(format!("Arch: {}", info.arch)).size(11),
                                                    Space::with_width(Length::Fill),
                                                    text(format!("Size: {}", info.size)).size(11),
                                                ]
                                                .width(Length::Fill),
                                            ]
                                            .spacing(5)
                                            .padding(Padding::new(12.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(PackageItemStyle)))
                                        .width(Length::Fill)
                                        .into()
                                    })
                                    .collect::<Vec<_>>()
                            )
                            .spacing(8)
                        )
                        .height(Length::Fixed(400.0)),
                    ]
                    .spacing(0)
                    .padding(Padding::new(20.0))
                )
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
            };
            
            let info_section = packages_list;

            let progress_section = if self.is_installing || self.is_complete {
                let progress_value = if self.is_complete { 1.0 } else { 0.7 };
                let progress_text = if self.is_complete {
                    "Installation completed successfully!".to_string()
                } else {
                    self.installation_progress.clone()
                };
                container(
                    column![
                        text("Installation Progress").size(16).style(iced::theme::Text::Color(theme.primary())),
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
                // Show only Exit button when installation is complete
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
                                .padding(Padding::new(14.0))
                        } else {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font),
                                    text(" Install")
                                ]
                                .spacing(4)
                                .align_items(Alignment::Center)
                            )
                                .on_press(Message::InstallPackages)
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
                format!("Install {} - FedoraForge", self.package_info[0].name)
            } else {
                format!("Install {} Packages - FedoraForge", self.package_info.len())
            }
        } else {
            "Install Package - FedoraForge".to_string()
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
                // Don't close automatically, let user click Exit
                iced::Command::none()
            }
            Message::InstallationError(_msg) => {
                // Installation error occurred
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
    
    // Load info for all packages in parallel
    let mut futures = Vec::new();
    for package_name in &package_names {
        let name = package_name.clone();
        futures.push(async move {
            load_single_package_info(name).await
        });
    }
    
    // Wait for all to complete
    let results: Vec<Result<PackageInfo, String>> = future::join_all(futures).await;
    
    // Collect successful results, skip failures
    let mut infos = Vec::new();
    for result in results {
        match result {
            Ok(info) => infos.push(info),
            Err(e) => {
                eprintln!("Warning: Failed to load package info: {}", e);
                // Continue with other packages even if one fails
            }
        }
    }
    
    if infos.is_empty() {
        return Err("Failed to load information for any packages".to_string());
    }
    
    Ok(infos)
}

async fn load_single_package_info(package_name: String) -> Result<PackageInfo, String> {
    // Use dnf info for packages that aren't installed yet
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
        // Handle fields that may have variable spacing - use contains and split on ':'
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
            // Prefer "Installed size" over "Download size" - only update if size is empty or we found "Installed size"
            let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if line.starts_with("Installed size") || info.size.is_empty() {
                // DNF info shows size as "79.1 MiB" or "253.1 MiB", parse it
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
            // Continue collecting description lines until we hit another field
            if line.is_empty() {
                // Empty line might be end of description, but continue if next line is not a field
            } else if line.starts_with("               :") {
                // Continuation of description (dnf info format with leading spaces and colon)
                let desc_cont = line.trim_start_matches("               :").trim();
                if !desc_cont.is_empty() {
                    description_lines.push(desc_cont.to_string());
                }
            } else if line.contains(':') {
                // Check if it's a new field (not continuation of description)
                let field_name = line.split(':').next().unwrap_or("").trim();
                // Known field names that end description
                let known_fields = ["URL", "License", "Vendor", "Source", "Repository", "Epoch"];
                if known_fields.iter().any(|&f| field_name.starts_with(f)) ||
                   (field_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && 
                    field_name.len() < 20 &&
                    !field_name.eq_ignore_ascii_case("description")) {
                    in_description = false;
                } else {
                    // Continuation of description (might have ":" in the text)
                    description_lines.push(line.to_string());
                }
            } else {
                // Regular description line
                description_lines.push(line.to_string());
            }
        }
    }

    // Combine all description lines
    info.description = description_lines.join(" ").trim().to_string();
    if info.description.is_empty() {
        info.description = info.summary.clone();
    }
    
    // If name wasn't found, use the package name we tried to load
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
    
    // Handle MiB, KiB, GiB, TiB (binary) and MB, KB, GB, TB (decimal)
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

struct PackageItemStyle;

impl iced::widget::container::StyleSheet for PackageItemStyle {
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
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
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

