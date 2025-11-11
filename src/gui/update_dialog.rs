use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use crate::gui::update_settings_dialog::UpdateSettings;

#[derive(Debug, Clone)]
pub enum Message {
    InstallUpdates,
    InstallationProgress(String),
    InstallationComplete,
    InstallationError(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub repository: String,
}

#[derive(Debug)]
pub struct UpdateDialog {
    updates: Vec<UpdateInfo>,
    is_checking: bool,
    is_installing: bool,
    is_complete: bool,
    installation_progress: String,
    terminal_output: String,
    show_dialog: bool,
}

impl UpdateDialog {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
            is_checking: false,
            is_installing: true, // Start installing immediately
            is_complete: false,
            installation_progress: String::new(),
            terminal_output: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window() -> Result<(), iced::Error> {
        let dialog = Self::new();
        
        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(900.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(700.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;
        
        let default_font = crate::gui::fonts::get_inter_font();

        <UpdateDialog as Application>::run(iced::Settings {
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

        let material_font = crate::gui::fonts::get_material_symbols_font();

        let content = if self.is_checking {
            container(
                column![
                    text("Checking for updates...").size(18),
                    Space::with_height(Length::Fixed(20.0)),
                    progress_bar(0.0..=1.0, 0.5).width(Length::Fill),
                ]
                .spacing(15)
                .align_items(Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if self.is_installing {
            // Show installation progress with terminal output
            let progress_text = if self.installation_progress.is_empty() {
                "Installing updates...".to_string()
            } else {
                self.installation_progress.clone()
            };

            let terminal_scroll: Element<Message> = scrollable(
                container(
                    text(&self.terminal_output)
                        .font(iced::Font::MONOSPACE)
                        .size(11)
                        .shaping(iced::widget::text::Shaping::Advanced)
                )
                .padding(Padding::new(12.0))
                .width(Length::Fill)
            )
            .height(Length::Fill)
            .into();

            container(
                column![
                    text("Installing Updates").size(20).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(8.0)),
                    progress_bar(0.0..=1.0, 0.5).width(Length::Fill),
                    Space::with_height(Length::Fixed(8.0)),
                    text(&progress_text).size(13),
                    Space::with_height(Length::Fixed(8.0)),
                    terminal_scroll,
                ]
                .spacing(8)
                .padding(Padding::new(16.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if self.is_complete {
            container(
                column![
                    text("Updates Installed Successfully!").size(18).style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.8, 0.0))),
                    Space::with_height(Length::Fixed(20.0)),
                    {
                        if !self.terminal_output.is_empty() {
                            let scroll: Element<Message> = scrollable(
                                container(
                                    text(&self.terminal_output)
                                        .font(iced::Font::MONOSPACE)
                                        .size(11)
                                        .shaping(iced::widget::text::Shaping::Advanced)
                                )
                                .padding(Padding::new(12.0))
                                .width(Length::Fill)
                            )
                            .height(Length::Fill)
                            .into();
                            scroll
                        } else {
                            text("All updates have been installed successfully.").size(14).into()
                        }
                    },
                    Space::with_height(Length::Fixed(20.0)),
                    button(
                        row![
                            text("✓"),
                            text(" Close")
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                    })))
                    .padding(Padding::new(12.0))
                ]
                .spacing(10)
                .align_items(Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if self.updates.is_empty() {
            container(
                column![
                    text("No Updates Available").size(18).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Your system is up to date!").size(14),
                    Space::with_height(Length::Fixed(20.0)),
                    button(
                        row![
                            text("✓"),
                            text(" Close")
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                    })))
                    .padding(Padding::new(12.0))
                ]
                .spacing(10)
                .align_items(Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else {
            // Show list of updates with install button
            let title = container(
                text(format!("{} Update(s) Available", self.updates.len()))
                    .size(20)
                    .style(iced::theme::Text::Color(theme.primary()))
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0));

            let updates_list: Element<Message> = scrollable(
                column(
                    self.updates
                        .iter()
                        .map(|update| {
                            container(
                                column![
                                    row![
                                        text(&update.name).size(16).width(Length::FillPortion(3)),
                                        text(&update.current_version).size(14).width(Length::FillPortion(2)),
                                        text("→").size(14),
                                        text(&update.available_version).size(14).width(Length::FillPortion(2)),
                                    ]
                                    .spacing(12)
                                    .align_items(Alignment::Center),
                                    Space::with_height(Length::Fixed(5.0)),
                                    text(&update.repository).size(12).style(iced::theme::Text::Color(iced::Color::from_rgba(0.6, 0.6, 0.6, 1.0))),
                                ]
                                .padding(12)
                            )
                            .style(iced::theme::Container::Custom(Box::new(UpdateItemStyle)))
                            .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(6)
                .padding(10),
            )
            .height(Length::Fixed(400.0))
            .into();

            let install_button = button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font),
                    text(format!(" Install {} Update(s)", self.updates.len()))
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::InstallUpdates)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
            })))
            .padding(Padding::new(14.0));

            let cancel_button = button(
                row![
                    text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font),
                    text(" Cancel")
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::Cancel)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::new(14.0));

            container(
                column![
                    title,
                    updates_list,
                    row![
                        Space::with_width(Length::Fill),
                        cancel_button,
                        Space::with_width(Length::Fixed(10.0)),
                        install_button,
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .padding(Padding::new(20.0))
                ]
                .spacing(10)
            )
            .width(Length::Fill)
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

impl Application for UpdateDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        // Start installation immediately
        let cmd = dialog.update(Message::InstallUpdates);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        "System Updates - FedoraForge".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InstallUpdates => {
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                self.terminal_output = String::new();
                iced::Command::perform(install_updates_streaming(), |result| {
                    match result {
                        Ok(output) => Message::InstallationProgress(output),
                        Err(e) => Message::InstallationError(e),
                    }
                })
            }
            Message::InstallationProgress(output) => {
                // Append new output to terminal
                if !self.terminal_output.is_empty() {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);
                
                // Update progress text - try to extract meaningful progress info
                let output_lower = output.to_lowercase();
                if output_lower.contains("downloading") || output_lower.contains("download") {
                    self.installation_progress = "Downloading packages...".to_string();
                } else if output_lower.contains("installing") || output_lower.contains("install") {
                    self.installation_progress = "Installing packages...".to_string();
                } else if output_lower.contains("verifying") || output_lower.contains("verify") {
                    self.installation_progress = "Verifying packages...".to_string();
                } else if output_lower.contains("complete") || output_lower.contains("finished") {
                    self.installation_progress = "Installation complete!".to_string();
                } else {
                    self.installation_progress = output.clone();
                }
                
                // Check if we should mark as complete
                let output_lower = output.to_lowercase();
                if output_lower.contains("complete") || 
                   output_lower.contains("finished") ||
                   output_lower.contains("nothing to do") {
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
            Message::InstallationError(msg) => {
                self.is_installing = false;
                self.is_checking = false;
                if !self.terminal_output.is_empty() {
                    self.terminal_output.push_str("\n\n");
                }
                self.terminal_output.push_str(&format!("Error: {}", msg));
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

async fn install_updates_streaming() -> Result<String, String> {
    // Load settings
    let settings = UpdateSettings::load();
    let mut dnf_args: Vec<String> = vec!["dnf".to_string(), "upgrade".to_string(), "-y".to_string(), "--assumeyes".to_string()];
    
    // Add settings-based arguments
    dnf_args.extend(settings.to_dnf_args());
    
    // Use pkexec for privilege escalation (better than sudo for GUI apps)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(&dnf_args);
    
    // Use spawn to get streaming output
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute dnf upgrade: {}", e))?;

    // Read output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str("Starting system update...\n");
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

    if !success {
        return Err(format!("Update installation failed (exit code: {}):\n{}", exit_code, combined_output));
    }

    // Success - return the output
    if combined_output.trim().is_empty() || combined_output.trim() == "Starting system update...\n--- Output ---\n" {
        Ok("Updates installed successfully!".to_string())
    } else {
        Ok(combined_output)
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
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
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
            ..Default::default()
        }
    }
}

struct UpdateItemStyle;

impl iced::widget::container::StyleSheet for UpdateItemStyle {
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
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
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

