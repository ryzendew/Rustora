use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
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
    // View mode switching
    SwitchToSearch,
    SwitchToInstalled,
    SwitchToUpdates,
    // Search operations
    SearchQueryChanged(String),
    Search,
    SearchResult(Vec<FlatpakInfo>),
    // Install operations
    TogglePackage(String),
    InstallSelected,
    InstallComplete,
    // Installed list operations
    LoadInstalled,
    InstalledLoaded(Vec<FlatpakInfo>),
    // Remove operations
    RemoveSelected,
    RemoveComplete,
    // Update operations
    CheckUpdates,
    UpdatesFound(Vec<FlatpakInfo>),
    InstallUpdates,
    UpdatesInstalled,
    // Package details panel
    PackageSelected(String, Option<String>), // application_id, remote
    PackageDetailsLoaded(FlatpakDetails),
    ClosePanel,
    // Error handling
    Error(String),
}

#[derive(Debug, Clone)]
pub struct FlatpakInfo {
    pub name: String,
    pub application_id: String,
    pub description: String,
    pub version: String,
    pub remote: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlatpakDetails {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub branch: String,
    pub arch: String,
    pub summary: String,
    pub description: String,
    pub size: String,
    pub remote: Option<String>,
    pub runtime: String,
    pub license: String,
}

#[derive(Debug)]
pub struct FlatpakTab {
    // Search state
    search_query: String,
    search_results: Vec<FlatpakInfo>,
    is_searching: bool,
    
    // Install state
    selected_packages: std::collections::HashSet<String>,
    is_installing: bool,
    
    // Installed list state
    installed_flatpaks: Vec<FlatpakInfo>,
    is_loading_installed: bool,
    
    // Remove state
    is_removing: bool,
    
    // Update state
    updates: Vec<FlatpakInfo>,
    is_checking_updates: bool,
    is_updating: bool,
    update_error: Option<String>,
    
    // View mode
    view_mode: ViewMode,
    
