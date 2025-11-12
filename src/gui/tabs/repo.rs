use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    LoadRepositories,
    RepositoriesLoaded(Vec<RepositoryInfo>),
    SearchQueryChanged(String),
    RepositorySelected(String),
    RepositoryDetailsLoaded(RepositoryDetails),
    ClosePanel,
    ToggleRepository(String),
    ToggleRepositoryComplete(Result<String, String>),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub id: String,
    pub name: String,
    pub baseurl: Option<String>,
    pub metalink: Option<String>,
    pub enabled: bool,
    pub file_path: String,
    #[allow(dead_code)]
    pub gpgcheck: Option<bool>,
    #[allow(dead_code)]
    pub repo_gpgcheck: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct RepositoryDetails {
    pub id: String,
    pub name: String,
    pub baseurl: Option<String>,
    pub metalink: Option<String>,
    pub enabled: bool,
    pub file_path: String,
    pub gpgcheck: Option<bool>,
    pub repo_gpgcheck: Option<bool>,
    pub gpgkey: Option<String>,
    #[allow(dead_code)]
    pub _metadata_expire: Option<String>,
    #[allow(dead_code)]
    pub _skip_if_unavailable: Option<bool>,
    #[allow(dead_code)]
    pub _countme: Option<String>,
    #[allow(dead_code)]
    pub _repo_type: Option<String>,
}

#[derive(Debug)]
pub struct RepoTab {
    repositories: Vec<RepositoryInfo>,
    filtered_repositories: Vec<RepositoryInfo>,
    search_query: String,
    is_loading: bool,
    selected_repository: Option<String>,
    repository_details: Option<RepositoryDetails>,
    panel_open: bool,
}

