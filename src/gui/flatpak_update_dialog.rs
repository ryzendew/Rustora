use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme, Color};
use crate::gui::dialog_design::DialogDesign;
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use std::path::PathBuf;
use tokio::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatpakUpdateInfo {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub remote: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartUpdate,
    UpdateProgress(String),
    UpdateComplete,
    UpdateError(String),
    Cancel,
}

#[derive(Debug)]
pub struct FlatpakUpdateDialog {
    packages: Vec<FlatpakUpdateInfo>,
    is_updating: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
    current_package: Option<String>,
}

impl FlatpakUpdateDialog {
    pub fn new(packages: Vec<FlatpakUpdateInfo>) -> Self {
        Self {
            packages: packages.clone(),
            is_updating: false,
            is_complete: false,
            has_error: false,
            progress_text: format!("Ready to update {} package(s)", packages.len()),
            terminal_output: String::new(),
            current_package: None,
        }
    }

    pub fn run_separate_window(packages: Vec<FlatpakUpdateInfo>) -> Result<(), iced::Error> {
        let dialog = Self::new(packages);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(700.0, 550.0);
        window_settings.min_size = Some(iced::Size::new(500.0, 400.0));
        window_settings.max_size = None;
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <FlatpakUpdateDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }

    fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let settings = crate::gui::settings::AppSettings::load();
        let title_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_size = (settings.font_size_body * settings.scale_body).round();
        let button_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let header = container(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL)
                    .font(material_font)
                    .size(title_size * 1.2)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_width(DialogDesign::space_small()),
                column![
                    text("Update Flatpak Applications")
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.primary())),
                    text(format!("{} package(s) to update", self.packages.len()))
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

        let packages_section = container(
            column![
                text("Packages to Update")
                    .size(body_size * 1.1)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(DialogDesign::space_small()),
                scrollable(
                    column(
                        self.packages
                            .iter()
                            .map(|pkg| {
                                container(
                                    row![
                                        text(&pkg.name).size(body_size).width(Length::FillPortion(3)),
                                        text(&pkg.version).size(body_size * 0.9).width(Length::FillPortion(2)),
                                        text(pkg.remote.as_deref().unwrap_or("local"))
                                            .size(body_size * 0.9)
                                            .width(Length::FillPortion(2))
                                            .style(iced::theme::Text::Color(theme.secondary_text())),
                                    ]
                                    .spacing(DialogDesign::SPACE_SMALL)
                                    .align_items(Alignment::Center)
                                    .padding(DialogDesign::pad_small())
                                )
                                .style(iced::theme::Container::Custom(Box::new(PackageItemStyle {
                                    is_updating: self.current_package.as_ref().map(|cp| cp == &pkg.application_id).unwrap_or(false),
                                })))
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(DialogDesign::SPACE_SMALL)
                )
                .height(Length::Fixed(200.0)),
            ]
            .spacing(0)
            .padding(DialogDesign::pad_medium())
        )
        .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)));

        let progress_section = if self.is_updating || self.is_complete || self.has_error {
            let value = if self.is_complete { 1.0 } else if self.has_error { 0.0 } else { 0.7 };
            let progress_text = if self.is_complete {
                "Update completed successfully!".to_string()
            } else if self.has_error {
                "Update failed".to_string()
            } else {
                self.progress_text.clone()
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
                        } else if self.has_error {
                            theme.danger()
                        } else {
                            theme.text()
                        })),
                    if let Some(ref current) = self.current_package {
                        text(format!("Updating: {}", current))
                            .size(body_size * 0.9)
                            .style(iced::theme::Text::Color(theme.primary()))
                    } else {
                        text("").size(0)
                    },
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
            } else if self.has_error {
                row![
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::CANCEL_SYMBOL).font(material_font).size(button_size * 1.1),
                            text(" Close").size(button_size)
                        ]
                        .spacing(DialogDesign::SPACE_TINY)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: false })))
                    .padding(DialogDesign::pad_small()),
                    Space::with_width(Length::Fill),
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
                        if self.is_updating {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(button_size * 1.1),
                                    text(" Updating...").size(button_size)
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
                                    text(" Start Update").size(button_size)
                                ]
                                .spacing(DialogDesign::SPACE_TINY)
                                .align_items(Alignment::Center)
                            )
                            .on_press(Message::StartUpdate)
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
                        packages_section,
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
        .into()
    }
}

