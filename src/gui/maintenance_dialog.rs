use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone)]
pub enum Message {
    StartTask,
    TaskProgress(String),
    #[allow(dead_code)]
    TaskComplete,
    TaskError(String),
    Close,
}

#[derive(Debug, Clone)]
pub enum MaintenanceTask {
    RebuildKernelModules,
    RegenerateInitramfs,
    RemoveOrphanedPackages,
    CleanPackageCache,
}

#[derive(Debug)]
pub struct MaintenanceDialog {
    task: MaintenanceTask,
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
}

impl MaintenanceDialog {
    pub fn new(task: MaintenanceTask) -> Self {
        Self {
            task: task.clone(),
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: match task {
                MaintenanceTask::RebuildKernelModules => "Rebuilding kernel modules...".to_string(),
                MaintenanceTask::RegenerateInitramfs => "Regenerating initramfs...".to_string(),
                MaintenanceTask::RemoveOrphanedPackages => "Removing orphaned packages...".to_string(),
                MaintenanceTask::CleanPackageCache => "Cleaning package cache...".to_string(),
            },
            terminal_output: String::new(),
        }
    }

    pub fn run_separate_window(task: MaintenanceTask) -> Result<(), iced::Error> {
        let dialog = Self::new(task);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(900.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(700.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <MaintenanceDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }

    fn get_task_title(&self) -> &str {
        match self.task {
            MaintenanceTask::RebuildKernelModules => "Rebuild Kernel Modules",
            MaintenanceTask::RegenerateInitramfs => "Regenerate Initramfs",
            MaintenanceTask::RemoveOrphanedPackages => "Remove Orphaned Packages",
            MaintenanceTask::CleanPackageCache => "Clean Package Cache",
        }
    }

    fn get_task_command(&self) -> (&str, Vec<&str>) {
        match self.task {
            MaintenanceTask::RebuildKernelModules => ("pkexec", vec!["akmods", "--force", "--rebuild"]),
            MaintenanceTask::RegenerateInitramfs => ("pkexec", vec!["dracut", "-f", "regenerate-all"]),
            MaintenanceTask::RemoveOrphanedPackages => ("pkexec", vec!["dnf", "autoremove", "-y", "--assumeyes"]),
            MaintenanceTask::CleanPackageCache => ("pkexec", vec!["dnf", "clean", "all"]),
        }
    }
}

impl Application for MaintenanceDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        // Start task immediately
        let cmd = dialog.update(Message::StartTask);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        format!("{} - Rustora", self.get_task_title())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartTask => {
                self.is_running = true;
                self.terminal_output.clear();
                let (cmd_name, args) = self.get_task_command();
                let cmd_name = cmd_name.to_string();
                let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

                iced::Command::perform(
                    run_maintenance_task_streaming(cmd_name, args),
                    |result| {
                        match result {
                            Ok(output) => Message::TaskProgress(output),
                            Err(e) => Message::TaskError(e),
                        }
                    },
                )
            }
            Message::TaskProgress(output) => {
                // Set the complete output
                self.terminal_output = output;
                self.is_running = false;
                self.is_complete = true;
                Command::none()
            }
            Message::TaskComplete => {
                self.is_running = false;
                self.is_complete = true;
                Command::none()
            }
            Message::TaskError(error) => {
                self.is_running = false;
                self.has_error = true;
                self.terminal_output = error;
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

impl MaintenanceDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let content = if self.is_running {
            // Show running state with terminal output
            let terminal_scroll: Element<Message> = scrollable(
                container(
                    text(&self.terminal_output)
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
                    text(self.get_task_title())
                        .size(22)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(12.0)),
                    progress_bar(0.0..=1.0, 0.5)
                        .width(Length::Fill),
                    Space::with_height(Length::Fixed(8.0)),
                    text(&self.progress_text)
                        .size(14),
                    Space::with_height(Length::Fixed(16.0)),
                    container(
                        column![
                            text("Output:")
                                .size(13)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_height(Length::Fixed(8.0)),
                            terminal_scroll,
                        ]
                        .spacing(0)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::new(12.0))
                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle))),
                ]
                .spacing(0)
                .padding(Padding::new(24.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if self.has_error {
            // Show error state
            let terminal_scroll: Element<Message> = if !self.terminal_output.is_empty() {
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
                .height(Length::Fill)
                .into()
            } else {
                container(Space::with_height(Length::Shrink))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            };

            container(
                column![
                    text("Task Failed")
                        .size(22)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
                    Space::with_height(Length::Fixed(12.0)),
                    text("The maintenance task encountered an error.")
                        .size(14),
                    Space::with_height(Length::Fixed(16.0)),
                    container(
                        column![
                            text("Error Output:")
                                .size(13)
                                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
                            Space::with_height(Length::Fixed(8.0)),
                            terminal_scroll,
                        ]
                        .spacing(0)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::new(12.0))
                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle))),
                    Space::with_height(Length::Fixed(16.0)),
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(18),
                            text(" Close")
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Close)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                    })))
                    .padding(Padding::new(12.0)),
                ]
                .spacing(0)
                .padding(Padding::new(24.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else {
            // Show success state
            let terminal_scroll: Element<Message> = if !self.terminal_output.is_empty() {
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
                .height(Length::Fill)
                .into()
            } else {
                container(Space::with_height(Length::Shrink))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            };

            container(
                column![
                    text("Task Completed Successfully")
                        .size(22)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.8, 0.0))),
                    Space::with_height(Length::Fixed(12.0)),
                    text(format!("{} completed successfully.", self.get_task_title()))
                        .size(14),
                    Space::with_height(Length::Fixed(16.0)),
                    container(
                        column![
                            text("Output:")
                                .size(13)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_height(Length::Fixed(8.0)),
                            terminal_scroll,
                        ]
                        .spacing(0)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::new(12.0))
                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle))),
                    Space::with_height(Length::Fixed(16.0)),
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(18),
                            text(" Close")
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Close)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                    })))
                    .padding(Padding::new(12.0)),
                ]
                .spacing(0)
                .padding(Padding::new(24.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        };

        content.into()
    }
}

async fn run_maintenance_task_streaming(cmd_name: String, args: Vec<String>) -> Result<String, String> {
    let mut cmd = TokioCommand::new(&cmd_name);
    cmd.args(&args);

    // Use spawn to get streaming output
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute {}: {}", cmd_name, e))?;

    // Read output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut combined_output = String::new();
    combined_output.push_str(&format!("$ {} {}\n", cmd_name, args.join(" ")));
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
                        return Err(format!("Error reading stdout: {}", e));
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
                        return Err(format!("Error reading stderr: {}", e));
                    }
                }
            }
        }
    }

    // Wait for process to complete
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status.success() {
        return Err(format!("Process failed (exit code: {}):\n{}",
            status.code().unwrap_or(-1), combined_output));
    }

    Ok(combined_output)
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
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct TerminalContainerStyle;

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
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
                radius: 8.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
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

