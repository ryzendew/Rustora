use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Border, Theme as IcedTheme, Color};
use crate::gui::dialog_design::DialogDesign;
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
        window_settings.size = iced::Size::new(700.0, 550.0);
        window_settings.min_size = Some(iced::Size::new(500.0, 400.0));
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
        let settings = crate::gui::settings::AppSettings::load();
        let title_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_size = (settings.font_size_body * settings.scale_body).round();
        let button_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let content = if self.is_loading {
            container(
                column![
                    text("Loading changelog...")
                        .size(body_size)
                        .style(iced::theme::Text::Color(theme.text()))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                ]
                .spacing(DialogDesign::SPACE_MEDIUM)
                .align_items(Alignment::Center)
                .padding(DialogDesign::pad_large())
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
        } else if let Some(ref error) = self.error {
            let header = container(
                row![
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL)
                        .font(material_font)
                        .size(title_size * 1.2)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_width(DialogDesign::space_small()),
                    text("Error")
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_width(Length::Fill),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding(DialogDesign::pad_medium());

            container(
                column![
                    header,
                    container(Space::with_height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                    scrollable(
                        column![
                            text(error)
                                .size(body_size)
                                .style(iced::theme::Text::Color(theme.danger())),
                        ]
                        .spacing(0)
                        .padding(DialogDesign::pad_medium())
                    )
                    .height(Length::Fill),
                    container(Space::with_height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                    container(
                        row![
                            Space::with_width(Length::Fill),
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(button_size * 1.1),
                                    text(" Close").size(button_size)
                                ]
                                .spacing(DialogDesign::SPACE_TINY)
                                .align_items(Alignment::Center)
                            )
                            .on_press(Message::Close)
                            .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
                            .padding(DialogDesign::pad_small()),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL)
                    )
                    .width(Length::Fill)
                    .padding(DialogDesign::pad_medium()),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(WindowContainerStyle {
                background: theme.background(),
            })))
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

            let header = container(
                row![
                    text(crate::gui::fonts::glyphs::INFO_SYMBOL)
                        .font(material_font)
                        .size(title_size * 1.2)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(DialogDesign::space_small()),
                    text(format!("{} {}", self.runner_title, self.build_title))
                        .size(title_size)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(Length::Fill),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding(DialogDesign::pad_medium());

            container(
                column![
                    header,
                    container(Space::with_height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                    scrollable(
                        column![
                            if !self.description.is_empty() {
                                container(
                                    column![
                                        text("Description")
                                            .size(body_size * 1.05)
                                            .style(iced::theme::Text::Color(theme.primary())),
                                        Space::with_height(DialogDesign::space_small()),
                                        text(&self.description)
                                            .size(body_size)
                                            .style(iced::theme::Text::Color(theme.text()))
                                            .line_height(1.6),
                                    ]
                                    .spacing(0)
                                    .padding(DialogDesign::pad_medium())
                                )
                                .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle)))
                            } else {
                                container(Space::with_height(Length::Fixed(0.0)))
                                    .width(Length::Fill)
                                    .into()
                            },
                            Space::with_height(DialogDesign::space_medium()),
                            container(
                                column![
                                    text("Changelog")
                                        .size(body_size * 1.1)
                                        .style(iced::theme::Text::Color(theme.primary())),
                                    Space::with_height(DialogDesign::space_small()),
                                    text(&full_text)
                                        .size(body_size * 0.95)
                                        .style(iced::theme::Text::Color(theme.text()))
                                        .line_height(1.6),
                                ]
                                .spacing(0)
                                .padding(DialogDesign::pad_medium())
                            )
                            .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
                            Space::with_height(DialogDesign::space_medium()),
                            container(
                                row![
                                    text("View on GitHub:")
                                        .size(body_size * 0.9)
                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                    Space::with_width(DialogDesign::space_small()),
                                    text(&self.page_url)
                                        .size(body_size * 0.9)
                                        .style(iced::theme::Text::Color(theme.primary())),
                                ]
                                .spacing(0)
                                .padding(DialogDesign::pad_small())
                            )
                            .style(iced::theme::Container::Custom(Box::new(CleanContainerStyle))),
                        ]
                        .spacing(0)
                        .padding(DialogDesign::pad_medium())
                    )
                    .height(Length::Fill),
                    container(Space::with_height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                    container(
                        row![
                            Space::with_width(Length::Fill),
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(button_size * 1.1),
                                    text(" Close").size(button_size)
                                ]
                                .spacing(DialogDesign::SPACE_TINY)
                                .align_items(Alignment::Center)
                            )
                            .on_press(Message::Close)
                            .style(iced::theme::Button::Custom(Box::new(CleanButtonStyle { is_primary: true })))
                            .padding(DialogDesign::pad_small()),
                        ]
                        .spacing(DialogDesign::SPACE_SMALL)
                    )
                    .width(Length::Fill)
                    .padding(DialogDesign::pad_medium()),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(WindowContainerStyle {
                background: theme.background(),
            })))
        };

        content.into()
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

struct CleanContainerStyle;

impl iced::widget::container::StyleSheet for CleanContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(
                palette.background.r * 0.98,
                palette.background.g * 0.98,
                palette.background.b * 0.98,
                1.0,
            ))),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.2),
            },
            ..Default::default()
        }
    }
}

struct DividerStyle;

impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 0.2))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
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
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct CleanButtonStyle {
    is_primary: bool,
}

impl ButtonStyleSheet for CleanButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                palette.primary
            } else {
                Color::from_rgba(0.4, 0.4, 0.4, 0.2)
            })),
            border: Border {
                radius: DialogDesign::RADIUS.into(),
                width: 1.0,
                color: if self.is_primary {
                    palette.primary
                } else {
                    Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                },
            },
            text_color: if self.is_primary { Color::WHITE } else { palette.text },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        let palette = style.palette();
        if self.is_primary {
            appearance.background = Some(iced::Background::Color(
                Color::from_rgba(palette.primary.r * 0.85, palette.primary.g * 0.85, palette.primary.b * 0.85, 1.0)
            ));
        } else {
            appearance.background = Some(iced::Background::Color(Color::from_rgba(0.4, 0.4, 0.4, 0.3)));
        }
        appearance
    }
}