impl RepoTab {
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
            filtered_repositories: Vec::new(),
            search_query: String::new(),
            is_loading: false,
            selected_repository: None,
            repository_details: None,
            panel_open: false,
        }
    }

    fn filter_repositories(&mut self) {
        if self.search_query.trim().is_empty() {
            self.filtered_repositories = self.repositories.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_repositories = self.repositories
                .iter()
                .filter(|repo| {
                    repo.id.to_lowercase().contains(&query_lower) ||
                    repo.name.to_lowercase().contains(&query_lower) ||
                    repo.file_path.to_lowercase().contains(&query_lower) ||
                    repo.baseurl.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                    repo.metalink.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false)
                })
                .cloned()
                .collect();
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::LoadRepositories => {
                self.is_loading = true;
                iced::Command::perform(load_repositories(), |result| {
                    match result {
                        Ok(repos) => Message::RepositoriesLoaded(repos),
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::RepositoriesLoaded(repos) => {
                self.is_loading = false;
                self.repositories = repos;
                self.filter_repositories();
                
                // Reload details if panel is open to reflect changes
                if let Some(ref repo_id) = self.selected_repository {
                    let repo_id = repo_id.clone();
                    return iced::Command::perform(load_repository_details(repo_id), Message::RepositoryDetailsLoaded);
                }
                
                iced::Command::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.filter_repositories();
                iced::Command::none()
            }
            Message::RepositorySelected(id) => {
                self.selected_repository = Some(id.clone());
                self.panel_open = true;
                iced::Command::perform(load_repository_details(id), Message::RepositoryDetailsLoaded)
            }
            Message::RepositoryDetailsLoaded(details) => {
                let details_id = details.id.clone();
                let details_enabled = details.enabled;
                self.repository_details = Some(details);
                // Also update the repository in the list if it exists
                if let Some(repo) = self.repositories.iter_mut().find(|r| r.id == details_id) {
                    repo.enabled = details_enabled;
                    self.filter_repositories();
                }
                iced::Command::none()
            }
            Message::ClosePanel => {
                self.panel_open = false;
                self.selected_repository = None;
                self.repository_details = None;
                iced::Command::none()
            }
            Message::ToggleRepository(repo_id) => {
                // Find the repository to get its current state
                if let Some(repo) = self.repositories.iter().find(|r| r.id == repo_id) {
                    let new_state = !repo.enabled;
                    iced::Command::perform(
                        toggle_repository(repo_id.clone(), new_state),
                        Message::ToggleRepositoryComplete,
                    )
                } else {
                    iced::Command::none()
                }
            }
            Message::ToggleRepositoryComplete(result) => {
                match result {
                    Ok(_) => {
                        // Reload repositories to reflect the change
                        self.is_loading = true;
                        iced::Command::perform(load_repositories(), |result| {
                            match result {
                                Ok(repos) => Message::RepositoriesLoaded(repos),
                                Err(e) => Message::Error(e),
                            }
                        })
                    }
                    Err(_e) => {
                        self.is_loading = false;
                        iced::Command::none()
                    }
                }
            }
            Message::Error(_msg) => {
                self.is_loading = false;
                iced::Command::none()
            }
        }
    }

    fn view_panel(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        if let Some(ref details) = self.repository_details {
            let material_font = crate::gui::fonts::get_material_symbols_font();
            
            container(
                scrollable(
                    column![
                        // Header with close button
                        row![
                            text("Repository Details")
                                .size(18)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_width(Length::Fill),
                            button(
                                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(18)
                            )
                            .on_press(Message::ClosePanel)
                            .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
                            .padding(Padding::new(6.0)),
                        ]
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        // Repository ID
                        container(
                            column![
                                text("Repository ID")
                                    .size(13)
                                    .style(iced::theme::Text::Color(theme.primary())),
                                Space::with_height(Length::Fixed(4.0)),
                                text(&details.id)
                                    .size(15)
                                    .width(Length::Fill),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle))),
                        Space::with_height(Length::Fixed(12.0)),
                        // Name
                        container(
                            column![
                                text("Name")
                                    .size(13)
                                    .style(iced::theme::Text::Color(theme.primary())),
                                Space::with_height(Length::Fixed(4.0)),
                                text(&details.name)
                                    .size(16) // Larger size for emphasis
                                    .style(iced::theme::Text::Color(theme.text())) // Darker for better visibility
                                    .width(Length::Fill),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle))),
                        Space::with_height(Length::Fixed(12.0)),
                        // Enabled status with toggle button
                        container(
                            row![
                                text("Enabled:")
                                    .size(13)
                                    .style(iced::theme::Text::Color(theme.primary()))
                                    .width(Length::Fixed(100.0)),
                                text(if details.enabled { "Yes" } else { "No" })
                                    .size(13)
                                    .style(iced::theme::Text::Color(
                                        if details.enabled {
                                            iced::Color::from_rgb(0.0, 0.8, 0.0)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }
                                    )),
                                Space::with_width(Length::Fill),
                                {
                                    let repo_id = details.id.clone();
                                    button(
                                        row![
                                            text(if details.enabled {
                                                crate::gui::fonts::glyphs::DELETE_SYMBOL
                                            } else {
                                                crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL
                                            })
                                            .font(material_font)
                                            .size(16),
                                            text(if details.enabled { " Disable" } else { " Enable" })
                                        ]
                                        .spacing(4)
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::ToggleRepository(repo_id))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: true,
                                    })))
                                    .padding(Padding::new(10.0))
                                }
                            ]
                            .spacing(12)
                            .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle))),
                        Space::with_height(Length::Fixed(12.0)),
                        // Base URL or Metalink
                        if let Some(ref baseurl) = details.baseurl {
                            container(
                                column![
                                    text("Base URL")
                                        .size(13)
                                        .style(iced::theme::Text::Color(theme.primary())),
                                    Space::with_height(Length::Fixed(4.0)),
                                    text(baseurl)
                                        .size(12)
                                        .width(Length::Fill),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(16.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
                        } else if let Some(ref metalink) = details.metalink {
                            container(
                                column![
                                    text("Metalink")
                                        .size(13)
                                        .style(iced::theme::Text::Color(theme.primary())),
                                    Space::with_height(Length::Fixed(4.0)),
                                    text(metalink)
                                        .size(12)
                                        .width(Length::Fill),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(16.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
                        } else {
                            container(Space::with_height(Length::Shrink))
                                .width(Length::Fill)
                                .into()
                        },
                        Space::with_height(Length::Fixed(12.0)),
                        // File path
                        container(
                            column![
                                text("Source File")
                                    .size(13)
                                    .style(iced::theme::Text::Color(theme.primary())),
                                Space::with_height(Length::Fixed(4.0)),
                                text(&details.file_path)
                                    .size(12)
                                    .width(Length::Fill),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle))),
                        // Additional details
                        if details.gpgcheck.is_some() || details.repo_gpgcheck.is_some() || details.gpgkey.is_some() {
                            column![
                                Space::with_height(Length::Fixed(12.0)),
                                text("Security")
                                    .size(14)
                                    .style(iced::theme::Text::Color(theme.primary())),
                                Space::with_height(Length::Fixed(8.0)),
                                if let Some(gpgcheck) = details.gpgcheck {
                                    container(
                                        row![
                                            text("GPG Check:")
                                                .size(12)
                                                .width(Length::Fixed(120.0)),
                                            text(if gpgcheck { "Enabled" } else { "Disabled" })
                                                .size(12),
                                        ]
                                        .spacing(12)
                                    )
                                    .width(Length::Fill)
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
                                } else {
                                    container(Space::with_height(Length::Shrink))
                                        .width(Length::Fill)
                                        .into()
                                },
                                if let Some(repo_gpgcheck) = details.repo_gpgcheck {
                                    container(
                                        row![
                                            text("Repo GPG Check:")
                                                .size(12)
                                                .width(Length::Fixed(120.0)),
                                            text(if repo_gpgcheck { "Enabled" } else { "Disabled" })
                                                .size(12),
                                        ]
                                        .spacing(12)
                                    )
                                    .width(Length::Fill)
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
                                } else {
                                    container(Space::with_height(Length::Shrink))
                                        .width(Length::Fill)
                                        .into()
                                },
                                if let Some(ref gpgkey) = details.gpgkey {
                                    container(
                                        column![
                                            text("GPG Key:")
                                                .size(12)
                                                .style(iced::theme::Text::Color(theme.primary())),
                                            Space::with_height(Length::Fixed(4.0)),
                                            text(gpgkey)
                                                .size(11)
                                                .width(Length::Fill),
                                        ]
                                        .spacing(0)
                                    )
                                    .width(Length::Fill)
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle)))
                                } else {
                                    container(Space::with_height(Length::Shrink))
                                        .width(Length::Fill)
                                        .into()
                                },
                            ]
                            .spacing(8)
                            .width(Length::Fill)
                        } else {
                            column![].spacing(0).width(Length::Fill)
                        },
                    ]
                    .spacing(0)
                    .padding(Padding::new(25.0))
                )
                .height(Length::Fill)
            )
            .width(Length::Fixed(450.0))
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
                            .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
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
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        // Header section
        let header = container(
            column![
                text("Repositories")
                    .size(28)
                    .style(iced::theme::Text::Color(theme.primary()))
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                Space::with_height(Length::Fixed(8.0)),
                text("Manage and view DNF/YUM repository configurations")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(0.0));

        // Search input
        let search_input = text_input("Search repositories...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .size(16)
            .width(Length::Fill)
            .padding(14)
            .style(iced::theme::TextInput::Custom(Box::new(RoundedTextInputStyle)));

        // Refresh button
        let refresh_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font),
                text(" Refresh")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::LoadRepositories)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(14.0));

        let header_row = row![
            search_input,
            Space::with_width(Length::Fixed(12.0)),
            refresh_button,
        ]
        .spacing(0)
        .align_items(Alignment::Center);

        // Content
        let content: Element<Message> = if self.is_loading {
            container(
                text("Loading repositories...").size(16)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
            .into()
        } else if self.filtered_repositories.is_empty() {
            container(
                {
                    let message = if self.search_query.trim().is_empty() {
                        "No repositories found. Click 'Refresh' to load repositories.".to_string()
                    } else {
                        format!("No repositories found matching '{}'", self.search_query)
                    };
                    text(message)
                        .size(14)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                }
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle)))
            .into()
        } else {
            scrollable(
                column(
                    self.filtered_repositories
                        .iter()
                        .map(|repo| {
                            let repo_id = repo.id.clone();
                            let enabled_color = if repo.enabled {
                                iced::Color::from_rgb(0.0, 0.8, 0.0)
                            } else {
                                iced::Color::from_rgb(0.6, 0.6, 0.6)
                            };
                            
                            let url_display = repo.baseurl.as_ref()
                                .or(repo.metalink.as_ref())
                                .map(|u| {
                                    if u.len() > 60 {
                                        format!("{}...", &u[..60])
                                    } else {
                                        u.clone()
                                    }
                                })
                                .unwrap_or_else(|| "No URL".to_string());

                            button(
                                container(
                                    row![
                                        column![
                                            row![
                                                text(&repo.id)
                                                    .size(16)
                                                    .style(iced::theme::Text::Color(theme.primary()))
                                                    .width(Length::Fill),
                                                Space::with_width(Length::Fixed(12.0)),
                                                container(
                                                    text(if repo.enabled { "Enabled" } else { "Disabled" })
                                                        .size(11)
                                                        .style(iced::theme::Text::Color(enabled_color))
                                                )
                                                .padding(Padding::new(6.0))
                                                .style(iced::theme::Container::Custom(Box::new(StatusBadgeStyle {
                                                    enabled: repo.enabled,
                                                }))),
                                            ]
                                            .spacing(0)
                                            .align_items(Alignment::Center)
                                            .width(Length::Fill),
                                            Space::with_height(Length::Fixed(6.0)),
                                            text(&repo.name)
                                                .size(14) // Larger size for emphasis
                                                .style(iced::theme::Text::Color(theme.text())) // Darker for better visibility
                                                .width(Length::Fill),
                                            Space::with_height(Length::Fixed(4.0)),
                                            row![
                                                text(&url_display)
                                                    .size(12)
                                                    .style(iced::theme::Text::Color(theme.secondary_text()))
                                                    .width(Length::Fill),
                                                Space::with_width(Length::Fixed(8.0)),
                                                text(&repo.file_path)
                                                    .size(11)
                                                    .style(iced::theme::Text::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 1.0))),
                                            ]
                                            .spacing(0)
                                            .align_items(Alignment::Center)
                                            .width(Length::Fill),
                                        ]
                                        .spacing(0)
                                        .width(Length::Fill),
                                    ]
                                    .spacing(0)
                                    .align_items(Alignment::Start)
                                    .padding(16)
                                )
                                .style(iced::theme::Container::Custom(Box::new(RepoItemStyle)))
                            )
                            .on_press(Message::RepositorySelected(repo_id))
                            .style(iced::theme::Button::Text)
                            .padding(0)
                            .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(8)
                .padding(10),
            )
            .into()
        };

        let main_content = container(
            column![
                header_row,
                Space::with_height(Length::Fixed(16.0)),
                content,
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(20.0));

        // Create the slide-out panel
        let panel = if self.panel_open {
            self.view_panel(theme)
        } else {
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(0.0))
                .height(Length::Fill)
                .into()
        };

        container(
            column![
                header,
                Space::with_height(Length::Fixed(24.0)),
                row![main_content, panel]
                    .spacing(15)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(32.0))
        .into()
    }
}

