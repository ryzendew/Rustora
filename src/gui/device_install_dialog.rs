use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone)]
pub enum Message {
    StartInstallation,
    InstallationProgress(String),
    #[allow(dead_code)]
    InstallationComplete,
    InstallationError(String),
    PostInstallProgress(String),
    PostInstallComplete,
    PostInstallError(String),
    Close,
}

#[derive(Debug)]
pub struct DeviceInstallDialog {
    profile_name: String,
    install_script: String,
    device_info: DeviceInfo,
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    is_post_install: bool,
    post_install_complete: bool,
    progress_text: String,
    terminal_output: String,
    post_install_output: String,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_name: String,
    pub device_name: String,
    pub driver: String,
    pub driver_version: String,
    pub bus_id: String,
    pub vendor_id: String,
    pub device_id: String,
    pub repositories: Vec<String>, // Repositories that will be enabled/used
}

impl DeviceInstallDialog {
    pub fn new(profile_name: String, install_script: String, device_info: DeviceInfo) -> Self {
        Self {
            profile_name: profile_name.clone(),
            install_script,
            device_info,
            is_running: true,
            is_complete: false,
            has_error: false,
            is_post_install: false,
            post_install_complete: false,
            progress_text: format!("Installing driver: {}...", profile_name),
            terminal_output: String::new(),
            post_install_output: String::new(),
        }
    }

    pub fn run_separate_window(profile_name: String, install_script: String, device_info: DeviceInfo) -> Result<(), iced::Error> {
        let dialog = Self::new(profile_name, install_script, device_info);
        
        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(1000.0, 700.0);
        window_settings.min_size = Some(iced::Size::new(800.0, 500.0));
        window_settings.resizable = true;
        window_settings.decorations = true;
        
        let default_font = crate::gui::fonts::get_inter_font();

        <DeviceInstallDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }
}

impl Application for DeviceInstallDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        // Start installation immediately
        let cmd = dialog.update(Message::StartInstallation);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        format!("Installing Driver: {} - FedoraForge", self.profile_name)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartInstallation => {
                self.is_running = true;
                self.terminal_output.clear();
                let script = self.install_script.clone();
                
                iced::Command::perform(
                    execute_install_script(script),
                    |result| match result {
                        Ok(output) => Message::InstallationProgress(output),
                        Err(e) => Message::InstallationError(e),
                    },
                )
            }
            Message::InstallationProgress(output) => {
                // Set the complete output (installation finished successfully)
                self.terminal_output = output;
                self.is_running = false;
                self.is_complete = true;
                
                // Check if this is an NVIDIA driver and run post-install commands
                let is_nvidia = self.device_info.vendor_id == "10de" || 
                                self.device_info.driver.to_lowercase().contains("nvidia");
                
                if is_nvidia {
                    // Start post-installation steps
                    self.is_post_install = true;
                    self.progress_text = "Rebuilding kernel modules...".to_string();
                    iced::Command::perform(
                        run_nvidia_post_install(),
                        |result| match result {
                            Ok(output) => Message::PostInstallProgress(output),
                            Err(e) => Message::PostInstallError(e),
                        },
                    )
                } else {
                    Command::none()
                }
            }
            Message::InstallationComplete => {
                self.is_running = false;
                self.is_complete = true;
                Command::none()
            }
            Message::InstallationError(error) => {
                self.is_running = false;
                self.has_error = true;
                if !self.terminal_output.is_empty() {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&format!("Error: {}", error));
                Command::none()
            }
            Message::PostInstallProgress(output) => {
                self.post_install_output = output;
                self.progress_text = "Regenerating initramfs...".to_string();
                // After akmods, run dracut
                let current_output = self.post_install_output.clone();
                iced::Command::perform(
                    run_dracut_regenerate(),
                    move |result| match result {
                        Ok(dracut_output) => {
                            let mut combined = current_output;
                            if !combined.is_empty() {
                                combined.push('\n');
                            }
                            combined.push_str(&dracut_output);
                            Message::PostInstallComplete
                        }
                        Err(e) => Message::PostInstallError(e),
                    },
                )
            }
            Message::PostInstallComplete => {
                self.is_post_install = false;
                self.post_install_complete = true;
                self.progress_text = "All post-installation steps completed successfully!".to_string();
                Command::none()
            }
            Message::PostInstallError(error) => {
                self.is_post_install = false;
                self.has_error = true;
                if !self.post_install_output.is_empty() {
                    self.post_install_output.push('\n');
                }
                self.post_install_output.push_str(&format!("Error: {}", error));
                Command::none()
            }
            Message::Close => {
                iced::window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = crate::gui::Theme::Dark;
        self.view_impl(&theme)
    }

    fn theme(&self) -> IcedTheme {
        crate::gui::Theme::Dark.iced_theme()
    }
}

