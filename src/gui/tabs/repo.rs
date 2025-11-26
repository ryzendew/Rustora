use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding, Border, Color};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use crate::gui::app::CustomScrollableStyle;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoView {
    All,
    Nvidia,
    RpmFusion,
}

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

    OpenAddRepoTerminal,
    CloseAddRepoTerminal,
    TerminalCommandChanged(String),
    ExecuteTerminalCommand,
    TerminalCommandOutput(String, String, bool),
    TerminalPromptResponse(bool),

    SwitchView(RepoView),
    InstallNvidiaRepo,
    InstallNvidiaRepoComplete(Result<(), String>),
    InstallRpmFusionRepos,
    InstallRpmFusionReposComplete(Result<(), String>),
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

    terminal_open: bool,
    terminal_command: String,
    terminal_output: Vec<String>,
    terminal_is_executing: bool,
    terminal_pending_prompt: Option<String>,

    current_view: RepoView,
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
            terminal_pending_prompt: None,
            current_view: RepoView::All,
        }
    }

    fn filter_repositories(&mut self) {
        self.filtered_repositories.clear();
        
        let query_lower = if self.search_query.trim().is_empty() {
            String::new()
        } else {
            self.search_query.to_lowercase()
        };
        
        let view_filter = match self.current_view {
            RepoView::Nvidia => Some("nvidia"),
            RepoView::RpmFusion => Some("rpmfusion"),
            RepoView::All => None,
        };
        
        self.filtered_repositories.reserve(self.repositories.len().min(200));
        
        for repo in &self.repositories {
            let matches_query = query_lower.is_empty() || 
                repo.id.to_lowercase().contains(&query_lower) ||
                repo.name.to_lowercase().contains(&query_lower) ||
                repo.file_path.to_lowercase().contains(&query_lower) ||
                repo.baseurl.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                repo.metalink.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false);
            
            let matches_view = view_filter.map(|filter| {
                repo.id.to_lowercase().contains(filter) ||
                repo.name.to_lowercase().contains(filter) ||
                (filter == "rpmfusion" && repo.name.to_lowercase().contains("rpm fusion")) ||
                repo.baseurl.as_ref().map(|u| u.to_lowercase().contains(filter)).unwrap_or(false) ||
                repo.metalink.as_ref().map(|u| u.to_lowercase().contains(filter)).unwrap_or(false)
            }).unwrap_or(true);
            
            if matches_query && matches_view {
                self.filtered_repositories.push(repo.clone());
                if self.filtered_repositories.len() >= 200 {
                    break;
                }
            }
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

                        self.is_loading = true;

                        if let Some(ref repo_id) = self.selected_repository {
                            let repo_id = repo_id.clone();
                            return iced::Command::batch([
                                iced::Command::perform(load_repositories(), |result| {
                                    match result {
                                        Ok(repos) => Message::RepositoriesLoaded(repos),
                                        Err(e) => Message::Error(e),
                                    }
                                }),
                                iced::Command::perform(load_repository_details(repo_id), Message::RepositoryDetailsLoaded),
                            ]);
                        }
                        iced::Command::perform(load_repositories(), |result| {
                            match result {
                                Ok(repos) => Message::RepositoriesLoaded(repos),
                                Err(e) => Message::Error(e),
                            }
                        })
                    }
                    Err(e) => {
                        self.is_loading = false;

                        iced::Command::perform(async {}, |_| Message::Error(e))
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
                                             "# Commands requiring root privileges (dnf, yum, rpm) will prompt for sudo password".to_string(),
                                             "# Enter a command to add a repository, e.g.:".to_string(),
                                             "# dnf config-manager --add-repo https://example.com/repo.repo".to_string(),
                                             "# or".to_string(),
                                             "# dnf config-manager --add-repo 'https://example.com/repo.repo'".to_string(),
                                             "# Note: sudo will be automatically requested when needed".to_string(),
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
                // Check if we're waiting for a prompt response
                if self.terminal_pending_prompt.is_some() {
                    // User needs to respond to the prompt first
                    return iced::Command::none();
                }
                let command = self.terminal_command.clone();
                self.terminal_is_executing = true;
                self.terminal_output.push(format!("$ {}", command));
                iced::Command::perform(execute_terminal_command_interactive(command), |result| {
                    match result {
                        Ok((stdout, stderr, success, _prompt)) => {
                            // Prompt detection is handled in TerminalCommandOutput handler
                            Message::TerminalCommandOutput(stdout, stderr, success)
                        }
                        Err(e) => Message::TerminalCommandOutput(String::new(), e, false),
                    }
                })
            }
            Message::TerminalPromptResponse(response) => {
                if let Some(prompt_text) = self.terminal_pending_prompt.take() {
                    let command = self.terminal_command.clone();
                    let response_char = if response { "y" } else { "n" };
                    self.terminal_output.push(format!("[User responded: {}]", if response { "Yes" } else { "No" }));
                    iced::Command::perform(
                        send_prompt_response(command, prompt_text, response_char.to_string()),
                        |result| {
                            match result {
                                Ok((stdout, stderr, success)) => Message::TerminalCommandOutput(stdout, stderr, success),
                                Err(e) => Message::TerminalCommandOutput(String::new(), e, false),
                            }
                        }
                    )
                } else {
                    iced::Command::none()
                }
            }
            Message::TerminalCommandOutput(stdout, stderr, success) => {
                // Check if output contains a prompt
                let full_output = format!("{}\n{}", stdout, stderr);
                if let Some(prompt_text) = detect_prompt(&full_output) {
                    // Store the prompt and show dialog
                    self.terminal_pending_prompt = Some(prompt_text.clone());
                    self.terminal_output.push(format!("[Prompt detected: {}]", prompt_text));
                    // Show dialog using zenity/kdialog
                    return iced::Command::perform(
                        show_prompt_dialog(prompt_text),
                        |result| {
                            if let Some(response) = result {
                                Message::TerminalPromptResponse(response)
                            } else {
                                // User cancelled, treat as no
                                Message::TerminalPromptResponse(false)
                            }
                        }
                    );
                }

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
                    self.terminal_output.push("[OK] Command completed successfully".to_string());
                    // Auto-refresh repositories if command was successful
                    self.terminal_command = String::new();
                    return iced::Command::batch(vec![
                        iced::Command::perform(async {}, |_| Message::LoadRepositories),
                    ]);
                } else {
                    self.terminal_output.push("[FAIL] Command failed".to_string());
                }
                self.terminal_output.push("".to_string());
                iced::Command::none()
            }
            Message::SwitchView(view) => {
                self.current_view = view;
                self.filter_repositories();
                iced::Command::none()
            }
            Message::InstallNvidiaRepo => {
                self.is_loading = true;
                iced::Command::perform(install_nvidia_repo(), |result| {
                    match result {
                        Ok(_) => Message::InstallNvidiaRepoComplete(Ok(())),
                        Err(e) => Message::InstallNvidiaRepoComplete(Err(e)),
                    }
                })
            }
            Message::InstallNvidiaRepoComplete(result) => {
                self.is_loading = false;
                match result {
                    Ok(_) => {
                        // Refresh repository list after installation
                        iced::Command::perform(load_repositories(), |result| {
                            match result {
                                Ok(repos) => Message::RepositoriesLoaded(repos),
                                Err(e) => Message::Error(e),
                            }
                        })
                    }
                    Err(e) => {
                        iced::Command::perform(async {}, move |_| {
                            Message::Error(format!("Failed to install NVIDIA repository: {}", e))
                        })
                    }
                }
            }
            Message::InstallRpmFusionRepos => {
                self.is_loading = true;
                iced::Command::perform(install_rpmfusion_repos(), |result| {
                    match result {
                        Ok(_) => Message::InstallRpmFusionReposComplete(Ok(())),
                        Err(e) => Message::InstallRpmFusionReposComplete(Err(e)),
                    }
                })
            }
            Message::InstallRpmFusionReposComplete(result) => {
                self.is_loading = false;
                match result {
                    Ok(_) => {
                        // Refresh repository list after installation
                        iced::Command::perform(load_repositories(), |result| {
                            match result {
                                Ok(repos) => Message::RepositoriesLoaded(repos),
                                Err(e) => Message::Error(e),
                            }
                        })
                    }
                    Err(e) => {
                        iced::Command::perform(async {}, move |_| {
                            Message::Error(format!("Failed to install RPM Fusion repositories: {}", e))
                        })
                    }
                }
            }
        }
    }

    fn view_panel(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        // Calculate font sizes from settings
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        if let Some(ref details) = self.repository_details {
            let material_font = crate::gui::fonts::get_material_symbols_font();

            container(
                scrollable(
                    column![
                        // Header with close button
                        row![
                            text("Repository Details")
                                .size(title_font_size * 0.64)
                                .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                            Space::with_width(Length::Fill),
                            button(
                                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_size)
                            )
                            .on_press(Message::ClosePanel)
                            .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle {
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::new(6.0)),
                        ]
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        // Repository ID
                        container(
                            column![
                                text("Repository ID")
                                    .size(body_font_size * 0.93)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                Space::with_height(Length::Fixed(4.0)),
                                text(&details.id)
                                    .size(body_font_size * 1.07)
                                    .width(Length::Fill),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                            radius: settings.border_radius,
                        }))),
                        Space::with_height(Length::Fixed(12.0)),
                        // Name
                        container(
                            column![
                                text("Name")
                                    .size(body_font_size * 0.93)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                Space::with_height(Length::Fixed(4.0)),
                                text(&details.name)
                                    .size(body_font_size * 1.14) // Larger size for emphasis
                                    .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings)))) // Darker for better visibility
                                    .width(Length::Fill),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                            radius: settings.border_radius,
                        }))),
                        Space::with_height(Length::Fixed(12.0)),
                        // Enabled status with toggle button
                        container(
                            row![
                                text("Enabled:")
                                    .size(body_font_size * 0.93)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                                    .width(Length::Fixed(100.0)),
                                text(if details.enabled { "Yes" } else { "No" })
                                    .size(body_font_size * 0.93)
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
                                            .size(icon_size),
                                            text(if details.enabled { " Disable" } else { " Enable" }).size(button_font_size)
                                        ]
                                        .spacing(4)
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::ToggleRepository(repo_id))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: true,
                                        radius: settings.border_radius,
                                    })))
                                    .padding(Padding::new(10.0))
                                }
                            ]
                            .spacing(12)
                            .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                            radius: settings.border_radius,
                        }))),
                        Space::with_height(Length::Fixed(12.0)),
                        // Base URL or Metalink
                        if let Some(ref baseurl) = details.baseurl {
                            container(
                                column![
                                    text("Base URL")
                                        .size(body_font_size * 0.93)
                                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    Space::with_height(Length::Fixed(4.0)),
                                    text(baseurl)
                                        .size(body_font_size * 0.86)
                                        .width(Length::Fill),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(16.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                                radius: settings.border_radius,
                            })))
                        } else if let Some(ref metalink) = details.metalink {
                            container(
                                column![
                                    text("Metalink")
                                        .size(body_font_size * 0.93)
                                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                    Space::with_height(Length::Fixed(4.0)),
                                    text(metalink)
                                        .size(body_font_size * 0.86)
                                        .width(Length::Fill),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(16.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                                radius: settings.border_radius,
                            })))
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
                                    .size(body_font_size * 0.93)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                Space::with_height(Length::Fixed(4.0)),
                                text(&details.file_path)
                                    .size(body_font_size * 0.86)
                                    .width(Length::Fill),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                            radius: settings.border_radius,
                        }))),
                        // Additional details
                        if details.gpgcheck.is_some() || details.repo_gpgcheck.is_some() || details.gpgkey.is_some() {
                            column![
                                Space::with_height(Length::Fixed(12.0)),
                                text("Security")
                                    .size(body_font_size)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                Space::with_height(Length::Fixed(8.0)),
                                if let Some(gpgcheck) = details.gpgcheck {
                                    container(
                                        row![
                                            text("GPG Check:")
                                                .size(body_font_size * 0.86)
                                                .width(Length::Fixed(120.0)),
                                            text(if gpgcheck { "Enabled" } else { "Disabled" })
                                                .size(body_font_size * 0.86),
                                        ]
                                        .spacing(12)
                                    )
                                    .width(Length::Fill)
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                                radius: settings.border_radius,
                            })))
                                } else {
                                    container(Space::with_height(Length::Shrink))
                                        .width(Length::Fill)
                                        .into()
                                },
                                if let Some(repo_gpgcheck) = details.repo_gpgcheck {
                                    container(
                                        row![
                                            text("Repo GPG Check:")
                                                .size(body_font_size * 0.86)
                                                .width(Length::Fixed(120.0)),
                                            text(if repo_gpgcheck { "Enabled" } else { "Disabled" })
                                                .size(body_font_size * 0.86),
                                        ]
                                        .spacing(12)
                                    )
                                    .width(Length::Fill)
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                                radius: settings.border_radius,
                            })))
                                } else {
                                    container(Space::with_height(Length::Shrink))
                                        .width(Length::Fill)
                                        .into()
                                },
                                if let Some(ref gpgkey) = details.gpgkey {
                                    container(
                                        column![
                                            text("GPG Key:")
                                                .size(body_font_size * 0.86)
                                                .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                            Space::with_height(Length::Fixed(4.0)),
                                            text(gpgkey)
                                                .size(body_font_size * 0.79)
                                                .width(Length::Fill),
                                        ]
                                        .spacing(0)
                                    )
                                    .width(Length::Fill)
                                    .padding(Padding::new(12.0))
                                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                                radius: settings.border_radius,
                            })))
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
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                    Color::from(settings.background_color.clone()),
                    settings.border_radius,
                ))))
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
                        {
                            let material_font = crate::gui::fonts::get_material_symbols_font();
                            button(
                                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_size)
                            )
                            .on_press(Message::ClosePanel)
                            .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle {
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::new(8.0))
                        },
                    ]
                    .width(Length::Fill),
                    Space::with_height(Length::Fill),
                    text("Loading...").size(body_font_size * 1.14).horizontal_alignment(iced::alignment::Horizontal::Center),
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

    pub fn view(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        // Calculate font sizes from settings
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let input_font_size = (settings.font_size_inputs * settings.scale_inputs).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let tab_font_size = (settings.font_size_tabs * settings.scale_tabs).round();

        // Header section
        let header = container(
            column![
                text("Repositories")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                Space::with_height(Length::Fixed(8.0)),
                text("Manage and view DNF/YUM repository configurations")
                    .size(body_font_size)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(0.0));

        // Search input
        let search_input = text_input("Search repositories...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .size(input_font_size)
            .width(Length::Fill)
            .padding(14)
            .style(iced::theme::TextInput::Custom(Box::new(RoundedTextInputStyle {
                radius: settings.border_radius,
            })));

        // Add Repository button
        let add_repo_button = button(
            row![
                text(crate::gui::fonts::glyphs::ADD_SYMBOL).font(material_font).size(icon_size),
                text(" Add Repository").size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::OpenAddRepoTerminal)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(14.0));

        // Refresh button
        let refresh_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                text(" Refresh").size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::LoadRepositories)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(14.0));

        // Sub-tabs for All, NVIDIA, and RPM Fusion
        let sub_tabs = container(
            row![
                button(
                    text("All")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == RepoView::All {
                            iced::Color::WHITE
                        } else {
                            theme.text_with_settings(Some(settings))
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == RepoView::All,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(RepoView::All))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
                button(
                    text("NVIDIA")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == RepoView::Nvidia {
                            iced::Color::WHITE
                        } else {
                            theme.text_with_settings(Some(settings))
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == RepoView::Nvidia,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(RepoView::Nvidia))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
                button(
                    text("RPM Fusion")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == RepoView::RpmFusion {
                            iced::Color::WHITE
                        } else {
                            theme.text_with_settings(Some(settings))
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == RepoView::RpmFusion,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(RepoView::RpmFusion))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
            ]
            .spacing(12)
        )
        .width(Length::Fill)
        .padding(Padding::from([0.0, 0.0, 20.0, 0.0]));

        let header_row = row![
            search_input,
            Space::with_width(Length::Fixed(16.0)),
            add_repo_button,
            Space::with_width(Length::Fixed(12.0)),
            refresh_button,
        ]
        .spacing(0)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Content - show install buttons based on current view
        let content: Element<Message> = if self.current_view == RepoView::Nvidia && !self.is_loading {
            // NVIDIA sub-tab content
            let nvidia_install_button = button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                    text(" Install NVIDIA Driver Repository")
                        .size(button_font_size)
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .on_press(Message::InstallNvidiaRepo)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(16.0));

            let nvidia_info = container(
                column![
                    text("NVIDIA Driver Repository")
                        .size(title_font_size * 0.71)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(12.0)),
                    text("This will install the NVIDIA driver repository from negativo17.org")
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(8.0)),
                    text("Repository URL: https://negativo17.org/repos/fedora-nvidia.repo")
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                        .font(iced::Font::MONOSPACE),
                    Space::with_height(Length::Fixed(24.0)),
                    nvidia_install_button,
                ]
                .spacing(8)
                .padding(Padding::new(24.0))
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })));

            let repo_list: Element<Message> = if self.filtered_repositories.is_empty() {
                container(
                    text("No NVIDIA repositories found. Click 'Install NVIDIA Driver Repository' to add one.")
                        .size(body_font_size)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .padding(20)
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
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

                                container(
                                    row![
                                button(
                                    container(
                                        row![
                                            column![
                                                row![
                                                            text(shorten_repo_id(&repo.id))
                                                        .size(body_font_size * 1.14)
                                                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                                                        .width(Length::Fill),
                                                    Space::with_width(Length::Fixed(12.0)),
                                                    container(
                                                        text(if repo.enabled { "Enabled" } else { "Disabled" })
                                                            .size(body_font_size * 0.79)
                                                            .style(iced::theme::Text::Color(enabled_color))
                                                    )
                                                    .padding(Padding::new(6.0))
                                                    .style(iced::theme::Container::Custom(Box::new(StatusBadgeStyle {
                                                        enabled: repo.enabled,
                                                        radius: settings.border_radius,
                                                    }))),
                                                ]
                                                .spacing(0)
                                                .align_items(Alignment::Center)
                                                .width(Length::Fill),
                                                Space::with_height(Length::Fixed(6.0)),
                                                text(&repo.name)
                                                    .size(body_font_size)
                                                    .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings))))
                                                    .width(Length::Fill),
                                                Space::with_height(Length::Fixed(4.0)),
                                                text(&url_display)
                                                    .size(body_font_size * 0.86)
                                                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                                    .width(Length::Fill),
                                            ]
                                            .spacing(0)
                                            .width(Length::Fill),
                                        ]
                                        .spacing(0)
                                        .align_items(Alignment::Start)
                                        .padding(16)
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(RepoItemStyle {
                                        radius: settings.border_radius,
                                    })))
                                )
                                        .on_press(Message::RepositorySelected(repo_id.clone()))
                                .style(iced::theme::Button::Text)
                                .padding(0)
                                        .width(Length::Fill),
                                        Space::with_width(Length::Fixed(12.0)),
                                        {
                                            let repo_id_toggle = repo_id.clone();
                                            button(
                                                row![
                                                    text(if repo.enabled {
                                                        crate::gui::fonts::glyphs::DELETE_SYMBOL
                                                    } else {
                                                        crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL
                                                    })
                                                    .font(material_font)
                                                    .size(icon_size),
                                                    text(if repo.enabled { " Disable" } else { " Enable" })
                                                        .size(button_font_size * 0.9)
                                                ]
                                                .spacing(4)
                                                .align_items(Alignment::Center)
                                            )
                                            .on_press(Message::ToggleRepository(repo_id_toggle))
                                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                is_primary: !repo.enabled,
                                                radius: settings.border_radius,
                                            })))
                                            .padding(Padding::new(10.0))
                                        }
                                    ]
                                    .spacing(0)
                                    .align_items(Alignment::Center)
                                    .width(Length::Fill)
                                )
                                .width(Length::Fill)
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(8)
                    .padding(10),
                )
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                    Color::from(settings.background_color.clone()),
                    settings.border_radius,
                ))))
                .into()
            };

            column![
                nvidia_info,
                Space::with_height(Length::Fixed(16.0)),
                repo_list,
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.current_view == RepoView::RpmFusion && !self.is_loading {
            // RPM Fusion sub-tab content
            let rpmfusion_install_button: Element<Message> = button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                    text(" Install RPM Fusion Repositories")
                        .size(button_font_size)
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .on_press(Message::InstallRpmFusionRepos)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(16.0))
            .into();

            let rpmfusion_info = container(
                column![
                    text("RPM Fusion Repositories")
                        .size(title_font_size * 0.71)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(12.0)),
                    text("This will install both RPM Fusion Free and Nonfree repositories")
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(8.0)),
                    text("These repositories provide additional software including NVIDIA drivers, multimedia codecs, and other packages not available in the official Fedora repositories")
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(24.0)),
                    rpmfusion_install_button,
                ]
                .spacing(8)
                .padding(Padding::new(24.0))
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })));

            let repo_list: Element<Message> = if self.filtered_repositories.is_empty() {
                container(
                    text("No RPM Fusion repositories found. Click 'Install RPM Fusion Repositories' to add them.")
                        .size(body_font_size)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .padding(20)
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })))
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

                            container(
                                row![
                                button(
                                    container(
                                        row![
                                            column![
                                                row![
                                                    text(&repo.id)
                                                        .size(body_font_size * 1.14)
                                                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                                                        .width(Length::Fill),
                                                    Space::with_width(Length::Fixed(12.0)),
                                                    container(
                                                        text(if repo.enabled { "Enabled" } else { "Disabled" })
                                                            .size(body_font_size * 0.79)
                                                            .style(iced::theme::Text::Color(enabled_color))
                                                    )
                                                    .padding(Padding::new(6.0))
                                                    .style(iced::theme::Container::Custom(Box::new(StatusBadgeStyle {
                                                        enabled: repo.enabled,
                                                        radius: settings.border_radius,
                                                    }))),
                                                ]
                                                .spacing(0)
                                                .align_items(Alignment::Center)
                                                .width(Length::Fill),
                                                Space::with_height(Length::Fixed(6.0)),
                                                text(&repo.name)
                                                    .size(body_font_size)
                                                    .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings))))
                                                    .width(Length::Fill),
                                                Space::with_height(Length::Fixed(4.0)),
                                                text(&url_display)
                                                    .size(body_font_size * 0.86)
                                                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                                    .width(Length::Fill),
                                            ]
                                            .spacing(0)
                                            .width(Length::Fill),
                                        ]
                                        .spacing(0)
                                        .align_items(Alignment::Start)
                                        .padding(16)
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(RepoItemStyle {
                                        radius: settings.border_radius,
                                    })))
                                )
                                    .on_press(Message::RepositorySelected(repo_id.clone()))
                                .style(iced::theme::Button::Text)
                                .padding(0)
                                    .width(Length::Fill),
                                    Space::with_width(Length::Fixed(12.0)),
                                    {
                                        let repo_id_toggle = repo_id.clone();
                                        button(
                                            row![
                                                text(if repo.enabled {
                                                    crate::gui::fonts::glyphs::DELETE_SYMBOL
                                                } else {
                                                    crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL
                                                })
                                                .font(material_font)
                                                .size(icon_size),
                                                text(if repo.enabled { " Disable" } else { " Enable" })
                                                    .size(button_font_size * 0.9)
                                            ]
                                            .spacing(4)
                                            .align_items(Alignment::Center)
                                        )
                                        .on_press(Message::ToggleRepository(repo_id_toggle))
                                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                            is_primary: !repo.enabled,
                                            radius: settings.border_radius,
                                        })))
                                        .padding(Padding::new(10.0))
                                    }
                                ]
                                .spacing(0)
                                .align_items(Alignment::Center)
                                .width(Length::Fill)
                            )
                            .width(Length::Fill)
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(8)
                    .padding(10),
                )
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                    Color::from(settings.background_color.clone()),
                    settings.border_radius,
                ))))
                .into()
            };

            column![
                rpmfusion_info,
                Space::with_height(Length::Fixed(16.0)),
                repo_list,
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.is_loading {
            container(
                text("Loading repositories...").size(body_font_size * 1.14)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })))
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
                        .size(body_font_size)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                }
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else {
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
                            .cloned()
                                .unwrap_or_else(|| "No URL".to_string());

                        // Clean, spacious card design
                                container(
                                    row![
                                    // Main clickable repository card - takes most space
                                    button(
                                        container(
                                        column![
                                                // Top row: ID and status badge
                                            row![
                                                    text(shorten_repo_id(&repo.id))
                                                        .size(body_font_size * 1.2)
                                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                                                    .width(Length::Fill),
                                                    Space::with_width(Length::Fixed(16.0)),
                                                container(
                                                    text(if repo.enabled { "Enabled" } else { "Disabled" })
                                                            .size(body_font_size * 0.8)
                                                        .style(iced::theme::Text::Color(enabled_color))
                                                )
                                                    .padding(Padding::from([6.0, 12.0, 6.0, 12.0]))
                                                .style(iced::theme::Container::Custom(Box::new(StatusBadgeStyle {
                                                    radius: settings.border_radius,
                                                    enabled: repo.enabled,
                                                }))),
                                            ]
                                            .spacing(0)
                                            .align_items(Alignment::Center)
                                            .width(Length::Fill),
                                                Space::with_height(Length::Fixed(14.0)),
                                                // Repository name
                                            text(&repo.name)
                                                    .size(body_font_size)
                                                    .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings))))
                                                .width(Length::Fill),
                                                Space::with_height(Length::Fixed(14.0)),
                                                // URL - full width, wraps properly
                                                column![
                                                    text("URL")
                                                        .size(body_font_size * 0.85)
                                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                                            Space::with_height(Length::Fixed(4.0)),
                                                text(&url_display)
                                                        .size(body_font_size * 0.9)
                                                        .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings))))
                                                        .width(Length::Fill)
                                                        .shaping(iced::widget::text::Shaping::Advanced),
                                            ]
                                            .spacing(0)
                                            .width(Length::Fill),
                                                Space::with_height(Length::Fixed(12.0)),
                                                // File path - full width
                                                column![
                                                    text("File")
                                                        .size(body_font_size * 0.85)
                                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                                                    Space::with_height(Length::Fixed(4.0)),
                                                    text(&repo.file_path)
                                                        .size(body_font_size * 0.85)
                                                        .style(iced::theme::Text::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 1.0)))
                                                        .width(Length::Fill)
                                                        .shaping(iced::widget::text::Shaping::Advanced),
                                        ]
                                        .spacing(0)
                                        .width(Length::Fill),
                                    ]
                                    .spacing(0)
                                            .width(Length::Fill)
                                            .padding(Padding::from([22.0, 26.0, 22.0, 26.0]))
                                )
                                .style(iced::theme::Container::Custom(Box::new(RepoItemStyle {
                                    radius: settings.border_radius,
                                })))
                            )
                                    .on_press(Message::RepositorySelected(repo_id.clone()))
                            .style(iced::theme::Button::Text)
                            .padding(0)
                                    .width(Length::Fill),
                                    Space::with_width(Length::Fixed(20.0)),
                                    // Toggle button - vertical layout
                                    {
                                        let repo_id_toggle = repo_id.clone();
                                        button(
                                            column![
                                                text(if repo.enabled {
                                                    crate::gui::fonts::glyphs::DELETE_SYMBOL
                                                } else {
                                                    crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL
                                                })
                                                .font(material_font)
                                                .size(icon_size * 1.3),
                                                Space::with_height(Length::Fixed(8.0)),
                                                text(if repo.enabled { "Disable" } else { "Enable" })
                                                    .size(button_font_size * 0.9)
                                            ]
                                            .spacing(0)
                                            .align_items(Alignment::Center)
                                        )
                                        .on_press(Message::ToggleRepository(repo_id_toggle))
                                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                            is_primary: !repo.enabled,
                                            radius: settings.border_radius,
                                        })))
                                        .padding(Padding::from([18.0, 24.0, 18.0, 24.0]))
                                    }
                                ]
                                .spacing(0)
                                .align_items(Alignment::Start)
                                .width(Length::Fill)
                            )
                        .width(Length::Fill)
                            .into()
                        })
                            .collect::<Vec<_>>(),
                    )
                .spacing(16)
                .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
                .width(Length::Fill)
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
                                theme.secondary_text_with_settings(Some(settings))
                            } else if line.starts_with('$') {
                                theme.primary_with_settings(Some(settings))
                            } else if line.starts_with("[OK]") {
                                iced::Color::from_rgb(0.1, 0.5, 0.1)
                            } else if line.starts_with("[FAIL]") || line.starts_with("[stderr]") {
                                iced::Color::from_rgb(0.9, 0.2, 0.2)
                            } else if line.starts_with("[Prompt detected:") || line.starts_with("[User responded:") {
                                iced::Color::from_rgb(0.9, 0.7, 0.1)
                            } else {
                                theme.text_with_settings(Some(settings))
                            };
                            text(line)
                                .size(body_font_size * 0.93)
                                .style(iced::theme::Text::Color(line_color))
                                .font(iced::Font::MONOSPACE)
                                .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(2)
                .padding(12)
            )
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                Color::from(settings.background_color.clone()),
                settings.border_radius,
            ))))
            .width(Length::Fill)
            .height(Length::Fill);

            let command_input = if self.terminal_pending_prompt.is_some() {
                // Show waiting message when prompt is pending
                text_input("Waiting for prompt response...", &self.terminal_command)
                    .size(input_font_size)
                    .width(Length::Fill)
                    .padding(12)
                    .style(iced::theme::TextInput::Custom(Box::new(TerminalInputStyle {
                        radius: settings.border_radius,
                    })))
                    .font(iced::Font::MONOSPACE)
            } else {
                text_input("Enter command...", &self.terminal_command)
                    .on_input(Message::TerminalCommandChanged)
                    .on_submit(Message::ExecuteTerminalCommand)
                    .size(input_font_size)
                    .width(Length::Fill)
                    .padding(12)
                    .style(iced::theme::TextInput::Custom(Box::new(TerminalInputStyle {
                        radius: settings.border_radius,
                    })))
                    .font(iced::Font::MONOSPACE)
            };

            let execute_button = if self.terminal_pending_prompt.is_some() {
                // Disable execute button when waiting for prompt
                button(
                    row![
                        text("Waiting...")
                            .size(button_font_size)
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::new(12.0))
            } else {
                button(
                    row![
                        text("Execute")
                            .size(button_font_size)
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::ExecuteTerminalCommand)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: settings.border_radius,
                })))
                .padding(Padding::new(12.0))
            };

            let close_button = button(
                row![
                    text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_size),
                    text(" Close").size(button_font_size)
                ]
                .spacing(4)
                .align_items(Alignment::Center)
            )
            .on_press(Message::CloseAddRepoTerminal)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(12.0));

            container(
                column![
                    row![
                        text("Add Repository Terminal")
                            .size(title_font_size * 0.71)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
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
            .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle {
                radius: settings.border_radius,
            })))
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
                scrollable(
                column![
                    header_row,
                        Space::with_height(Length::Fixed(20.0)),
                    content,
                ]
                .spacing(0)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                    Color::from(settings.background_color.clone()),
                    settings.border_radius,
                ))))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
            .into()
        };

        // Create the slide-out panel
        let panel = if self.panel_open && !self.terminal_open {
            self.view_panel(theme, settings)
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
                sub_tabs,
                Space::with_height(Length::Fixed(16.0)),
                if self.terminal_open {
                    terminal_ui
                } else {
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

/// Shorten repository ID for display, especially COPR repositories
fn shorten_repo_id(id: &str) -> String {
    // For COPR repos like "copr:copr.fedorainfracloud.org:bieszcachyos"
    // Extract just the last part after the final colon
    if id.starts_with("copr:") {
        if let Some(last_colon) = id.rfind(':') {
            if last_colon < id.len() - 1 {
                return id[last_colon + 1..].to_string();
            }
        }
    }

    // For other long IDs, truncate if too long
    if id.len() > 40 {
        format!("{}...", &id[..37])
    } else {
        id.to_string()
    }
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

    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(["dnf", "config-manager", &format!("--{}", action), &repo_id]);

    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute dnf config-manager: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if user cancelled the password dialog
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }

        return Err(format!("Failed to {} repository: {}\n{}",
            if enable { "enable" } else { "disable" },
            stderr, stdout));
    }

    Ok(format!("Repository {} {}",
        repo_id,
        if enable { "enabled" } else { "disabled" }))
}

