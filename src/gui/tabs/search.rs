use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
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
    pub release: String,
    pub arch: String,
    pub summary: String,
    pub size: String,
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
                // Debounce: only search if query is at least 2 characters and not empty
                if !query.trim().is_empty() && query.len() >= 2 {
                    self.is_searching = true;
                    // Use a small delay to debounce rapid typing
                    let query_clone = query.clone();
                    iced::Command::perform(
                        async move {
                            // Small delay to debounce rapid typing
                            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                            search_packages(query_clone).await
                        },
                        |result| {
                            match result {
                                Ok(packages) => Message::SearchResult(packages),
                                Err(e) => Message::Error(e),
                            }
                        }
                    )
                } else {
                    self.packages.clear();
                    self.is_searching = false;
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
                // Spawn a separate window for package installation
                let packages: Vec<String> = self.selected_packages.iter().cloned().collect();
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("fedoraforge"));
                        TokioCommand::new(&exe_path)
                            .arg("install-dialog")
                            .args(packages)
                            .spawn()
                            .ok();
                    },
                    |_| Message::InstallComplete, // Dummy message
                )
            }
            Message::InstallComplete => {
                self.is_installing = false;
                self.selected_packages.clear();
                // Refresh search results after installation
                if !self.search_query.trim().is_empty() {
                    let query = self.search_query.clone();
                    self.is_searching = true;
                    iced::Command::perform(search_packages(query), |result| {
                        match result {
                            Ok(packages) => Message::SearchResult(packages),
                            Err(e) => Message::Error(e),
                        }
                    })
                } else {
                    iced::Command::none()
                }
            }
            Message::Error(_msg) => {
                // Error occurred, just reset state
                self.is_installing = false;
                iced::Command::none()
            }
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
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
                                let pkg_name_for_toggle = pkg.name.clone();
                                let is_selected = self.selected_packages.contains(&pkg.name);
                                
                                // Professional card layout
                                let checkbox_widget = checkbox("", is_selected)
                                    .on_toggle(move |_| Message::TogglePackage(pkg_name_for_toggle.clone()))
                                    .style(iced::theme::Checkbox::Custom(Box::new(RoundedCheckboxStyle)));
                                
                                // Package header with name and version
                                let version_info: Element<Message> = if !pkg.version.is_empty() || !pkg.release.is_empty() {
                                    let version_text = if !pkg.version.is_empty() && !pkg.release.is_empty() {
                                        format!("{} {}", pkg.version, pkg.release)
                                    } else if !pkg.version.is_empty() {
                                        pkg.version.clone()
                                    } else {
                                        pkg.release.clone()
                                    };
                                    row![
                                        text("Version:")
                                            .size(12)
                                            .style(iced::theme::Text::Color(theme.secondary_text())),
                                        text(&version_text)
                                            .size(12),
                                    ]
                                    .spacing(6)
                                    .into()
                                } else {
                                    Space::with_height(Length::Shrink).into()
                                };
                                
                                let header = row![
                                    checkbox_widget,
                                    column![
                                        row![
                                            text(&pkg.name)
                                            .size(18) // Larger size for emphasis
                                            .style(iced::theme::Text::Color(theme.text())) // Darker for better visibility
                                                .size(17)
                                                .style(iced::theme::Text::Color(theme.primary()))
                                                .width(Length::Fill),
                                        ]
                                        .width(Length::Fill)
                                        .spacing(8),
                                        version_info,
                                    ]
                                    .spacing(4)
                                    .width(Length::Fill),
                                ]
                                .spacing(12)
                                .align_items(Alignment::Start)
                                .width(Length::Fill);
                                
                                // Package details section
                                let details = if !pkg.summary.is_empty() || !pkg.description.is_empty() {
                                    let summary_text = if !pkg.summary.is_empty() {
                                        &pkg.summary
                                    } else {
                                        &pkg.description
                                    };
                                    // Truncate long descriptions
                                    let display_text = if summary_text.len() > 120 {
                                        format!("{}...", &summary_text[..120])
                                    } else {
                                        summary_text.clone()
                                    };
                                    
                                    column![
                                        text(&display_text)
                                            .size(13)
                                            .shaping(iced::widget::text::Shaping::Advanced)
                                            .width(Length::Fill),
                                    ]
                                    .spacing(4)
                                    .width(Length::Fill)
                                } else {
                                    column![].spacing(0).width(Length::Fill)
                                };
                                
                                // Package metadata (arch, size)
                                let arch_info: Element<Message> = if !pkg.arch.is_empty() {
                                    row![
                                        text("Arch:")
                                            .size(11)
                                            .style(iced::theme::Text::Color(theme.secondary_text())),
                                        text(&pkg.arch)
                                            .size(11),
                                    ]
                                    .spacing(4)
                                    .into()
                                } else {
                                    Space::with_width(Length::Shrink).into()
                                };
                                
                                let size_info: Element<Message> = if !pkg.size.is_empty() {
                                    row![
                                        text("Size:")
                                            .size(11)
                                            .style(iced::theme::Text::Color(theme.secondary_text())),
                                        text(&pkg.size)
                                            .size(11),
                                    ]
                                    .spacing(4)
                                    .into()
                                } else {
                                    Space::with_width(Length::Shrink).into()
                                };
                                
                                let metadata = row![
                                    arch_info,
                                    Space::with_width(Length::Fill),
                                    size_info,
                                ]
                                .width(Length::Fill)
                                .spacing(8);
                                
                                container(
                                    column![
                                        header,
                                        Space::with_height(Length::Fixed(8.0)),
                                        details,
                                        Space::with_height(Length::Fixed(6.0)),
                                        metadata,
                                    ]
                                    .spacing(0)
                                    .padding(Padding::new(16.0))
                                    .width(Length::Fill)
                                )
                                .style(iced::theme::Container::Custom(Box::new(PackageCardStyle {
                                    is_selected,
                                })))
                                .width(Length::Fill)
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(10)
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
    // Use repoquery which is much faster than dnf search
    // Get all info in one query using --qf (queryformat) to avoid multiple dnf info calls
    let queryformat = "%{name}|%{version}|%{release}|%{arch}|%{summary}|%{description}|%{size}";
    
    let output = tokio::process::Command::new("dnf")
        .args([
            "repoquery",
            "--quiet",
            "--cacheonly", // Use cached metadata only - much faster
            "--qf", queryformat,
            &format!("*{}*", query.trim()), // Search pattern
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf: {}", e))?;

    // If cacheonly fails, try without it (metadata might not be cached)
    let output = if !output.status.success() {
        tokio::process::Command::new("dnf")
            .args([
                "repoquery",
                "--quiet",
                "--qf", queryformat,
                &format!("*{}*", query.trim()),
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute dnf: {}", e))?
    } else {
        output
    };

    if !output.status.success() {
        return Err(format!("DNF repoquery failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse pipe-separated values: name|version|release|arch|summary|description|size
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 {
            continue;
        }

        let name = parts[0].trim();
        // Skip duplicates (same package from different repos)
        if seen_names.contains(name) {
            continue;
        }
        seen_names.insert(name.to_string());

        let version = parts.get(1).unwrap_or(&"").trim();
        let release = parts.get(2).unwrap_or(&"").trim();
        let arch = parts.get(3).unwrap_or(&"").trim();
        let summary = parts.get(4).unwrap_or(&"").trim();
        let description = parts.get(5).unwrap_or(&"").trim();
        let size_str = parts.get(6).unwrap_or(&"").trim();

        // Parse size
        let size = if !size_str.is_empty() {
            if let Ok(size_bytes) = parse_size(size_str) {
                format_size(size_bytes)
            } else {
                size_str.to_string()
            }
        } else {
            String::new()
        };

        packages.push(PackageInfo {
            name: name.to_string(),
            description: if !description.is_empty() {
                description.to_string()
            } else {
                summary.to_string()
            },
            version: version.to_string(),
            release: release.to_string(),
            arch: arch.to_string(),
            summary: if !summary.is_empty() {
                summary.to_string()
            } else if !description.is_empty() {
                // Use first part of description as summary
                let summary_len = description.len().min(100);
                description[..summary_len].to_string()
            } else {
                String::new()
            },
            size,
        });

        // Limit to first 50 packages for performance
        if packages.len() >= 50 {
            break;
        }
    }
    
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

#[allow(dead_code)]
async fn load_package_details(package_name: String) -> Result<PackageInfo, String> {
    // Use dnf info to get detailed package information
    let output = tokio::process::Command::new("dnf")
        .args(["info", &package_name])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf: {}", e))?;

    if !output.status.success() {
        return Err(format!("Failed to read package info: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = PackageInfo {
        name: package_name.clone(),
        description: String::new(),
        version: String::new(),
        release: String::new(),
        arch: String::new(),
        summary: String::new(),
        size: String::new(),
    };

    let mut description_lines = Vec::new();
    let mut in_description = false;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name") && line.contains(':') {
            let name = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !name.is_empty() {
                info.name = name.to_string();
            }
            in_description = false;
        } else if line.starts_with("Version") && line.contains(':') {
            info.version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Release") && line.contains(':') {
            info.release = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Architecture") && line.contains(':') {
            info.arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if line.starts_with("Summary") && line.contains(':') {
            info.summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            in_description = false;
        } else if (line.starts_with("Installed size") || line.starts_with("Download size") || line.starts_with("Size")) && line.contains(':') {
            let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if line.starts_with("Installed size") || info.size.is_empty() {
                if let Ok(size_bytes) = parse_size(size_str) {
                    info.size = format_size(size_bytes);
                } else {
                    info.size = size_str.to_string();
                }
            }
            in_description = false;
        } else if line.starts_with("Description") && line.contains(':') {
            in_description = true;
            let desc = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            if !desc.is_empty() {
                description_lines.push(desc.to_string());
            }
        } else if in_description {
            if line.starts_with("               :") {
                let desc_cont = line.trim_start_matches("               :").trim();
                if !desc_cont.is_empty() {
                    description_lines.push(desc_cont.to_string());
                }
            } else if line.contains(':') {
                let field_name = line.split(':').next().unwrap_or("").trim();
                let known_fields = ["URL", "License", "Vendor", "Source", "Repository", "Epoch"];
                if known_fields.iter().any(|&f| field_name.starts_with(f)) ||
                   (field_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && 
                    field_name.len() < 20 &&
                    !field_name.eq_ignore_ascii_case("description")) {
                    in_description = false;
                } else {
                    description_lines.push(line.to_string());
                }
            } else {
                description_lines.push(line.to_string());
            }
        }
    }

    info.description = description_lines.join(" ").trim().to_string();
    if info.description.is_empty() {
        info.description = info.summary.clone();
    }
    if info.summary.is_empty() && !info.description.is_empty() {
        // Use first part of description as summary if summary is empty
        let summary_len = info.description.len().min(100);
        info.summary = info.description[..summary_len].to_string();
    }

    Ok(info)
}

fn parse_size(size_str: &str) -> Result<u64, ()> {
    let size_str = size_str.trim();
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(());
    }
    
    let number: f64 = parts[0].parse().map_err(|_| ())?;
    let unit = if parts.len() > 1 {
        parts[1].to_lowercase()
    } else {
        "b".to_string()
    };
    
    let multiplier = match unit.as_str() {
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    
    Ok((number * multiplier) as u64)
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}


struct PackageCardStyle {
    is_selected: bool,
}

impl iced::widget::container::StyleSheet for PackageCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(if self.is_selected {
                iced::Color::from_rgba(palette.primary.r, palette.primary.g, palette.primary.b, 0.15)
            } else {
                iced::Color::from_rgba(
                    palette.background.r * 0.98,
                    palette.background.g * 0.98,
                    palette.background.b * 0.98,
                    1.0,
                )
            })),
            border: Border {
                radius: 12.0.into(),
                width: if self.is_selected { 2.0 } else { 1.0 },
                color: if self.is_selected {
                    palette.primary
                } else {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15)
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

