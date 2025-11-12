use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space, image};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::checkbox::Appearance as CheckboxAppearance;
use iced::widget::checkbox::StyleSheet as CheckboxStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone)]
pub enum Message {
    LoadPackages,
    PackagesLoaded(Vec<PackageInfo>),
    SearchQueryChanged(String),
    TogglePackage(String),
    PackageSelected(String),
    PackageDetailsLoaded(PackageDetails),
    ClosePanel,
    RemoveSelected,
    RemoveComplete,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub repository: String,
}

#[derive(Debug, Clone)]
pub struct PackageDetails {
    pub name: String,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub summary: String,
    pub description: String,
    pub size: String,
    pub icon_path: Option<String>,
}

#[derive(Debug)]
pub struct InstalledTab {
    packages: Vec<PackageInfo>,
    filtered_packages: Vec<PackageInfo>,
    search_query: String,
    selected_packages: std::collections::HashSet<String>,
    is_loading: bool,
    is_removing: bool,
    selected_package: Option<String>,
    package_details: Option<PackageDetails>,
    panel_open: bool,
}

impl InstalledTab {
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            filtered_packages: Vec::new(),
            search_query: String::new(),
            selected_packages: std::collections::HashSet::new(),
            is_loading: false,
            is_removing: false,
            selected_package: None,
            package_details: None,
            panel_open: false,
        }
    }

    fn filter_packages(&mut self) {
        if self.search_query.trim().is_empty() {
            self.filtered_packages = self.packages.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_packages = self.packages
                .iter()
                .filter(|pkg| {
                    pkg.name.to_lowercase().contains(&query_lower) ||
                    pkg.version.to_lowercase().contains(&query_lower) ||
                    pkg.repository.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect();
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::LoadPackages => {
                self.is_loading = true;
                iced::Command::perform(load_installed_packages(), |result| {
                    match result {
                        Ok(packages) => Message::PackagesLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::PackagesLoaded(packages) => {
                self.is_loading = false;
                self.packages = packages;
                self.filter_packages();
                iced::Command::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.filter_packages();
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
            Message::PackageSelected(name) => {
                self.selected_package = Some(name.clone());
                self.panel_open = true;
                iced::Command::perform(load_package_details(name), Message::PackageDetailsLoaded)
            }
            Message::PackageDetailsLoaded(details) => {
                self.package_details = Some(details);
                iced::Command::none()
            }
            Message::ClosePanel => {
                self.panel_open = false;
                self.selected_package = None;
                self.package_details = None;
                iced::Command::none()
            }
            Message::RemoveSelected => {
                if self.selected_packages.is_empty() {
                    return iced::Command::none();
                }
                // Spawn a separate window for package removal
                let packages: Vec<String> = self.selected_packages.iter().cloned().collect();
                let _packages_str = packages.join(" ");
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("fedoraforge"));
                        TokioCommand::new(&exe_path)
                            .arg("remove-dialog")
                            .args(packages)
                            .spawn()
                            .ok();
                    },
                    |_| Message::RemoveComplete, // Dummy message
                )
            }
            Message::RemoveComplete => {
                // Refresh the package list after removal dialog closes
                self.selected_packages.clear();
                iced::Command::perform(load_installed_packages(), |result| {
                    match result {
                        Ok(packages) => Message::PackagesLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::Error(_msg) => {
                // Error occurred, just reset state
                self.is_removing = false;
                iced::Command::none()
            }
        }
    }

    fn view_panel(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        if let Some(ref details) = self.package_details {
            // Icon widget - try to load actual icon
            let icon_widget = if let Some(ref icon_path) = details.icon_path {
                if std::path::Path::new(icon_path).exists() {
                    let icon_handle = iced::widget::image::Handle::from_path(icon_path);
                    container(
                        image(icon_handle)
                            .width(Length::Fixed(96.0))
                            .height(Length::Fixed(96.0))
                            .content_fit(iced::ContentFit::Contain)
                    )
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(120.0))
                    .center_x()
                    .center_y()
                    .style(iced::theme::Container::Custom(Box::new(IconContainerStyle)))
                } else {
                    container(
                        text("📦")
                            .size(48)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(120.0))
                    .center_x()
                    .center_y()
                    .style(iced::theme::Container::Custom(Box::new(IconContainerStyle)))
                }
            } else {
                container(
                    text("📦")
                        .size(48)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                )
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(IconContainerStyle)))
            };

            container(
                scrollable(
                    column![
                        // Header with close button
                        row![
                            text("Package Details").size(16).style(iced::theme::Text::Color(theme.primary())),
                            Space::with_width(Length::Fill),
                            {
                                let material_font = crate::gui::fonts::get_material_symbols_font();
                                button(
                                    text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(18)
                                )
                                    .on_press(Message::ClosePanel)
                                    .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
                                    .padding(Padding::new(6.0))
                            },
                        ]
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        // Icon
                        icon_widget,
                        Space::with_height(Length::Fixed(20.0)),
                        // Package name
                        text(&details.name)
                            .size(22) // Larger size for emphasis
                            .style(iced::theme::Text::Color(theme.text())) // Darker for better visibility
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        // Package details
                        container(
                            column![
                                row![
                                    text("Version:").size(13).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary())),
                                    text(&details.version).size(13).width(Length::Fill),
                                ]
                                .spacing(12),
                                Space::with_height(Length::Fixed(8.0)),
                                row![
                                    text("Release:").size(13).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary())),
                                    text(&details.release).size(13).width(Length::Fill),
                                ]
                                .spacing(12),
                                Space::with_height(Length::Fixed(8.0)),
                                row![
                                    text("Architecture:").size(13).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary())),
                                    text(&details.arch).size(13).width(Length::Fill),
                                ]
                                .spacing(12),
                                Space::with_height(Length::Fixed(8.0)),
                                row![
                                    text("Size:").size(13).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary())),
                                    text(&details.size).size(13).width(Length::Fill),
                                ]
                                .spacing(12),
                            ]
                            .spacing(0)
                        )
                        .padding(Padding::new(18.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle))),
                        Space::with_height(Length::Fixed(20.0)),
                        // Summary
                        text("Summary").size(15).style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        text(&details.summary).size(13),
                        Space::with_height(Length::Fixed(20.0)),
                        // Description
                        text("Description").size(15).style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        text(&details.description).size(13).width(Length::Fill),
                    ]
                    .spacing(0)
                    .padding(Padding::new(25.0))
                )
                .height(Length::Fill)
            )
            .width(Length::Fixed(420.0))
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(PanelStyle)))
            .into()
        } else {
            container(
                column![
                    row![
                        Space::with_width(Length::Fill),
                        {
                            let material_font = crate::gui::fonts::get_material_symbols_font();
                            button(
                                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(20)
                            )
                                .on_press(Message::ClosePanel)
                                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                    is_primary: false,
                                })))
                                .padding(Padding::new(8.0))
                        },
                    ]
                    .width(Length::Fill),
                    Space::with_height(Length::Fill),
                    text("Loading...").size(16).horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fill),
                ]
                .padding(Padding::new(20.0))
            )
            .width(Length::Fixed(400.0))
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(PanelStyle)))
            .into()
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let search_input = text_input("Search installed packages...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .size(16)
            .width(Length::Fill)
            .padding(14)
            .style(iced::theme::TextInput::Custom(Box::new(RoundedTextInputStyle)));

        let material_font = crate::gui::fonts::get_material_symbols_font();
        let refresh_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font),
                text(" Refresh")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::LoadPackages)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::new(14.0));

        let remove_button = if self.selected_packages.is_empty() {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font),
                    text(" Remove Selected")
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
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font),
                    text(format!(" Remove {} Package(s)", self.selected_packages.len()))
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
                .on_press(Message::RemoveSelected)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                })))
                .padding(Padding::new(14.0))
        };

        let header = row![refresh_button, Space::with_width(Length::Fill), remove_button]
            .spacing(10)
            .align_items(Alignment::Center);

        let content: Element<Message> = if self.is_loading {
            container(text("Loading installed packages...").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
                .into()
        } else {
            let package_list: Element<Message> = if self.packages.is_empty() {
                container(text("No packages found").size(14))
                    .width(Length::Fill)
                    .padding(20)
                    .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
                    .into()
            } else if self.filtered_packages.is_empty() && !self.search_query.trim().is_empty() {
                container(text(format!("No packages found matching '{}'", self.search_query)).size(14))
                    .width(Length::Fill)
                    .padding(20)
                    .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
                    .into()
            } else {
                scrollable(
                    column(
                        self.filtered_packages
                            .iter()
                            .map(|pkg| {
                                let pkg_name_for_toggle = pkg.name.clone();
                                let is_selected = self.selected_packages.contains(&pkg.name);
                                let checkbox_widget = checkbox("", is_selected)
                                    .on_toggle(move |_| Message::TogglePackage(pkg_name_for_toggle.clone()))
                                    .style(iced::theme::Checkbox::Custom(Box::new(RoundedCheckboxStyle)));
                                let pkg_name_for_click = pkg.name.clone();
                                button(
                                    container(
                                        row![
                                            checkbox_widget,
                                            text(&pkg.name)
                                                .size(17) // Larger size for emphasis
                                                .style(iced::theme::Text::Color(theme.text())) // Darker for better visibility
                                                .width(Length::FillPortion(3)),
                                            text(&pkg.version).size(14).width(Length::FillPortion(2)),
                                            text(&pkg.repository).size(14).width(Length::FillPortion(2)),
                                        ]
                                        .spacing(12)
                                        .align_items(Alignment::Center)
                                        .padding(12)
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(PackageItemStyle {
                                        is_selected,
                                    })))
                                )
                                .on_press(Message::PackageSelected(pkg_name_for_click))
                                .style(iced::theme::Button::Text)
                                .padding(0)
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(6)
                    .padding(10),
                )
                .into()
            };
            column![header, package_list].spacing(10).into()
        };

        // Create the slide-out panel
        let panel = if self.panel_open {
            self.view_panel(theme)
        } else {
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(0.0))
                .height(Length::Fill)
                .into()
        };

        let main_content = container(column![search_input, content].spacing(15).padding(20))
            .width(Length::Fill)
            .height(Length::Fill);

        container(
            row![main_content, panel]
                .spacing(15)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(15.0))
        .into()
    }
}