impl Application for FlatpakUpdateDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        (flags, Command::none())
    }

    fn title(&self) -> String {
        format!("Update Flatpak Applications - Rustora")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartUpdate => {
                self.is_updating = true;
                self.has_error = false;
                self.progress_text = "Starting update...".to_string();
                self.terminal_output = String::new();

                let packages_to_update: Vec<String> = self.packages.iter()
                    .map(|p| p.application_id.clone())
                    .collect();

                iced::Command::perform(update_flatpaks_streaming(packages_to_update), |result| {
                    match result {
                        Ok(output) => Message::UpdateProgress(output),
                        Err(e) => Message::UpdateError(e),
                    }
                })
            }
            Message::UpdateProgress(output) => {
                if !self.terminal_output.is_empty() {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);

                self.progress_text = output.clone();

                let output_lower = output.to_lowercase();
                for pkg in &self.packages {
                    if output_lower.contains(&pkg.name.to_lowercase()) ||
                       output_lower.contains(&pkg.application_id.to_lowercase()) {
                        self.current_package = Some(pkg.application_id.clone());
                        break;
                    }
                }

                if output.contains("Complete") ||
                   output.contains("Installed") ||
                   output.contains("complete") ||
                   output.to_lowercase().contains("success") ||
                   output.contains("Nothing to do") {
                    iced::Command::perform(async {}, |_| Message::UpdateComplete)
                } else {
                    iced::Command::none()
                }
            }
            Message::UpdateComplete => {
                self.is_updating = false;
                self.is_complete = true;
                self.current_package = None;
                self.progress_text = "Update completed successfully!".to_string();
                if !self.terminal_output.contains("completed successfully") {
                    self.terminal_output.push_str("\n[OK] Update completed successfully!");
                }
                iced::Command::none()
            }
            Message::UpdateError(msg) => {
                self.is_updating = false;
                self.has_error = true;
                self.current_package = None;
                self.progress_text = format!("Update failed: {}", msg);
                if !self.terminal_output.contains("failed") {
                    self.terminal_output.push_str(&format!("\n[FAIL] Update failed: {}", msg));
                }
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

async fn update_flatpaks_streaming(packages: Vec<String>) -> Result<String, String> {
    let mut cmd = TokioCommand::new("flatpak");
    cmd.args(["update", "--app", "-y", "--noninteractive", "--verbose"]);

    if !packages.is_empty() {
        cmd.args(&packages);
    }

    let command_str = format!("flatpak update --app -y --noninteractive --verbose {}",
        packages.join(" "));

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute flatpak update: {}", e))?;

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

    let packages_str = packages.join(", ");
    write_flatpak_log("update", &packages_str, None, &combined_output, success).await;

    if !success {
        let output_lower = combined_output.to_lowercase();
        if output_lower.contains("nothing to do") ||
           output_lower.contains("no updates") {
            // This is actually a success case
            return Ok(format!("No updates needed.\n\n{}", combined_output));
        }

        return Err(format!("Update failed (exit code: {}):\n{}", exit_code, combined_output));
    }

    if combined_output.trim().is_empty() || combined_output.trim() == format!("Command: {}\n--- Output ---\n", command_str).trim() {
        Ok("Update Complete!".to_string())
    } else {
        let output_lower = combined_output.to_lowercase();
        if output_lower.contains("complete") ||
           output_lower.contains("installed") ||
           output_lower.contains("success") {
            Ok(combined_output)
        } else {
            Ok(format!("Update completed.\n\n{}", combined_output))
        }
    }
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

struct PackageItemStyle {
    is_updating: bool,
}

impl iced::widget::container::StyleSheet for PackageItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(if self.is_updating {
                Color::from_rgba(palette.primary.r, palette.primary.g, palette.primary.b, 0.15)
            } else {
                Color::from_rgba(palette.background.r * 0.96, palette.background.g * 0.96, palette.background.b * 0.96, 1.0)
            })),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: if self.is_updating {
                    palette.primary
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 0.15)
                },
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