async fn load_repositories() -> Result<Vec<RepositoryInfo>, String> {
    let repos_dir = PathBuf::from("/etc/yum.repos.d");
    
    if !repos_dir.exists() {
        return Err("Repository directory not found".to_string());
    }

    let mut repositories = Vec::new();

    // Read all .repo files
    let entries = std::fs::read_dir(&repos_dir)
        .map_err(|e| format!("Failed to read repository directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("repo") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let repos = parse_repo_file(&content, path.to_string_lossy().to_string());
                repositories.extend(repos);
            }
        }
    }

    // Sort by ID
    repositories.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(repositories)
}

fn parse_repo_file(content: &str, file_path: String) -> Vec<RepositoryInfo> {
    let mut repositories = Vec::new();
    let mut current_section: Option<String> = None;
    let mut current_data: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for section header [section]
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous section if exists
            if let Some(section_id) = current_section.take() {
                if let Some(repo) = build_repository_info(&section_id, &current_data, &file_path) {
                    repositories.push(repo);
                }
            }
            current_data.clear();
            current_section = Some(line[1..line.len()-1].to_string());
        } else if current_section.is_some() {
            // Parse key=value pairs
            if let Some(equal_pos) = line.find('=') {
                let key = line[..equal_pos].trim().to_lowercase();
                let value = line[equal_pos+1..].trim().to_string();
                current_data.insert(key, value);
            }
        }
    }

    // Save last section
    if let Some(section_id) = current_section {
        if let Some(repo) = build_repository_info(&section_id, &current_data, &file_path) {
            repositories.push(repo);
        }
    }

    repositories
}

