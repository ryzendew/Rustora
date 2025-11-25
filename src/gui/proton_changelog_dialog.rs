use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;

#[derive(Debug, Clone)]
pub enum Message {
    LoadChangelog,
    ChangelogLoaded(Result<String, String>),
    Close,
}

#[derive(Debug)]
pub struct ProtonChangelogDialog {
    runner_title: String,
    build_title: String,
    page_url: String,
    description: String,
    changelog: Option<String>,
    is_loading: bool,
    error: Option<String>,
}

impl ProtonChangelogDialog {
    pub fn new(
        runner_title: String,
        build_title: String,
        description: String,
        page_url: String,
    ) -> Self {
        Self {
            runner_title,
            build_title,
            page_url,
            description,
            changelog: None,
            is_loading: true,
            error: None,
        }
    }

    pub fn run_separate_window(
        runner_title: String,
        build_title: String,
        description: String,
        page_url: String,
    ) -> Result<(), iced::Error> {
        let dialog = Self::new(runner_title, build_title, description, page_url);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(900.0, 700.0);
        window_settings.min_size = Some(iced::Size::new(700.0, 500.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <ProtonChangelogDialog as Application>::run(iced::Settings {
            window: window_settings,
            flags: dialog,
            default_font,
            default_text_size: iced::Pixels::from(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
        })
    }
}

impl Application for ProtonChangelogDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::LoadChangelog);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        format!("{} {} - Changelog - Rustora", self.runner_title, self.build_title)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadChangelog => {
                self.is_loading = true;
                let page_url = self.page_url.clone();
                Command::perform(fetch_changelog(page_url), Message::ChangelogLoaded)
            }
            Message::ChangelogLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(changelog) => {
                        self.changelog = Some(changelog);
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Command::none()
            }
            Message::Close => {
                iced::window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = crate::gui::Theme::Dark;
        self.view_impl(&theme)
    }

    fn theme(&self) -> IcedTheme {
        crate::gui::Theme::Dark.iced_theme()
    }
}

impl ProtonChangelogDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let close_button = button(
            text("Close")
                .size(16.0)
                .style(iced::theme::Text::Color(iced::Color::WHITE))
        )
        .on_press(Message::Close)
        .padding(Padding::from([12.0, 24.0, 12.0, 24.0]))
        .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)));

        let content = if self.is_loading {
            container(
                column![
                    text("Loading changelog...")
                        .size(16.0)
                        .style(iced::theme::Text::Color(theme.text()))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                ]
                .spacing(16)
                .align_items(Alignment::Center)
                .padding(24)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        } else if let Some(ref error) = self.error {
            container(
                column![
                    text("Error")
                        .size(20.0)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_height(Length::Fixed(16.0)),
                    text(error)
                        .size(14.0)
                        .style(iced::theme::Text::Color(theme.text())),
                    Space::with_height(Length::Fixed(24.0)),
                    close_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center)
                .padding(24)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        } else {
            let changelog_text = self.changelog.as_ref()
                .map(|c| c.as_str())
                .unwrap_or("No changelog available.");

            let description_text = if !self.description.is_empty() {
                format!("{}\n\n", self.description)
            } else {
                String::new()
            };

            let full_text = format!("{}{}", description_text, changelog_text);

            container(
                column![
                    // Header
                    row![
                        text(format!("{} {}", self.runner_title, self.build_title))
                            .size(20.0)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_width(Length::Fill),
                        close_button,
                    ]
                    .spacing(12)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    // Description
                    if !self.description.is_empty() {
                        let desc_container: Element<Message> = container(
                            text(&self.description)
                                .size(14.0)
                                .style(iced::theme::Text::Color(theme.text()))
                        )
                        .padding(16)
                        .style(iced::theme::Container::Custom(Box::new(DescriptionContainerStyle {
                            _theme: *theme,
                        })))
                        .width(Length::Fill)
                        .into();
                        desc_container
                    } else {
                        Space::with_height(Length::Fixed(0.0)).into()
                    },
                    Space::with_height(Length::Fixed(16.0)),
                    // Changelog title
                    text("Changelog")
                        .size(18.0)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(8.0)),
                    // Changelog content
                    scrollable(
                        text(&full_text)
                            .size(13.0)
                            .style(iced::theme::Text::Color(theme.text()))
                            .line_height(1.6)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(ChangelogScrollableStyle)))
                    .width(Length::Fill)
                    .height(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    // GitHub link
                    row![
                        text("View on GitHub:")
                            .size(12.0)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_width(Length::Fixed(8.0)),
                        text(&self.page_url)
                            .size(12.0)
                            .style(iced::theme::Text::Color(theme.primary()))
                    ]
                    .spacing(4),
                ]
                .spacing(0)
                .padding(24)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle {
                theme: *theme,
            })))
            .into()
    }
}

async fn fetch_changelog(page_url: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    if page_url.contains("github.com") && page_url.contains("/releases/") {
        if let Some(api_url) = convert_to_github_api_url(&page_url) {
            match client.get(&api_url)
                .header("User-Agent", "Rustora/1.0")
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if let Some(body) = json.get("body").and_then(|v| v.as_str()) {
                                if !body.is_empty() {
                                    return Ok(body.to_string());
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    Ok(format!("Changelog available at:\n{}\n\n(Full changelog can be viewed on the GitHub release page)", page_url))
}

fn convert_to_github_api_url(release_url: &str) -> Option<String> {
    if let Some(stripped) = release_url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = stripped.split('/').collect();
        if parts.len() >= 4 && parts[2] == "releases" && parts[3] == "tag" {
            let user = parts[0];
            let repo = parts[1];
            let tag = parts.get(4)?;
            return Some(format!("https://api.github.com/repos/{}/{}/releases/tags/{}", user, repo, tag));
        }
    }
    None
}

struct DialogContainerStyle {
    theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(self.theme.background().into()),
            border: Border::default(),
            shadow: Default::default(),
        }
    }
}

struct DescriptionContainerStyle {
    _theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for DescriptionContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(iced::Color::from_rgb(0.15, 0.15, 0.15).into()),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            shadow: Default::default(),
        }
    }
}

struct ChangelogScrollableStyle;

impl iced::widget::scrollable::StyleSheet for ChangelogScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: None,
                border: iced::Border::default(),
                scroller: iced::widget::scrollable::Scroller {
                    color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                    border: iced::Border::default(),
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_mouse_over: bool) -> iced::widget::scrollable::Appearance {
        self.active(_style)
    }
}

struct CloseButtonStyle;

impl ButtonStyleSheet for CloseButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Color::from_rgb(0.2, 0.5, 0.8).into()),
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            text_color: iced::Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Color::from_rgb(0.25, 0.6, 0.9).into());
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Color::from_rgb(0.15, 0.4, 0.7).into());
        appearance
    }
}
