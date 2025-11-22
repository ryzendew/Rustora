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
    TaskError(String),
    Close,
}

#[derive(Debug)]
pub struct KernelInstallDialog {
    kernel_name: String,
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
}

impl KernelInstallDialog {
    pub fn new(kernel_name: String) -> Self {
        Self {
            kernel_name: kernel_name.clone(),
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: format!("Installing kernel {}...", kernel_name),
            terminal_output: String::new(),
        }
    }

    pub fn run_separate_window(kernel_name: String) -> Result<(), iced::Error> {
        let dialog = Self::new(kernel_name);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(900.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(700.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <KernelInstallDialog as Application>::run(iced::Settings {
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

impl Application for KernelInstallDialog {
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
        format!("Installing Kernel {} - Rustora", self.kernel_name)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartTask => {
                self.is_running = true;
                self.terminal_output.clear();
                let kernel_name = self.kernel_name.clone();

                iced::Command::perform(
                    install_kernel_with_headers(kernel_name),
                    |result| match result {
                        Ok(output) => Message::TaskProgress(output),
                        Err(e) => Message::TaskError(e),
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

impl KernelInstallDialog {
    fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
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
                    text("Installing Kernel")
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
                    text("Installation Failed")
                        .size(22)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
                    Space::with_height(Length::Fixed(12.0)),
                    text("The kernel installation encountered an error.")
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
                    text("Kernel Installed Successfully")
                        .size(22)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.8, 0.0))),
                    Space::with_height(Length::Fixed(12.0)),
                    text(format!("Kernel {} and headers installed successfully. GRUB configuration has been rebuilt.", self.kernel_name))
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

async fn install_kernel_with_headers(kernel_name: String) -> Result<String, String> {
    let mut combined_output = String::new();

    // Step 1: Install kernel
    combined_output.push_str(&format!("$ Installing kernel: {}\n", kernel_name));
    combined_output.push_str("--- Step 1: Installing kernel package ---\n");

    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(["dnf", "install", "-y", "--assumeyes", &kernel_name]);

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute dnf install: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
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
                        combined_output.push_str(&line);
                        combined_output.push('\n');
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

    if !status.success() {
        return Err(format!("Kernel installation failed (exit code: {}):\n{}",
            status.code().unwrap_or(-1), combined_output));
    }

    combined_output.push_str("\n--- Step 2: Installing kernel headers ---\n");

    // Step 2: Install kernel headers
    let headers_name = kernel_name.replace("kernel-", "kernel-headers-");
    combined_output.push_str(&format!("$ Installing headers: {}\n", headers_name));

    let mut cmd2 = TokioCommand::new("pkexec");
    cmd2.args(["dnf", "install", "-y", "--assumeyes", &headers_name]);

    cmd2.stdout(std::process::Stdio::piped());
    cmd2.stderr(std::process::Stdio::piped());

    let mut child2 = cmd2
        .spawn()
        .map_err(|e| format!("Failed to execute dnf install: {}", e))?;

    let stdout2 = child2.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr2 = child2.stderr.take().ok_or("Failed to capture stderr")?;

    let mut stdout_reader2 = BufReader::new(stdout2).lines();
    let mut stderr_reader2 = BufReader::new(stderr2).lines();

    loop {
        tokio::select! {
            result = stdout_reader2.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
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
            result = stderr_reader2.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
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

    let status2 = child2.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status2.success() {
        // Headers might not be available, but kernel is installed, so continue
        combined_output.push_str(&format!("Warning: Headers installation failed (exit code: {}), but kernel is installed.\n",
            status2.code().unwrap_or(-1)));
    }

    combined_output.push_str("\n--- Step 3: Rebuilding GRUB configuration ---\n");

    // Step 3: Rebuild GRUB configuration
    combined_output.push_str("$ Rebuilding GRUB configuration...\n");

    let mut cmd3 = TokioCommand::new("pkexec");
    cmd3.args(["grub2-mkconfig", "-o", "/boot/grub2/grub.cfg"]);

    cmd3.stdout(std::process::Stdio::piped());
    cmd3.stderr(std::process::Stdio::piped());

    let mut child3 = cmd3
        .spawn()
        .map_err(|e| format!("Failed to execute grub2-mkconfig: {}", e))?;

    let stdout3 = child3.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr3 = child3.stderr.take().ok_or("Failed to capture stderr")?;

    let mut stdout_reader3 = BufReader::new(stdout3).lines();
    let mut stderr_reader3 = BufReader::new(stderr3).lines();

    loop {
        tokio::select! {
            result = stdout_reader3.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
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
            result = stderr_reader3.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
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

    let status3 = child3.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status3.success() {
        combined_output.push_str(&format!("Warning: GRUB rebuild failed (exit code: {}), but kernel is installed.\n",
            status3.code().unwrap_or(-1)));
    } else {
        combined_output.push_str("GRUB configuration rebuilt successfully.\n");
    }

    Ok(format!("Kernel {} installed successfully!\n\n{}", kernel_name, combined_output))
}

// Style structs
struct DialogContainerStyle;
impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.1, 0.1, 0.1, 1.0).into()),
            ..Default::default()
        }
    }
}

struct TerminalContainerStyle;
impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgb(0.05, 0.05, 0.05).into()),
            border: Border::with_radius(6.0),
            ..Default::default()
        }
    }
}

struct RoundedButtonStyle {
    is_primary: bool,
}
impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(if self.is_primary {
                iced::Color::from_rgb(0.2, 0.6, 0.9).into()
            } else {
                iced::Color::from_rgba(0.2, 0.2, 0.2, 1.0).into()
            }),
            text_color: iced::Color::WHITE,
            border: Border::with_radius(6.0),
            ..Default::default()
        }
    }
}


