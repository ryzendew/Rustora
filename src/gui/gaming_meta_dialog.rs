use iced::widget::{button, column, container, row, scrollable, text, Space, progress_bar};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use crate::gui::app::CustomScrollableStyle;
use crate::gui::settings::AppSettings;

#[derive(Debug, Clone)]
pub enum Message {
    StartInstallation,
    InstallationProgress(String, f32), // Output and progress (0.0 to 1.0)
    InstallationComplete(Result<(), String>),
    Close,
}

#[derive(Debug)]
pub struct GamingMetaDialog {
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
    current_step: String,
    progress: f32, // 0.0 to 1.0
}

impl GamingMetaDialog {
    pub fn new() -> Self {
        Self {
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: "Installing Gaming Meta...".to_string(),
            terminal_output: String::new(),
            current_step: String::new(),
            progress: 0.0,
        }
    }

    pub fn run_separate_window() -> Result<(), iced::Error> {
        let dialog = Self::new();
        
        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(1000.0, 700.0);
        window_settings.min_size = Some(iced::Size::new(800.0, 500.0));
        window_settings.resizable = true;
        window_settings.decorations = true;
        
        let default_font = crate::gui::fonts::get_inter_font();

        <GamingMetaDialog as Application>::run(iced::Settings {
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

impl Application for GamingMetaDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::StartInstallation);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        "Gaming Meta Installation - Rustora".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartInstallation => {
                self.is_running = true;
                self.is_complete = false;
                self.has_error = false;
                self.progress = 0.0;
                self.terminal_output.clear();
                self.terminal_output.push_str("Starting Gaming Meta installation...\n");
                self.terminal_output.push_str("=====================================\n\n");
                
                Command::perform(install_gaming_meta_streaming(), |result| {
                    match result {
                        Ok((output, progress)) => Message::InstallationProgress(output, progress),
                        Err(e) => Message::InstallationComplete(Err(e)),
                    }
                })
            }
            Message::InstallationProgress(output, progress) => {
                // Update terminal output and progress
                self.terminal_output = output.clone();
                self.progress = progress;
                
                // Calculate progress based on which step we're on
                let step_progress = if output.contains("Step 1 completed") {
                    self.progress = 1.0 / 7.0;
                    "Step 1/7: Installing core gaming tools..."
                } else if output.contains("Step 2 completed") {
                    self.progress = 2.0 / 7.0;
                    "Step 2/7: Checking Flatpak availability..."
                } else if output.contains("Step 3 completed") {
                    self.progress = 3.0 / 7.0;
                    "Step 3/7: Installing MangoJuice..."
                } else if output.contains("Step 4 completed") {
                    self.progress = 4.0 / 7.0;
                    "Step 4/7: Installing ProtonPlus..."
                } else if output.contains("Step 5 completed") {
                    self.progress = 5.0 / 7.0;
                    "Step 5/7: Fetching Heroic release info..."
                } else if output.contains("Step 6 completed") {
                    self.progress = 6.0 / 7.0;
                    "Step 6/7: Downloading Heroic Games Launcher..."
                } else if output.contains("Step 7 completed") {
                    self.progress = 7.0 / 7.0;
                    "Step 7/7: Installing Heroic Games Launcher..."
                } else if output.contains("Step 1:") && !output.contains("Step 1 completed") {
                    self.progress = 0.1 / 7.0;
                    "Step 1/7: Installing core gaming tools..."
                } else if output.contains("Step 2:") && !output.contains("Step 2 completed") {
                    self.progress = 1.1 / 7.0;
                    "Step 2/7: Checking Flatpak availability..."
                } else if output.contains("Step 3:") && !output.contains("Step 3 completed") {
                    self.progress = 2.1 / 7.0;
                    "Step 3/7: Installing MangoJuice..."
                } else if output.contains("Step 4:") && !output.contains("Step 4 completed") {
                    self.progress = 3.1 / 7.0;
                    "Step 4/7: Installing ProtonPlus..."
                } else if output.contains("Step 5:") && !output.contains("Step 5 completed") {
                    self.progress = 4.1 / 7.0;
                    "Step 5/7: Fetching Heroic release info..."
                } else if output.contains("Step 6:") && !output.contains("Step 6 completed") {
                    self.progress = 5.1 / 7.0;
                    "Step 6/7: Downloading Heroic Games Launcher..."
                } else if output.contains("Step 7:") && !output.contains("Step 7 completed") {
                    self.progress = 6.1 / 7.0;
                    "Step 7/7: Installing Heroic Games Launcher..."
                } else {
                    // Use provided progress if available, otherwise keep current
                    if progress > 0.0 {
                        self.progress = progress;
                    }
                    "Installing Gaming Meta..."
                };
                
                self.progress_text = step_progress.to_string();
                
                // Check if installation is complete (look for success/error indicators)
                if output.contains("✓ ALL STEPS COMPLETED SUCCESSFULLY!") {
                    self.is_running = false;
                    self.is_complete = true;
                    self.progress = 1.0;
                    self.progress_text = "Installation completed successfully!".to_string();
                } else if output.contains("✗ Step") && output.contains("failed") {
                    // Check if it's a final failure (not just a step that might continue)
                    if output.matches("✗").count() > 1 || output.contains("return Err") {
                        self.is_running = false;
                        self.has_error = true;
                        self.progress_text = "Installation failed".to_string();
                    }
                }
                Command::none()
            }
            Message::InstallationComplete(result) => {
                self.is_running = false;
                match result {
                    Ok(_) => {
                        self.is_complete = true;
                        self.progress_text = "Installation completed successfully!".to_string();
                        if !self.terminal_output.contains("✓ All gaming tools installed successfully") {
                            self.terminal_output.push_str("\n✓ All gaming tools installed successfully!\n");
                        }
                    }
                    Err(e) => {
                        self.has_error = true;
                        self.progress_text = format!("Installation failed: {}", e);
                        self.terminal_output.push_str(&format!("\n✗ Error: {}\n", e));
                    }
                }
                Command::none()
            }
            Message::Close => {
                window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = crate::gui::Theme::Dark;
        let settings = AppSettings::load();
        self.view_impl(&theme, &settings)
    }

    fn theme(&self) -> IcedTheme {
        crate::gui::Theme::Dark.iced_theme()
    }
}

impl GamingMetaDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme, settings: &AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        let title_text = if self.is_complete {
            if self.has_error {
                "Installation Failed"
            } else {
                "Installation Complete"
            }
        } else {
            "Installing Gaming Meta"
        };
        
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let icon_font_size = (settings.font_size_icons * settings.scale_icons).round();
        
        let progress_display = if !self.current_step.is_empty() {
            text(&self.current_step).size(body_font_size)
        } else {
            text(&self.progress_text).size(body_font_size)
        };

        let terminal_output = scrollable(
            text(&self.terminal_output)
                .font(iced::Font::MONOSPACE)
                .size(body_font_size * 0.86)
        )
        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
            theme.background(),
            settings.border_radius,
        ))))
        .width(Length::Fill)
        .height(Length::Fill);

        let close_button: Element<Message> = button(
            row![
                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_font_size),
                text(" Close").size(body_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::Close)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: !self.has_error && self.is_complete,
            radius: settings.border_radius,
            theme: *theme,
        })))
        .padding(Padding::new(12.0))
        .into();

        container(
            column![
                // Header
                row![
                    text(title_text).size(title_font_size).style(iced::theme::Text::Color(
                        if self.has_error {
                            theme.danger()
                        } else if self.is_complete {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            theme.primary()
                        }
                    )),
                    Space::with_width(Length::Fill),
                    if self.is_complete || self.has_error {
                        close_button
                    } else {
                        Space::with_width(Length::Fixed(0.0)).into()
                    },
                ]
                .spacing(12)
                .align_items(Alignment::Center)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(16.0)),
                // Progress bar
                progress_bar(0.0..=1.0, self.progress)
                    .width(Length::Fill)
                    .height(Length::Fixed(8.0)),
                Space::with_height(Length::Fixed(8.0)),
                // Progress text
                progress_display.style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(16.0)),
                // Terminal output
                container(terminal_output)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(12)
                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    }))),
            ]
            .spacing(0)
            .padding(24)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle {
            theme: *theme,
        })))
        .into()
    }
}