async fn load_installed_packages() -> Result<Vec<PackageInfo>, String> {
    let output = TokioCommand::new("dnf")
        .args(["list", "--installed", "--quiet", "--assumeyes"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf: {}", e))?;

    if !output.status.success() {
        return Err(format!("DNF list failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line == "Installed packages" {
            continue;
        }
        // Format: "package.arch  version  repository"
        // Split by whitespace but handle multiple spaces
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0].split('.').next().unwrap_or(parts[0]);
            packages.push(PackageInfo {
                name: name.to_string(),
                version: parts[1].to_string(),
                repository: parts[2].to_string(),
            });
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

async fn load_package_details(package_name: String) -> PackageDetails {
    // Get package info from rpm -qi
    let rpm_output = TokioCommand::new("rpm")
        .args(["-qi", &package_name])
        .output()
        .await;

    let mut name = package_name.clone();
    let mut version = String::new();
    let mut release = String::new();
    let mut arch = String::new();
    let mut summary = String::new();
    let mut description = String::new();
    let mut size = String::new();
    let mut icon_path: Option<String> = None;

    if let Ok(output) = rpm_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut in_description = false;
            let mut description_lines = Vec::new();

            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("Name        :") {
                    name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Version     :") {
                    version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Release     :") {
                    release = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Architecture:") {
                    arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Summary     :") {
                    summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Size        :") {
                    let size_str = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                    let size_bytes: u64 = size_str.parse().unwrap_or(0);
                    size = format_size(size_bytes);
                } else if line.starts_with("Description :") {
                    in_description = true;
                    let desc = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                    if !desc.is_empty() {
                        description_lines.push(desc.to_string());
                    }
                } else if in_description {
                    if line.is_empty() || (line.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && line.ends_with(':') && line.len() < 20) {
                        in_description = false;
                    } else {
                        description_lines.push(line.to_string());
                    }
                }
            }

            if !description_lines.is_empty() {
                description = description_lines.join(" ").trim().to_string();
            }
            if description.is_empty() {
                description = summary.clone();
            }
            if summary.is_empty() {
                summary = format!("Package: {}", name);
            }
        }
    }

    // Try to find .desktop file and icon
    let desktop_output = TokioCommand::new("rpm")
        .args(["-ql", &package_name])
        .output()
        .await;

    if let Ok(output) = desktop_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().ends_with(".desktop") {
                    // Found a desktop file, try to read it
                    if let Ok(desktop_content) = std::fs::read_to_string(line.trim()) {
                        for desktop_line in desktop_content.lines() {
                            if desktop_line.trim().starts_with("Icon=") {
                                let icon_value = desktop_line.splitn(2, '=').nth(1).unwrap_or("").trim();
                                // Try to resolve icon path
                                if icon_value.starts_with('/') {
                                    // Absolute path
                                    if std::path::Path::new(icon_value).exists() {
                                        icon_path = Some(icon_value.to_string());
                                    }
                                } else {
                                    // Icon name - try common locations using gtk-update-icon-cache paths
                                    let icon_dirs = [
                                        "/usr/share/icons/hicolor/256x256/apps",
                                        "/usr/share/icons/hicolor/128x128/apps",
                                        "/usr/share/icons/hicolor/96x96/apps",
                                        "/usr/share/icons/hicolor/64x64/apps",
                                        "/usr/share/icons/hicolor/48x48/apps",
                                        "/usr/share/icons/hicolor/32x32/apps",
                                        "/usr/share/pixmaps",
                                        "/usr/share/applications",
                                    ];
                                    for dir in &icon_dirs {
                                        for ext in &["png", "svg", "xpm", "ico"] {
                                            let icon_file = format!("{}/{}.{}", dir, icon_value, ext);
                                            if std::path::Path::new(&icon_file).exists() {
                                                icon_path = Some(icon_file);
                                                break;
                                            }
                                        }
                                        if icon_path.is_some() {
                                            break;
                                        }
                                    }
                                    // Also try without extension in pixmaps
                                    if icon_path.is_none() {
                                        let pixmap_file = format!("/usr/share/pixmaps/{}", icon_value);
                                        if std::path::Path::new(&pixmap_file).exists() {
                                            icon_path = Some(pixmap_file);
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    PackageDetails {
        name,
        version,
        release,
        arch,
        summary,
        description,
        size,
        icon_path,
    }
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

#[allow(dead_code)]
async fn remove_packages(packages: Vec<String>) -> Result<(), String> {
    let status = TokioCommand::new("sudo")
        .arg("dnf")
        .arg("remove")
        .arg("-y")
        .args(&packages)
        .status()
        .await
        .map_err(|e| format!("Failed to execute dnf remove: {}", e))?;

    if !status.success() {
        return Err("Package removal failed".to_string());
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

struct RoundedTextInputStyle;

impl TextInputStyleSheet for RoundedTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        TextInputAppearance {
            background: iced::Background::Color(palette.background),
            border: Border {
                radius: 16.0.into(),
                width: 2.0,
                color: palette.primary,
            },
            icon_color: palette.text,
        }
    }

    fn focused(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        TextInputAppearance {
            background: iced::Background::Color(palette.background),
            border: Border {
                radius: 16.0.into(),
                width: 2.0,
                color: palette.primary,
            },
            icon_color: palette.primary,
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

    fn disabled(&self, _style: &Self::Style) -> TextInputAppearance {
        TextInputAppearance {
            background: iced::Background::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1)),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            icon_color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        }
    }
}

struct InfoContainerStyle;

impl iced::widget::container::StyleSheet for InfoContainerStyle {
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
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}

struct PanelStyle;

impl iced::widget::container::StyleSheet for PanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.97,
                palette.background.g * 0.97,
                palette.background.b * 0.97,
                1.0,
            ))),
            border: Border {
                radius: 20.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct IconContainerStyle;

impl iced::widget::container::StyleSheet for IconContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.9,
                palette.background.g * 0.9,
                palette.background.b * 0.9,
                1.0,
            ))),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
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
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            text_color: palette.text,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Background::Color(iced::Color::from_rgba(0.7, 0.2, 0.2, 0.2)));
        appearance.border.color = iced::Color::from_rgba(0.7, 0.2, 0.2, 0.5);
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