// Style structs
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

struct RepoItemStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for RepoItemStyle {
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

struct SubTabButtonStyle {
    is_active: bool,
    radius: f32,
}

impl ButtonStyleSheet for SubTabButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        ButtonAppearance {
            background: Some(if self.is_active {
                palette.primary.into()
            } else {
                if is_dark {
                    iced::Color::from_rgba(0.2, 0.2, 0.2, 1.0).into()
                } else {
                    iced::Color::from_rgba(0.9, 0.9, 0.91, 1.0).into()
                }
            }),
            text_color: if self.is_active { iced::Color::WHITE } else { palette.text },
            border: Border::with_radius(self.radius * 0.375),
            shadow: Default::default(),
            shadow_offset: iced::Vector::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        if !self.is_active {
            let palette = style.palette();
            appearance.background = Some(palette.primary.into());
            appearance.text_color = iced::Color::WHITE;
        }
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
        self.active(style)
    }
}

struct StatusBadgeStyle {
    enabled: bool,
    radius: f32,
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
                radius: self.radius.into(),
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

struct InfoContainerStyle {
    radius: f32,
}

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

struct CloseButtonStyle {
    radius: f32,
}

impl ButtonStyleSheet for CloseButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        ButtonAppearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
            border: Border {
                radius: self.radius.into(),
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
                radius: self.radius.into(),
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

// Install NVIDIA repository
async fn install_nvidia_repo() -> Result<(), String> {
    use tokio::process::Command as TokioCommand;

    // Check if repository already exists
    let check_cmd = TokioCommand::new("dnf")
        .arg("repoinfo")
        .arg("fedora-nvidia")
        .output()
        .await;

    // If repo exists, return success without adding
    if let Ok(output) = check_cmd {
        if output.status.success() {
            return Ok(()); // Repository already exists
        }
    }

    // Use pkexec to run with elevated privileges
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("config-manager");
    cmd.arg("addrepo");
    cmd.arg("--from-repofile");
    cmd.arg("https://negativo17.org/repos/fedora-nvidia.repo");

    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd.output().await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to install repository: {}", stderr))
    }
}

async fn install_rpmfusion_repos() -> Result<(), String> {
    use tokio::process::Command as TokioCommand;
    use std::process::Command as StdCommand;

    // First, get the Fedora version
    let fedora_version_output = StdCommand::new("rpm")
        .args(&["-E", "%fedora"])
        .output()
        .map_err(|e| format!("Failed to get Fedora version: {}", e))?;

    if !fedora_version_output.status.success() {
        return Err("Failed to determine Fedora version".to_string());
    }

    let fedora_version = String::from_utf8_lossy(&fedora_version_output.stdout)
        .trim()
        .to_string();

    if fedora_version.is_empty() {
        return Err("Fedora version is empty".to_string());
    }

    // Build URLs for RPM Fusion repos
    let free_repo_url = format!(
        "https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-{}.noarch.rpm",
        fedora_version
    );
    let nonfree_repo_url = format!(
        "https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-{}.noarch.rpm",
        fedora_version
    );

    // Install free repo first
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.arg(&free_repo_url);

    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd.output().await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to install RPM Fusion Free repository: {}", stderr));
    }

