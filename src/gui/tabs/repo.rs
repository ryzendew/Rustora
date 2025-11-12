use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;

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
    // Terminal messages
    OpenAddRepoTerminal,
    CloseAddRepoTerminal,
    TerminalCommandChanged(String),
    ExecuteTerminalCommand,
    TerminalCommandOutput(String, String, bool), // stdout, stderr, success
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
    // Terminal state
    terminal_open: bool,
    terminal_command: String,
    terminal_output: Vec<String>,
    terminal_is_executing: bool,
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
            terminal_open: false,
            terminal_command: String::new(),
            terminal_output: Vec::new(),
            terminal_is_executing: false,
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
            Message::OpenAddRepoTerminal => {
                self.terminal_open = true;
                self.terminal_command = String::new();
                self.terminal_output = vec!["# Add Repository Terminal".to_string(), 
                                             "# Enter a command to add a repository, e.g.:".to_string(),
                                             "# dnf config-manager --add-repo https://example.com/repo.repo".to_string(),
                                             "# or".to_string(),
                                             "# dnf config-manager --add-repo 'https://example.com/repo.repo'".to_string(),
                                             "".to_string()];
                iced::Command::none()
            }
            Message::CloseAddRepoTerminal => {
                self.terminal_open = false;
                self.terminal_command = String::new();
                self.terminal_output = Vec::new();
                iced::Command::none()
            }
            Message::TerminalCommandChanged(cmd) => {
                self.terminal_command = cmd;
                iced::Command::none()
            }
            Message::ExecuteTerminalCommand => {
                if self.terminal_is_executing || self.terminal_command.trim().is_empty() {
                    return iced::Command::none();
                }
                let command = self.terminal_command.clone();
                self.terminal_is_executing = true;
                self.terminal_output.push(format!("$ {}", command));
                iced::Command::perform(execute_terminal_command(command), |result| {
                    match result {
                        Ok((stdout, stderr, success)) => Message::TerminalCommandOutput(stdout, stderr, success),
                        Err(e) => Message::TerminalCommandOutput(String::new(), e, false),
                    }
                })
            }
            Message::TerminalCommandOutput(stdout, stderr, success) => {
                self.terminal_is_executing = false;
                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        self.terminal_output.push(line.to_string());
                    }
                }
                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        self.terminal_output.push(format!("[stderr] {}", line));
                    }
                }
                if success {
                    self.terminal_output.push("✓ Command completed successfully".to_string());
                    // Auto-refresh repositories if command was successful
                    self.terminal_command = String::new();
                    return iced::Command::batch(vec![
                        iced::Command::perform(async {}, |_| Message::LoadRepositories),
                    ]);
                } else {
                    self.terminal_output.push("✗ Command failed".to_string());
                }
                self.terminal_output.push("".to_string());
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

        // Add Repository button
        let add_repo_button = button(
            row![
                text(crate::gui::fonts::glyphs::ADD_SYMBOL).font(material_font),
                text(" Add Repository")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::OpenAddRepoTerminal)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
        })))
        .padding(Padding::new(14.0));

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
            add_repo_button,
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

        // Terminal UI
        let terminal_ui: Element<Message> = if self.terminal_open {
            let terminal_output = scrollable(
                column(
                    self.terminal_output
                        .iter()
                        .map(|line| {
                            let line_color = if line.starts_with('#') {
                                theme.secondary_text()
                            } else if line.starts_with('$') {
                                theme.primary()
                            } else if line.starts_with("✓") {
                                iced::Color::from_rgb(0.1, 0.5, 0.1)
                            } else if line.starts_with("✗") || line.starts_with("[stderr]") {
                                iced::Color::from_rgb(0.9, 0.2, 0.2)
                            } else {
                                theme.text()
                            };
                            text(line)
                                .size(13)
                                .style(iced::theme::Text::Color(line_color))
                                .font(iced::Font::MONOSPACE)
                                .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(2)
                .padding(12)
            )
            .width(Length::Fill)
            .height(Length::Fill);

            let command_input = text_input("Enter command...", &self.terminal_command)
                .on_input(Message::TerminalCommandChanged)
                .on_submit(Message::ExecuteTerminalCommand)
                .size(14)
                .width(Length::Fill)
                .padding(12)
                .style(iced::theme::TextInput::Custom(Box::new(TerminalInputStyle)))
                .font(iced::Font::MONOSPACE);

            let execute_button = button(
                row![
                    text("Execute")
                        .size(14)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::ExecuteTerminalCommand)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
            })))
            .padding(Padding::new(12.0));

            let close_button = button(
                row![
                    text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(18),
                    text(" Close")
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::CloseAddRepoTerminal)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::new(12.0));

            container(
                column![
                    row![
                        text("Add Repository Terminal")
                            .size(20)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_width(Length::Fill),
                        close_button,
                    ]
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    terminal_output,
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        command_input,
                        Space::with_width(Length::Fixed(12.0)),
                        execute_button,
                    ]
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle)))
            .into()
        } else {
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(0.0))
                .height(Length::Fill)
                .into()
        };

        let main_content: Element<Message> = if self.terminal_open {
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(0.0))
                .height(Length::Fill)
                .into()
        } else {
            container(
                column![
                    header_row,
                    Space::with_height(Length::Fixed(16.0)),
                    content,
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(20.0))
            .into()
        };

        // Create the slide-out panel
        let panel = if self.panel_open && !self.terminal_open {
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
                if self.terminal_open {
                    terminal_ui
                } else {
                    row![main_content, panel]
                        .spacing(15)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                },
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

async fn execute_terminal_command(command: String) -> Result<(String, String, bool), String> {
    // Parse the command - support simple commands like "dnf config-manager --add-repo ..."
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let program = parts[0];
    let args = &parts[1..];

    // Use pkexec for commands that need root privileges (like dnf config-manager)
    let needs_root = program == "dnf" || program == "yum" || program == "rpm";
    
    let output = if needs_root {
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg(program);
        cmd.args(args);
        cmd.output().await
    } else {
        let mut cmd = TokioCommand::new(program);
        cmd.args(args);
        cmd.output().await
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let success = output.status.success();
            Ok((stdout, stderr, success))
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

struct TerminalContainerStyle;

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.08, 0.08, 0.08, 1.0).into()
            } else {
                iced::Color::from_rgba(0.95, 0.95, 0.96, 1.0).into()
            }),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: if is_dark {
                    iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0)
                } else {
                    iced::Color::from_rgba(0.7, 0.7, 0.72, 0.4)
                },
            },
            ..Default::default()
        }
    }
}

