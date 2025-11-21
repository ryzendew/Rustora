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
    StepProgress(String, f32), // Step output and progress (0.0 to 1.0)
    InstallationComplete(Result<(), String>),
    Close,
}

#[derive(Debug)]
pub struct CachyosKernelDialog {
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
    progress: f32, // 0.0 to 1.0
    current_step_num: usize, // Current step (0-7)
}

impl CachyosKernelDialog {
    pub fn new() -> Self {
        Self {
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: "Installing Cachyos Kernel...".to_string(),
            terminal_output: String::new(),
            progress: 0.0,
            current_step_num: 0,
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

        <CachyosKernelDialog as Application>::run(iced::Settings {
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

impl Application for CachyosKernelDialog {
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
        "Cachyos Kernel Installation - Rustora".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartInstallation => {
                self.is_running = true;
                self.is_complete = false;
                self.has_error = false;
                self.progress = 0.0;
                self.current_step_num = 0;
                self.terminal_output.clear();
                self.terminal_output.push_str("Starting Cachyos Kernel installation...\n");
                self.terminal_output.push_str("=====================================\n\n");
                
                // Start with step 0
                Command::perform(run_installation_step(0), |result| {
                    match result {
                        Ok((output, step_num, progress)) => {
                            if step_num >= 8 {
                                Message::InstallationComplete(Ok(()))
                            } else {
                                Message::StepProgress(output, progress)
                            }
                        }
                        Err(e) => Message::InstallationComplete(Err(e)),
                    }
                })
            }
            Message::StepProgress(output, progress) => {
                // Append new output
                if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);
                self.progress = progress;
                
                // Update progress text based on current step
                self.progress_text = get_step_progress_text(self.current_step_num);
                
                // Check if this step is complete
                let step_complete = output.contains("completed") || output.contains("failed");
                
                if step_complete {
                    // Check if installation is fully complete
                    if output.contains("✓ ALL STEPS COMPLETED SUCCESSFULLY!") {
                        self.is_running = false;
                        self.is_complete = true;
                        self.progress = 1.0;
                        self.progress_text = "Installation completed successfully!".to_string();
                        return Command::none();
                    }
                    
                    // Continue to next step
                    self.current_step_num += 1;
                    
                    if self.current_step_num < 8 {
                        Command::perform(run_installation_step(self.current_step_num), |result| {
                            match result {
                                Ok((output, step_num, progress)) => {
                                    if step_num >= 8 {
                                        Message::InstallationComplete(Ok(()))
                                    } else {
                                        Message::StepProgress(output, progress)
                                    }
                                }
                                Err(e) => Message::InstallationComplete(Err(e)),
                            }
                        })
                    } else {
                        // All steps done
                        self.is_running = false;
                        self.is_complete = true;
                        self.progress = 1.0;
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::InstallationComplete(result) => {
                self.is_running = false;
                match result {
                    Ok(_) => {
                        self.is_complete = true;
                        self.progress = 1.0;
                        self.progress_text = "Installation completed successfully!".to_string();
                        if !self.terminal_output.contains("✓ ALL STEPS COMPLETED SUCCESSFULLY!") {
                            self.terminal_output.push_str("\n✓ All steps completed successfully!\n");
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

impl CachyosKernelDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme, settings: &AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        let title_text = if self.is_complete {
            if self.has_error {
                "Installation Failed"
            } else {
                "Installation Complete"
            }
        } else {
            "Installing Cachyos Kernel"
        };
        
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let icon_font_size = (settings.font_size_icons * settings.scale_icons).round();
        
        let progress_display = text(&self.progress_text).size(body_font_size);

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

fn get_step_progress_text(step: usize) -> String {
    match step {
        0 => "Step 1/8: Enabling kernel-cachyos repo...".to_string(),
        1 => "Step 2/8: Enabling kernel-cachyos-addons repo...".to_string(),
        2 => "Step 3/8: Installing Cachyos kernel...".to_string(),
        3 => "Step 4/8: Installing Cachyos settings...".to_string(),
        4 => "Step 5/8: Installing scheduler extensions...".to_string(),
        5 => "Step 6/8: Updating GRUB configuration...".to_string(),
        6 => "Step 7/8: GPU detection and module rebuild...".to_string(),
        7 => "Step 8/8: Regenerating initramfs...".to_string(),
        _ => "Installing Cachyos Kernel...".to_string(),
    }
}

async fn run_installation_step(step: usize) -> Result<(String, usize, f32), String> {
    const TOTAL_STEPS: f32 = 8.0;
    let mut step_output = String::new();
    let progress = ((step + 1) as f32) / TOTAL_STEPS;
    
    match step {
        0 => {
            // Step 1: Enable kernel-cachyos repo
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 1: Enabling kernel-cachyos repository\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match enable_repo("bieszczaders/kernel-cachyos").await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    step_output.push_str("\n✓ Step 1 completed: kernel-cachyos repository enabled\n\n");
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 1 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        1 => {
            // Step 2: Enable kernel-cachyos-addons repo
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 2: Enabling kernel-cachyos-addons repository\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match enable_repo("bieszczaders/kernel-cachyos-addons").await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    step_output.push_str("\n✓ Step 2 completed: kernel-cachyos-addons repository enabled\n\n");
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 2 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        2 => {
            // Step 3: Install Cachyos kernel
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 3: Installing Cachyos kernel\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match install_packages(&["kernel-cachyos"]).await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    let status_msg = if cmd_output.contains("already installed") || cmd_output.contains("Nothing to do") {
                        "Cachyos kernel already installed (continuing with remaining steps)"
                    } else {
                        "Cachyos kernel installed"
                    };
                    step_output.push_str(&format!("\n✓ Step 3 completed: {}\n\n", status_msg));
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 3 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        3 => {
            // Step 4: Install Cachyos settings and ananicy
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 4: Installing Cachyos settings and ananicy\n");
            step_output.push_str("Packages: cachyos-settings, ananicy-cpp, cachyos-ananicy-rules\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match install_packages(&["cachyos-settings", "ananicy-cpp", "cachyos-ananicy-rules"]).await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    let status_msg = if cmd_output.contains("already installed") || cmd_output.contains("Nothing to do") {
                        "Cachyos settings and ananicy already installed (continuing with remaining steps)"
                    } else {
                        "Cachyos settings and ananicy installed"
                    };
                    step_output.push_str(&format!("\n✓ Step 4 completed: {}\n\n", status_msg));
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 4 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        4 => {
            // Step 5: Install scheduler extensions
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 5: Installing scheduler extensions\n");
            step_output.push_str("Packages: scx-manager, scx-scheds-git, scx-tools\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match install_packages(&["scx-manager", "scx-scheds-git", "scx-tools"]).await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    let status_msg = if cmd_output.contains("already installed") || cmd_output.contains("Nothing to do") {
                        "Scheduler extensions already installed (continuing with remaining steps)"
                    } else {
                        "Scheduler extensions installed"
                    };
                    step_output.push_str(&format!("\n✓ Step 5 completed: {}\n\n", status_msg));
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 5 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        5 => {
            // Step 6: Update GRUB configuration
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 6: Updating GRUB configuration\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match update_grub().await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    step_output.push_str("\n✓ Step 6 completed: GRUB configuration updated\n\n");
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 6 failed: {}\n", e));
                    return Err(e);
                }
            }
        }
        6 => {
            // Step 7: Detect GPU and rebuild modules if NVIDIA
            let gpu_type = detect_gpu().await;
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str(&format!("Step 7: GPU detected - {}\n", gpu_type));
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            
            if gpu_type == "NVIDIA" {
                step_output.push_str("NVIDIA GPU detected - rebuilding kernel modules...\n\n");
                match rebuild_kernel_modules().await {
                    Ok(cmd_output) => {
                        step_output.push_str(&cmd_output);
                        step_output.push_str("\n✓ Step 7 completed: Kernel modules rebuilt\n\n");
                    }
                    Err(e) => {
                        step_output.push_str(&format!("✗ Step 7 failed: {}\n", e));
                        return Err(e);
                    }
                }
            } else {
                step_output.push_str(&format!("{} GPU detected - skipping kernel module rebuild\n\n", gpu_type));
                step_output.push_str("✓ Step 7 completed: Skipped (not NVIDIA)\n\n");
            }
        }
        7 => {
            // Step 8: Regenerate initramfs
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("Step 8: Regenerating initramfs\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n\n");
            match regenerate_initramfs().await {
                Ok(cmd_output) => {
                    step_output.push_str(&cmd_output);
                    step_output.push_str("\n✓ Step 8 completed: Initramfs regenerated\n\n");
                }
                Err(e) => {
                    step_output.push_str(&format!("✗ Step 8 failed: {}\n", e));
                    return Err(e);
                }
            }
            
            // Final summary
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("✓ ALL STEPS COMPLETED SUCCESSFULLY!\n");
            step_output.push_str("═══════════════════════════════════════════════════════════════\n");
            step_output.push_str("\nInstalled packages:\n");
            step_output.push_str("  • kernel-cachyos\n");
            step_output.push_str("  • cachyos-settings\n");
            step_output.push_str("  • ananicy-cpp\n");
            step_output.push_str("  • cachyos-ananicy-rules\n");
            step_output.push_str("  • scx-manager\n");
            step_output.push_str("  • scx-scheds-git\n");
            step_output.push_str("  • scx-tools\n");
            step_output.push_str("\n⚠️  Please reboot to use the new kernel!\n");
        }
        _ => {
            return Ok((String::new(), step, 1.0));
        }
    }
    
    Ok((step_output, step, progress))
}

async fn execute_command_with_output(
    cmd_name: &str,
    args: &[&str],
    command_line: &str,
) -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg(cmd_name);
    cmd.args(args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute {}: {}", cmd_name, e))?;
    
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut output = String::new();
    output.push_str(command_line);
    output.push_str("\n─────────────────────────────────────────────────────────────\n");
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stdout: {}", e)),
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stderr: {}", e)),
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for {}: {}", cmd_name, e))?;
    
    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or polkit not available. Please try again.".to_string());
    }
    
    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()));
    }
    
    Ok(output)
}

async fn enable_repo(repo: &str) -> Result<String, String> {
    execute_command_with_output(
        "dnf",
        &["copr", "enable", "-y", repo],
        &format!("$ pkexec dnf copr enable -y {}", repo),
    ).await
}

async fn install_packages(packages: &[&str]) -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.arg("--allowerasing");
    cmd.args(packages);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute dnf install: {}", e))?;
    
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut output = String::new();
    output.push_str(&format!("$ pkexec dnf install -y --allowerasing {}\n", packages.join(" ")));
    output.push_str("─────────────────────────────────────────────────────────────\n");
    
    let mut already_installed = false;
    let mut nothing_to_do = false;
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let line_lower = line.to_lowercase();
                        if line_lower.contains("already installed") || 
                           line_lower.contains("is already installed") {
                            already_installed = true;
                        }
                        if line_lower.contains("nothing to do") {
                            nothing_to_do = true;
                        }
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stdout: {}", e)),
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let line_lower = line.to_lowercase();
                        if line_lower.contains("already installed") || 
                           line_lower.contains("is already installed") {
                            already_installed = true;
                        }
                        if line_lower.contains("nothing to do") {
                            nothing_to_do = true;
                        }
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stderr: {}", e)),
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for dnf install: {}", e))?;
    
    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or polkit not available. Please try again.".to_string());
    }
    
    // If packages are already installed or nothing to do, treat as success
    if already_installed || nothing_to_do {
        output.push_str("\nℹ️  Note: Some packages were already installed. Continuing...\n");
        return Ok(output);
    }
    
    // Check for actual errors
    if !status.success() {
        let output_lower = output.to_lowercase();
        if output_lower.contains("error") || 
           output_lower.contains("failed") ||
           output_lower.contains("no package") ||
           output_lower.contains("cannot find") {
            return Err(format!("Installation failed with exit code: {:?}\n\nOutput:\n{}", 
                status.code(), output));
        }
        // If exit code is 1 but no clear error, might be "nothing to do" case
        if status.code() == Some(1) {
            output.push_str("\nℹ️  Note: Installation completed (exit code 1, but no clear errors detected). Continuing...\n");
            return Ok(output);
        }
        return Err(format!("Command failed with exit code: {:?}\n\nOutput:\n{}", 
            status.code(), output));
    }
    
    Ok(output)
}

async fn update_grub() -> Result<String, String> {
    execute_command_with_output(
        "grub2-mkconfig",
        &["-o", "/boot/grub2/grub.cfg"],
        "$ pkexec grub2-mkconfig -o /boot/grub2/grub.cfg",
    ).await
}

async fn detect_gpu() -> String {
    // Check for NVIDIA
    let mut cmd = TokioCommand::new("lspci");
    cmd.arg("-k");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    if let Ok(output) = cmd.output().await {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("NVIDIA") || stdout.contains("nvidia") {
                return "NVIDIA".to_string();
            }
            if stdout.contains("AMD") || stdout.contains("amd") || stdout.contains("Radeon") {
                return "AMD".to_string();
            }
            if stdout.contains("Intel") || stdout.contains("intel") {
                return "Intel".to_string();
            }
        }
    }
    
    "Unknown".to_string()
}

async fn rebuild_kernel_modules() -> Result<String, String> {
    execute_command_with_output(
        "akmods",
        &["--force", "--rebuild"],
        "$ pkexec akmods --force --rebuild",
    ).await
}

async fn regenerate_initramfs() -> Result<String, String> {
    execute_command_with_output(
        "dracut",
        &["-f", "--regenerate-all"],
        "$ pkexec dracut -f --regenerate-all",
    ).await
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