    // Install nonfree repo
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.arg(&nonfree_repo_url);

    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd.output().await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to install RPM Fusion Nonfree repository: {}", stderr))
    }
}

// Detect if output contains a y/n prompt
fn detect_prompt(output: &str) -> Option<String> {
    let prompt_patterns = [
        "[y/N]",
        "[Y/n]",
        "[yes/no]",
        "(y/n)",
        "(Y/n)",
        "Is this ok",
        "Continue?",
        "Proceed?",
    ];

    // Check each line for prompt patterns
    for line in output.lines().rev() {
        let line_lower = line.to_lowercase();
        for pattern in prompt_patterns.iter() {
            if line_lower.contains(&pattern.to_lowercase()) {
                // Found a prompt, return the line
                return Some(line.trim().to_string());
            }
        }
        // Also check for question marks that might indicate a prompt
        if line.contains('?') && (line_lower.contains("y") || line_lower.contains("n") || line_lower.contains("yes") || line_lower.contains("no")) {
            return Some(line.trim().to_string());
        }
    }
    None
}

// Show a yes/no dialog using zenity or kdialog
async fn show_prompt_dialog(prompt_text: String) -> Option<bool> {
    use tokio::process::Command as TokioCommand;

    // Try zenity first (GNOME)
    let zenity_cmd = TokioCommand::new("zenity")
        .args([
            "--question",
            "--title=Command Prompt",
            "--text",
            &prompt_text,
            "--ok-label=Yes",
            "--cancel-label=No",
        ])
        .output()
        .await;

    if let Ok(output) = zenity_cmd {
        // zenity returns 0 for yes, 1 for no
        return Some(output.status.code() == Some(0));
    }

    // Fallback to kdialog (KDE)
    let kdialog_cmd = TokioCommand::new("kdialog")
        .args([
            "--yesno",
            &prompt_text,
            "--title=Command Prompt",
        ])
        .output()
        .await;

    if let Ok(output) = kdialog_cmd {
        // kdialog returns 0 for yes, 1 for no
        return Some(output.status.code() == Some(0));
    }

    // If both fail, return None (user cancelled or no dialog available)
    None
}

