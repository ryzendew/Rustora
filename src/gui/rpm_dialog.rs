use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme, Color};
use crate::gui::dialog_design::DialogDesign;
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
        let title_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_size = (settings.font_size_body * settings.scale_body).round();
        let button_size = (settings.font_size_buttons * settings.scale_buttons).round();

        let content = if self.is_loading {
            container(
                column![
                    text("Loading RPM information...")
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
        } else if let Some(ref info) = self.rpm_info {
            let material_font = crate::gui::fonts::get_material_symbols_font();
            let header = container(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL)
                        .font(material_font)
                        .size(title_size * 1.2)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(DialogDesign::space_small()),
                    column![
                        text(format!("Install {}", info.name))
                            .size(title_size)
                            .style(iced::theme::Text::Color(theme.primary())),
                        text(format!("{} {} ({})", info.version, info.release, info.arch))
                            .size(body_size * 0.8)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(DialogDesign::SPACE_TINY),
                    Space::with_width(Length::Fill),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding(DialogDesign::pad_medium());

            let label_w = 90.0;
            let info_section = container(
                column![
                    text("Package Details")
                        .size(body_size * 1.1)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(DialogDesign::space_small()),
                    row![
                        column![
                            row![
                                text("Name:").size(body_size).width(Length::Fixed(label_w))
                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.name).size(body_size).width(Length::Fill),
                            ]
                            .spacing(DialogDesign::SPACE_SMALL),
                            Space::with_height(DialogDesign::space_tiny()),
                            row![
                                text("Version:").size(body_size).width(Length::Fixed(label_w))
                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.version).size(body_size).width(Length::Fill),
                            ]
                            .spacing(DialogDesign::SPACE_SMALL),
                            Space::with_height(DialogDesign::space_tiny()),
                            row![
                                text("Release:").size(body_size).width(Length::Fixed(label_w))
                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.release).size(body_size).width(Length::Fill),
                            ]
                            .spacing(DialogDesign::SPACE_SMALL),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                        Space::with_width(DialogDesign::space_medium()),
                        column![
                            row![
                                text("Arch:").size(body_size).width(Length::Fixed(label_w))
                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.arch).size(body_size).width(Length::Fill),
                            ]
                            .spacing(DialogDesign::SPACE_SMALL),
                            Space::with_height(DialogDesign::space_tiny()),
                            row![
                                text("Size:").size(body_size).width(Length::Fixed(label_w))
                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.size).size(body_size).width(Length::Fill),
                            ]
                            .spacing(DialogDesign::SPACE_SMALL),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(0),
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
                        .size(body_size * 0.95),
                    Space::with_height(DialogDesign::space_medium()),
                    text("Description")
                        .size(body_size * 1.05)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(DialogDesign::space_tiny()),
                    scrollable(
                        text(&info.description)
                            .size(body_size * 0.95)
                    )
                    .height(Length::Fixed(120.0)),
                ]
                .spacing(0)
                .padding(DialogDesign::pad_medium())
            )
            .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)));

            let deps_section = if !info.dependencies.is_empty() {
                let material_font = crate::gui::fonts::get_material_symbols_font();
                container(
                    column![
                        row![
                            text(crate::gui::fonts::glyphs::SETTINGS_SYMBOL)
                                .font(material_font)
                                .size(body_size * 1.1)
                                .style(iced::theme::Text::Color(theme.primary())),
                            text(format!(" Dependencies ({})", info.dependencies.len()))
                                .size(body_size * 1.05)
                                .style(iced::theme::Text::Color(theme.primary()))
                        ]
                        .spacing(DialogDesign::SPACE_TINY)
                        .align_items(Alignment::Center),
                        Space::with_height(DialogDesign::space_small()),
                        scrollable(
                            column(
                                info.dependencies
                                    .iter()
                                    .map(|dep| {
                                        container(
                                            text(dep).size(body_size * 0.9)
                                        )
                                        .padding(DialogDesign::pad_small())
                                        .style(iced::theme::Container::Custom(Box::new(DependencyItemStyle)))
                                        .width(Length::Fill)
                                        .into()
                                    })
                                    .collect::<Vec<_>>()
                            )
                            .spacing(DialogDesign::SPACE_TINY)
                        )
                        .height(Length::Fixed(130.0)),
                    ]
                    .spacing(0)
                    .padding(DialogDesign::pad_medium())
                )
                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
            } else {
                container(Space::with_height(Length::Shrink))
            };

            let progress_section = if self.is_installing || self.is_complete {
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
                                .on_press(Message::InstallRpm)
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
                            info_section,
                            deps_section,
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
                    text("Failed to load RPM information")
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

struct DependencyItemStyle;

impl iced::widget::container::StyleSheet for DependencyItemStyle {
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