    // Package details panel
    selected_package: Option<String>,
    package_details: Option<FlatpakDetails>,
    panel_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Search,
    Installed,
    Updates,
}

impl FlatpakTab {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            selected_packages: std::collections::HashSet::new(),
            is_installing: false,
            installed_flatpaks: Vec::new(),
            is_loading_installed: false,
            is_removing: false,
            updates: Vec::new(),
            is_checking_updates: false,
            is_updating: false,
            update_error: None,
            view_mode: ViewMode::Search,
            selected_package: None,
            package_details: None,
            panel_open: false,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::SwitchToSearch => {
                self.view_mode = ViewMode::Search;
                iced::Command::none()
            }
            Message::SwitchToInstalled => {
                self.view_mode = ViewMode::Installed;
                // Auto-load installed packages when switching to this view
                self.is_loading_installed = true;
                iced::Command::perform(load_installed_flatpaks(), |result| {
                    match result {
                        Ok(packages) => Message::InstalledLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::SwitchToUpdates => {
                self.view_mode = ViewMode::Updates;
                // Auto-check for updates when switching to this view
                self.is_checking_updates = true;
                iced::Command::perform(check_flatpak_updates(), |result| {
                    match result {
                        Ok(updates) => Message::UpdatesFound(updates),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query.clone();
                if !query.trim().is_empty() && query.len() >= 2 {
                    self.is_searching = true;
                    iced::Command::perform(search_flatpaks(query), |result| {
                        match result {
                            Ok(packages) => Message::SearchResult(packages),
                            Err(e) => Message::Error(e),
                        }
                    })
                } else {
                    self.search_results.clear();
                    iced::Command::none()
                }
            }
            Message::Search => {
                if self.search_query.trim().is_empty() {
                    return iced::Command::none();
                }
                self.is_searching = true;
                let query = self.search_query.clone();
                iced::Command::perform(search_flatpaks(query), |result| {
                    match result {
                        Ok(packages) => Message::SearchResult(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::SearchResult(packages) => {
                self.is_searching = false;
                self.search_results = packages;
                self.view_mode = ViewMode::Search;
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
                // Spawn separate window for Flatpak installation
                // Install packages one by one (open dialog for first, others can be queued)
                let packages: Vec<String> = self.selected_packages.iter().cloned().collect();
                if let Some(first_pkg) = packages.first() {
                    // Find the remote for this package from search results
                    let remote = self.search_results
                        .iter()
                        .find(|p| &p.application_id == first_pkg)
                        .and_then(|p| p.remote.clone());
                    
                    let app_id = first_pkg.clone();
                    let remote_clone = remote.clone();
                    iced::Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                            let mut cmd = TokioCommand::new(&exe_path);
                            cmd.arg("flatpak-install-dialog").arg(&app_id);
                            if let Some(ref r) = remote_clone {
                                cmd.arg("--remote").arg(r);
                            }
                            let _ = cmd.spawn();
                        },
                        |_| Message::InstallComplete,
                    )
                } else {
                    iced::Command::none()
                }
            }
            Message::InstallComplete => {
                self.is_installing = false;
                self.selected_packages.clear();
                // Refresh installed list
                iced::Command::perform(load_installed_flatpaks(), |result| {
                    match result {
                        Ok(packages) => Message::InstalledLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::LoadInstalled => {
                self.is_loading_installed = true;
                self.view_mode = ViewMode::Installed;
                iced::Command::perform(load_installed_flatpaks(), |result| {
                    match result {
                        Ok(packages) => Message::InstalledLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::InstalledLoaded(packages) => {
                self.is_loading_installed = false;
                self.installed_flatpaks = packages;
                iced::Command::none()
            }
            Message::RemoveSelected => {
                if self.selected_packages.is_empty() {
                    return iced::Command::none();
                }
                // Spawn separate window for Flatpak removal
                let packages: Vec<String> = self.selected_packages.iter().cloned().collect();
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let _ = TokioCommand::new(&exe_path)
                            .arg("flatpak-remove-dialog")
                            .args(packages)
                            .spawn()
                            .ok();
                    },
                    |_| Message::RemoveComplete,
                )
            }
            Message::RemoveComplete => {
                self.is_removing = false;
                self.selected_packages.clear();
                // Refresh installed list
                iced::Command::perform(load_installed_flatpaks(), |result| {
                    match result {
                        Ok(packages) => Message::InstalledLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::CheckUpdates => {
                self.is_checking_updates = true;
                self.update_error = None;
                self.view_mode = ViewMode::Updates;
                iced::Command::perform(check_flatpak_updates(), |result| {
                    match result {
                        Ok(updates) => Message::UpdatesFound(updates),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::UpdatesFound(updates) => {
                self.is_checking_updates = false;
                self.updates = updates;
                self.update_error = None;
                iced::Command::none()
            }
            Message::InstallUpdates => {
                if self.updates.is_empty() {
                    return iced::Command::none();
                }
                // Spawn separate window for Flatpak updates
                let updates: Vec<crate::gui::flatpak_update_dialog::FlatpakUpdateInfo> = self.updates
                    .iter()
                    .map(|u| crate::gui::flatpak_update_dialog::FlatpakUpdateInfo {
                        name: u.name.clone(),
                        application_id: u.application_id.clone(),
                        version: u.version.clone(),
                        remote: u.remote.clone(),
                    })
                    .collect();
                
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        use base64::{Engine as _, engine::general_purpose};
                        
                        // Serialize to JSON and encode as base64
                        let json = serde_json::to_string(&updates)
                            .unwrap_or_else(|_| "[]".to_string());
                        let encoded = general_purpose::STANDARD.encode(json.as_bytes());
                        
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let _ = TokioCommand::new(&exe_path)
                            .arg("flatpak-update-dialog")
                            .arg("--packages-b64")
                            .arg(&encoded)
                            .spawn();
                    },
                    |_| Message::UpdatesInstalled,
                )
            }
            Message::UpdatesInstalled => {
                self.is_updating = false;
                self.updates.clear();
                // Refresh installed list
                iced::Command::perform(load_installed_flatpaks(), |result| {
                    match result {
                        Ok(packages) => Message::InstalledLoaded(packages),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::PackageSelected(app_id, remote) => {
                self.selected_package = Some(app_id.clone());
                self.panel_open = true;
                iced::Command::perform(load_flatpak_details(app_id, remote), Message::PackageDetailsLoaded)
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
            Message::Error(msg) => {
                self.is_searching = false;
                self.is_installing = false;
                self.is_loading_installed = false;
                self.is_removing = false;
                self.is_checking_updates = false;
                self.is_updating = false;
                // Store error message if we're in updates view
                if self.view_mode == ViewMode::Updates {
                    self.update_error = Some(msg);
                }
                iced::Command::none()
            }
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let tab_font_size = settings.font_size_tabs * settings.scale_tabs;
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        // Mode selector buttons
        let search_mode_button = button(
            row![
                text(crate::gui::fonts::glyphs::SEARCH_SYMBOL).font(material_font).size(icon_size),
                text(" Search").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::SwitchToSearch)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: self.view_mode == ViewMode::Search,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));

        let installed_mode_button = button(
            row![
                text(crate::gui::fonts::glyphs::INSTALLED_SYMBOL).font(material_font).size(icon_size),
                text(" Installed").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::SwitchToInstalled)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: self.view_mode == ViewMode::Installed,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));

        let updates_mode_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                text(" Updates").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::SwitchToUpdates)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: self.view_mode == ViewMode::Updates,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));

        let mode_selector = row![
            search_mode_button,
            installed_mode_button,
            updates_mode_button
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        // Content based on view mode
        let content: Element<Message> = match self.view_mode {
            ViewMode::Search => self.view_search(theme, settings, material_font),
            ViewMode::Installed => self.view_installed(theme, settings, material_font),
            ViewMode::Updates => self.view_updates(theme, settings, material_font),
        };

        container(column![mode_selector, content].spacing(15).padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_search(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings, material_font: iced::Font) -> Element<'_, Message> {
        let input_font_size = settings.font_size_inputs * settings.scale_inputs;
        let body_font_size = settings.font_size_body * settings.scale_body;
        let button_font_size = settings.font_size_buttons * settings.scale_buttons;
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let package_name_size = settings.font_size_package_names * settings.scale_package_cards;
        let package_detail_size = settings.font_size_package_details * settings.scale_package_cards;
        let search_input = text_input("Search Flatpak applications...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::Search)
            .size(input_font_size)
            .width(Length::Fill)
            .padding(14)
            .style(iced::theme::TextInput::Custom(Box::new(RoundedTextInputStyle {
                radius: settings.border_radius,
            })));

        let search_button = button(
            row![
                text(crate::gui::fonts::glyphs::SEARCH_SYMBOL).font(material_font).size(icon_size),
                text(" Search").size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::Search)
        .padding(Padding::new(14.0))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })));

        let search_row = row![search_input, search_button]
            .spacing(10)
            .align_items(Alignment::Center);

        let install_button = if self.selected_packages.is_empty() {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font),
                    text(" Install Selected")
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
        } else {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                    text(format!(" Install {} Package(s)", self.selected_packages.len())).size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::InstallSelected)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
        };

        let content: Element<Message> = if self.is_searching {
            container(text("Searching...").size(body_font_size))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
                .into()
        } else if self.search_results.is_empty() && !self.search_query.is_empty() {
            container(text("No Flatpak applications found").size(body_font_size))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
                .into()
        } else {
            let package_list: Element<Message> = if self.search_results.is_empty() {
                container(text("Enter a search query to find Flatpak applications").size(body_font_size))
                    .width(Length::Fill)
                    .padding(20)
                    .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                        radius: settings.border_radius,
                    })))
                    .into()
            } else {
                scrollable(
                    column(
                        self.search_results
                            .iter()
                            .map(|pkg| {
                                let pkg_name = pkg.name.clone();
                                let pkg_id = pkg.application_id.clone();
                                let is_selected = self.selected_packages.contains(&pkg_id);
                                let checkbox_widget = checkbox("", is_selected)
                                    .on_toggle(move |_| Message::TogglePackage(pkg_id.clone()))
                                    .style(iced::theme::Checkbox::Custom(Box::new(RoundedCheckboxStyle {
                                        radius: settings.border_radius,
                                    })));
                                let pkg_id_for_click = pkg.application_id.clone();
                                let pkg_remote_for_click = pkg.remote.clone();
                                button(
                                    container(
                                        row![
                                            checkbox_widget,
                                            text(&pkg_name).size(package_name_size).width(Length::FillPortion(2)),
                                            text(&pkg.version).size(package_detail_size).width(Length::FillPortion(1)),
                                            text(&pkg.description).size(package_detail_size).width(Length::FillPortion(4)),
                                        ]
                                        .spacing(12)
                                        .align_items(Alignment::Center)
                                        .padding(12)
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(PackageItemStyle {
                                        is_selected,
                                        radius: settings.border_radius,
                                    })))
                                )
                                .on_press(Message::PackageSelected(pkg_id_for_click, pkg_remote_for_click))
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
            column![install_button, package_list].spacing(10).into()
        };

        // Create the slide-out panel
        let panel = if self.panel_open {
            self.view_panel(theme, settings, material_font)
        } else {
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(0.0))
                .height(Length::Fill)
                .into()
        };

        let main_content = container(column![search_row, content].spacing(15))
            .width(Length::Fill)
            .height(Length::Fill);

        container(
            row![
                container(main_content)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill),
                container(panel)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
            ]
            .spacing(15)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_panel(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings, material_font: iced::Font) -> Element<'_, Message> {
        let title_font_size = settings.font_size_titles * settings.scale_titles;
        let body_font_size = settings.font_size_body * settings.scale_body;
        let package_detail_size = settings.font_size_package_details * settings.scale_package_cards;
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        if let Some(ref details) = self.package_details {
            container(
                scrollable(
                    column![
                        // Header with close button
                        row![
                            text("Flatpak Details").size(title_font_size).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                            Space::with_width(Length::Fill),
                            button(
                                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_size)
                            )
                            .on_press(Message::ClosePanel)
                            .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
                            .padding(Padding::new(6.0))
                        ]
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        // Package name
                        text(&details.name)
                            .size(title_font_size * 1.1)
                            .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                        Space::with_height(Length::Fixed(10.0)),
                        text(&details.application_id)
                            .size(package_detail_size * 0.9)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        // Package details
                        container({
                            let mut items: Vec<Element<Message>> = vec![
                                row![
                                    text("Version:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    text(&details.version).size(package_detail_size).width(Length::Fill),
                                ]
                                .spacing(12)
                                .into(),
                                Space::with_height(Length::Fixed(8.0)).into(),
                                row![
                                    text("Branch:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    text(&details.branch).size(package_detail_size).width(Length::Fill),
                                ]
                                .spacing(12)
                                .into(),
                                Space::with_height(Length::Fixed(8.0)).into(),
                                row![
                                    text("Architecture:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    text(&details.arch).size(package_detail_size).width(Length::Fill),
                                ]
                                .spacing(12)
                                .into(),
                                Space::with_height(Length::Fixed(8.0)).into(),
                                row![
                                    text("Size:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    text(&details.size).size(package_detail_size).width(Length::Fill),
                                ]
                                .spacing(12)
                                .into(),
                                Space::with_height(Length::Fixed(8.0)).into(),
                                row![
                                    text("Runtime:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    text(&details.runtime).size(package_detail_size).width(Length::Fill),
                                ]
                                .spacing(12)
                                .into(),
                                Space::with_height(Length::Fixed(8.0)).into(),
                                row![
                                    text("Remote:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    text(details.remote.as_deref().unwrap_or("N/A")).size(package_detail_size).width(Length::Fill),
                                ]
                                .spacing(12)
                                .into(),
                            ];
                            
                            if !details.license.is_empty() {
                                items.push(Space::with_height(Length::Fixed(8.0)).into());
                                items.push(
                                    row![
                                        text("License:").size(package_detail_size).width(Length::Fixed(110.0)).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                        text(&details.license).size(package_detail_size).width(Length::Fill),
                                    ]
                                    .spacing(12)
                                    .into()
                                );
                            }
                            
                            column(items).spacing(0)
                        })
                        .padding(Padding::new(18.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                            radius: settings.border_radius,
                        }))),
                        Space::with_height(Length::Fixed(20.0)),
                        // Summary
                        text("Summary").size(title_font_size * 0.9).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                        Space::with_height(Length::Fixed(8.0)),
                        text(&details.summary).size(package_detail_size),
                        Space::with_height(Length::Fixed(20.0)),
                        // Description
                        text("Description").size(title_font_size * 0.9).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                        Space::with_height(Length::Fixed(8.0)),
                        text(&details.description).size(package_detail_size).width(Length::Fill),
                    ]
                    .spacing(0)
                    .padding(Padding::new(25.0))
                )
                .height(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(PanelStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else {
            container(
                column![
                    row![
                        Space::with_width(Length::Fill),
                        button(
                            text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_size * 1.2)
                        )
                        .on_press(Message::ClosePanel)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: false,
                            radius: settings.border_radius,
                        })))
                        .padding(Padding::new(8.0))
                    ]
                    .width(Length::Fill),
                    Space::with_height(Length::Fill),
                    text("Loading...").size(body_font_size).horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fill),
                ]
                .padding(Padding::new(20.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(PanelStyle {
                radius: settings.border_radius,
            })))
            .into()
        }
    }

    fn view_installed(&self, _theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings, material_font: iced::Font) -> Element<'_, Message> {
        let body_font_size = settings.font_size_body * settings.scale_body;
        let button_font_size = settings.font_size_buttons * settings.scale_buttons;
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let package_name_size = settings.font_size_package_names * settings.scale_package_cards;
        let package_detail_size = settings.font_size_package_details * settings.scale_package_cards;
        
        let refresh_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                text(" Refresh").size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::LoadInstalled)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(14.0));

        let remove_button = if self.selected_packages.is_empty() {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font).size(icon_size),
                    text(" Remove Selected").size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
        } else {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font).size(icon_size),
                    text(format!(" Remove {} Package(s)", self.selected_packages.len())).size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::RemoveSelected)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
        };

        let header = row![refresh_button, Space::with_width(Length::Fill), remove_button]
            .spacing(10)
            .align_items(Alignment::Center);

        let content: Element<Message> = if self.is_loading_installed {
            container(text("Loading installed Flatpaks...").size(body_font_size))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
                .into()
        } else if self.installed_flatpaks.is_empty() {
            container(text("No Flatpak applications installed").size(body_font_size))
                .width(Length::Fill)
                .padding(20)
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
                .into()
        } else {
            scrollable(
                column(
                    self.installed_flatpaks
                        .iter()
                        .map(|pkg| {
                            let pkg_id = pkg.application_id.clone();
                            let is_selected = self.selected_packages.contains(&pkg_id);
                            let checkbox_widget = checkbox("", is_selected)
                                .on_toggle(move |_| Message::TogglePackage(pkg_id.clone()))
                                .style(iced::theme::Checkbox::Custom(Box::new(RoundedCheckboxStyle {
                                    radius: settings.border_radius,
                                })));
                            container(
                                row![
                                    checkbox_widget,
                                    text(&pkg.name).size(package_name_size).width(Length::FillPortion(3)),
                                    text(&pkg.version).size(package_detail_size).width(Length::FillPortion(2)),
                                    text(pkg.remote.as_deref().unwrap_or("local")).size(package_detail_size).width(Length::FillPortion(2)),
                                ]
                                .spacing(12)
                                .align_items(Alignment::Center)
                                .padding(12)
                            )
                            .style(iced::theme::Container::Custom(Box::new(PackageItemStyle {
                                is_selected,
                                radius: settings.border_radius,
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

        column![header, content].spacing(10).into()
    }

    fn view_updates(&self, _theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings, material_font: iced::Font) -> Element<'_, Message> {
        let body_font_size = settings.font_size_body * settings.scale_body;
        let button_font_size = settings.font_size_buttons * settings.scale_buttons;
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let package_name_size = settings.font_size_package_names * settings.scale_package_cards;
        let package_detail_size = settings.font_size_package_details * settings.scale_package_cards;
        
        let check_button = if self.is_checking_updates {
            button(
                row![
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                    text(" Checking...").size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
        } else {
            button(
                row![
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                    text(" Check for Updates").size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::CheckUpdates)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
        };

        let install_button: Element<Message> = if self.updates.is_empty() || self.is_updating {
            if self.is_updating {
                button(
                    row![
                        text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                        text(" Updating...").size(button_font_size)
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::new(14.0))
                .into()
            } else {
                button(text("No Updates Available").size(button_font_size))
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: false,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::new(14.0))
                    .into()
            }
        } else {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                    text(format!(" Update {} Package(s)", self.updates.len())).size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::InstallUpdates)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(14.0))
            .into()
        };

        let header = row![check_button, Space::with_width(Length::Fill), install_button]
            .spacing(10)
            .align_items(Alignment::Center);

        let content: Element<Message> = if self.is_checking_updates {
            container(text("Checking for updates...").size(body_font_size))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
                .into()
        } else if let Some(ref error) = self.update_error {
            let material_font = crate::gui::fonts::get_material_symbols_font();
            container(
                column![
                    text("Update Error").size(body_font_size).style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3))),
                    Space::with_height(Length::Fixed(10.0)),
                    text(error).size(package_detail_size),
                    Space::with_height(Length::Fixed(15.0)),
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                            text(" Try Again").size(button_font_size)
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::CheckUpdates)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::new(12.0)),
                ]
                .spacing(8)
                .align_items(Alignment::Center)
                .padding(20)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else if self.updates.is_empty() {
            container(text("Click 'Check for Updates' to see available updates").size(body_font_size))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
                .into()
        } else {
            scrollable(
                column(
                    self.updates
                        .iter()
                        .map(|update| {
                            container(
                                row![
                                    text(&update.name).size(package_name_size).width(Length::FillPortion(3)),
                                    text(&update.version).size(package_detail_size).width(Length::FillPortion(2)),
                                    text(update.remote.as_deref().unwrap_or("local")).size(package_detail_size).width(Length::FillPortion(2)),
                                ]
                                .spacing(12)
                                .align_items(Alignment::Center)
                                .padding(12)
                            )
                            .style(iced::theme::Container::Custom(Box::new(UpdateItemStyle {
                                radius: settings.border_radius,
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

        column![header, content].spacing(15).into()
    }
}

// Flatpak command implementations
async fn search_flatpaks(query: String) -> Result<Vec<FlatpakInfo>, String> {
    let output = TokioCommand::new("flatpak")
        .args(["search", "--columns=name,application,description,version,remotes", &query])
        .output()
        .await
        .map_err(|e| format!("Failed to execute flatpak search: {}", e))?;

    if !output.status.success() {
        return Err(format!("Flatpak search failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Parse tab-separated values: name, application, description, version, remotes
        // Format: name<TAB>application<TAB>description<TAB>version<TAB>remotes
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim().to_string();
            let application_id = parts[1].trim().to_string();
            let description = parts.get(2).map(|s| s.trim()).unwrap_or("").to_string();
            let version = parts.get(3).map(|s| s.trim()).unwrap_or("").to_string();
            // Remotes can be comma-separated (e.g., "fedora,flathub")
            let remote = parts.get(4)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());
            
            results.push(FlatpakInfo {
                name,
                application_id,
                description,
                version,
                remote,
            });
        }
    }

    Ok(results)
}

#[allow(dead_code)]
async fn install_flatpaks(packages: Vec<String>) -> Result<(), String> {
    // Use --assumeyes (-y) for non-interactive installation
    // Use --app flag to ensure we're installing applications
    let status = TokioCommand::new("flatpak")
        .args(["install", "--app", "-y", "--noninteractive"])
        .args(&packages)
        .status()
        .await
        .map_err(|e| format!("Failed to execute flatpak install: {}", e))?;

    if !status.success() {
        return Err("Flatpak installation failed".to_string());
    }
    Ok(())
}

async fn load_installed_flatpaks() -> Result<Vec<FlatpakInfo>, String> {
    // List all installed items (applications, runtimes, extensions)
    // Use --columns to get structured output
    let output = TokioCommand::new("flatpak")
        .args(["list", "--columns=name,application,version,origin"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute flatpak list: {}", e))?;

    if !output.status.success() {
        return Err(format!("Flatpak list failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Parse tab-separated values: name, application, version, origin
        // Note: version can be empty for some entries (like runtimes)
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim().to_string();
            let application_id = parts[1].trim().to_string();
            let version = parts.get(2).map(|s| s.trim()).unwrap_or("").to_string();
            let remote = parts.get(3).map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string());
            
            packages.push(FlatpakInfo {
                name,
                application_id,
                description: String::new(),
                version,
                remote,
            });
        }
    }

    Ok(packages)
}

#[allow(dead_code)]
async fn remove_flatpaks(packages: Vec<String>) -> Result<(), String> {
    // Use --assumeyes (-y) for non-interactive uninstallation
    // Use --app flag to ensure we're uninstalling applications
    let status = TokioCommand::new("flatpak")
        .args(["uninstall", "--app", "-y", "--noninteractive"])
        .args(&packages)
        .status()
        .await
        .map_err(|e| format!("Failed to execute flatpak uninstall: {}", e))?;

    if !status.success() {
        return Err("Flatpak removal failed".to_string());
    }
    Ok(())
}

async fn check_flatpak_updates() -> Result<Vec<FlatpakInfo>, String> {
    // First, update the appstream to get latest information
    let _ = TokioCommand::new("flatpak")
        .args(["update", "--appstream", "-y"])
        .status()
        .await;

    // Get list of installed applications first
    let installed_output = TokioCommand::new("flatpak")
        .args(["list", "--app", "--columns=application,version,origin"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute flatpak list: {}", e))?;

    if !installed_output.status.success() {
        return Err(format!("Failed to list installed flatpaks: {}", 
            String::from_utf8_lossy(&installed_output.stderr)));
    }

    let installed_stdout = String::from_utf8_lossy(&installed_output.stdout);
    let mut updates = Vec::new();

    // For each installed app, check if there's an update available
    for line in installed_stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse: application_id<TAB>version<TAB>origin
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let application_id = parts[0].trim().to_string();
            let installed_version = parts[1].trim().to_string();
            let remote = parts[2].trim().to_string();

            if remote.is_empty() {
                continue;
            }

            // Check remote version
            let remote_info_output = TokioCommand::new("flatpak")
                .args(["remote-info", &remote, &application_id])
                .output()
                .await;

            if let Ok(remote_info) = remote_info_output {
                if remote_info.status.success() {
                    let remote_stdout = String::from_utf8_lossy(&remote_info.stdout);
                    let mut remote_version = String::new();
                    let mut name = application_id.clone();

                    // Parse remote-info output
                    for info_line in remote_stdout.lines() {
                        let info_line = info_line.trim();
                        if info_line.starts_with("Name:") {
                            name = info_line.splitn(2, ':')
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .to_string();
                        } else if info_line.starts_with("Version:") {
                            remote_version = info_line.splitn(2, ':')
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .to_string();
                        }
                    }

                    // Compare versions - if remote version differs from installed, there's an update
                    if !remote_version.is_empty() && remote_version != installed_version {
                        updates.push(FlatpakInfo {
                            name: if name.is_empty() { application_id.clone() } else { name },
                            application_id,
                            description: String::new(),
                            version: remote_version,
                            remote: Some(remote),
                        });
                    }
                }
            }
        }
    }

    // Alternative: Also try the remote-ls --updates method as a fallback
    // This might catch updates that the version comparison missed
    let remote_ls_output = TokioCommand::new("flatpak")
        .args(["remote-ls", "--updates", "--app", "--columns=name,application,version,origin"])
        .output()
        .await;

    if let Ok(remote_ls) = remote_ls_output {
        if remote_ls.status.success() {
            let remote_ls_stdout = String::from_utf8_lossy(&remote_ls.stdout);
            for line in remote_ls_stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim().to_string();
                    let application_id = parts[1].trim().to_string();
                    let version = parts.get(2).map(|s| s.trim()).unwrap_or("").to_string();
                    let remote = parts.get(3).map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string());
                    
                    // Only add if not already in updates list
                    if !updates.iter().any(|u| u.application_id == application_id) {
                        updates.push(FlatpakInfo {
                            name,
                            application_id,
                            description: String::new(),
                            version,
                            remote,
                        });
                    }
                }
            }
        }
    }

    Ok(updates)
}


async fn load_flatpak_details(app_id: String, remote: Option<String>) -> FlatpakDetails {
    // Try to get info from remote first, then fallback to installed
    let mut name = app_id.clone();
    let mut version = String::new();
    let mut branch = String::new();
    let mut arch = String::new();
    let mut summary = String::new();
    let mut description = String::new();
    let mut size = String::new();
    let mut runtime = String::new();
    let mut license = String::new();

    // Try remote-info first if remote is provided
    if let Some(ref remote_name) = remote {
        let output = TokioCommand::new("flatpak")
            .args(["remote-info", remote_name, &app_id])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.starts_with("Name:") {
                        name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Version:") {
                        version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Branch:") {
                        branch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Arch:") {
                        arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Summary:") {
                        summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Description:") {
                        description = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Download size:") {
                        size = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Runtime:") {
                        runtime = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("License:") {
                        license = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    }
                }
            }
        }
    }

    // Fallback to info if remote-info didn't work or no remote provided
    if version.is_empty() {
        let output = TokioCommand::new("flatpak")
            .args(["info", &app_id])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.starts_with("Name:") && name == app_id {
                        name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Version:") && version.is_empty() {
                        version = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Branch:") && branch.is_empty() {
                        branch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Arch:") && arch.is_empty() {
                        arch = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Summary:") && summary.is_empty() {
                        summary = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Description:") && description.is_empty() {
                        description = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Installed size:") && size.is_empty() {
                        size = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Runtime:") && runtime.is_empty() {
                        runtime = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("License:") && license.is_empty() {
                        license = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                    }
                }
            }
        }
    }

    // Use app_id as name if name is still empty
    if name.is_empty() {
        name = app_id.clone();
    }

    FlatpakDetails {
        name,
        application_id: app_id,
        version: if version.is_empty() { "N/A".to_string() } else { version },
        branch: if branch.is_empty() { "stable".to_string() } else { branch },
        arch: if arch.is_empty() { "x86_64".to_string() } else { arch },
        summary: if summary.is_empty() { "No summary available".to_string() } else { summary },
        description: if description.is_empty() { "No description available".to_string() } else { description },
        size: if size.is_empty() { "Unknown".to_string() } else { size },
        remote,
        runtime: if runtime.is_empty() { "N/A".to_string() } else { runtime },
        license: if license.is_empty() { String::new() } else { license },
    }
}

// Style implementations
struct PackageItemStyle {
    is_selected: bool,
    radius: f32,
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
                radius: self.radius.into(),
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

struct RoundedMessageStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for RoundedMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct RoundedButtonStyle {
    is_primary: bool,
    radius: f32,
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
                radius: self.radius.into(),
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

struct RoundedTextInputStyle {
    radius: f32,
}

impl TextInputStyleSheet for RoundedTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        TextInputAppearance {
            background: iced::Background::Color(palette.background),
            border: Border {
                radius: self.radius.into(),
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
                radius: self.radius.into(),
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
                radius: self.radius.into(),
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

struct RoundedCheckboxStyle {
    radius: f32,
}

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
                radius: self.radius.into(),
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

struct UpdateItemStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for UpdateItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}

struct PanelStyle {
    radius: f32,
}

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
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct InfoContainerStyle {
    radius: f32,
}

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
                radius: self.radius.into(),
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

