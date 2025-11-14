use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use crate::gui::update_settings_dialog::UpdateSettings;
use serde_json;

#[derive(Debug, Clone)]
pub enum Message {
    LoadPackageInfo,
    PackageInfoLoaded(Vec<UpdateInfo>),
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

#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    Pending,
    Installing,
    Installed,
    Failed,
}

#[derive(Debug)]
pub struct UpdateDialog {
    updates: Vec<UpdateInfo>,
    packages_to_install: Vec<String>, // Specific packages to install (empty = all)
    packages_with_info: Vec<UpdateInfo>, // Packages to install with their update info
    package_status: std::collections::HashMap<String, InstallStatus>, // Track installation status per package
    is_loading_info: bool,
    is_installing: bool,
    is_complete: bool,
    installation_progress: String,
    terminal_output: String,
    show_dialog: bool,
}

impl UpdateDialog {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::new_with_packages(Vec::new())
    }
    
    pub fn new_with_packages(packages: Vec<String>) -> Self {
        Self {
            updates: Vec::new(),
            packages_to_install: packages.clone(),
            packages_with_info: Vec::new(),
            package_status: packages.iter().map(|p| (p.clone(), InstallStatus::Pending)).collect(),
            is_loading_info: true,
            is_installing: false,
            is_complete: false,
            installation_progress: String::new(),
            terminal_output: String::new(),
            show_dialog: true,
        }
    }

    pub fn run_separate_window(packages_b64: Option<String>) -> Result<(), iced::Error> {
        // Decode packages if provided
        let packages: Vec<String> = if let Some(b64) = packages_b64 {
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(decoded) = general_purpose::STANDARD.decode(&b64) {
                if let Ok(json_str) = String::from_utf8(decoded) {
                    serde_json::from_str(&json_str).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        let dialog = Self::new_with_packages(packages);
        
        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(1000.0, 700.0);
        window_settings.min_size = Some(iced::Size::new(800.0, 500.0));
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

        let content = if self.is_loading_info {
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
            // Show installation progress with packages list and terminal output
            let progress_text = if self.installation_progress.is_empty() {
                "Installing updates...".to_string()
            } else {
                self.installation_progress.clone()
            };

            // Show packages being installed with status
            let packages_list: Element<Message> = if !self.packages_with_info.is_empty() {
                scrollable(
                    column(
                        self.packages_with_info
                            .iter()
                            .map(|update| {
                                let status = self.package_status.get(&update.name)
                                    .cloned()
                                    .unwrap_or(InstallStatus::Pending);
                                let status_text = match status {
                                    InstallStatus::Pending => ("⏳", "Pending", iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0)),
                                    InstallStatus::Installing => ("⏳", "Installing...", theme.primary()),
                                    InstallStatus::Installed => ("✓", "Installed", iced::Color::from_rgb(0.0, 0.8, 0.0)),
                                    InstallStatus::Failed => ("✗", "Failed", iced::Color::from_rgb(0.8, 0.0, 0.0)),
                                };
                                
                                container(
                                    row![
                                        text(status_text.0).size(18).style(iced::theme::Text::Color(status_text.2)),
                                        text(&update.name).size(16).width(Length::FillPortion(3)),
                                        text(&update.current_version).size(14).width(Length::FillPortion(2)),
                                        text("→").size(14),
                                        text(&update.available_version).size(14).width(Length::FillPortion(2)),
                                        text(status_text.1).size(12).style(iced::theme::Text::Color(status_text.2)).width(Length::FillPortion(2)),
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
                .height(Length::Fixed(200.0))
                .into()
            } else {
                Space::with_height(Length::Fixed(0.0)).into()
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
                    if !self.packages_with_info.is_empty() {
                        let packages_column: Element<Message> = column![
                            text("Packages:").size(14).style(iced::theme::Text::Color(theme.primary())),
                            packages_list,
                        ]
                        .spacing(8)
                        .into();
                        packages_column
                    } else {
                        Space::with_height(Length::Fixed(0.0)).into()
                    },
                    Space::with_height(Length::Fixed(8.0)),
                    text("Output:").size(14).style(iced::theme::Text::Color(theme.primary())),
                    terminal_scroll,
                ]
                .spacing(8)
                .padding(Padding::new(16.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle)))
        } else if self.is_complete {
            // Show installed packages list
            let packages_list: Element<Message> = if !self.packages_with_info.is_empty() {
                scrollable(
                    column(
                        self.packages_with_info
                            .iter()
                            .map(|update| {
                                let status = self.package_status.get(&update.name)
                                    .cloned()
                                    .unwrap_or(InstallStatus::Installed);
                                let status_text = match status {
                                    InstallStatus::Installed => ("✓", "Installed", iced::Color::from_rgb(0.0, 0.8, 0.0)),
                                    InstallStatus::Failed => ("✗", "Failed", iced::Color::from_rgb(0.8, 0.0, 0.0)),
                                    _ => ("?", "Unknown", iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0)),
                                };
                                
                                container(
                                    row![
                                        text(status_text.0).size(18).style(iced::theme::Text::Color(status_text.2)),
                                        text(&update.name).size(16).width(Length::FillPortion(3)),
                                        text(&update.current_version).size(14).width(Length::FillPortion(2)),
                                        text("→").size(14),
                                        text(&update.available_version).size(14).width(Length::FillPortion(2)),
                                        text(status_text.1).size(12).style(iced::theme::Text::Color(status_text.2)).width(Length::FillPortion(2)),
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
                .height(Length::Fixed(250.0))
                .into()
            } else {
                Space::with_height(Length::Fixed(0.0)).into()
            };

            // Check if any packages failed
            let has_failed = self.package_status.values().any(|s| *s == InstallStatus::Failed);
            let title_text = if has_failed {
                "Installation Failed".to_string()
            } else {
                "Updates Installed Successfully!".to_string()
            };
            let title_color = if has_failed {
                iced::Color::from_rgb(0.8, 0.0, 0.0)
            } else {
                iced::Color::from_rgb(0.0, 0.8, 0.0)
            };
            
            container(
                column![
                    text(&title_text).size(18).style(iced::theme::Text::Color(title_color)),
                    Space::with_height(Length::Fixed(20.0)),
                    if !self.packages_with_info.is_empty() {
                        let packages_column: Element<Message> = column![
                            text("Installed Packages:").size(14).style(iced::theme::Text::Color(theme.primary())),
                            packages_list,
                        ]
                        .spacing(8)
                        .into();
                        packages_column
                    } else {
                        Space::with_height(Length::Fixed(0.0)).into()
                    },
                    Space::with_height(Length::Fixed(10.0)),
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
                            .height(Length::Fixed(200.0))
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
            // Show list of packages to install with install button
            let packages_to_show = if !self.packages_with_info.is_empty() {
                &self.packages_with_info
            } else {
                &self.updates
            };
            
            let title = container(
                text(format!("{} Package(s) to Install", packages_to_show.len()))
                    .size(20)
                    .style(iced::theme::Text::Color(theme.primary()))
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0));

            let packages_list: Element<Message> = scrollable(
                column(
                    packages_to_show
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
                    text(format!(" Install {} Package(s)", packages_to_show.len()))
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
                    packages_list,
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
        // Load package info first, then show them before installation
        let cmd = dialog.update(Message::LoadPackageInfo);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        "System Updates - Rustora".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPackageInfo => {
                let packages = self.packages_to_install.clone();
                // If no packages specified, we'll show all available updates
                // Otherwise, load info for specific packages
                if packages.is_empty() {
                    // Load all available updates
                    iced::Command::perform(load_all_updates(), |result| {
                        Message::PackageInfoLoaded(result.unwrap_or_default())
                    })
                } else {
                    // Load info for specific packages
                    iced::Command::perform(load_package_update_info(packages), |result| {
                        Message::PackageInfoLoaded(result.unwrap_or_default())
                    })
                }
            }
            Message::PackageInfoLoaded(updates) => {
                self.packages_with_info = updates.clone();
                self.is_loading_info = false;
                // Initialize status for all packages
                for update in &updates {
                    self.package_status.insert(update.name.clone(), InstallStatus::Pending);
                }
                // Auto-start installation after showing packages briefly
                iced::Command::perform(async {}, |_| Message::InstallUpdates)
            }
            Message::InstallUpdates => {
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                self.terminal_output = String::new();
                
                // Determine which packages to install
                // Priority: use packages_with_info if available (they were verified to have updates)
                // Otherwise fall back to packages_to_install (the original selected packages)
                let packages: Vec<String> = if !self.packages_with_info.is_empty() {
                    // Use packages that were found in check-update (verified to have updates)
                    self.packages_with_info.iter().map(|u| u.name.clone()).collect()
                } else if !self.packages_to_install.is_empty() {
                    // Fall back to originally selected packages if packages_with_info is empty
                    // This handles the case where packages weren't found in check-update
                    eprintln!("[DEBUG] Using original packages_to_install: {:?}", self.packages_to_install);
                    self.packages_to_install.clone()
                } else {
                    // No packages specified - this shouldn't happen
                    eprintln!("[DEBUG] WARNING: No packages to install!");
                    return iced::Command::perform(async {}, |_| Message::InstallationError("No packages selected for installation".to_string()));
                };
                
                if packages.is_empty() {
                    return iced::Command::perform(async {}, |_| Message::InstallationError("No packages available for installation".to_string()));
                }
                
                eprintln!("[DEBUG] Installing packages: {:?}", packages);
                
                // Mark all packages as installing
                for pkg in &packages {
                    self.package_status.insert(pkg.clone(), InstallStatus::Installing);
                }
                
                let packages_with_info = self.packages_with_info.clone();
                let package_names: std::collections::HashSet<String> = packages.iter().cloned().collect();
                iced::Command::perform(install_updates_streaming(packages, packages_with_info, package_names), |result| {
                    match result {
                        Ok(output) => Message::InstallationProgress(output),
                        Err(e) => Message::InstallationError(e),
                    }
                })
            }
            Message::InstallationProgress(output) => {
                // Always append the output, even if it contains errors
                // This ensures users can see what happened
                if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                    self.terminal_output.push('\n');
                }
                // Only append new content that we haven't seen
                let current_output = self.terminal_output.clone();
                let new_content: String = output
                    .lines()
                    .filter(|line| !current_output.contains(line))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                if !new_content.is_empty() {
                    self.terminal_output.push_str(&new_content);
                    if !self.terminal_output.ends_with('\n') {
                        self.terminal_output.push('\n');
                    }
                }
                
                // Parse output line by line to track individual package progress
                let lines: Vec<&str> = output.lines().collect();
                let mut new_lines = Vec::new();
                
                for line in lines {
                    // Skip if we've already processed this line
                    if new_lines.contains(&line) {
                        continue;
                    }
                    new_lines.push(line);
                    
                    let line_lower = line.to_lowercase();
                    
                    // Parse dnf output patterns to detect package installation
                    // DNF output formats:
                    // - "Installing: package-name-version.arch"
                    // - "Upgrading: package-name-version.arch"  
                    // - "Installed: package-name-version.arch"
                    // - "Complete!"
                    // - Progress: "[====>  ] 50% package-name"
                    
                    // Extract package name from dnf output lines
                    // Format: "Installing: package-name-version.arch" or "Upgrading: package-name-version.arch"
                    if line_lower.starts_with("installing:") || 
                       line_lower.starts_with("upgrading:") ||
                       line_lower.starts_with("installed:") {
                        // Extract package name after the colon
                        if let Some(colon_pos) = line.find(':') {
                            let after_colon = line[colon_pos + 1..].trim();
                            // Get the first word (package name with version)
                            let pkg_with_version = after_colon.split_whitespace().next().unwrap_or("");
                            // Remove architecture suffix (e.g., ".x86_64")
                            let pkg_name_part = pkg_with_version.split('.').next().unwrap_or(pkg_with_version);
                            // Try to match with our package names
                            // Package names in dnf output are usually "base-name-version", we need to match "base-name"
                            for (pkg_name, status) in &mut self.package_status {
                                let pkg_lower = pkg_name.to_lowercase();
                                // Check if the package name matches (handle version suffixes)
                                if pkg_name_part.to_lowercase().starts_with(&pkg_lower) || 
                                   pkg_lower == pkg_name_part.to_lowercase().split('-').next().unwrap_or("") {
                                    if line_lower.starts_with("installed:") {
                                        *status = InstallStatus::Installed;
                                    } else if line_lower.starts_with("installing:") || 
                                              line_lower.starts_with("upgrading:") {
                                        *status = InstallStatus::Installing;
                                    }
                                }
                            }
                        }
                    }
                    
                    // Also check for package mentions in progress lines and other contexts
                    for (pkg_name, status) in &mut self.package_status {
                        let pkg_lower = pkg_name.to_lowercase();
                        
                        // Check if this line mentions the package (more flexible matching)
                        if line_lower.contains(&pkg_lower) || 
                           (pkg_lower.len() > 3 && line_lower.contains(&pkg_lower[..pkg_lower.len().min(10)])) {
                            // Detect installation completion
                            if line_lower.contains("installed") || 
                               line_lower.contains("upgraded") || 
                               (line_lower.contains("package") && line_lower.contains("already installed")) ||
                               (line_lower.contains("complete") && line_lower.contains(&pkg_lower)) {
                                *status = InstallStatus::Installed;
                            } 
                            // Detect errors
                            else if line_lower.contains("error") || 
                                    line_lower.contains("failed") ||
                                    line_lower.contains("cannot") ||
                                    line_lower.contains("dependency") {
                                *status = InstallStatus::Failed;
                            }
                            // Detect active installation (if not already installed)
                            else if (*status != InstallStatus::Installed) && 
                                    (line_lower.contains("installing") || 
                                     line_lower.contains("upgrading") ||
                                     line_lower.contains("downloading") ||
                                     line_lower.contains("verifying")) {
                                *status = InstallStatus::Installing;
                            }
                        }
                    }
                    
                    // Update overall progress text
                    if line_lower.contains("downloading") {
                        self.installation_progress = "Downloading packages...".to_string();
                    } else if line_lower.contains("installing") || line_lower.contains("upgrading") {
                        // Try to extract which package is being installed
                        for pkg_name in self.package_status.keys() {
                            if line_lower.contains(&pkg_name.to_lowercase()) {
                                self.installation_progress = format!("Installing {}...", pkg_name);
                                break;
                            }
                        }
                        if self.installation_progress == "Preparing installation..." {
                            self.installation_progress = "Installing packages...".to_string();
                        }
                    } else if line_lower.contains("verifying") {
                        self.installation_progress = "Verifying packages...".to_string();
                    } else if line_lower.contains("complete") || line_lower.contains("finished") {
                        self.installation_progress = "Installation complete!".to_string();
                    }
                }
                
                // Append new lines to terminal output
                if !new_lines.is_empty() {
                    if !self.terminal_output.is_empty() {
                        self.terminal_output.push('\n');
                    }
                    self.terminal_output.push_str(&new_lines.join("\n"));
                }
                
                // Check if we should mark as complete
                let output_lower = output.to_lowercase();
                if output_lower.contains("complete") || 
                   output_lower.contains("finished") ||
                   output_lower.contains("nothing to do") ||
                   output_lower.contains("no packages marked") {
                    // Mark all remaining packages as installed
                    for status in self.package_status.values_mut() {
                        if *status == InstallStatus::Installing || *status == InstallStatus::Pending {
                            *status = InstallStatus::Installed;
                        }
                    }
                    iced::Command::perform(async {}, |_| Message::InstallationComplete)
                } else {
                    iced::Command::none()
                }
            }
            Message::InstallationComplete => {
                self.is_installing = false;
                self.is_complete = true;
                self.installation_progress = "Installation completed successfully!".to_string();
                // Mark any remaining installing packages as installed
                for status in self.package_status.values_mut() {
                    if *status == InstallStatus::Installing || *status == InstallStatus::Pending {
                        *status = InstallStatus::Installed;
                    }
                }
                if !self.terminal_output.contains("completed successfully") {
                    self.terminal_output.push_str("\n✓ Installation completed successfully!");
                }
                iced::Command::none()
            }
            Message::InstallationError(msg) => {
                self.is_installing = false;
                self.is_loading_info = false;
                self.is_complete = true; // Mark as complete so UI shows the error state
                // Mark all packages as failed
                for status in self.package_status.values_mut() {
                    if *status == InstallStatus::Installing || *status == InstallStatus::Pending {
                        *status = InstallStatus::Failed;
                    }
                }
                // The error message contains the full output after the error prefix
                // Extract the output part and set it as terminal_output
                if msg.contains("Update installation failed") || msg.contains("encountered errors") {
                    // The format is: "Update installation failed (exit code: X):\n\n<output>"
                    // Extract everything after the first "\n\n" as that's the actual dnf output
                    if let Some(output_start) = msg.find("\n\n") {
                        let dnf_output = &msg[output_start + 2..];
                        // Set terminal_output to show the dnf output
                        if self.terminal_output.is_empty() {
                            self.terminal_output = dnf_output.to_string();
                        } else {
                            // Append if we already have some output
                            if !self.terminal_output.ends_with('\n') {
                                self.terminal_output.push('\n');
                            }
                            self.terminal_output.push_str(dnf_output);
                        }
                        // Add error header at the beginning
                        self.terminal_output = format!("=== ERROR ===\nExit code indicates failure\n\n{}", self.terminal_output);
                    } else {
                        // Fallback: use the whole message
                        self.terminal_output = msg.clone();
                    }
                } else {
                    // Simple error message, just append it
                    if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                        self.terminal_output.push('\n');
                    }
                    self.terminal_output.push_str(&format!("\n=== ERROR ===\n{}", msg));
                }
                eprintln!("[ERROR] Installation failed: {}", msg);
                eprintln!("[ERROR] Terminal output length: {} bytes", self.terminal_output.len());
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

async fn load_all_updates() -> Result<Vec<UpdateInfo>, String> {
    // Get all available updates
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

async fn load_package_update_info(packages: Vec<String>) -> Result<Vec<UpdateInfo>, String> {
    eprintln!("[DEBUG] load_package_update_info called with {} packages: {:?}", packages.len(), packages);
    
    // Get all available updates
    let output = TokioCommand::new("dnf")
        .args(["check-update", "--quiet"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf check-update: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    eprintln!("[DEBUG] dnf check-update output length: {} bytes", stdout.len());
    
    // If stdout is empty, no updates available
    if stdout.trim().is_empty() {
        eprintln!("[DEBUG] No updates available according to dnf check-update");
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

    // Create a set of package names to filter (case-insensitive)
    let packages_set: std::collections::HashSet<String> = packages.iter()
        .map(|p| p.to_lowercase())
        .collect();

    eprintln!("[DEBUG] Looking for packages: {:?}", packages_set);

    let mut updates = Vec::new();
    let mut found_packages = std::collections::HashSet::new();
    
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
            let name_lower = name.to_lowercase();
            
            // Only include packages that are in our list
            if packages_set.contains(&name_lower) {
                let available_version = parts[1].to_string();
                let repository = parts[2].to_string();
                
                // Get current version from installed packages
                let current_version = installed_versions
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());
                
                eprintln!("[DEBUG] Found package in updates: {} ({} -> {})", name, current_version, available_version);
                
                updates.push(UpdateInfo {
                    name: name.to_string(),
                    current_version,
                    available_version,
                    repository,
                });
                found_packages.insert(name_lower);
            }
        }
    }
    
    // Log packages that weren't found
    for pkg in &packages_set {
        if !found_packages.contains(pkg) {
            eprintln!("[DEBUG] WARNING: Package '{}' not found in dnf check-update output", pkg);
        }
    }
    
    eprintln!("[DEBUG] Found {}/{} packages in updates", updates.len(), packages.len());

    Ok(updates)
}

async fn install_updates_streaming(
    packages: Vec<String>, 
    _packages_with_info: Vec<UpdateInfo>,
    _package_names: std::collections::HashSet<String>
) -> Result<String, String> {
    // Load settings
    let settings = UpdateSettings::load();
    
    // Clone packages for display before moving
    let packages_display = packages.clone();
    
    eprintln!("[DEBUG] install_updates_streaming called with {} packages", packages.len());
    eprintln!("[DEBUG] Package names: {:?}", packages);
    
    if packages.is_empty() {
        return Err("No packages specified for installation".to_string());
    }
    
    // Use dnf upgrade with package names
    // dnf upgrade will upgrade to the latest available version for each package
    let mut dnf_args: Vec<String> = vec!["dnf".to_string(), "upgrade".to_string(), "-y".to_string(), "--assumeyes".to_string()];
    
    // Add settings-based arguments first (flags should come before package names)
    dnf_args.extend(settings.to_dnf_args());
    
    // Add package names to upgrade
    dnf_args.extend(packages.clone());
    
    eprintln!("[DEBUG] dnf command: pkexec {}", dnf_args.join(" "));
    
    // Use pkexec for privilege escalation (better than sudo for GUI apps)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(&dnf_args);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
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
    if !packages_display.is_empty() {
        combined_output.push_str(&format!("Upgrading packages: {}\n", packages_display.join(", ")));
    }
    combined_output.push_str("--- Output ---\n");
    
    // Read both stdout and stderr line by line
    // Send each line as it comes for real-time progress tracking
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

    eprintln!("[DEBUG] dnf command finished with exit code: {}", exit_code);
    eprintln!("[DEBUG] Output length: {} bytes", combined_output.len());
    eprintln!("[DEBUG] Output preview: {}", combined_output.chars().take(500).collect::<String>());
    
    // Check for common error patterns in output even if exit code is 0
    let output_lower = combined_output.to_lowercase();
    let has_error = output_lower.contains("error") || 
                    output_lower.contains("failed") ||
                    output_lower.contains("cannot") ||
                    output_lower.contains("no package") ||
                    output_lower.contains("nothing provides") ||
                    output_lower.contains("problem") ||
                    output_lower.contains("conflict");
    
    // dnf upgrade returns exit code 0 on success, but also returns 0 if nothing to do
    // Check if there's actual output indicating work was done
    let has_output = !combined_output.trim().is_empty() && 
                     combined_output.trim() != "Starting system update...\n--- Output ---\n" &&
                     !combined_output.trim().ends_with("Starting system update...\n--- Output ---");
    
    // Always return the output so it can be displayed, even on failure
    // The UI will handle showing errors appropriately
    if !success {
        eprintln!("[DEBUG] Command failed, but returning output for display");
        // Return the output with error prefix so UI can display it
        // Include the full output so users can see what went wrong
        return Err(format!("Update installation failed (exit code: {}):\n\n{}", exit_code, combined_output));
    }
    
    // Check for errors in output even if exit code is 0
    if has_error {
        return Err(format!("Update installation encountered errors:\n{}", combined_output));
    }

    // Success - return the output
    if !has_output {
        // Check if packages were actually installed by looking for "Installed:" or "Upgraded:" in output
        if output_lower.contains("installed:") || output_lower.contains("upgraded:") {
            Ok(combined_output)
        } else {
            Ok("No updates were needed - packages are already at the latest version.".to_string())
        }
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

