use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Padding, Theme as IcedTheme, Border, Color};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::scrollable::{Appearance as ScrollableAppearance, StyleSheet as ScrollableStyleSheet};
use crate::gui::tabs::{SearchTab, InstalledTab, UpdateTab, FlatpakTab, MaintenanceTab, RepoTab, KernelTab, DeviceTab, FpmTab, TweaksTab};
use crate::gui::Theme as AppTheme;
use crate::gui::settings::AppSettings;
use crate::gui::tabs::search;
use crate::gui::tabs::installed;
use crate::gui::tabs::update;
use crate::gui::tabs::flatpak;
use crate::gui::tabs::maintenance;
use crate::gui::tabs::repo;
use crate::gui::tabs::kernel;
use crate::gui::tabs::device;
use crate::gui::tabs::fpm;
use crate::gui::tabs::tweaks;
use crate::gui::rpm_dialog::RpmDialog;
use std::path::PathBuf;

async fn open_file_picker() -> Option<PathBuf> {
    use tokio::process::Command;

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
    FpmTabMessage(fpm::Message),
    TweaksTabMessage(tweaks::Message),
    ThemeToggled,
    OpenRpmFilePicker,
    RpmFileSelected(Option<PathBuf>),
    OpenSettings,
    SettingsCheck,
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
    Fpm,
    Tweaks,
}

#[derive(Debug)]
pub struct RustoraApp {
    current_tab: Tab,
    search_tab: SearchTab,
    installed_tab: InstalledTab,
    update_tab: UpdateTab,
    flatpak_tab: FlatpakTab,
    maintenance_tab: MaintenanceTab,
    repo_tab: RepoTab,
    kernel_tab: KernelTab,
    device_tab: DeviceTab,
    fpm_tab: FpmTab,
    tweaks_tab: TweaksTab,
    theme: AppTheme,
    #[allow(dead_code)]
    rpm_dialog: Option<RpmDialog>,
    settings: AppSettings,
}

