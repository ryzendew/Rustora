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
        window_settings.size = iced::Size::new(600.0, 550.0);
        window_settings.min_size = Some(iced::Size::new(480.0, 400.0));
        window_settings.max_size = None;
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <RpmDialog as Application>::run(iced::Settings {
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
        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.2).round();
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();

        let content =         if self.is_loading {
            container(
                column![
                    text("Loading RPM information...").size(title_font_size),
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
            let material_font = crate::gui::fonts::get_material_symbols_font();
            let title = container(
                row![
                    column![
                        text(format!("Install {}", info.name))
                            .size(title_font_size)
                            .style(iced::theme::Text::Color(theme.primary())),
                        text(format!("{} {} ({})", info.version, info.release, info.arch))
                            .size(body_font_size * 0.7)
                            .style(iced::theme::Text::Color(iced::Color::from_rgba(0.6, 0.6, 0.6, 1.0))),
                    ]
                    .spacing(2),
                    Space::with_width(Length::Fill),
                    button(
                        text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_size)
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

            let info_section = container(
                column![
                    text("Package Information").size(body_font_size * 0.9).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(10.0)),
                    row![
                        column![
                            row![
                                text("Name:").size(body_font_size * 0.75).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.name).size(body_font_size * 0.75).width(Length::Fill),
                            ]
                            .spacing(8),
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Version:").size(body_font_size * 0.75).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.version).size(body_font_size * 0.75).width(Length::Fill),
                            ]
                            .spacing(8),
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Release:").size(body_font_size * 0.75).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.release).size(body_font_size * 0.75).width(Length::Fill),
                            ]
                            .spacing(8),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                        Space::with_width(Length::Fixed(12.0)),
                        column![
                            row![
                                text("Arch:").size(body_font_size * 0.75).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.arch).size(body_font_size * 0.75).width(Length::Fill),
                            ]
                            .spacing(8),
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Size:").size(body_font_size * 0.75).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.size).size(body_font_size * 0.75).width(Length::Fill),
                            ]
                            .spacing(8),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(0)
                    .padding(Padding::new(12.0)),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Summary").size(body_font_size * 0.85).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(4.0)),
                    text(&info.summary).size(body_font_size * 0.75),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Description").size(body_font_size * 0.85).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(4.0)),
                    scrollable(
                        text(&info.description).size(body_font_size * 0.75)
                    )
                    .height(Length::Fixed(110.0)),
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
                            text(crate::gui::fonts::glyphs::SETTINGS_SYMBOL).font(material_font).size(icon_size * 0.8).style(iced::theme::Text::Color(theme.primary())),
                            text(format!(" Dependencies ({}):", info.dependencies.len())).size(body_font_size * 0.9).style(iced::theme::Text::Color(theme.primary()))
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
                                            text(dep).size(body_font_size * 0.75)
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
                        .height(Length::Fixed(130.0)),
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
                        text("Installation Progress").size(body_font_size * 0.9).style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        progress_bar(0.0..=1.0, progress_value).width(Length::Fill),
                        Space::with_height(Length::Fixed(5.0)),
                        text(&progress_text).size(body_font_size * 0.8)
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
                            text(crate::gui::fonts::glyphs::EXIT_SYMBOL).font(material_font).size(icon_size * 0.9),
                            text(" Exit").size(button_font_size)
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
                    .size(button_font_size)
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
            format!("Install {} - Rustora", info.name)
        } else {
            "Install RPM Package - Rustora".to_string()
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

    let mut found_name = false;
    let mut found_version = false;
    let mut found_release = false;
    let mut found_arch = false;
    let mut found_summary = false;
    let mut found_size = false;
    let mut found_description = false;
    let mut in_description = false;
    let mut description_lines = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if found_name && found_version && found_release && found_arch && found_summary && found_size && found_description && !in_description {
            break;
        }

        if line.starts_with("Name        :") && !found_name {
            info.name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_name = true;
            in_description = false;
        } else if line.starts_with("Version     :") && !found_version {
            info.version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_version = true;
            in_description = false;
        } else if line.starts_with("Release     :") && !found_release {
            info.release = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_release = true;
            in_description = false;
        } else if line.starts_with("Architecture:") && !found_arch {
            info.arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_arch = true;
            in_description = false;
        } else if line.starts_with("Summary     :") && !found_summary {
            info.summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            found_summary = true;
            in_description = false;
        } else if line.starts_with("Size        :") && !found_size {
            let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if let Ok(size_bytes) = size_str.parse::<u64>() {
                info.size = format_size(size_bytes);
            }
            found_size = true;
            in_description = false;
        } else if line.starts_with("Description :") && !found_description {
            let desc_start = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !desc_start.is_empty() {
                description_lines.push(desc_start.to_string());
            }
            found_description = true;
            in_description = true;
        } else if in_description {
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

    let mut cmd = TokioCommand::new("pkexec");
    cmd.args([
        "dnf",
        "install",
        "-y",
        "--assumeyes",
        "--nogpgcheck",
        &path_str
    ]);

    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute installation: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let has_debian_deps = stderr.contains("libc6") || stderr.contains("libgtk-3-0") ||
                             stderr.contains("libwebkit") || stderr.contains("libxdo") ||
                             stdout.contains("libc6") || stdout.contains("libgtk-3-0") ||
                             stdout.contains("libwebkit") || stdout.contains("libxdo");
        let has_file_conflicts = stderr.contains("conflicts with file") || stdout.contains("conflicts with file");
        let has_missing_deps = stderr.contains("nothing provides") || stdout.contains("nothing provides");

        let error_msg = if has_debian_deps || has_missing_deps {
            format!(
                "Installation failed due to dependency resolution issues.\n\n\
                This converted package contains Debian/Ubuntu package names that don't exist in Fedora.\n\n\
                Common Debian → Fedora package name mappings:\n\
                • libc6 → glibc (usually already installed)\n\
                • libgtk-3-0 → gtk3\n\
                • libwebkit2gtk-4.1-0 → webkit2gtk4.1\n\
                • libxdo3 → xdotool\n\n\
                Error details:\n{}\n{}\n\n\
                [WARN] WARNING: Package conversion cannot automatically map dependencies.\n\
                \n\
                Recommended solutions:\n\
                1. Check if there's a native RPM version available (preferred)\n\
                2. Look for Flatpak or AppImage versions of the application\n\
                3. Manually install the Fedora equivalents of missing dependencies, then try:\n\
                   dnf install --nogpgcheck --skip-broken {}\n\
                4. Extract the package manually and install files directly (advanced)\n\
                5. Report the issue to the package maintainer to provide native RPM support\n\
                \n\
                Note: FPM conversion preserves original dependency names and cannot automatically\n\
                map them to Fedora package names. This is a known limitation of package conversion.",
                stderr, stdout, rpm_path.to_string_lossy()
            )
        } else if has_file_conflicts {
            format!(
                "Installation failed due to file conflicts.\n\n\
                Converted packages may try to claim ownership of system directories like /usr/bin, /usr/lib, etc., \
                which are owned by the filesystem package.\n\n\
                Error details:\n{}\n{}\n\n\
                [WARN] WARNING: Using --allowerasing or --force could remove critical system files and break your system.\n\
                \n\
                Recommended solutions:\n\
                1. Check if there's a native RPM version available (preferred)\n\
                2. Look for Flatpak or AppImage versions of the application\n\
                3. Manually extract and install the package contents (advanced)\n\
                4. Report the issue to the package maintainer to provide native RPM support\n\
                \n\
                Note: Package conversion may have limitations depending on the source package structure.",
                stderr, stdout
            )
        } else {
            format!("Installation failed:\n{}\n{}", stderr, stdout)
        };

        return Err(error_msg);
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
