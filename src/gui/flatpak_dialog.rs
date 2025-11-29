use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Border, Theme as IcedTheme};
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
    FlatpakInfoLoaded(FlatpakInfo),
    InstallFlatpak,
    InstallationProgress(String),
    InstallationComplete,
    InstallationError(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct FlatpakInfo {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub branch: String,
    pub arch: String,
    pub summary: String,
    pub description: String,
    pub size: String,
    pub runtime: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub struct FlatpakDialog {
    pub application_id: String,
    pub remote: Option<String>,
    pub flatpak_info: Option<FlatpakInfo>,
    pub is_loading: bool,
    pub is_installing: bool,
    pub is_complete: bool,
    pub installation_progress: String,
    pub terminal_output: String,
    pub show_dialog: bool,
}

impl FlatpakDialog {
    pub fn new(application_id: String, remote: Option<String>) -> Self {
        Self {
            application_id,
            remote,
            flatpak_info: None,
            is_loading: true,
            is_installing: false,
            is_complete: false,
            installation_progress: String::new(),
            terminal_output: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window(application_id: String, remote: Option<String>) -> Result<(), iced::Error> {
        let dialog = Self::new(application_id, remote);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(600.0, 550.0);
        window_settings.min_size = Some(iced::Size::new(480.0, 400.0));
        window_settings.max_size = None;
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <FlatpakDialog as Application>::run(iced::Settings {
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
        } else if let Some(ref info) = self.flatpak_info {
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
                        text(&info.application_id)
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

            let label_w = 85.0;
            let info_section = container(
                column![
                    text("Package Details")
                        .size(body_size * 1.1)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(DialogDesign::space_small()),
                    row![
                        column![
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
                                text("Branch:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.branch).size(body_size).width(Length::Fill),
                            ]
                            .spacing(DialogDesign::SPACE_SMALL),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                        Space::with_width(DialogDesign::space_medium()),
                        column![
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
                            Space::with_height(DialogDesign::space_tiny()),
                            row![
                                text("Runtime:").size(body_size).width(Length::Fixed(label_w)).style(iced::theme::Text::Color(theme.secondary_text())),
                                text(&info.runtime).size(body_size).width(Length::Fill),
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
                        {
                            if !self.terminal_output.is_empty() {
                                column![
                                    Space::with_height(DialogDesign::space_medium()),
                                    text("Terminal Output")
                                        .size(body_size * 1.0)
                                        .style(iced::theme::Text::Color(theme.primary())),
                                    Space::with_height(DialogDesign::space_small()),
                                    container(
                                        scrollable(
                                            text(&self.terminal_output)
                                                .size(body_size * 0.85)
                                                .font(iced::Font::MONOSPACE)
                                                .width(Length::Fill)
                                        )
                                        .height(Length::Fixed(180.0))
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle)))
                                    .width(Length::Fill)
                                    .padding(DialogDesign::pad_small())
                                ]
                                .spacing(0)
                            } else {
                                column![].spacing(0)
                            }
                        },
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
                                .on_press(Message::InstallFlatpak)
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

impl Application for FlatpakDialog {
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
        if let Some(ref info) = self.flatpak_info {
            format!("Install {} - Rustora", info.name)
        } else {
            "Install Flatpak - Rustora".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadFlatpakInfo => {
                self.is_loading = true;
                let app_id = self.application_id.clone();
                let remote = self.remote.clone();
                iced::Command::perform(load_flatpak_info(app_id, remote), |result| {
                    match result {
                        Ok(info) => Message::FlatpakInfoLoaded(info),
                        Err(e) => Message::InstallationError(e.to_string()),
                    }
                })
            }
            Message::FlatpakInfoLoaded(info) => {
                self.is_loading = false;
                self.flatpak_info = Some(info);
                iced::Command::none()
            }
            Message::InstallFlatpak => {
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                self.terminal_output = String::new();
                let app_id = self.application_id.clone();
                let remote = self.remote.clone();
                iced::Command::perform(install_flatpak_streaming(app_id, remote), |result| {
                    match result {
                        Ok(output) => Message::InstallationProgress(output),
                        Err(e) => Message::InstallationError(e.to_string()),
                    }
                })
            }
            Message::InstallationProgress(output) => {
                if !self.terminal_output.is_empty() {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);

                self.installation_progress = output.clone();

                if output.contains("Complete") ||
                   output.contains("Installed") ||
                   output.contains("complete") ||
                   output.to_lowercase().contains("success") ||
                   output.contains("already installed") {
                    iced::Command::perform(async {}, |_| Message::InstallationComplete)
                } else {
                    iced::Command::none()
                }
            }
            Message::InstallationComplete => {
                self.is_installing = false;
                self.is_complete = true;
                self.installation_progress = "Installation completed successfully!".to_string();
                if !self.terminal_output.contains("completed successfully") {
                    self.terminal_output.push_str("\n[OK] Installation completed successfully!");
                }
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

async fn load_flatpak_info(app_id: String, remote: Option<String>) -> Result<FlatpakInfo, String> {
    let mut name = app_id.clone();
    let mut version = String::new();
    let mut branch = String::new();
    let mut arch = String::new();
    let mut summary = String::new();
    let mut description = String::new();
    let mut size = String::new();
    let mut runtime = String::new();
    let dependencies = Vec::new();

    if let Some(ref remote_name) = remote {
        let output = TokioCommand::new("flatpak")
            .args(["remote-info", remote_name, &app_id])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.starts_with("Name:") {
                        name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Version:") {
                        version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Branch:") {
                        branch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Arch:") {
                        arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Summary:") {
                        summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Description:") {
                        description = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Download size:") {
                        size = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Runtime:") {
                        runtime = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    }
                }
            }
        }
    }

    if version.is_empty() {
        let output = TokioCommand::new("flatpak")
            .args(["info", &app_id])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.starts_with("Name:") && name == app_id {
                        name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Version:") && version.is_empty() {
                        version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Branch:") && branch.is_empty() {
                        branch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Arch:") && arch.is_empty() {
                        arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Summary:") && summary.is_empty() {
                        summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Description:") && description.is_empty() {
                        description = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Installed size:") && size.is_empty() {
                        size = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Runtime:") && runtime.is_empty() {
                        runtime = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    }
                }
            }
        }
    }

    if name.is_empty() {
        name = app_id.clone();
    }

    Ok(FlatpakInfo {
        name,
        application_id: app_id,
        version: if version.is_empty() { "N/A".to_string() } else { version },
        branch: if branch.is_empty() { "stable".to_string() } else { branch },
        arch: if arch.is_empty() { "x86_64".to_string() } else { arch },
        summary: if summary.is_empty() { "No summary available".to_string() } else { summary },
        description: if description.is_empty() { "No description available".to_string() } else { description },
        size: if size.is_empty() { "Unknown".to_string() } else { size },
        runtime: if runtime.is_empty() { "N/A".to_string() } else { runtime },
        dependencies,
    })
}

async fn write_flatpak_log(operation: &str, app_id: &str, remote: Option<&String>, output: &str, success: bool) {
    if let Ok(home) = std::env::var("HOME") {
        let log_dir = PathBuf::from(&home).join(".rustora");
        if let Err(_e) = fs::create_dir_all(&log_dir).await {
            return;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_file = log_dir.join(format!("flatpak_{}_{}.log", operation, timestamp));

        let mut log_content = String::new();
        log_content.push_str(&format!("=== Flatpak {} Log ===\n", operation));
        log_content.push_str(&format!("Timestamp: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        log_content.push_str(&format!("Application ID: {}\n", app_id));
        if let Some(remote_name) = remote {
            log_content.push_str(&format!("Remote: {}\n", remote_name));
        }
        log_content.push_str(&format!("Status: {}\n", if success { "SUCCESS" } else { "FAILED" }));
        log_content.push_str("\n--- Command Output ---\n");
        log_content.push_str(output);
        log_content.push_str("\n--- End of Log ---\n");

        if let Err(_e) = fs::write(&log_file, log_content).await {
        }
    } else {
    }
}

async fn install_flatpak_streaming(app_id: String, remote: Option<String>) -> Result<String, String> {
    let mut cmd = TokioCommand::new("flatpak");
    cmd.args(["install", "-y", "--noninteractive", "--verbose"]);

    if let Some(ref remote_name) = remote {
        if !remote_name.is_empty() {
            cmd.arg(remote_name);
        }
    }

    cmd.arg(&app_id);

    let command_str = format!("flatpak install -y --noninteractive --verbose {} {}",
        remote.as_ref().map(|r| r.as_str()).unwrap_or(""),
        &app_id);

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute flatpak install: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut combined_output = String::new();
    combined_output.push_str(&format!("Command: {}\n", command_str));
    combined_output.push_str("--- Output ---\n");

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stdout: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stderr: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
        }
    }

    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    let success = status.success();
    let exit_code = status.code().unwrap_or(-1);

    write_flatpak_log("install", &app_id, remote.as_ref(), &combined_output, success).await;

    if !success {
        let output_lower = combined_output.to_lowercase();
        if output_lower.contains("already installed") ||
           output_lower.contains("is already installed") ||
           output_lower.contains("nothing to do") {
            // This is actually a success case
            return Ok(format!("Application is already installed.\n\n{}", combined_output));
        }

        return Err(format!("Installation failed (exit code: {}):\n{}", exit_code, combined_output));
    }

    if combined_output.trim().is_empty() || combined_output.trim() == format!("Command: {}\n--- Output ---\n", command_str).trim() {
        Ok("Installation Complete!".to_string())
    } else {
        let output_lower = combined_output.to_lowercase();
        if output_lower.contains("complete") ||
           output_lower.contains("installed") ||
           output_lower.contains("success") {
            Ok(combined_output)
        } else {
            Ok(format!("Installation completed.\n\n{}", combined_output))
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

struct TerminalContainerStyle;

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            },
            ..Default::default()
        }
    }
}