impl Application for RustoraApp {
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
                fpm_tab: FpmTab::new(),
                tweaks_tab: TweaksTab::new(),
                theme: AppTheme::Dark,
                rpm_dialog: None,
                settings: AppSettings::load(),
            },
            load_command,
        )
    }

    fn title(&self) -> String {
        "Rustora - Package Manager".to_string()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(50))
            .map(|_| Message::SettingsCheck)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SettingsCheck => {
                let new_settings = AppSettings::load();

                let old_settings = self.settings.clone();
                self.settings = new_settings.clone();

                if old_settings.font_size == self.settings.font_size
                    && old_settings.font_family == self.settings.font_family
                    && old_settings.scaling == self.settings.scaling
                    && old_settings.border_radius == self.settings.border_radius
                    && old_settings.background_color.r == self.settings.background_color.r
                    && old_settings.background_color.g == self.settings.background_color.g
                    && old_settings.background_color.b == self.settings.background_color.b
                    && old_settings.text_color.r == self.settings.text_color.r
                    && old_settings.text_color.g == self.settings.text_color.g
                    && old_settings.text_color.b == self.settings.text_color.b
                    && old_settings.primary_color.r == self.settings.primary_color.r
                    && old_settings.primary_color.g == self.settings.primary_color.g
                    && old_settings.primary_color.b == self.settings.primary_color.b
                    && old_settings.secondary_text_color.r == self.settings.secondary_text_color.r
                    && old_settings.secondary_text_color.g == self.settings.secondary_text_color.g
                    && old_settings.secondary_text_color.b == self.settings.secondary_text_color.b
                    && old_settings.font_size_buttons == self.settings.font_size_buttons
                    && old_settings.font_size_titles == self.settings.font_size_titles
                    && old_settings.font_size_body == self.settings.font_size_body
                    && old_settings.font_size_inputs == self.settings.font_size_inputs
                    && old_settings.font_size_tabs == self.settings.font_size_tabs
                    && old_settings.font_size_icons == self.settings.font_size_icons
                    && old_settings.scale_buttons == self.settings.scale_buttons
                    && old_settings.scale_titles == self.settings.scale_titles
                    && old_settings.scale_body == self.settings.scale_body
                    && old_settings.scale_inputs == self.settings.scale_inputs
                    && old_settings.scale_tabs == self.settings.scale_tabs
                    && old_settings.scale_icons == self.settings.scale_icons {

                    self.settings.tab_visibility = new_settings.tab_visibility;
                }
                Command::none()
            }
            Message::TabSelected(tab) => {
                self.current_tab = tab;
                if tab == Tab::Installed {

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
                } else if tab == Tab::Tweaks {
                    return Command::perform(async {}, |_| {
                        Message::TweaksTabMessage(tweaks::Message::LoadDnfConfig)
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
            Message::FpmTabMessage(msg) => {
                self.fpm_tab.update(msg).map(Message::FpmTabMessage)
            }
            Message::TweaksTabMessage(msg) => {
                self.tweaks_tab.update(msg).map(Message::TweaksTabMessage)
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

                let rpm_path_str = rpm_path.to_string_lossy().to_string();
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
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
            Message::OpenSettings => {

                Command::perform(
                    async {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let _ = TokioCommand::new(&exe_path)
                            .arg("settings")
                            .spawn();
                    },
                    |_| Message::TabSelected(Tab::Search),
                )
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

        // Use individual font sizes and scales
        let button_font_size = self.settings.font_size_buttons * self.settings.scale_buttons;
        let tab_font_size = self.settings.font_size_tabs * self.settings.scale_tabs;
        let icon_size = (self.settings.font_size_icons * self.settings.scale_icons).round();
        let title_font_size = (self.settings.font_size_titles * self.settings.scale_titles).round();

        let install_rpm_button = button(
            row![
                download_icon.size(icon_size),
                text(" Install RPM").size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::OpenRpmFilePicker)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let theme_button = button(
            row![
                theme_icon.size(icon_size),
                text(if self.theme == AppTheme::Dark { " Light" } else { " Dark" }).size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::ThemeToggled)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let settings_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let settings_button = button(
            row![
                settings_icon.size(icon_size),
                text(" Settings").size(button_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::OpenSettings)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let search_button = button(
            row![
                search_icon.size(icon_size),
                text(" Search").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Search))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Search,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let installed_button = button(
            row![
                installed_icon.size(icon_size),
                text(" Installed").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Installed))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Installed,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let update_button = button(
            row![
                refresh_icon.size(icon_size),
                text(" Updates").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Update))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Update,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let flatpak_icon = text(glyphs::INSTALLED_SYMBOL).font(material_font);
        let flatpak_button = button(
            row![
                flatpak_icon.size(icon_size),
                text(" Flatpak").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Flatpak))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Flatpak,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let maintenance_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let maintenance_button = button(
            row![
                maintenance_icon.size(icon_size),
                text(" Manage").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Maintenance))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Maintenance,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let repo_icon = text(glyphs::INSTALLED_SYMBOL).font(material_font);
        let repo_button = button(
            row![
                repo_icon.size(icon_size),
                text(" Repos").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Repo))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Repo,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let kernel_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let kernel_button = button(
            row![
                kernel_icon.size(icon_size),
                text(" Kernels").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Kernel))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Kernel,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let device_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let device_button = button(
            row![
                device_icon.size(icon_size),
                text(" Devices").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Device))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Device,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let fpm_icon = text(glyphs::DOWNLOAD_SYMBOL).font(material_font);
        let fpm_button = button(
            row![
                fpm_icon.size(icon_size),
                text(" FPM").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Fpm))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Fpm,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        let tweaks_icon = text(glyphs::SETTINGS_SYMBOL).font(material_font);
        let tweaks_button = button(
            row![
                tweaks_icon.size(icon_size),
                text(" Tweaks").size(tab_font_size)
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
            .on_press(Message::TabSelected(Tab::Tweaks))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: self.current_tab == Tab::Tweaks,
                radius: self.settings.border_radius,
                primary_color: Color::from(self.settings.primary_color.clone()),
                text_color: Color::from(self.settings.text_color.clone()),
                background_color: Color::from(self.settings.background_color.clone()),
            })))
            .width(Length::Shrink)
            .padding(Padding::new(14.0));

        use iced::widget::scrollable;

        // Determine if we should use sidebar layout based on scale
        let use_sidebar_layout = self.settings.scale_tabs > 1.8 ||
                                  (self.settings.font_size_tabs * self.settings.scale_tabs) > 20.0;

        // Show only icons when scale is very high
        let show_icons_only = self.settings.scale_tabs > 2.8 ||
                              (self.settings.font_size_tabs * self.settings.scale_tabs) > 28.0;

        // Build tab buttons for horizontal layout (original buttons)
        let mut tab_buttons_horizontal = Vec::new();
        if self.settings.is_tab_visible("Search") {
            tab_buttons_horizontal.push(search_button.into());
        }
        if self.settings.is_tab_visible("Installed") {
            tab_buttons_horizontal.push(installed_button.into());
        }
        if self.settings.is_tab_visible("Updates") {
            tab_buttons_horizontal.push(update_button.into());
        }
        if self.settings.is_tab_visible("Flatpak") {
            tab_buttons_horizontal.push(flatpak_button.into());
        }
        if self.settings.is_tab_visible("Maintenance") {
            tab_buttons_horizontal.push(maintenance_button.into());
        }
        if self.settings.is_tab_visible("Repositories") {
            tab_buttons_horizontal.push(repo_button.into());
        }
        if self.settings.is_tab_visible("Kernel") {
            tab_buttons_horizontal.push(kernel_button.into());
        }
        if self.settings.is_tab_visible("Device") {
            tab_buttons_horizontal.push(device_button.into());
        }
        if self.settings.is_tab_visible("FPM") {
            tab_buttons_horizontal.push(fpm_button.into());
        }
        if self.settings.is_tab_visible("Tweaks") {
            tab_buttons_horizontal.push(tweaks_button.into());
        }

        // Create sidebar buttons (separate instances to avoid move issues)
        let install_rpm_button_sidebar: Element<Message> = {
            let download_icon_sidebar = text(glyphs::DOWNLOAD_SYMBOL).font(material_font);
            let icon_size_for_button = if show_icons_only {
                // Make icons larger when icons-only to fill the button space
                (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
            } else {
                icon_size
            };
            let button_content: Element<Message> = if show_icons_only {
                container(download_icon_sidebar.size(icon_size_for_button))
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center)
                    .into()
            } else {
                row![download_icon_sidebar.size(icon_size), text(" Install RPM").size(button_font_size)]
                    .spacing(4)
                    .align_items(Alignment::Center)
                    .into()
            };
            button(button_content)
                .on_press(Message::OpenRpmFilePicker)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: self.settings.border_radius,
                    primary_color: Color::from(self.settings.primary_color.clone()),
                    text_color: Color::from(self.settings.text_color.clone()),
                    background_color: Color::from(self.settings.background_color.clone()),
                })))
                .width(Length::Fill)
                .padding(Padding::new(14.0))
                .into()
        };

        let theme_button_sidebar: Element<Message> = {
            let theme_icon_sidebar = text(glyphs::THEME_SYMBOL).font(material_font);
            let icon_size_for_button = if show_icons_only {
                // Make icons larger when icons-only to fill the button space
                (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
            } else {
                icon_size
            };
            let button_content: Element<Message> = if show_icons_only {
                container(theme_icon_sidebar.size(icon_size_for_button))
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center)
                    .into()
            } else {
                row![theme_icon_sidebar.size(icon_size), text(if self.theme == AppTheme::Dark { " Light" } else { " Dark" }).size(button_font_size)]
                    .spacing(4)
                    .align_items(Alignment::Center)
                    .into()
            };
            button(button_content)
                .on_press(Message::ThemeToggled)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: self.settings.border_radius,
                    primary_color: Color::from(self.settings.primary_color.clone()),
                    text_color: Color::from(self.settings.text_color.clone()),
                    background_color: Color::from(self.settings.background_color.clone()),
                })))
                .width(Length::Fill)
                .padding(Padding::new(14.0))
                .into()
        };

        let settings_button_sidebar: Element<Message> = {
            let settings_icon_sidebar = text(glyphs::SETTINGS_SYMBOL).font(material_font);
            let icon_size_for_button = if show_icons_only {
                // Make icons larger when icons-only to fill the button space
                (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
            } else {
                icon_size
            };
            let button_content: Element<Message> = if show_icons_only {
                container(settings_icon_sidebar.size(icon_size_for_button))
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center)
                    .into()
            } else {
                row![settings_icon_sidebar.size(icon_size), text(" Settings").size(button_font_size)]
                    .spacing(4)
                    .align_items(Alignment::Center)
                    .into()
            };
            button(button_content)
                .on_press(Message::OpenSettings)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: self.settings.border_radius,
                    primary_color: Color::from(self.settings.primary_color.clone()),
                    text_color: Color::from(self.settings.text_color.clone()),
                    background_color: Color::from(self.settings.background_color.clone()),
                })))
                .width(Length::Fill)
                .padding(Padding::new(14.0))
                .into()
        };

        // Top bar with title and optionally action buttons
        let top_bar = if use_sidebar_layout {
            // Sidebar layout: title only in top bar
            container(
                row![
                    text("Rustora").size(title_font_size).style(iced::theme::Text::Color(Color::from(self.settings.primary_color.clone()))),
                    Space::with_width(Length::Fill),
                ]
                .spacing(8)
                .align_items(Alignment::Center)
                .padding(Padding::from([12.0, 16.0]))
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(TabBarStyle {
                radius: self.settings.border_radius,
            })))
        } else {
            // Horizontal layout: title and action buttons in top bar
            container(
                row![
                    text("Rustora").size(title_font_size).style(iced::theme::Text::Color(Color::from(self.settings.primary_color.clone()))),
                    Space::with_width(Length::Fixed(12.0)),
                    Space::with_width(Length::Fill),
                    install_rpm_button,
                    theme_button,
                    settings_button,
                ]
                .spacing(8)
                .align_items(Alignment::Center)
                .padding(Padding::from([12.0, 16.0]))
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(TabBarStyle {
                radius: self.settings.border_radius,
            })))
        };

        // Left sidebar with tabs and action buttons (when scale is high)
        // Calculate sidebar width dynamically based on button content
        let sidebar_width = if show_icons_only {
            // Icons-only mode: width = icon_size + padding (left + right) + minimal extra space
            let icon_size_for_calc = (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0);
            (icon_size_for_calc + 14.0 * 2.0 + 8.0).max(70.0).min(90.0) + 24.0 // icon + padding + small extra, clamped tighter + 24px total
        } else {
            // Text + icon mode: wider to accommodate text
            220.0 + 24.0 // 24px more total
        };

        // Tab bar for horizontal layout (when scale is low)
        let (sidebar, tab_bar) = if use_sidebar_layout {
            // Create tab buttons for sidebar (separate instances)
            let mut tab_buttons_sidebar = Vec::new();
            if self.settings.is_tab_visible("Search") {
                let search_icon_sidebar = text(glyphs::SEARCH_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(search_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![search_icon_sidebar.size(icon_size), text(" Search").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Search)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Search, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Installed") {
                let installed_icon_sidebar = text(glyphs::INSTALLED_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(installed_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![installed_icon_sidebar.size(icon_size), text(" Installed").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Installed)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Installed, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Updates") {
                let refresh_icon_sidebar = text(glyphs::REFRESH_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(refresh_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![refresh_icon_sidebar.size(icon_size), text(" Updates").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Update)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Update, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Flatpak") {
                let flatpak_icon_sidebar = text(glyphs::INSTALLED_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(flatpak_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![flatpak_icon_sidebar.size(icon_size), text(" Flatpak").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Flatpak)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Flatpak, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Maintenance") {
                let maintenance_icon_sidebar = text(glyphs::SETTINGS_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(maintenance_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![maintenance_icon_sidebar.size(icon_size), text(" Manage").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Maintenance)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Maintenance, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Repositories") {
                let repo_icon_sidebar = text(glyphs::INSTALLED_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(repo_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![repo_icon_sidebar.size(icon_size), text(" Repos").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Repo)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Repo, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Kernel") {
                let kernel_icon_sidebar = text(glyphs::SETTINGS_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(kernel_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![kernel_icon_sidebar.size(icon_size), text(" Kernels").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Kernel)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Kernel, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Device") {
                let device_icon_sidebar = text(glyphs::SETTINGS_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(device_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![device_icon_sidebar.size(icon_size), text(" Devices").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Device)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Device, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("FPM") {
                let fpm_icon_sidebar = text(glyphs::DOWNLOAD_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    // Make icons larger when icons-only to fill the button space
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(fpm_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![fpm_icon_sidebar.size(icon_size), text(" FPM").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Fpm)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Fpm, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }
            if self.settings.is_tab_visible("Tweaks") {
                let tweaks_icon_sidebar = text(glyphs::SETTINGS_SYMBOL).font(material_font);
                let icon_size_for_button = if show_icons_only {
                    (self.settings.font_size_icons * self.settings.scale_icons * 1.5).max(24.0).min(40.0)
                } else {
                    icon_size
                };
                let button_content: Element<Message> = if show_icons_only {
                    container(tweaks_icon_sidebar.size(icon_size_for_button))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    row![tweaks_icon_sidebar.size(icon_size), text(" Tweaks").size(tab_font_size)]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                };
                tab_buttons_sidebar.push(button(button_content).on_press(Message::TabSelected(Tab::Tweaks)).style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle { is_primary: self.current_tab == Tab::Tweaks, radius: self.settings.border_radius, primary_color: Color::from(self.settings.primary_color.clone()), text_color: Color::from(self.settings.text_color.clone()), background_color: Color::from(self.settings.background_color.clone()) }))).width(Length::Fill).padding(Padding::new(14.0)).into());
            }

            let sidebar = container(
                column![
                    scrollable(
                        column(tab_buttons_sidebar)
                            .spacing(4)
                            .width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                        Color::from(self.settings.background_color.clone()),
                        self.settings.border_radius,
                    )))),
                    Space::with_height(Length::Fixed(12.0)),
                    // Divider or separator
                    container(Space::with_height(Length::Fixed(1.0)))
                        .width(Length::Fill)
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle))),
                    Space::with_height(Length::Fixed(12.0)),
                    // Action buttons at the bottom
                    column![
                        install_rpm_button_sidebar,
                        theme_button_sidebar,
                        settings_button_sidebar,
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::Fill)
                .height(Length::Fill)
            )
            .width(Length::Fixed(sidebar_width))
            .height(Length::Fill)
            .padding(Padding::from([12.0, 16.0]))
            .style(iced::theme::Container::Custom(Box::new(SidebarStyle {
                radius: self.settings.border_radius,
            })));
            let tab_bar = container(Space::with_height(Length::Fixed(0.0)))
                .width(Length::Fill)
                .height(Length::Fixed(0.0));
            (sidebar, tab_bar)
        } else {
            let sidebar = container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(0.0))
                .height(Length::Fill);
            let tab_bar = container(
                scrollable(
                    row(tab_buttons_horizontal)
                        .spacing(8)
                        .align_items(Alignment::Center)
                        .padding(Padding::from([10.0, 12.0, 10.0, 12.0]))
                )
                .width(Length::Fill)
                .height(Length::Shrink)
                .direction(iced::widget::scrollable::Direction::Horizontal(iced::widget::scrollable::Properties::default()))
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                    background_color: Color::from(self.settings.background_color.clone()),
                    border_radius: self.settings.border_radius * 0.5,
                })))
            )
            .style(iced::theme::Container::Custom(Box::new(TabBarStyle {
                radius: self.settings.border_radius,
            })))
            .width(Length::Fill)
            .height(Length::Shrink);
            (sidebar, tab_bar)
        };

        // Right content area
        let content = match self.current_tab {
            Tab::Search => self.search_tab.view(&self.theme, &self.settings).map(Message::SearchTabMessage),
            Tab::Installed => self.installed_tab.view(&self.theme, &self.settings).map(Message::InstalledTabMessage),
            Tab::Update => self.update_tab.view(&self.theme, &self.settings).map(Message::UpdateTabMessage),
            Tab::Flatpak => self.flatpak_tab.view(&self.theme, &self.settings).map(Message::FlatpakTabMessage),
            Tab::Maintenance => self.maintenance_tab.view(&self.theme, &self.settings).map(Message::MaintenanceTabMessage),
            Tab::Repo => self.repo_tab.view(&self.theme, &self.settings).map(Message::RepoTabMessage),
            Tab::Kernel => self.kernel_tab.view(&self.theme, &self.settings).map(Message::KernelTabMessage),
            Tab::Device => self.device_tab.view(&self.theme, &self.settings).map(Message::DeviceTabMessage),
            Tab::Fpm => self.fpm_tab.view(&self.theme, &self.settings).map(Message::FpmTabMessage),
            Tab::Tweaks => self.tweaks_tab.view(&self.theme, &self.settings).map(Message::TweaksTabMessage),
        };

        let content_pane = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle {
                background: Color::from(self.settings.background_color.clone()),
                radius: self.settings.border_radius,
            })));

        let main_content: Element<Message> = if use_sidebar_layout {
            row![
                sidebar,
                content_pane,
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            column![
                tab_bar,
                content_pane,
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        container(
            column![
                top_bar,
                main_content,
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle {
            background: Color::from(self.settings.background_color.clone()),
            radius: self.settings.border_radius,
        })))
        .into()
    }

    fn theme(&self) -> IcedTheme {
        self.theme.iced_theme()
    }
}

struct RoundedButtonStyle {
    is_primary: bool,
    radius: f32,
    primary_color: iced::Color,
    text_color: iced::Color,
    background_color: iced::Color,
}

impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                self.primary_color
            } else {
                let is_dark = self.background_color.r < 0.5;
                if is_dark {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1)
                } else {
                    iced::Color::from_rgba(0.85, 0.85, 0.87, 0.3) // Softer light mode button
                }
            })),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: if self.is_primary {
                    self.primary_color
                } else {
                    let is_dark = self.background_color.r < 0.5;
                    if is_dark {
                        iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                    } else {
                        iced::Color::from_rgba(0.7, 0.7, 0.72, 0.4) // Softer light mode border
                    }
                },
            },
            text_color: self.text_color,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(_style);
        let is_dark = self.background_color.r < 0.5;
        appearance.background = Some(iced::Background::Color(if self.is_primary {
            iced::Color::from_rgba(self.primary_color.r * 0.9, self.primary_color.g * 0.9, self.primary_color.b * 0.9, 1.0)
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

struct TabBarStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for TabBarStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct SidebarStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for SidebarStyle {
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
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct DividerStyle;

impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.text.r * 0.2,
                palette.text.g * 0.2,
                palette.text.b * 0.2,
                0.3,
            ))),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ContainerStyle {
    background: iced::Color,
    radius: f32,
}

impl iced::widget::container::StyleSheet for ContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.background)),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

