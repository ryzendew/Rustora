use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Theme as IcedTheme, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use crate::gui::tabs::{SearchTab, InstalledTab, UpdateTab, FlatpakTab, MaintenanceTab, RepoTab, KernelTab, DeviceTab};
use crate::gui::Theme as AppTheme;
use crate::gui::tabs::search;
use crate::gui::tabs::installed;
use crate::gui::tabs::update;
use crate::gui::tabs::flatpak;
use crate::gui::tabs::maintenance;
use crate::gui::tabs::repo;
use crate::gui::tabs::kernel;
use crate::gui::tabs::device;
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
    FlatpakTabMessage(flatpak::Message),
    MaintenanceTabMessage(maintenance::Message),
    RepoTabMessage(repo::Message),
    KernelTabMessage(kernel::Message),
    DeviceTabMessage(device::Message),
    ThemeToggled,
    OpenRpmFilePicker,
    RpmFileSelected(Option<PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Search,
    Installed,
    Update,
    Flatpak,
    Maintenance,
    Repo,
    Kernel,
    Device,
}

#[derive(Debug)]
pub struct FedoraForgeApp {
    current_tab: Tab,
    search_tab: SearchTab,
    installed_tab: InstalledTab,
    update_tab: UpdateTab,
    flatpak_tab: FlatpakTab,
    maintenance_tab: MaintenanceTab,
    repo_tab: RepoTab,
    kernel_tab: KernelTab,
    device_tab: DeviceTab,
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
                flatpak_tab: FlatpakTab::new(),
                maintenance_tab: MaintenanceTab::new(),
                repo_tab: RepoTab::new(),
                kernel_tab: KernelTab::new(),
                device_tab: DeviceTab::new(),
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
                    // LoadPackages handler will check if packages are already loaded
                    // This prevents flickering when switching back to the tab
                    return Command::perform(async {}, |_| {
                        Message::InstalledTabMessage(installed::Message::LoadPackages)
                    });
                } else if tab == Tab::Repo {
                    return Command::perform(async {}, |_| {
                        Message::RepoTabMessage(repo::Message::LoadRepositories)
                    });
                } else if tab == Tab::Kernel {
                    return Command::perform(async {}, |_| {
                        Message::KernelTabMessage(kernel::Message::LoadBranches)
                    });
                } else if tab == Tab::Device {
                    return Command::perform(async {}, |_| {
                        Message::DeviceTabMessage(device::Message::RequestPermissions)
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
            Message::FlatpakTabMessage(msg) => {
                self.flatpak_tab.update(msg).map(Message::FlatpakTabMessage)
            }
            Message::MaintenanceTabMessage(msg) => {
                self.maintenance_tab.update(msg).map(Message::MaintenanceTabMessage)
            }
            Message::RepoTabMessage(msg) => {
                self.repo_tab.update(msg).map(Message::RepoTabMessage)
            }
            Message::KernelTabMessage(msg) => {
                self.kernel_tab.update(msg).map(Message::KernelTabMessage)
            }
            Message::DeviceTabMessage(msg) => {
                self.device_tab.update(msg).map(Message::DeviceTabMessage)
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
                download_icon.size(16),
                text(" Install RPM").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::OpenRpmFilePicker)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let theme_button = button(
            row![
                theme_icon.size(16),
                text(if self.theme == AppTheme::Dark { " Light" } else { " Dark" }).size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::ThemeToggled)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let search_button = button(
            row![
                search_icon.size(16),
                text(" Search").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Search))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Search,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let installed_button = button(
            row![
                installed_icon.size(16),
                text(" Installed").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Installed))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Installed,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let update_button = button(
            row![
                refresh_icon.size(16),
                text(" Updates").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Update))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Update,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let flatpak_icon = text(glyphs::INSTALLED_SYMBOL).font(material_font);
        let flatpak_button = button(
            row![
                flatpak_icon.size(16),
                text(" Flatpak").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Flatpak))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Flatpak,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let maintenance_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let maintenance_button = button(
            row![
                maintenance_icon.size(16),
                text(" Maintenance").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Maintenance))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Maintenance,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let repo_icon = text(glyphs::INSTALLED_SYMBOL).font(material_font);
        let repo_button = button(
            row![
                repo_icon.size(16),
                text(" Repositories").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Repo))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Repo,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let kernel_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let kernel_button = button(
            row![
                kernel_icon.size(16),
                text(" Kernels").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Kernel))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Kernel,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        let device_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let device_button = button(
            row![
                device_icon.size(16),
                text(" Devices").size(13)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Device))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Device,
            })))
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]));

        use iced::widget::scrollable;
        
        let tab_bar = container(
            scrollable(
                row![
                    text("FedoraForge").size(18).style(iced::theme::Text::Color(self.theme.primary())),
                    Space::with_width(Length::Fixed(12.0)),
                    search_button,
                    installed_button,
                    update_button,
                    flatpak_button,
                    maintenance_button,
                    repo_button,
                    kernel_button,
                    device_button,
                    Space::with_width(Length::Fill),
                    install_rpm_button,
                    theme_button,
                ]
                .spacing(8)
                .align_items(Alignment::Center)
                .padding(Padding::from([10.0, 12.0, 10.0, 12.0]))
            )
            .width(Length::Fill)
            .height(Length::Shrink)
        )
        .style(iced::theme::Container::Custom(Box::new(TabBarStyle)))
        .width(Length::Fill);

        let content = match self.current_tab {
            Tab::Search => self.search_tab.view(&self.theme).map(Message::SearchTabMessage),
            Tab::Installed => self.installed_tab.view(&self.theme).map(Message::InstalledTabMessage),
            Tab::Update => self.update_tab.view(&self.theme).map(Message::UpdateTabMessage),
            Tab::Flatpak => self.flatpak_tab.view(&self.theme).map(Message::FlatpakTabMessage),
            Tab::Maintenance => self.maintenance_tab.view(&self.theme).map(Message::MaintenanceTabMessage),
            Tab::Repo => self.repo_tab.view(&self.theme).map(Message::RepoTabMessage),
            Tab::Kernel => self.kernel_tab.view(&self.theme).map(Message::KernelTabMessage),
            Tab::Device => self.device_tab.view(&self.theme).map(Message::DeviceTabMessage),
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
                let is_dark = palette.background.r < 0.5;
                if is_dark {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1)
                } else {
                    iced::Color::from_rgba(0.85, 0.85, 0.87, 0.3) // Softer light mode button
                }
            })),
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: if self.is_primary {
                    palette.primary
                } else {
                    let is_dark = palette.background.r < 0.5;
                    if is_dark {
                        iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                    } else {
                        iced::Color::from_rgba(0.7, 0.7, 0.72, 0.4) // Softer light mode border
                    }
                },
            },
            text_color: palette.text,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        appearance.background = Some(iced::Background::Color(if self.is_primary {
            iced::Color::from_rgba(palette.primary.r * 0.9, palette.primary.g * 0.9, palette.primary.b * 0.9, 1.0)
        } else {
            if is_dark {
                iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15)
            } else {
                iced::Color::from_rgba(0.8, 0.8, 0.82, 0.4) // Softer light mode hover
            }
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