struct TerminalInputStyle;

impl TextInputStyleSheet for TerminalInputStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        TextInputAppearance {
            background: if is_dark {
                iced::Color::from_rgba(0.1, 0.1, 0.1, 1.0).into()
            } else {
                iced::Color::from_rgba(0.98, 0.98, 0.99, 1.0).into()
            },
            border: Border::with_radius(8.0),
            icon_color: palette.text,
        }
    }

    fn focused(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        TextInputAppearance {
            background: if is_dark {
                iced::Color::from_rgba(0.1, 0.1, 0.1, 1.0).into()
            } else {
                iced::Color::from_rgba(0.98, 0.98, 0.99, 1.0).into()
            },
            border: Border {
                radius: 8.0.into(),
                width: 2.0,
                color: palette.primary,
            },
            icon_color: palette.primary,
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> iced::Color {
        let palette = style.palette();
        iced::Color::from_rgba(palette.text.r, palette.text.g, palette.text.b, 0.5)
    }

    fn value_color(&self, style: &Self::Style) -> iced::Color {
        style.palette().text
    }

    fn disabled_color(&self, style: &Self::Style) -> iced::Color {
        let palette = style.palette();
        iced::Color::from_rgba(palette.text.r, palette.text.g, palette.text.b, 0.5)
    }

    fn selection_color(&self, style: &Self::Style) -> iced::Color {
        style.palette().primary
    }

    fn disabled(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        TextInputAppearance {
            background: if is_dark {
                iced::Color::from_rgba(0.05, 0.05, 0.05, 1.0).into()
            } else {
                iced::Color::from_rgba(0.95, 0.95, 0.95, 1.0).into()
            },
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: if is_dark {
                    iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0)
                } else {
                    iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0)
                },
            },
            icon_color: iced::Color::from_rgba(palette.text.r, palette.text.g, palette.text.b, 0.5),
        }
    }
}