fn build_repository_info(section_id: &str, data: &HashMap<String, String>, file_path: &str) -> Option<RepositoryInfo> {
    let name = data.get("name").cloned().unwrap_or_else(|| section_id.to_string());
    let baseurl = data.get("baseurl").filter(|s| !s.trim().is_empty() && !s.starts_with('#'))
        .map(|s| s.replace("$releasever", "40").replace("$basearch", "x86_64"));
    let metalink = data.get("metalink").filter(|s| !s.trim().is_empty() && !s.starts_with('#'))
        .map(|s| s.replace("$releasever", "40").replace("$basearch", "x86_64"));
    
    let enabled = data.get("enabled")
        .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    
    let gpgcheck = data.get("gpgcheck")
        .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"));
    
    let repo_gpgcheck = data.get("repo_gpgcheck")
        .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"));

    Some(RepositoryInfo {
        id: section_id.to_string(),
        name,
        baseurl,
        metalink,
        enabled,
        file_path: file_path.to_string(),
        gpgcheck,
        repo_gpgcheck,
    })
}

async fn load_repository_details(repo_id: String) -> RepositoryDetails {
    // Reload repositories to get full details
    let repos = load_repositories().await.unwrap_or_default();
    
    if let Some(repo) = repos.iter().find(|r| r.id == repo_id) {
        // Read the file again to get all details
        if let Ok(content) = std::fs::read_to_string(&repo.file_path) {
            if let Some(details) = parse_repo_details(&content, &repo_id, &repo.file_path) {
                return details;
            }
        }
    }

    // Fallback to basic info
    RepositoryDetails {
        id: repo_id,
        name: String::new(),
        baseurl: None,
        metalink: None,
        enabled: false,
        file_path: String::new(),
        gpgcheck: None,
        repo_gpgcheck: None,
        gpgkey: None,
        _metadata_expire: None,
        _skip_if_unavailable: None,
        _countme: None,
        _repo_type: None,
    }
}

