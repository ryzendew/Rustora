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
    Error(String),
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
                        Err(e) => Message::Error(e),
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
                self.is_installing = true;
                iced::Command::perform(install_updates(), |result| {
                    match result {
                        Ok(_) => Message::UpdatesInstalled,
                        Err(e) => Message::Error(e.to_string()),
                    }
                })
            }
            Message::UpdatesInstalled => {
                self.is_installing = false;
                self.updates.clear();
                self.has_updates = false;
                iced::Command::none()
            }
            Message::Error(_msg) => {
                // Error occurred, just reset state
                self.is_checking = false;
                self.is_installing = false;
                iced::Command::none()
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

        let header = row![check_button, Space::with_width(Length::Fill), install_button]
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
    let output = TokioCommand::new("sudo")
        .arg("dnf")
        .args(["check-update", "--refresh", "--assumeyes"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf check-update: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut updates = Vec::new();
    let lines: Vec<&str> = stdout.lines().collect();
    
    for i in 0..lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with("Last metadata") || line.starts_with("Dependencies") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0].split('.').next().unwrap_or(parts[0]);
            let current = if i > 0 && !lines[i-1].trim().is_empty() {
                String::new()
            } else {
                String::new()
            };
            updates.push(UpdateInfo {
                name: name.to_string(),
                current_version: current,
                available_version: parts[1].to_string(),
                repository: parts[2].to_string(),
            });
        }
    }

    Ok(updates)
}

async fn install_updates() -> Result<(), String> {
    let status = TokioCommand::new("sudo")
        .arg("dnf")
        .args(["upgrade", "-y"])
        .status()
        .await
        .map_err(|e| format!("Failed to execute dnf upgrade: {}", e))?;

    if !status.success() {
        return Err("Update installation failed".to_string());
    }
    Ok(())
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

