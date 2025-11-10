use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Theme as IcedTheme, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use crate::gui::tabs::{SearchTab, InstalledTab, UpdateTab};
use crate::gui::Theme as AppTheme;
use crate::gui::tabs::search;
use crate::gui::tabs::installed;
use crate::gui::tabs::update;
use crate::gui::rpm_dialog::RpmDialog;
use std::path::PathBuf;

async fn open_file_picker() -> Option<PathBuf> {
    use tokio::process::Command;
    
    // Try zenity first (GNOME), then kdialog (KDE), then fallback to a simple approach
    let output = Command::new("zenity")
        .args(["--file-selection", "--title=Select RPM Package to Install", "--file-filter=*.rpm"])
        .output()
        .await;
    
    if let Ok(output) = output {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }
    
    // Fallback to kdialog
    let output = Command::new("kdialog")
        .args(["--getopenfilename", ".", "*.rpm"])
        .output()
        .await;
    
    if let Ok(output) = output {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }
    
    None
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    SearchTabMessage(search::Message),
    InstalledTabMessage(installed::Message),
    UpdateTabMessage(update::Message),
    ThemeToggled,
    OpenRpmFilePicker,
    RpmFileSelected(Option<PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Search,
    Installed,
    Update,
}

#[derive(Debug)]
pub struct FedoraForgeApp {
    current_tab: Tab,
    search_tab: SearchTab,
    installed_tab: InstalledTab,
    update_tab: UpdateTab,
    theme: AppTheme,
    #[allow(dead_code)]
    rpm_dialog: Option<RpmDialog>,
}

impl Application for FedoraForgeApp {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Option<PathBuf>;

    fn new(_flags: Option<PathBuf>) -> (Self, Command<Message>) {
        let installed_tab = InstalledTab::new();
        let load_command = Command::perform(async {}, |_| {
            Message::InstalledTabMessage(installed::Message::LoadPackages)
        });

        (
            Self {
                current_tab: Tab::Search,
                search_tab: SearchTab::new(),
                installed_tab,
                update_tab: UpdateTab::new(),
                theme: AppTheme::Dark,
                rpm_dialog: None,
            },
            load_command,
        )
    }

    fn title(&self) -> String {
        "FedoraForge - Package Manager".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.current_tab = tab;
                if tab == Tab::Installed {
                    return Command::perform(async {}, |_| {
                        Message::InstalledTabMessage(installed::Message::LoadPackages)
                    });
                }
                Command::none()
            }
            Message::SearchTabMessage(msg) => {
                self.search_tab.update(msg).map(Message::SearchTabMessage)
            }
            Message::InstalledTabMessage(msg) => {
                self.installed_tab.update(msg).map(Message::InstalledTabMessage)
            }
            Message::UpdateTabMessage(msg) => {
                self.update_tab.update(msg).map(Message::UpdateTabMessage)
            }
            Message::ThemeToggled => {
                self.theme = match self.theme {
                    AppTheme::Light => AppTheme::Dark,
                    AppTheme::Dark => AppTheme::Light,
                };
                Command::none()
            }
            Message::OpenRpmFilePicker => {
                Command::perform(open_file_picker(), Message::RpmFileSelected)
            }
            Message::RpmFileSelected(Some(rpm_path)) => {
                // Spawn a separate window process for the RPM dialog
                let rpm_path_str = rpm_path.to_string_lossy().to_string();
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("fedoraforge"));
                        let _ = TokioCommand::new(&exe_path)
                            .arg("gui")
                            .arg(&rpm_path_str)
                            .spawn();
                    },
                    |_| Message::TabSelected(Tab::Search),
                )
            }
            Message::RpmFileSelected(None) => {
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        
        // Use cached Material Symbols font (optimized - loaded once)
        let material_font = glyphs::material_font();
        
        // Create icon text with Material Symbols font
        let download_icon = text(glyphs::DOWNLOAD_SYMBOL).font(material_font);
        let theme_icon = text(glyphs::THEME_SYMBOL).font(material_font);
        let search_icon = text(glyphs::SEARCH_SYMBOL).font(material_font);
        let installed_icon = text(glyphs::INSTALLED_SYMBOL).font(material_font);
        let refresh_icon = text(glyphs::REFRESH_SYMBOL).font(material_font);
        
        let install_rpm_button = button(
            row![
                download_icon,
                text(" Install RPM")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::OpenRpmFilePicker)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::new(12.0));

        let theme_button = button(
            row![
                theme_icon,
                text(if self.theme == AppTheme::Dark { " Light" } else { " Dark" })
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::ThemeToggled)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::new(12.0));

        let search_button = button(
            row![
                search_icon,
                text(" Search")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Search))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Search,
            })))
            .padding(Padding::new(14.0));

        let installed_button = button(
            row![
                installed_icon,
                text(" Installed")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Installed))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Installed,
            })))
            .padding(Padding::new(14.0));

        let update_button = button(
            row![
                refresh_icon,
                text(" Updates")
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Update))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Update,
            })))
            .padding(Padding::new(14.0));

        let tab_bar = container(
            row![
                text("FedoraForge").size(20).style(iced::theme::Text::Color(self.theme.primary())),
                Space::with_width(Length::Fill),
                search_button,
                installed_button,
                update_button,
                Space::with_width(Length::Fill),
                install_rpm_button,
                theme_button,
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .padding(Padding::new(15.0))
        )
        .style(iced::theme::Container::Custom(Box::new(TabBarStyle)))
        .width(Length::Fill);

        let content = match self.current_tab {
            Tab::Search => self.search_tab.view(&self.theme).map(Message::SearchTabMessage),
            Tab::Installed => self.installed_tab.view(&self.theme).map(Message::InstalledTabMessage),
            Tab::Update => self.update_tab.view(&self.theme).map(Message::UpdateTabMessage),
        };

        container(column![tab_bar, content].spacing(0))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle {
                background: self.theme.background(),
            })))
            .into()
    }

    fn theme(&self) -> IcedTheme {
        self.theme.iced_theme()
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

struct TabBarStyle;

impl iced::widget::container::StyleSheet for TabBarStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: 20.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ContainerStyle {
    background: iced::Color,
}

impl iced::widget::container::StyleSheet for ContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.background)),
            border: Border {
                radius: 20.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}