fn parse_repo_details(content: &str, repo_id: &str, file_path: &str) -> Option<RepositoryDetails> {
    let mut in_section = false;
    let mut data: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len()-1];
            if section == repo_id {
                in_section = true;
            } else if in_section {
                break;
            }
        } else if in_section {
            if let Some(equal_pos) = line.find('=') {
                let key = line[..equal_pos].trim().to_lowercase();
                let value = line[equal_pos+1..].trim().to_string();
                data.insert(key, value);
            }
        }
    }

    if !in_section {
        return None;
    }

    let name = data.get("name").cloned().unwrap_or_else(|| repo_id.to_string());
    let baseurl = data.get("baseurl").filter(|s| !s.trim().is_empty() && !s.starts_with('#'))
        .map(|s| s.replace("$releasever", "40").replace("$basearch", "x86_64"));
    let metalink = data.get("metalink").filter(|s| !s.trim().is_empty() && !s.starts_with('#'))
        .map(|s| s.replace("$releasever", "40").replace("$basearch", "x86_64"));
    
    let enabled = data.get("enabled")
        .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    Some(RepositoryDetails {
        id: repo_id.to_string(),
        name,
        baseurl,
        metalink,
        enabled,
        file_path: file_path.to_string(),
        gpgcheck: data.get("gpgcheck").map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true")),
        repo_gpgcheck: data.get("repo_gpgcheck").map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true")),
        gpgkey: data.get("gpgkey").cloned(),
        _metadata_expire: data.get("metadata_expire").cloned(),
        _skip_if_unavailable: data.get("skip_if_unavailable").map(|v| v.trim().eq_ignore_ascii_case("true")),
        _countme: data.get("countme").cloned(),
        _repo_type: data.get("type").cloned(),
    })
}

async fn toggle_repository(repo_id: String, enable: bool) -> Result<String, String> {
    use tokio::process::Command as TokioCommand;
    
    let action = if enable { "set-enabled" } else { "set-disabled" };
    
    let output = TokioCommand::new("pkexec")
        .args(["dnf", "config-manager", &format!("--{}", action), &repo_id])
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf config-manager: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Failed to {} repository: {}\n{}", 
            if enable { "enable" } else { "disable" },
            stderr, stdout));
    }

    Ok(format!("Repository {} {}", 
        repo_id,
        if enable { "enabled" } else { "disabled" }))
}

// Style structs
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

struct RepoItemStyle;

impl iced::widget::container::StyleSheet for RepoItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}

struct StatusBadgeStyle {
    enabled: bool,
}

impl iced::widget::container::StyleSheet for StatusBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(if self.enabled {
                iced::Color::from_rgba(0.0, 0.8, 0.0, 0.15)
            } else {
                iced::Color::from_rgba(0.6, 0.6, 0.6, 0.15)
            })),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: if self.enabled {
                    iced::Color::from_rgba(0.0, 0.8, 0.0, 0.3)
                } else {
                    iced::Color::from_rgba(0.6, 0.6, 0.6, 0.3)
                },
            },
            ..Default::default()
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
                palette.background.r * 0.96,
                palette.background.g * 0.96,
                palette.background.b * 0.96,
                1.0,
            ))),
            border: Border {
                radius: 12.0.into(),
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