async fn install_gaming_meta_streaming() -> Result<(String, f32), String> {
    let mut output = String::new();
    
    // Step 1: Install core gaming tools
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("Step 1: Installing core gaming tools\n");
    output.push_str("Packages: steam, lutris, mangohud, gamescope\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n\n");
    match install_core_gaming_tools().await {
        Ok(cmd_output) => {
            output.push_str(&cmd_output);
            output.push_str("\n✓ Step 1 completed: Core gaming tools installed successfully\n\n");
        }
        Err(e) => {
            output.push_str(&format!("✗ Step 1 failed: {}\n", e));
            return Err(e);
        }
    }
    
    // Step 2: Check Flatpak availability
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("Step 2: Checking Flatpak availability\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n\n");
    let flatpak_check = check_flatpak().await;
    match flatpak_check {
        Ok(msg) => {
            output.push_str(&msg);
            output.push_str("\n✓ Step 2 completed: Flatpak is available\n\n");
        }
        Err(e) => {
            output.push_str(&format!("✗ Step 2 failed: {}\n", e));
            return Err(e);
        }
    }
    
    // Step 3: Install MangoJuice
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("Step 3: Installing MangoJuice (io.github.radiolamp.mangojuice)\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n\n");
    match install_flatpak_package("io.github.radiolamp.mangojuice", "MangoJuice").await {
        Ok(cmd_output) => {
            output.push_str(&cmd_output);
            output.push_str("\n✓ Step 3 completed: MangoJuice installed successfully\n\n");
        }
        Err(e) => {
            output.push_str(&format!("✗ Step 3 failed: {}\n", e));
            return Err(e);
        }
    }
    
    // Step 4: Install ProtonPlus
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("Step 4: Installing ProtonPlus (com.vysp3r.ProtonPlus)\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n\n");
    match install_flatpak_package("com.vysp3r.ProtonPlus", "ProtonPlus").await {
        Ok(cmd_output) => {
            output.push_str(&cmd_output);
            output.push_str("\n✓ Step 4 completed: ProtonPlus installed successfully\n\n");
        }
        Err(e) => {
            output.push_str(&format!("✗ Step 4 failed: {}\n", e));
            return Err(e);
        }
    }
    
    // Step 5: Fetch Heroic Games Launcher release info
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("Step 5: Fetching Heroic Games Launcher release information\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n\n");
    let release_info = fetch_heroic_release_info().await;
    match release_info {
        Ok((download_url, filename, info_output)) => {
            output.push_str(&info_output);
            output.push_str("\n✓ Step 5 completed: Release information fetched\n\n");
            
            // Step 6: Download Heroic Games Launcher
            output.push_str("═══════════════════════════════════════════════════════════════\n");
            output.push_str("Step 6: Downloading Heroic Games Launcher\n");
            output.push_str(&format!("File: {}\n", filename));
            output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match download_heroic_rpm(&download_url, &filename).await {
                Ok((rpm_path, download_output)) => {
                    output.push_str(&download_output);
                    output.push_str("\n✓ Step 6 completed: Download completed\n\n");
                    
                    // Step 7: Install Heroic Games Launcher
                    output.push_str("═══════════════════════════════════════════════════════════════\n");
                    output.push_str("Step 7: Installing Heroic Games Launcher\n");
                    output.push_str(&format!("RPM: {}\n", filename));
                    output.push_str("═══════════════════════════════════════════════════════════════\n\n");
                    match install_heroic_rpm(&rpm_path).await {
                        Ok(install_output) => {
                            output.push_str(&install_output);
                            output.push_str("\n✓ Step 7 completed: Heroic Games Launcher installed successfully\n\n");
                        }
                        Err(e) => {
                            // Clean up downloaded file on error
                            let _ = std::fs::remove_file(&rpm_path);
                            output.push_str(&format!("✗ Step 7 failed: {}\n", e));
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    output.push_str(&format!("✗ Step 6 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        Err(e) => {
            output.push_str(&format!("✗ Step 5 failed: {}\n", e));
            return Err(e);
        }
    }
    
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("✓ ALL STEPS COMPLETED SUCCESSFULLY!\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("\nInstalled packages:\n");
    output.push_str("  • steam\n");
    output.push_str("  • lutris\n");
    output.push_str("  • mangohud\n");
    output.push_str("  • gamescope\n");
    output.push_str("  • io.github.radiolamp.mangojuice (Flatpak)\n");
    output.push_str("  • com.vysp3r.ProtonPlus (Flatpak)\n");
    output.push_str("  • Heroic Games Launcher\n");
    
    // Return output with 100% progress
    Ok((output, 1.0))
}

async fn install_core_gaming_tools() -> Result<String, String> {
    let packages = vec!["steam", "lutris", "mangohud", "gamescope"];
    
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.args(&packages);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute dnf install: {}", e))?;
    
    // Capture output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut output = String::new();
    output.push_str(&format!("$ pkexec dnf install -y {}\n", packages.join(" ")));
    output.push_str("─────────────────────────────────────────────────────────────\n");
    
    // Read both stdout and stderr
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(format!("Error reading stdout: {}", e));
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(format!("Error reading stderr: {}", e));
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for dnf install: {}", e))?;
    
    // Check for pkexec cancellation (exit codes 126/127)
    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or polkit not available. Please try again.".to_string());
    }
    
    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()));
    }
    
    Ok(output)
}

async fn check_flatpak() -> Result<String, String> {
    let mut cmd = TokioCommand::new("which");
    cmd.arg("flatpak");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let output = cmd.output().await
        .map_err(|e| format!("Failed to check flatpak: {}", e))?;
    
    let mut result = String::new();
    result.push_str("$ which flatpak\n");
    result.push_str("─────────────────────────────────────────────────────────────\n");
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        result.push_str(&stdout);
        result.push_str("\nFlatpak is installed and available.\n");
        Ok(result)
    } else {
        result.push_str("Flatpak not found in PATH.\n");
        Err("Flatpak is not installed. Please install it first.".to_string())
    }
}

async fn install_flatpak_package(package_id: &str, package_name: &str) -> Result<String, String> {
    let mut cmd = TokioCommand::new("flatpak");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.arg("flathub");
    cmd.arg(package_id);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute flatpak install: {}", e))?;
    
    // Capture output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut output = String::new();
    output.push_str(&format!("$ flatpak install -y flathub {}\n", package_id));
    output.push_str("─────────────────────────────────────────────────────────────\n");
    
    // Read both stdout and stderr
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(format!("Error reading stdout: {}", e));
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(format!("Error reading stderr: {}", e));
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for flatpak install: {}", e))?;
    
    if !status.success() {
        // Check if package is already installed (that's okay)
        if output.contains("already installed") || output.contains("is already installed") {
            output.push_str("\n(Note: Package was already installed)\n");
            return Ok(output);
        }
        return Err(format!("Failed to install {}: Command exited with code {:?}", package_name, status.code()));
    }
    
    Ok(output)
}

async fn fetch_heroic_release_info() -> Result<(String, String, String), String> {
    let client = reqwest::Client::new();
    let releases_url = "https://api.github.com/repos/Heroic-Games-Launcher/HeroicGamesLauncher/releases/latest";
    
    let mut output = String::new();
    output.push_str(&format!("$ curl -s {}\n", releases_url));
    output.push_str("─────────────────────────────────────────────────────────────\n");
    output.push_str("Fetching latest release information from GitHub...\n");
    
    let response = client
        .get(releases_url)
        .header("User-Agent", "Rustora/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch releases: HTTP {}", response.status()));
    }
    
    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release JSON: {}", e))?;
    
    let tag_name = release["tag_name"].as_str().unwrap_or("unknown");
    let release_name = release["name"].as_str().unwrap_or("unknown");
    
    output.push_str(&format!("Release: {}\n", release_name));
    output.push_str(&format!("Tag: {}\n", tag_name));
    
    // Find the x86_64 RPM file
    let assets = release["assets"].as_array()
        .ok_or("No assets found in release")?;
    
    let rpm_asset = assets.iter()
        .find(|asset| {
            let name = asset["name"].as_str().unwrap_or("");
            name.ends_with(".rpm") && name.contains("x86_64")
        })
        .ok_or("No x86_64 RPM file found in release")?;
    
    let download_url = rpm_asset["browser_download_url"].as_str()
        .ok_or("No download URL found")?;
    let filename = rpm_asset["name"].as_str()
        .ok_or("No filename found")?;
    let size = rpm_asset["size"].as_u64().unwrap_or(0);
    let size_mb = size as f64 / 1_048_576.0;
    
    output.push_str(&format!("RPM file: {}\n", filename));
    output.push_str(&format!("Size: {:.2} MB\n", size_mb));
    output.push_str(&format!("Download URL: {}\n", download_url));
    
    Ok((download_url.to_string(), filename.to_string(), output))
}

