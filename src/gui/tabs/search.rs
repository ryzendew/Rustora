use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::checkbox::Appearance as CheckboxAppearance;
use iced::widget::checkbox::StyleSheet as CheckboxStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;

#[derive(Debug, Clone)]
pub enum Message {
    SearchQueryChanged(String),
    Search,
    SearchResult(Vec<PackageInfo>),
    TogglePackage(String),
    InstallSelected,
    InstallComplete,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug)]
pub struct SearchTab {
    search_query: String,
    packages: Vec<PackageInfo>,
    selected_packages: std::collections::HashSet<String>,
    is_searching: bool,
    is_installing: bool,
}

impl SearchTab {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            packages: Vec::new(),
            selected_packages: std::collections::HashSet::new(),
            is_searching: false,
            is_installing: false,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::SearchQueryChanged(query) => {
                self.search_query = query.clone();
                if !query.trim().is_empty() && query.len() >= 2 {
                    self.is_searching = true;
                    iced::Command::perform(search_packages(query), |result| {
                        match result {
                            Ok(packages) => Message::SearchResult(packages),
                            Err(e) => Message::Error(e),
                        }
                    })
                } else {
                    self.packages.clear();
                    iced::Command::none()
                }
            }
            Message::Search => {
                if self.search_query.trim().is_empty() {
                    return iced::Command::none();
                }
                self.is_searching = true;
                let query = self.search_query.clone();
                iced::Command::perform(search_packages(query), |result| {
                    match result {
                        Ok(packages) => Message::SearchResult(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::SearchResult(packages) => {
                self.is_searching = false;
                self.packages = packages;
                iced::Command::none()
            }
            Message::TogglePackage(name) => {
                if self.selected_packages.contains(&name) {
                    self.selected_packages.remove(&name);
                } else {
                    self.selected_packages.insert(name);
                }
                iced::Command::none()
            }
            Message::InstallSelected => {
                if self.selected_packages.is_empty() {
                    return iced::Command::none();
                }
                self.is_installing = true;
                let packages: Vec<String> = self.selected_packages.iter().cloned().collect();
                iced::Command::perform(install_packages(packages), |result| {
                    match result {
                        Ok(_) => Message::InstallComplete,
                        Err(e) => Message::Error(e.to_string()),
                    }
                })
            }
            Message::InstallComplete => {
                self.is_installing = false;
                self.selected_packages.clear();
                iced::Command::none()
            }
            Message::Error(_msg) => {
                // Error occurred, just reset state
                self.is_installing = false;
                iced::Command::none()
            }
        }
    }

    pub fn view(&self, _theme: &crate::gui::Theme) -> Element<'_, Message> {
        let search_input = text_input("Search packages...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::Search)
            .size(16)
            .width(Length::Fill)
            .padding(14)
            .style(iced::theme::TextInput::Custom(Box::new(RoundedTextInputStyle)));

        use crate::gui::fonts::glyphs;
        
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        let search_button = button(
            row![
                text(glyphs::SEARCH_SYMBOL).font(material_font),
                text(" Search")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::Search)
            .padding(Padding::new(14.0))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
            })));

        let search_row = row![search_input, search_button]
            .spacing(10)
            .align_items(Alignment::Center);

        let install_button = if self.selected_packages.is_empty() {
            button(
                row![
                    text(glyphs::DOWNLOAD_SYMBOL).font(material_font),
                    text(" Install Selected")
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
                    text(glyphs::DOWNLOAD_SYMBOL).font(material_font),
                    text(format!(" Install {} Package(s)", self.selected_packages.len()))
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
                .on_press(Message::InstallSelected)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                })))
                .padding(Padding::new(14.0))
        };

        let content: Element<Message> = if self.is_searching {
            container(text("Searching...").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
                .into()
        } else if self.packages.is_empty() && !self.search_query.is_empty() {
            container(text("No packages found").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
                .into()
        } else {
            let package_list: Element<Message> = if self.packages.is_empty() {
                container(text("Enter a search query to find packages").size(14))
                    .width(Length::Fill)
                    .padding(20)
                    .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
                    .into()
            } else {
                scrollable(
                    column(
                        self.packages
                            .iter()
                            .map(|pkg| {
                                let pkg_name = pkg.name.clone();
                                let pkg_name_for_toggle = pkg.name.clone();
                                let is_selected = self.selected_packages.contains(&pkg.name);
                                let checkbox_widget = checkbox("", is_selected)
                                    .on_toggle(move |_| Message::TogglePackage(pkg_name_for_toggle.clone()))
                                    .style(iced::theme::Checkbox::Custom(Box::new(RoundedCheckboxStyle)));
                                container(
                                    row![
                                        checkbox_widget,
                                        text(&pkg_name).size(16).width(Length::FillPortion(2)),
                                        text(&pkg.version).size(14).width(Length::FillPortion(1)),
                                        text(&pkg.description).size(14).width(Length::FillPortion(4)),
                                    ]
                                    .spacing(12)
                                    .align_items(Alignment::Center)
                                    .padding(12)
                                )
                                .style(iced::theme::Container::Custom(Box::new(PackageItemStyle {
                                    is_selected,
                                })))
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(6)
                    .padding(10),
                )
                .into()
            };
            column![install_button, package_list].spacing(10).into()
        };

        container(column![search_row, content].spacing(15).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

async fn search_packages(query: String) -> Result<Vec<PackageInfo>, String> {
    let output = tokio::process::Command::new("dnf")
        .args(["search", "--quiet", "--assumeyes", &query])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf: {}", e))?;

    if !output.status.success() {
        return Err(format!("DNF search failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Matched fields:") || line.starts_with("Importing") || line.starts_with("Is this ok") {
            continue;
        }
        // DNF search output format: "package.arch<TAB>description" or "package.arch : description"
        let parts: Vec<&str> = if line.contains('\t') {
            line.splitn(2, '\t').collect()
        } else if line.contains(" : ") {
            line.splitn(2, " : ").collect()
        } else {
            continue;
        };
        if parts.len() == 2 {
            let name = parts[0].trim();
            let desc = parts[1].trim();
            let pkg_name = name.split('.').next().unwrap_or(name);
            results.push(PackageInfo {
                name: pkg_name.to_string(),
                description: desc.to_string(),
                version: String::new(),
            });
        }
    }

    results.sort_by(|a, b| a.name.cmp(&b.name));
    results.dedup_by(|a, b| a.name == b.name);
    Ok(results)
}

async fn install_packages(packages: Vec<String>) -> Result<(), String> {
    let status = tokio::process::Command::new("sudo")
        .arg("dnf")
        .arg("install")
        .arg("-y")
        .args(&packages)
        .status()
        .await
        .map_err(|e| format!("Failed to execute dnf install: {}", e))?;

    if !status.success() {
        return Err("Package installation failed".to_string());
    }
    Ok(())
}

struct PackageItemStyle {
    is_selected: bool,
}

impl iced::widget::container::StyleSheet for PackageItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(if self.is_selected {
                iced::Color::from_rgba(palette.primary.r, palette.primary.g, palette.primary.b, 0.1)
            } else {
                palette.background
            })),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: if self.is_selected {
                    palette.primary
                } else {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2)
                },
            },
            ..Default::default()
        }
    }
}

struct RoundedTextInputStyle;

impl TextInputStyleSheet for RoundedTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        TextInputAppearance {
            background: iced::Background::Color(palette.background),
            border: Border {
                radius: 18.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            icon_color: palette.text,
        }
    }

    fn focused(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        TextInputAppearance {
            background: iced::Background::Color(palette.background),
            border: Border {
                radius: 18.0.into(),
                width: 2.0,
                color: palette.primary,
            },
            icon_color: palette.primary,
        }
    }

    fn disabled(&self, _style: &Self::Style) -> TextInputAppearance {
        TextInputAppearance {
            background: iced::Background::Color(iced::Color::from_rgba(0.9, 0.9, 0.9, 1.0)),
            border: Border {
                radius: 18.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            icon_color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5)
    }

    fn value_color(&self, style: &Self::Style) -> iced::Color {
        style.palette().text
    }

    fn disabled_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5)
    }

    fn selection_color(&self, style: &Self::Style) -> iced::Color {
        style.palette().primary
    }
}

#[allow(dead_code)]
struct ButtonContainerStyle;

#[allow(dead_code)]
impl iced::widget::container::StyleSheet for ButtonContainerStyle {
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

struct RoundedMessageStyle;

impl iced::widget::container::StyleSheet for RoundedMessageStyle {
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
            text_color: if self.is_primary {
                palette.text
            } else {
                palette.text
            },
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

struct RoundedCheckboxStyle;

impl CheckboxStyleSheet for RoundedCheckboxStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style, is_checked: bool) -> CheckboxAppearance {
        let palette = style.palette();
        CheckboxAppearance {
            background: iced::Background::Color(if is_checked {
                palette.primary
            } else {
                iced::Color::from_rgba(0.9, 0.9, 0.9, 1.0)
            }),
            icon_color: if is_checked {
                palette.text
            } else {
                iced::Color::TRANSPARENT
            },
            border: Border {
                radius: 6.0.into(),
                width: 2.0,
                color: if is_checked {
                    palette.primary
                } else {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5)
                },
            },
            text_color: Some(palette.text),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> CheckboxAppearance {
        let mut appearance = self.active(style, is_checked);
        let palette = style.palette();
        appearance.border.color = palette.primary;
        appearance
    }
}