impl DeviceInstallDialog {
    fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {

        // Device information section
        let device_info_section = container(
            column![
                text("Device Information")
                    .size(18)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                row![
                    text("Vendor:").size(13).width(Length::Fixed(120.0))
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&self.device_info.vendor_name).size(13),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(6.0)),
                row![
                    text("Device:").size(13).width(Length::Fixed(120.0))
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&self.device_info.device_name).size(13),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(6.0)),
                row![
                    text("Driver to Install:").size(13).width(Length::Fixed(120.0))
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&self.device_info.driver).size(13)
                        .style(iced::theme::Text::Color(theme.primary())),
                ]
                .spacing(10)
                .width(Length::Fill),
                if !self.device_info.driver_version.is_empty() {
                    row![
                        text("Driver Version:").size(13).width(Length::Fixed(120.0))
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        text(&self.device_info.driver_version).size(13)
                            .style(iced::theme::Text::Color(theme.primary())),
                    ]
                    .spacing(10)
                    .width(Length::Fill)
                } else {
                    row![Space::with_width(Length::Shrink)]
                },
                Space::with_height(Length::Fixed(6.0)),
                row![
                    text("Bus ID:").size(13).width(Length::Fixed(120.0))
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&self.device_info.bus_id).size(13),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(6.0)),
                row![
                    text("Vendor ID:").size(13).width(Length::Fixed(120.0))
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&self.device_info.vendor_id).size(13),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(6.0)),
                row![
                    text("Device ID:").size(13).width(Length::Fixed(120.0))
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&self.device_info.device_id).size(13),
                ]
                .spacing(10)
                .width(Length::Fill),
                if !self.device_info.repositories.is_empty() {
                    column![
                        Space::with_height(Length::Fixed(6.0)),
                        row![
                            text("Repositories:").size(13).width(Length::Fixed(120.0))
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            column(
                                self.device_info.repositories.iter().map(|repo| {
                                    text(repo).size(12)
                                        .style(iced::theme::Text::Color(theme.primary()))
                                        .into()
                                }).collect::<Vec<_>>()
                            )
                            .spacing(2),
                        ]
                        .spacing(10)
                        .width(Length::Fill)
                        .align_items(Alignment::Start),
                    ]
                } else {
                    column![Space::with_width(Length::Shrink)]
                },
            ]
            .spacing(4)
            .padding(Padding::new(16.0))
        )
        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
        .width(Length::Fill);

        let content = if self.is_running || self.is_post_install {
            // Show running state with terminal output
            let title_text = if self.is_post_install {
                "Post-Installation Steps"
            } else {
                "Installing Driver"
            };
            
            let output_text = if self.is_post_install {
                "Post-Installation Output:"
            } else {
                "Installation Output:"
            };
            
            let combined_output = if self.is_post_install {
                let mut combined = self.terminal_output.clone();
                if !combined.is_empty() && !self.post_install_output.is_empty() {
                    combined.push_str("\n\n=== Post-Installation Steps ===\n");
                }
                combined.push_str(&self.post_install_output);
                combined
            } else {
                self.terminal_output.clone()
            };
            
            let terminal_scroll: Element<Message> = scrollable(
                container(
                    text(&combined_output)
                        .font(iced::Font::MONOSPACE)
                        .size(12)
                        .shaping(iced::widget::text::Shaping::Advanced)
                )
                .padding(Padding::new(16.0))
                .width(Length::Fill)
            )
            .height(Length::Fill)
            .into();
            
            container(
                column![
                    text(title_text)
                        .size(22)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(12.0)),
                    text(&self.profile_name)
                        .size(16)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(16.0)),
                    device_info_section,
                    Space::with_height(Length::Fixed(16.0)),
                    progress_bar(0.0..=1.0, 0.5)
                        .width(Length::Fill),
                    Space::with_height(Length::Fixed(8.0)),
                    text(&self.progress_text)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(16.0)),
                    text(output_text)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(8.0)),
                    terminal_scroll,
                ]
                .spacing(8)
                .padding(Padding::new(20.0))
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if self.has_error {
            // Show error state
            container(
                column![
                    text("Installation Failed")
                        .size(22)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_height(Length::Fixed(12.0)),
                    text(&self.profile_name)
                        .size(16)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(16.0)),
                    device_info_section,
                    Space::with_height(Length::Fixed(16.0)),
                    text("Error Output:")
                        .size(14)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_height(Length::Fixed(8.0)),
                    scrollable(
                        container(
                            text(&self.terminal_output)
                                .font(iced::Font::MONOSPACE)
                                .size(12)
                                .shaping(iced::widget::text::Shaping::Advanced)
                        )
                        .padding(Padding::new(16.0))
                        .width(Length::Fill)
                    )
                    .height(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    button(
                        row![
                            text("Close").size(14),
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Close)
                    .style(iced::theme::Button::Custom(Box::new(DialogButtonStyle)))
                    .padding(Padding::new(12.0)),
                ]
                .spacing(8)
                .padding(Padding::new(20.0))
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else {
            // Show success state
            let success_title = if self.post_install_complete {
                "Installation and Post-Installation Complete"
            } else {
                "Installation Complete"
            };
            
            let combined_output = if self.post_install_complete {
                let mut combined = self.terminal_output.clone();
                if !combined.is_empty() && !self.post_install_output.is_empty() {
                    combined.push_str("\n\n=== Post-Installation Steps ===\n");
                }
                combined.push_str(&self.post_install_output);
                combined
            } else {
                self.terminal_output.clone()
            };
            
            container(
                column![
                    text(success_title)
                        .size(22)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(12.0)),
                    text(&self.profile_name)
                        .size(16)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(16.0)),
                    device_info_section,
                    Space::with_height(Length::Fixed(16.0)),
                    if self.post_install_complete {
                        text("All post-installation steps completed successfully!")
                            .size(14)
                            .style(iced::theme::Text::Color(theme.primary()))
                    } else {
                        text("Installation completed successfully.")
                            .size(14)
                            .style(iced::theme::Text::Color(theme.primary()))
                    },
                    Space::with_height(Length::Fixed(16.0)),
                    text("Installation Output:")
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(8.0)),
                    scrollable(
                        container(
                            text(&combined_output)
                                .font(iced::Font::MONOSPACE)
                                .size(12)
                                .shaping(iced::widget::text::Shaping::Advanced)
                        )
                        .padding(Padding::new(16.0))
                        .width(Length::Fill)
                    )
                    .height(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    button(
                        row![
                            text("Close").size(14),
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Close)
                    .style(iced::theme::Button::Custom(Box::new(DialogButtonStyle)))
                    .padding(Padding::new(12.0)),
                ]
                .spacing(8)
                .padding(Padding::new(20.0))
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(0.0))
            .into()
    }
}

// Execute install script with streaming output
async fn execute_install_script(script: String) -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    // Write script to temporary file
    use std::io::Write;
    let mut temp_file = std::env::temp_dir();
    temp_file.push(format!("fedoraforge_install_{}.sh", std::process::id()));
    
    {
        let mut file = std::fs::File::create(&temp_file)
            .map_err(|e| format!("Failed to create temporary script file: {}", e))?;
        file.write_all(script.as_bytes())
            .map_err(|e| format!("Failed to write script: {}", e))?;
        file.write_all(b"\n")
            .map_err(|e| format!("Failed to write script: {}", e))?;
    }
    
    // Make script executable
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&temp_file)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&temp_file, perms)
        .map_err(|e| format!("Failed to set script permissions: {}", e))?;
    
    // Use pkexec to run the script with elevated privileges
    let script_path = temp_file.to_string_lossy().to_string();
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("bash");
    cmd.arg(&script_path);
    
    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    // Set up process with stdout and stderr captured
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|e| {
            let _ = std::fs::remove_file(&temp_file);
            format!("Failed to start installation: {}", e)
        })?;
    
    let mut output = String::new();
    
    // Read stdout and stderr concurrently
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    
    // Read stdout
    if let Some(stdout) = stdout {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if !line.is_empty() {
                        output.push_str(&line);
                        line.clear();
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    // Read stderr
    if let Some(stderr) = stderr {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if !line.is_empty() {
                        output.push_str(&format!("[stderr] {}", line));
                        line.clear();
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    // Wait for process to complete
    let status = child.wait().await
        .map_err(|e| {
            let _ = std::fs::remove_file(&temp_file);
            format!("Failed to wait for process: {}", e)
        })?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);
    
    if status.success() {
        if output.is_empty() {
            output = "Installation completed successfully.".to_string();
        }
        Ok(output)
    } else {
        let exit_code = status.code().unwrap_or(-1);
        Err(format!("Installation failed with exit code {}. Output:\n{}", exit_code, output))
    }
}

// Run akmods --force --rebuild for NVIDIA drivers
async fn run_nvidia_post_install() -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("akmods");
    cmd.arg("--force");
    cmd.arg("--rebuild");
    
    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start akmods: {}", e))?;
    
    let mut output = String::new();
    
    // Read stdout and stderr concurrently
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    
    // Read stdout
    if let Some(stdout) = stdout {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if !line.is_empty() {
                        output.push_str(&line);
                        line.clear();
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    // Read stderr
    if let Some(stderr) = stderr {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if !line.is_empty() {
                        output.push_str(&format!("[stderr] {}", line));
                        line.clear();
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    // Wait for process to complete
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for akmods: {}", e))?;
    
    if status.success() {
        if output.is_empty() {
            output = "akmods --force --rebuild completed successfully.".to_string();
        }
        Ok(output)
    } else {
        let exit_code = status.code().unwrap_or(-1);
        Err(format!("akmods failed with exit code {}. Output:\n{}", exit_code, output))
    }
}

// Run dracut -f --regenerate-all for NVIDIA drivers
async fn run_dracut_regenerate() -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dracut");
    cmd.arg("-f");
    cmd.arg("--regenerate-all");
    
    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start dracut: {}", e))?;
    
    let mut output = String::new();
    
    // Read stdout and stderr concurrently
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    
    // Read stdout
    if let Some(stdout) = stdout {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if !line.is_empty() {
                        output.push_str(&line);
                        line.clear();
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    // Read stderr
    if let Some(stderr) = stderr {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if !line.is_empty() {
                        output.push_str(&format!("[stderr] {}", line));
                        line.clear();
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    // Wait for process to complete
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for dracut: {}", e))?;
    
    if status.success() {
        if output.is_empty() {
            output = "dracut -f --regenerate-all completed successfully.".to_string();
        }
        Ok(output)
    } else {
        let exit_code = status.code().unwrap_or(-1);
        Err(format!("dracut failed with exit code {}. Output:\n{}", exit_code, output))
    }
}

// Info container style (for device information)
struct InfoContainerStyle;

impl iced::widget::container::StyleSheet for InfoContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

// Dialog container style
struct DialogContainerStyle;

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.1))),
            border: Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

// Dialog button style
struct DialogButtonStyle;

impl ButtonStyleSheet for DialogButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.5, 0.9))),
            border: Border {
                color: iced::Color::from_rgb(0.3, 0.6, 1.0),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: iced::Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Background::Color(iced::Color::from_rgb(0.25, 0.55, 0.95)));
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.45, 0.85)));
        appearance
    }
}