// Execute command with interactive prompt handling
async fn execute_terminal_command_interactive(command: String) -> Result<(String, String, bool, Option<String>), String> {
    // For now, use the regular execute_terminal_command and detect prompts in output
    // In the future, this could be enhanced to use spawn with pipes for real-time interaction
    execute_terminal_command(command).await
        .map(|(stdout, stderr, success)| {
            let full_output = format!("{}\n{}", stdout, stderr);
            let prompt = detect_prompt(&full_output);
            (stdout, stderr, success, prompt)
        })
}

// Send prompt response by re-running command with answer
async fn send_prompt_response(original_command: String, _prompt_text: String, response: String) -> Result<(String, String, bool), String> {
    use tokio::process::Command as TokioCommand;

    // Parse the original command
    let parts: Vec<&str> = original_command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let program = parts[0];
    let args = &parts[1..];

    // Use echo to pipe the response, then run the command
    // Format: echo "y" | pkexec dnf ...
    let needs_root = program == "dnf" || program == "yum" || program == "rpm" ||
                     (program == "sudo" && args.get(0).map(|a| *a == "dnf" || *a == "yum" || *a == "rpm").unwrap_or(false));

    let output = if needs_root {
        // Build command: echo "y" | pkexec dnf ...
        let mut cmd = TokioCommand::new("sh");
        cmd.arg("-c");

        let cmd_str = if program == "sudo" {
            format!("echo \"{}\" | pkexec {}", response, args.join(" "))
        } else {
            format!("echo \"{}\" | pkexec {} {}", response, program, args.join(" "))
        };
        cmd.arg(&cmd_str);

        // Ensure DISPLAY is set for GUI password dialog
        if let Ok(display) = std::env::var("DISPLAY") {
            cmd.env("DISPLAY", display);
        }

        cmd.output().await
    } else {
        let mut cmd = TokioCommand::new("sh");
        cmd.arg("-c");
        let cmd_str = format!("echo \"{}\" | {} {}", response, program, args.join(" "));
        cmd.arg(&cmd_str);
        cmd.output().await
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let success = output.status.success();

            if !success && (output.status.code() == Some(126) || output.status.code() == Some(127)) {
                return Err("Authentication cancelled or failed. Please try again.".to_string());
            }

            Ok((stdout, stderr, success))
        }
        Err(e) => Err(format!("Failed to execute command: {}. Make sure polkit is installed.", e)),
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
    // This includes dnf, yum, rpm, and any command that modifies system repositories
    let needs_root = program == "dnf" || program == "yum" || program == "rpm" ||
                     (program == "sudo" && args.get(0).map(|a| *a == "dnf" || *a == "yum" || *a == "rpm").unwrap_or(false));

    let output = if needs_root {
        let mut cmd = TokioCommand::new("pkexec");
        // If command starts with sudo, remove it since pkexec handles elevation
        if program == "sudo" {
            cmd.args(args);
        } else {
            cmd.arg(program);
            cmd.args(args);
        }

        // Ensure DISPLAY is set for GUI password dialog
        if let Ok(display) = std::env::var("DISPLAY") {
            cmd.env("DISPLAY", display);
        }

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

            // Check if user cancelled the password dialog (exit code 126 or 127)
            if !success && (output.status.code() == Some(126) || output.status.code() == Some(127)) {
                return Err("Authentication cancelled or failed. Please try again.".to_string());
            }

            Ok((stdout, stderr, success))
        }
        Err(e) => Err(format!("Failed to execute command: {}. Make sure polkit is installed.", e)),
    }
}

struct TerminalContainerStyle {
    radius: f32,
}

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
                radius: self.radius.into(),
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

struct TerminalInputStyle {
    radius: f32,
}

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
            border: Border::with_radius(self.radius * 0.5),
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
                radius: self.radius.into(),
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
