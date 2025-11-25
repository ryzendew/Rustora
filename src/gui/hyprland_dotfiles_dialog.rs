use iced::widget::{button, column, container, row, scrollable, text, Space, progress_bar};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use crate::gui::app::CustomScrollableStyle;
use crate::gui::settings::AppSettings;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    StartInstallation,
    StepProgress(String, f32),
    InstallationComplete(Result<(), String>),
    Close,
}

#[derive(Debug)]
pub struct HyprlandDotfilesDialog {
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
    progress: f32,
    current_step_num: usize,
}

impl HyprlandDotfilesDialog {
    pub fn new() -> Self {
        Self {
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: "Installing Dotfiles...".to_string(),
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

        <HyprlandDotfilesDialog as Application>::run(iced::Settings {
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

impl Application for HyprlandDotfilesDialog {
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
        "Hyprland Dotfiles Installation - Rustora".to_string()
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
                self.terminal_output.push_str("Starting dotfiles installation...\n");
                self.terminal_output.push_str("=====================================\n\n");

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
                if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);
                self.progress = progress;

                self.progress_text = get_step_progress_text(self.current_step_num);

                let step_complete = output.contains("completed") || output.contains("failed");

                if step_complete {
                    if output.contains("[OK] ALL STEPS COMPLETED SUCCESSFULLY!") {
                        self.is_running = false;
                        self.is_complete = true;
                        self.progress = 1.0;
                        self.progress_text = "Installation completed successfully!".to_string();
                        return Command::none();
                    }

                    self.current_step_num += 1;

                    if self.current_step_num < 2 {
                        Command::perform(run_installation_step(self.current_step_num), |result| {
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
                    } else {
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
                        if !self.terminal_output.contains("[OK] ALL STEPS COMPLETED SUCCESSFULLY!") {
                            self.terminal_output.push_str("\n[OK] All steps completed successfully!\n");
                        }
                    }
                    Err(e) => {
                        self.has_error = true;
                        self.progress_text = format!("Installation failed: {}", e);
                        self.terminal_output.push_str(&format!("\n[FAIL] Error: {}\n", e));
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

impl HyprlandDotfilesDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme, settings: &AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let title_text = if self.is_complete {
            if self.has_error {
                "Installation Failed"
            } else {
                "Installation Complete"
            }
        } else {
            "Installing Dotfiles"
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
                progress_bar(0.0..=1.0, self.progress)
                    .width(Length::Fill)
                    .height(Length::Fixed(8.0)),
                Space::with_height(Length::Fixed(8.0)),
                progress_display.style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(16.0)),
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
        0 => "Step 1/2: Cloning repository...".to_string(),
        1 => "Step 2/2: Copying configuration files...".to_string(),
        _ => "Installation complete!".to_string(),
    }
}

async fn run_installation_step(step: usize) -> Result<(String, usize, f32), String> {
    let progress = (step as f32 + 1.0) / 2.0;

    match step {
        0 => {
            let output = clone_repository().await?;
            Ok((format!("{}\n[OK] Step 1 completed: Repository cloned\n", output), 1, progress))
        }
        1 => {
            let output = copy_config_files().await?;
            Ok((format!("{}\n[OK] Step 2 completed: Configuration files copied\n\n[OK] ALL STEPS COMPLETED SUCCESSFULLY!\n", output), 2, progress))
        }
        _ => Err("Invalid step number".to_string()),
    }
}

async fn clone_repository() -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let repo_dir = temp_dir.join("Dark-Material-shell");

    if repo_dir.exists() {
        std::fs::remove_dir_all(&repo_dir)
            .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
    }

    let mut cmd = TokioCommand::new("git");
    cmd.arg("clone");
    cmd.arg("--depth");
    cmd.arg("1");
    cmd.arg("https://github.com/ryzendew/Dark-Material-shell.git");
    cmd.arg(&repo_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut output = String::new();
    output.push_str(&format!("$ git clone --depth 1 https://github.com/ryzendew/Dark-Material-shell.git {}\n", repo_dir.display()));
    output.push_str("─────────────────────────────────────────────────────────────\n");

    let child = cmd.spawn()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    let output_result = child.wait_with_output().await
        .map_err(|e| format!("Failed to wait for git clone: {}", e))?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output_result.stdout);
    output.push_str(&stdout);

    Ok(output)
}

async fn copy_config_files() -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let repo_dir = temp_dir.join("Dark-Material-shell");

    let home_dir = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set")?;
    let config_dir = PathBuf::from(&home_dir).join(".config");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create .config directory: {}", e))?;

    let mut output = String::new();
    output.push_str("Copying configuration files...\n");
    output.push_str("─────────────────────────────────────────────────────────────\n");

    let hypr_source = repo_dir.join("hypr");
    let hypr_dest = config_dir.join("hypr");

    if hypr_source.exists() {
        if hypr_dest.exists() {
            std::fs::remove_dir_all(&hypr_dest)
                .map_err(|e| format!("Failed to remove existing hypr directory: {}", e))?;
        }

        copy_dir_all(&hypr_source, &hypr_dest)
            .map_err(|e| format!("Failed to copy hypr directory: {}", e))?;
        output.push_str(&format!("[OK] Copied hypr → {}\n", hypr_dest.display()));
    } else {
        return Err("hypr directory not found in repository".to_string());
    }

    let quickshell_source = repo_dir.join("quickshell");
    let quickshell_dest = config_dir.join("quickshell");

    if quickshell_source.exists() {
        if quickshell_dest.exists() {
            std::fs::remove_dir_all(&quickshell_dest)
                .map_err(|e| format!("Failed to remove existing quickshell directory: {}", e))?;
        }

        copy_dir_all(&quickshell_source, &quickshell_dest)
            .map_err(|e| format!("Failed to copy quickshell directory: {}", e))?;
        output.push_str(&format!("[OK] Copied quickshell → {}\n", quickshell_dest.display()));
    } else {
        return Err("quickshell directory not found in repository".to_string());
    }

    std::fs::remove_dir_all(&repo_dir)
        .map_err(|e| format!("Failed to remove temporary directory: {}", e))?;
    output.push_str("[OK] Cleaned up temporary files\n");

    Ok(output)
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

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