async fn download_heroic_rpm(download_url: &str, filename: &str) -> Result<(std::path::PathBuf, String), String> {
    let client = reqwest::Client::new();
    let temp_dir = std::env::temp_dir();
    let rpm_path = temp_dir.join(filename);
    
    let mut output = String::new();
    output.push_str(&format!("$ wget {}\n", download_url));
    output.push_str("─────────────────────────────────────────────────────────────\n");
    output.push_str(&format!("Downloading to: {}\n", rpm_path.display()));
    
    let download_response = client
        .get(download_url)
        .header("User-Agent", "Rustora/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to download RPM: {}", e))?;
    
    if !download_response.status().is_success() {
        return Err(format!("Failed to download RPM: HTTP {}", download_response.status()));
    }
    
    let content_length = download_response.content_length();
    output.push_str("Downloading...\n");
    
    let bytes = download_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;
    
    if let Some(total) = content_length {
        output.push_str(&format!("Downloaded: {:.2} MB / {:.2} MB (100%)\n", 
            bytes.len() as f64 / 1_048_576.0, total as f64 / 1_048_576.0));
    } else {
        output.push_str(&format!("Downloaded: {:.2} MB\n", bytes.len() as f64 / 1_048_576.0));
    }
    
    std::fs::write(&rpm_path, bytes.as_ref())
        .map_err(|e| format!("Failed to save RPM file: {}", e))?;
    
    output.push_str(&format!("✓ Download completed: {}\n", rpm_path.display()));
    
    Ok((rpm_path, output))
}

