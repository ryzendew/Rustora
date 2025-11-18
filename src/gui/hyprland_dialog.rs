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
pub struct HyprlandDialog {
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
    progress: f32, // 0.0 to 1.0
    current_step_num: usize, // Current step (0-9)
}

impl HyprlandDialog {
    pub fn new() -> Self {
        Self {
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: "Installing Hyprland & Dependencies...".to_string(),
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

        <HyprlandDialog as Application>::run(iced::Settings {
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

impl Application for HyprlandDialog {
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
        "Hyprland Installation - Rustora".to_string()
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
                self.terminal_output.push_str("Starting Hyprland installation...\n");
                self.terminal_output.push_str("=====================================\n\n");
                
                // Start with step 0
                Command::perform(run_installation_step(0), |result| {
                    match result {
                        Ok((output, step_num, progress)) => {
                            if step_num >= 2 {
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
                    
                    if self.current_step_num < 10 {
                        Command::perform(run_installation_step(self.current_step_num), |result| {
                            match result {
                                Ok((output, step_num, progress)) => {
                                    if step_num >= 10 {
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

impl HyprlandDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme, settings: &AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        let title_text = if self.is_complete {
            if self.has_error {
                "Installation Failed"
            } else {
                "Installation Complete"
            }
        } else {
            "Installing Hyprland & Dependencies"
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
        0 => "Step 1/2: Enabling COPR repositories...".to_string(),
        1 => "Step 2/2: Updating cache and installing Hyprland dependencies...".to_string(),
        _ => "Installation complete!".to_string(),
    }
}

async fn run_installation_step(step: usize) -> Result<(String, usize, f32), String> {
    let progress = (step as f32 + 1.0) / 2.0;
    
    match step {
        0 => {
            // Enable COPR repos (only quickshell and hyprland)
            let output = enable_copr_repos().await?;
            Ok((format!("{}\n✓ Step 1 completed: COPR repositories enabled\n", output), 1, progress))
        }
        1 => {
            // Update package cache and install Hyprland dependencies in one command
            let output = update_cache_and_install().await?;
            Ok((format!("{}\n✓ Step 2 completed: Cache updated and Hyprland dependencies installed\n\n✓ ALL STEPS COMPLETED SUCCESSFULLY!\n", output), 2, progress))
        }
        _ => Err("Invalid step number".to_string()),
    }
}

async fn execute_command_with_output(cmd: &mut TokioCommand, description: &str) -> Result<String, String> {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute {}: {}", description, e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output = String::new();
    
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
        .map_err(|e| format!("Failed to wait for {}: {}", description, e))?;

    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or polkit not available. Please try again.".to_string());
    }

    if !status.success() {
        let output_lower = output.to_lowercase();
        if output_lower.contains("already installed") || output_lower.contains("is already installed") {
            output.push_str("\nℹ️  Note: Repository was already enabled. Continuing...\n");
            return Ok(output);
        }
        return Err(format!("Command failed with exit code: {:?}\n\nOutput:\n{}", status.code(), output));
    }

    Ok(output)
}

async fn enable_copr_repos() -> Result<String, String> {
    // Enable both repos in a single command
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("copr");
    cmd.arg("enable");
    cmd.arg("-y");
    cmd.arg("solopasha/hyprland");
    cmd.arg("errornointernet/quickshell");
    
    let mut output = String::new();
    output.push_str("$ pkexec dnf copr enable -y solopasha/hyprland errornointernet/quickshell\n");
    output.push_str("─────────────────────────────────────────────────────────────\n");
    
    let cmd_output = execute_command_with_output(&mut cmd, "COPR repositories").await?;
    output.push_str(&cmd_output);
    
    Ok(output)
}

async fn update_cache_and_install() -> Result<String, String> {
    // Update cache and install packages in a single command
    let packages = vec![
        "hyprland",
        "hyprpicker",
        "swww",
        "quickshell-git",
        "fuzzel",
        "wlogout",
        "cliphist",
        "brightnessctl",
        "grim",
        "slurp",
        "swappy",
    ];
    
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("sh");
    cmd.arg("-c");
    cmd.arg(&format!("dnf makecache && dnf install -y {}", packages.join(" ")));
    
    let mut output = String::new();
    output.push_str("$ pkexec dnf makecache && dnf install -y ");
    output.push_str(&packages.join(" "));
    output.push_str("\n");
    output.push_str("─────────────────────────────────────────────────────────────\n");
    
    let cmd_output = execute_command_with_output(&mut cmd, "cache update and package installation").await?;
    output.push_str(&cmd_output);
    
    Ok(output)
}


// Style structs (same as cachyos_kernel_dialog.rs)
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
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2)
        }));
        appearance
    }
}

struct TerminalContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
            border: Border {
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                width: 1.0,
                radius: self.radius.into(),
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