pub struct CustomScrollableStyle {
    background_color: iced::Color,
    border_radius: f32,
}

impl CustomScrollableStyle {
    pub fn new(background_color: iced::Color, border_radius: f32) -> Self {
        Self {
            background_color,
            border_radius,
        }
    }
}

impl ScrollableStyleSheet for CustomScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ScrollableAppearance {
        let is_dark = self.background_color.r < 0.5;
        let primary_color = if is_dark {
            iced::Color::from_rgb(0.2, 0.6, 0.9)
        } else {
            iced::Color::from_rgb(0.15, 0.45, 0.65)
        };

        ScrollableAppearance {
            container: Appearance {
                background: None,
                border: Border::default(),
                ..Default::default()
            },
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: if is_dark {
                        iced::Color::from_rgba(primary_color.r, primary_color.g, primary_color.b, 0.5)
                    } else {
                        iced::Color::from_rgba(primary_color.r * 0.7, primary_color.g * 0.7, primary_color.b * 0.7, 0.5)
                    },
                    border: Border {
                        radius: (self.border_radius * 0.5).into(),
                        width: 0.0,
                        color: iced::Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, style: &Self::Style, _is_mouse_over_scrollbar: bool) -> ScrollableAppearance {
        let mut appearance = self.active(style);
        let is_dark = self.background_color.r < 0.5;
        let primary_color = if is_dark {
            iced::Color::from_rgb(0.2, 0.6, 0.9)
        } else {
            iced::Color::from_rgb(0.15, 0.45, 0.65)
        };

        appearance.scrollbar.scroller.color = if is_dark {
            iced::Color::from_rgba(primary_color.r, primary_color.g, primary_color.b, 0.7)
        } else {
            iced::Color::from_rgba(primary_color.r * 0.7, primary_color.g * 0.7, primary_color.b * 0.7, 0.7)
        };
        appearance
    }

    fn dragging(&self, style: &Self::Style) -> ScrollableAppearance {
        let mut appearance = self.active(style);
        let is_dark = self.background_color.r < 0.5;
        let primary_color = if is_dark {
            iced::Color::from_rgb(0.2, 0.6, 0.9)
        } else {
            iced::Color::from_rgb(0.15, 0.45, 0.65)
        };

        appearance.scrollbar.scroller.color = if is_dark {
            iced::Color::from_rgba(primary_color.r, primary_color.g, primary_color.b, 0.9)
        } else {
            iced::Color::from_rgba(primary_color.r * 0.7, primary_color.g * 0.7, primary_color.b * 0.7, 0.9)
        };
        appearance
    }
}