async fn install_heroic_rpm(rpm_path: &std::path::PathBuf) -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.arg(rpm_path.to_str().ok_or("Invalid RPM path")?);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute dnf install: {}", e))?;
    
    // Capture output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut output = String::new();
    output.push_str(&format!("$ pkexec dnf install -y {}\n", rpm_path.display()));
    output.push_str("─────────────────────────────────────────────────────────────\n");
    
    // Read both stdout and stderr
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(format!("Error reading stdout: {}", e));
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(format!("Error reading stderr: {}", e));
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for dnf install: {}", e))?;
    
    // Clean up the downloaded file
    let _ = std::fs::remove_file(rpm_path);
    
    // Check for pkexec cancellation (exit codes 126/127)
    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or polkit not available. Please try again.".to_string());
    }
    
    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()));
    }
    
    Ok(output)
}

// Style structs
struct RoundedButtonStyle {
    is_primary: bool,
    radius: f32,
    theme: crate::gui::Theme,
}

impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                self.theme.primary()
            } else {
                iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1)
            })),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: if self.is_primary {
                    self.theme.primary()
                } else {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                },
            },
            text_color: self.theme.text(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(_style);
        appearance.background = Some(iced::Background::Color(if self.is_primary {
            let primary = self.theme.primary();
            iced::Color::from_rgba(primary.r * 0.9, primary.g * 0.9, primary.b * 0.9, 1.0)
        } else {
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15)
        }));
        appearance
    }
}

struct TerminalContainerStyle {
    radius: f32,
    theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.theme.background())),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: self.theme.secondary_text(),
            },
            ..Default::default()
        }
    }
}

struct DialogContainerStyle {
    theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.theme.background())),
            ..Default::default()
        }
    }
}

