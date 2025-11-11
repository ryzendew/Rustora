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
    #[allow(dead_code)]
    pub remote: Option<String>,
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
        window_settings.size = iced::Size::new(768.0, 612.0);
        window_settings.min_size = Some(iced::Size::new(640.0, 480.0));
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

        let content = if self.is_loading {
            container(
                column![
                    text("Loading Flatpak information...").size(18),
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
        } else if let Some(ref info) = self.flatpak_info {
            let material_font = crate::gui::fonts::get_material_symbols_font();
            let title = container(
                row![
                    column![
                        text(format!("Install {}", info.name))
                            .size(18)
                            .style(iced::theme::Text::Color(theme.primary())),
                        text(&info.application_id)
                            .size(12)
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
            .padding(Padding::new(16.0));

            let info_section = container(
                column![
                    text("Package Information").size(14).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(10.0)),
                    row![
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
                                text("Branch:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.branch).size(11).width(Length::Fill),
                            ]
                            .spacing(8),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(1)),
                        Space::with_width(Length::Fixed(12.0)),
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
                            Space::with_height(Length::Fixed(6.0)),
                            row![
                                text("Runtime:").size(11).width(Length::Fixed(85.0)).style(iced::theme::Text::Color(theme.primary())),
                                text(&info.runtime).size(11).width(Length::Fill),
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
                        // Embedded terminal output
                        {
                            if !self.terminal_output.is_empty() {
                                column![
                                    Space::with_height(Length::Fixed(10.0)),
                                    text("Terminal Output").size(13).style(iced::theme::Text::Color(theme.primary())),
                                    Space::with_height(Length::Fixed(6.0)),
                                    container(
                                        scrollable(
                                            text(&self.terminal_output)
                                                .size(11)
                                                .font(iced::Font::MONOSPACE)
                                                .width(Length::Fill)
                                        )
                                        .height(Length::Fixed(200.0))
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle)))
                                    .width(Length::Fill)
                                    .padding(Padding::new(10.0))
                                ]
                                .spacing(0)
                            } else {
                                column![].spacing(0)
                            }
                        },
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
                            .on_press(Message::InstallFlatpak)
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
            format!("Install {} - FedoraForge", info.name)
        } else {
            "Install Flatpak - FedoraForge".to_string()
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
                // Append new output to terminal
                if !self.terminal_output.is_empty() {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);
                
                // Update progress text
                self.installation_progress = output.clone();
                
                // Check if installation is complete
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
                    self.terminal_output.push_str("\n✓ Installation completed successfully!");
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

    // Try remote-info first if remote is provided
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

    // Fallback to info if remote-info didn't work
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
        remote: remote.clone(),
        dependencies,
    })
}

async fn write_flatpak_log(operation: &str, app_id: &str, remote: Option<&String>, output: &str, success: bool) {
    if let Ok(home) = std::env::var("HOME") {
        let log_dir = PathBuf::from(&home).join(".fedoraforge");
        if let Err(e) = fs::create_dir_all(&log_dir).await {
            eprintln!("Failed to create log directory: {}", e);
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
        
        if let Err(e) = fs::write(&log_file, log_content).await {
            eprintln!("Failed to write log file: {}", e);
        }
    } else {
        eprintln!("HOME environment variable not set, cannot write log");
    }
}

async fn install_flatpak_streaming(app_id: String, remote: Option<String>) -> Result<String, String> {
    let mut cmd = TokioCommand::new("flatpak");
    // Use verbose mode to get more output for debugging
    // --assumeyes (-y) and --noninteractive for automated installation
    cmd.args(["install", "-y", "--noninteractive", "--verbose"]);
    
    // If remote is provided, add it before the app_id
    // Format: flatpak install [OPTIONS] [REMOTE] [REF...]
    if let Some(ref remote_name) = remote {
        if !remote_name.is_empty() {
            cmd.arg(remote_name);
        }
    }
    
    // Add the application ID (full ref like org.app.id or just the ID)
    cmd.arg(&app_id);
    
    // Log the command being executed
    let command_str = format!("flatpak install -y --noninteractive --verbose {} {}", 
        remote.as_ref().map(|r| r.as_str()).unwrap_or(""), 
        &app_id);
    
    // Use spawn to get streaming output
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute flatpak install: {}", e))?;

    // Read output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str(&format!("Command: {}\n", command_str));
    combined_output.push_str("--- Output ---\n");
    
    // Read both stdout and stderr
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
    
    // Write log file
    write_flatpak_log("install", &app_id, remote.as_ref(), &combined_output, success).await;

    if !success {
        // Check if it's a known "no updates" or "already installed" case
        let output_lower = combined_output.to_lowercase();
        if output_lower.contains("already installed") || 
           output_lower.contains("is already installed") ||
           output_lower.contains("nothing to do") {
            // This is actually a success case
            return Ok(format!("Application is already installed.\n\n{}", combined_output));
        }
        
        // For other failures, return error with full output
        return Err(format!("Installation failed (exit code: {}):\n{}", exit_code, combined_output));
    }

    // Success - return the output or a success message
    if combined_output.trim().is_empty() || combined_output.trim() == format!("Command: {}\n--- Output ---\n", command_str).trim() {
        Ok("Installation Complete!".to_string())
    } else {
        // Check if output indicates success
        let output_lower = combined_output.to_lowercase();
        if output_lower.contains("complete") || 
           output_lower.contains("installed") ||
           output_lower.contains("success") {
            Ok(combined_output)
        } else {
            // Even if exit code is 0, check output for success indicators
            Ok(format!("Installation completed.\n\n{}", combined_output))
        }
    }
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

struct TerminalContainerStyle;

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        // Dark terminal-like background
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.1))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            },
            ..Default::default()
        }
    }
}

