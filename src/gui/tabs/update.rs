use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone)]
pub enum Message {
    CheckUpdates,
    UpdatesFound(Vec<UpdateInfo>),
    InstallUpdates,
    UpdatesInstalled,
    OpenSettings,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub repository: String,
}

#[derive(Debug)]
pub struct UpdateTab {
    updates: Vec<UpdateInfo>,
    is_checking: bool,
    is_installing: bool,
    has_updates: bool,
}

impl UpdateTab {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
            is_checking: false,
            is_installing: false,
            has_updates: false,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::CheckUpdates => {
                self.is_checking = true;
                self.has_updates = false;
                iced::Command::perform(check_for_updates(), |result| {
                    match result {
                        Ok(updates) => Message::UpdatesFound(updates),
                        Err(_) => Message::UpdatesFound(Vec::new()),
                    }
                })
            }
            Message::UpdatesFound(updates) => {
                self.is_checking = false;
                self.updates = updates.clone();
                self.has_updates = !updates.is_empty();
                iced::Command::none()
            }
            Message::InstallUpdates => {
                if self.updates.is_empty() {
                    return iced::Command::none();
                }
                // Spawn a separate window for update installation
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("fedoraforge"));
                        TokioCommand::new(&exe_path)
                            .arg("update-dialog")
                            .spawn()
                            .ok();
                    },
                    move |_| {
                        // Keep the updates list visible in the tab
                        Message::UpdatesInstalled
                    },
                )
            }
            Message::UpdatesInstalled => {
                // Updates are now handled in the dialog window
                // Optionally refresh the list after installation
                iced::Command::none()
            }
            Message::OpenSettings => {
                // Spawn a separate window for update settings
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("fedoraforge"));
                        TokioCommand::new(&exe_path)
                            .arg("update-settings-dialog")
                            .spawn()
                            .ok();
                    },
                    |_| Message::UpdatesInstalled, // Dummy message
                )
            }
        }
    }

    pub fn view(&self, _theme: &crate::gui::Theme) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        let check_button = if self.is_checking {
            button(
                row![
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font),
                    text(" Checking...")
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(14.0))
        } else {
            button(
                row![
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font),
                    text(" Check for Updates")
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
                .on_press(Message::CheckUpdates)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                })))
                .padding(Padding::new(14.0))
        };

        let install_button: Element<Message> = if self.updates.is_empty() || self.is_installing {
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
                        is_primary: false,
                    })))
                    .padding(Padding::new(14.0))
                    .into()
            } else {
                button(text("No Updates Available"))
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: false,
                    })))
                    .padding(Padding::new(14.0))
                    .into()
            }
        } else {
            button(
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
                .padding(Padding::new(14.0))
                .into()
        };

        let settings_button = button(
            text(crate::gui::fonts::glyphs::SETTINGS_SYMBOL).font(material_font)
        )
        .on_press(Message::OpenSettings)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(10.0));

        let header = row![
            check_button,
            Space::with_width(Length::Fill),
            settings_button,
            Space::with_width(Length::Fixed(10.0)),
            install_button
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let content: Element<Message> = if self.is_checking {
            container(text("Checking for updates...").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle)))
                .into()
        } else if self.updates.is_empty() && !self.has_updates {
            container(text("Click 'Check for Updates' to see available updates").size(14))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle)))
                .into()
        } else if self.updates.is_empty() && self.has_updates {
            container(text("System is up to date!").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle)))
                .into()
        } else {
            scrollable(
                column(
                    self.updates
                        .iter()
                        .map(|update| {
                            container(
                                row![
                                    text(&update.name).size(16).width(Length::FillPortion(3)),
                                    text(&update.current_version).size(14).width(Length::FillPortion(2)),
                                    text("→").size(14),
                                    text(&update.available_version).size(14).width(Length::FillPortion(2)),
                                    text(&update.repository).size(14).width(Length::FillPortion(2)),
                                ]
                                .spacing(12)
                                .align_items(Alignment::Center)
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
            .into()
        };

        container(column![header, content].spacing(15).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

async fn check_for_updates() -> Result<Vec<UpdateInfo>, String> {
    // dnf check-update doesn't require sudo - it just checks what's available
    // Exit code 100 means updates are available (this is normal, not an error)
    let output = TokioCommand::new("dnf")
        .args(["check-update", "--quiet"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf check-update: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // If stdout is empty, no updates available
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Get currently installed versions to compare
    let installed_output = TokioCommand::new("dnf")
        .args(["list", "--installed", "--quiet"])
        .output()
        .await;
    
    let mut installed_versions = std::collections::HashMap::new();
    if let Ok(installed) = installed_output {
        if installed.status.success() {
            let installed_stdout = String::from_utf8_lossy(&installed.stdout);
            for line in installed_stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].split('.').next().unwrap_or(parts[0]);
                    installed_versions.insert(name.to_string(), parts[1].to_string());
                }
            }
        }
    }

    let mut updates = Vec::new();
    
    // Parse check-update output: "package.arch  version  repository"
    for line in stdout.lines() {
        let line = line.trim();
        // Skip header lines and empty lines
        if line.is_empty() || 
           line.starts_with("Last metadata") || 
           line.starts_with("Dependencies") ||
           line.starts_with("Upgrade") ||
           line.starts_with("Obsoleting") ||
           line.contains("Matched fields:") {
            continue;
        }
        
        // Split by whitespace - format is: package.arch  version  repository
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let full_name = parts[0];
            let name = full_name.split('.').next().unwrap_or(full_name);
            let available_version = parts[1].to_string();
            let repository = parts[2].to_string();
            
            // Get current version from installed packages
            let current_version = installed_versions
                .get(name)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            
            updates.push(UpdateInfo {
                name: name.to_string(),
                current_version,
                available_version,
                repository,
            });
        }
    }

    Ok(updates)
}

struct RoundedContainerStyle;

impl iced::widget::container::StyleSheet for RoundedContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            border: Border {
                radius: 16.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
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

