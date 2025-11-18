use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, checkbox};
use iced::{Alignment, Element, Length, Padding, Border, Color};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use crate::gui::app::CustomScrollableStyle;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweaksView {
    GamingMeta,
    DnfConfig,
    CachyosKernel,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchView(TweaksView),
    CheckGamingMetaStatus,
    GamingMetaStatusChecked(Result<GamingMetaStatus, String>),
    InstallGamingMeta,
    GamingMetaComplete(Result<String, String>),
    CheckCachyosKernelStatus,
    CachyosKernelStatusChecked(Result<CachyosKernelStatus, String>),
    InstallCachyosKernel,
    LoadDnfConfig,
    DnfConfigLoaded(Result<DnfConfig, String>),
    ParallelDownloadsChanged(String),
    FastestMirrorToggled(bool),
    SaveDnfConfig,
    DnfConfigSaved(Result<(), String>),
}

#[derive(Debug, Clone)]
pub struct DnfConfig {
    pub max_parallel_downloads: Option<u32>,
    pub fastestmirror: bool,
}

#[derive(Debug, Clone)]
pub struct GamingMetaStatus {
    pub steam: bool,
    pub lutris: bool,
    pub mangohud: bool,
    pub gamescope: bool,
    pub mangojuice: bool,
    pub protonplus: bool,
    pub heroic: bool,
}

#[derive(Debug, Clone)]
pub struct CachyosKernelStatus {
    pub kernel_cachyos: bool,
    pub cachyos_settings: bool,
    pub ananicy_cpp: bool,
    pub cachyos_ananicy_rules: bool,
    pub scx_manager: bool,
    pub scx_scheds_git: bool,
    pub scx_tools: bool,
}

#[derive(Debug)]
pub struct TweaksTab {
    current_view: TweaksView,
    output_log: Vec<String>,
    // DNF config state
    dnf_config: Option<DnfConfig>,
    parallel_downloads_input: String,
    fastest_mirror_enabled: bool,
    is_loading_dnf_config: bool,
    is_saving_dnf_config: bool,
    dnf_config_error: Option<String>,
    // Gaming Meta status
    gaming_meta_status: Option<GamingMetaStatus>,
    is_checking_gaming_meta: bool,
    // Cachyos Kernel status
    cachyos_kernel_status: Option<CachyosKernelStatus>,
    is_checking_cachyos_kernel: bool,
}

impl TweaksTab {
    pub fn new() -> Self {
        let mut tab = Self {
            current_view: TweaksView::GamingMeta,
            output_log: Vec::new(),
            dnf_config: None,
            parallel_downloads_input: String::new(),
            fastest_mirror_enabled: false,
            is_loading_dnf_config: false,
            is_saving_dnf_config: false,
            dnf_config_error: None,
            gaming_meta_status: None,
            is_checking_gaming_meta: false,
            cachyos_kernel_status: None,
            is_checking_cachyos_kernel: false,
        };
        // Auto-check gaming meta status on init
        tab.is_checking_gaming_meta = true;
        tab
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::SwitchView(view) => {
                self.current_view = view;
                // Check gaming meta status when switching to that tab
                if view == TweaksView::GamingMeta && self.gaming_meta_status.is_none() {
                    self.is_checking_gaming_meta = true;
                    return iced::Command::perform(check_gaming_meta_status(), Message::GamingMetaStatusChecked);
                }
                // Auto-load DNF config when switching to that tab
                if view == TweaksView::DnfConfig && self.dnf_config.is_none() && !self.is_loading_dnf_config {
                    self.is_loading_dnf_config = true;
                    self.dnf_config_error = None;
                    return iced::Command::perform(load_dnf_config(), Message::DnfConfigLoaded);
                }
                // Check Cachyos kernel status when switching to that tab
                if view == TweaksView::CachyosKernel && self.cachyos_kernel_status.is_none() {
                    self.is_checking_cachyos_kernel = true;
                    return iced::Command::perform(check_cachyos_kernel_status(), Message::CachyosKernelStatusChecked);
                }
                iced::Command::none()
            }
            Message::CheckGamingMetaStatus => {
                self.is_checking_gaming_meta = true;
                iced::Command::perform(check_gaming_meta_status(), Message::GamingMetaStatusChecked)
            }
            Message::GamingMetaStatusChecked(result) => {
                self.is_checking_gaming_meta = false;
                match result {
                    Ok(status) => {
                        self.gaming_meta_status = Some(status);
                    }
                    Err(_) => {
                        // On error, just set all to false
                        self.gaming_meta_status = Some(GamingMetaStatus {
                            steam: false,
                            lutris: false,
                            mangohud: false,
                            gamescope: false,
                            mangojuice: false,
                            protonplus: false,
                            heroic: false,
                        });
                    }
                }
                iced::Command::none()
            }
            Message::InstallGamingMeta => {
                // Spawn a separate window for Gaming Meta installation
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("gaming-meta-dialog")
                            .spawn()
                            .ok();
                    },
                    |_| Message::GamingMetaComplete(Ok("Dialog opened".to_string())),
                )
            }
            Message::GamingMetaComplete(_result) => {
                // Dialog handles its own completion
                // Refresh status after installation
                self.is_checking_gaming_meta = true;
                iced::Command::perform(check_gaming_meta_status(), Message::GamingMetaStatusChecked)
            }
            Message::CheckCachyosKernelStatus => {
                self.is_checking_cachyos_kernel = true;
                iced::Command::perform(check_cachyos_kernel_status(), Message::CachyosKernelStatusChecked)
            }
            Message::CachyosKernelStatusChecked(result) => {
                self.is_checking_cachyos_kernel = false;
                match result {
                    Ok(status) => {
                        self.cachyos_kernel_status = Some(status);
                    }
                    Err(_) => {
                        // On error, just set all to false
                        self.cachyos_kernel_status = Some(CachyosKernelStatus {
                            kernel_cachyos: false,
                            cachyos_settings: false,
                            ananicy_cpp: false,
                            cachyos_ananicy_rules: false,
                            scx_manager: false,
                            scx_scheds_git: false,
                            scx_tools: false,
                        });
                    }
                }
                iced::Command::none()
            }
            Message::InstallCachyosKernel => {
                // Spawn a separate window for Cachyos kernel installation
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("cachyos-kernel-dialog")
                            .spawn()
                            .ok();
                    },
                    |_| Message::GamingMetaComplete(Ok("Dialog opened".to_string())),
                )
            }
            Message::LoadDnfConfig => {
                self.is_loading_dnf_config = true;
                self.dnf_config_error = None;
                iced::Command::perform(load_dnf_config(), Message::DnfConfigLoaded)
            }
            Message::DnfConfigLoaded(result) => {
                self.is_loading_dnf_config = false;
                match result {
                    Ok(config) => {
                        self.dnf_config = Some(config.clone());
                        // Default to 20 if not set
                        self.parallel_downloads_input = config.max_parallel_downloads
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "20".to_string());
                        self.fastest_mirror_enabled = config.fastestmirror;
                    }
                    Err(e) => {
                        self.dnf_config_error = Some(e);
                    }
                }
                iced::Command::none()
            }
            Message::ParallelDownloadsChanged(value) => {
                self.parallel_downloads_input = value;
                iced::Command::none()
            }
            Message::FastestMirrorToggled(enabled) => {
                self.fastest_mirror_enabled = enabled;
                iced::Command::none()
            }
            Message::SaveDnfConfig => {
                // Validate parallel downloads input
                // Default to 20 if empty
                let parallel_downloads = if self.parallel_downloads_input.trim().is_empty() {
                    Some(20)
                } else {
                    match self.parallel_downloads_input.trim().parse::<u32>() {
                        Ok(val) if val >= 1 && val <= 25 => Some(val),
                        _ => {
                            self.dnf_config_error = Some("Parallel downloads must be a number between 1 and 25".to_string());
                            return iced::Command::none();
                        }
                    }
                };
                
                let config = DnfConfig {
                    max_parallel_downloads: parallel_downloads,
                    fastestmirror: self.fastest_mirror_enabled,
                };
                
                self.is_saving_dnf_config = true;
                self.dnf_config_error = None;
                iced::Command::perform(save_dnf_config(config), Message::DnfConfigSaved)
            }
            Message::DnfConfigSaved(result) => {
                self.is_saving_dnf_config = false;
                match result {
                    Ok(_) => {
                        // Reload config to show updated values
                        self.is_loading_dnf_config = true;
                        iced::Command::perform(load_dnf_config(), Message::DnfConfigLoaded)
                    }
                    Err(e) => {
                        self.dnf_config_error = Some(e);
                        iced::Command::none()
                    }
                }
            }
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        // Calculate font sizes from settings
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        
        // Header section
        let header = container(
            column![
                text("Tweaks")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary()))
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                Space::with_height(Length::Fixed(8.0)),
                text("System tweaks and optimizations")
                    .size(body_font_size)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(0.0));

        // Output log
        let output_log: Element<Message> = if self.output_log.is_empty() {
            container(
                text("No output yet. Click 'Gaming Meta' to start installation.")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
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
                    self.output_log
                        .iter()
                        .map(|line| {
                            let line_color = if line.starts_with("✓") {
                                iced::Color::from_rgb(0.1, 0.5, 0.1)
                            } else if line.starts_with("✗") {
                                iced::Color::from_rgb(0.9, 0.2, 0.2)
                            } else {
                                theme.text()
                            };
                            text(line)
                                .size(body_font_size * 0.93)
                                .style(iced::theme::Text::Color(line_color))
                                .font(iced::Font::MONOSPACE)
                                .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(4)
                .padding(12)
            )
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                Color::from(settings.background_color.clone()),
                settings.border_radius,
            ))))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        // DNF Configuration section
        let input_font_size = (settings.font_size_inputs * settings.scale_inputs).round();
        
        // Display current config state
        let current_config_display: Element<Message> = if self.is_loading_dnf_config {
            container(
                text("Loading current configuration...")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
            )
            .padding(12)
            .into()
        } else if let Some(ref config) = self.dnf_config {
            container(
                column![
                    text("Current Configuration:")
                        .size(body_font_size * 0.93)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        text("Max Parallel Downloads: ")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.text())),
                        text(config.max_parallel_downloads
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "Not set (default: 20)".to_string()))
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.primary())),
                    ]
                    .spacing(4),
                    Space::with_height(Length::Fixed(4.0)),
                    row![
                        text("Fastest Mirror: ")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.text())),
                        text(if config.fastestmirror { "Enabled" } else { "Disabled" })
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(if config.fastestmirror {
                                iced::Color::from_rgb(0.1, 0.5, 0.1)
                            } else {
                                theme.secondary_text()
                            })),
                    ]
                    .spacing(4),
                ]
                .spacing(0)
            )
            .padding(12)
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else if let Some(ref error) = self.dnf_config_error {
            container(
                text(format!("Error loading config: {}", error))
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2)))
            )
            .padding(12)
            .into()
        } else {
            container(
                text("Click 'Load Current Config' to view current settings")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
            )
            .padding(12)
            .into()
        };
        
        let parallel_downloads_input = text_input(
            "Enter number (1-25), default is 20",
            &self.parallel_downloads_input
        )
        .on_input(Message::ParallelDownloadsChanged)
        .size(input_font_size)
        .width(Length::Fill)
        .padding(12)
        .style(iced::theme::TextInput::Custom(Box::new(RoundedTextInputStyle {
            radius: settings.border_radius,
        })));
        
        let fastest_mirror_checkbox = checkbox(
            "Enable fastest mirror selection",
            self.fastest_mirror_enabled,
        )
        .on_toggle(Message::FastestMirrorToggled)
        .size(body_font_size);
        
        let load_dnf_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                text(" Load Current Config").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::LoadDnfConfig)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));
        
        let save_dnf_button = button(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                text(" Save DNF Config").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::SaveDnfConfig)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));
        
        let dnf_config_status = if self.is_loading_dnf_config {
            text("Loading DNF configuration...")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
        } else if self.is_saving_dnf_config {
            text("Saving DNF configuration...")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
        } else if let Some(ref error) = self.dnf_config_error {
            text(format!("Error: {}", error))
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2)))
        } else if self.dnf_config.is_some() {
            text("✓ Configuration loaded")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.5, 0.1)))
        } else {
            text("Click 'Load Current Config' to view current settings")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
        };
        
        let dnf_config_info = container(
            column![
                text("DNF Speed Optimization")
                    .size(title_font_size * 0.71)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                text("Configure DNF to speed up package downloads")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(4.0)),
                text("Note: Saving changes requires administrator privileges (sudo)")
                    .size(body_font_size * 0.86)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.7, 0.1))),
                Space::with_height(Length::Fixed(16.0)),
                // Current config display
                current_config_display,
                Space::with_height(Length::Fixed(16.0)),
                text("Edit Configuration:")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                text("Max Parallel Downloads (1-25):")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(8.0)),
                parallel_downloads_input,
                Space::with_height(Length::Fixed(12.0)),
                text("Allows DNF to download multiple packages simultaneously. Recommended: 20")
                    .size(body_font_size * 0.86)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(16.0)),
                fastest_mirror_checkbox,
                Space::with_height(Length::Fixed(8.0)),
                text("Automatically selects the fastest mirror for your location")
                    .size(body_font_size * 0.86)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(20.0)),
                row![
                    load_dnf_button,
                    Space::with_width(Length::Fixed(12.0)),
                    save_dnf_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
                Space::with_height(Length::Fixed(12.0)),
                dnf_config_status,
            ]
            .spacing(0)
            .padding(Padding::new(24.0))
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
            radius: settings.border_radius,
        })));

        // Sub-tabs for Gaming Meta, DNF Config, and Cachyos Kernel
        let tab_font_size = (settings.font_size_tabs * settings.scale_tabs).round();
        let sub_tabs = container(
            row![
                button(
                    text("Gaming Meta")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == TweaksView::GamingMeta {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == TweaksView::GamingMeta,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(TweaksView::GamingMeta))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
                button(
                    text("DNF Config")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == TweaksView::DnfConfig {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == TweaksView::DnfConfig,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(TweaksView::DnfConfig))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
                button(
                    text("Cachyos Kernel")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == TweaksView::CachyosKernel {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == TweaksView::CachyosKernel,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(TweaksView::CachyosKernel))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
            ]
            .spacing(12)
        )
        .width(Length::Fill)
        .padding(Padding::from([0.0, 32.0, 20.0, 32.0]));

        // Cachyos Kernel status display
        let cachyos_kernel_status_display: Element<Message> = if self.is_checking_cachyos_kernel {
            text("Checking installation status...")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
                .into()
        } else if let Some(ref status) = self.cachyos_kernel_status {
            column![
                text("Installation Status:")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(8.0)),
                row![
                    text(if status.kernel_cachyos { "✓ kernel-cachyos" } else { "✗ kernel-cachyos" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.kernel_cachyos {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                    Space::with_width(Length::Fixed(16.0)),
                    text(if status.cachyos_settings { "✓ cachyos-settings" } else { "✗ cachyos-settings" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.cachyos_settings {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(4.0)),
                row![
                    text(if status.ananicy_cpp { "✓ ananicy-cpp" } else { "✗ ananicy-cpp" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.ananicy_cpp {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                    Space::with_width(Length::Fixed(16.0)),
                    text(if status.cachyos_ananicy_rules { "✓ cachyos-ananicy-rules" } else { "✗ cachyos-ananicy-rules" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.cachyos_ananicy_rules {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(4.0)),
                row![
                    text(if status.scx_manager { "✓ scx-manager" } else { "✗ scx-manager" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.scx_manager {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                    Space::with_width(Length::Fixed(16.0)),
                    text(if status.scx_scheds_git { "✓ scx-scheds-git" } else { "✗ scx-scheds-git" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.scx_scheds_git {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(4.0)),
                text(if status.scx_tools { "✓ scx-tools" } else { "✗ scx-tools" })
                    .size(body_font_size * 0.86)
                    .style(iced::theme::Text::Color(if status.scx_tools {
                        iced::Color::from_rgb(0.1, 0.5, 0.1)
                    } else {
                        iced::Color::from_rgb(0.7, 0.7, 0.7)
                    })),
            ]
            .spacing(0)
            .into()
        } else {
            text("Click 'Check Status' to see installed packages")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
                .into()
        };

        let check_cachyos_status_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                text(" Check Status").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::CheckCachyosKernelStatus)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));

        // Cachyos Kernel button
        let cachyos_kernel_button = button(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                text(" Install Cachyos Kernel").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::InstallCachyosKernel)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(16.0));

        // Cachyos Kernel info card
        let cachyos_kernel_info = container(
            column![
                text("Cachyos Kernel")
                    .size(title_font_size * 0.71)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                text("Installs Cachyos kernel with scheduler extensions:")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(8.0)),
                text("• kernel-cachyos + cachyos-settings")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                text("• ananicy-cpp, cachyos-ananicy-rules")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                text("• scx-manager, scx-scheds-git, scx-tools")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                text("• Auto-configures GRUB and regenerates initramfs")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(20.0)),
                row![
                    cachyos_kernel_button,
                    Space::with_width(Length::Fixed(12.0)),
                    check_cachyos_status_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
                Space::with_height(Length::Fixed(16.0)),
                cachyos_kernel_status_display,
            ]
            .spacing(8)
            .padding(Padding::new(24.0))
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
            radius: settings.border_radius,
        })));

        // Gaming Meta status display
        let gaming_meta_status_display: Element<Message> = if self.is_checking_gaming_meta {
            text("Checking installation status...")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
                .into()
        } else if let Some(ref status) = self.gaming_meta_status {
            column![
                text("Installation Status:")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(8.0)),
                row![
                    text(if status.steam { "✓ Steam" } else { "✗ Steam" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.steam {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                    Space::with_width(Length::Fixed(16.0)),
                    text(if status.lutris { "✓ Lutris" } else { "✗ Lutris" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.lutris {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(4.0)),
                row![
                    text(if status.mangohud { "✓ MangoHUD" } else { "✗ MangoHUD" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.mangohud {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                    Space::with_width(Length::Fixed(16.0)),
                    text(if status.gamescope { "✓ Gamescope" } else { "✗ Gamescope" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.gamescope {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(4.0)),
                row![
                    text(if status.mangojuice { "✓ MangoJuice" } else { "✗ MangoJuice" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.mangojuice {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                    Space::with_width(Length::Fixed(16.0)),
                    text(if status.protonplus { "✓ ProtonPlus" } else { "✗ ProtonPlus" })
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(if status.protonplus {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        })),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(4.0)),
                text(if status.heroic { "✓ Heroic Games Launcher" } else { "✗ Heroic Games Launcher" })
                    .size(body_font_size * 0.86)
                    .style(iced::theme::Text::Color(if status.heroic {
                        iced::Color::from_rgb(0.1, 0.5, 0.1)
                    } else {
                        iced::Color::from_rgb(0.7, 0.7, 0.7)
                    })),
            ]
            .spacing(0)
            .into()
        } else {
            text("Click 'Check Status' to see installed packages")
                .size(body_font_size * 0.93)
                .style(iced::theme::Text::Color(theme.secondary_text()))
                .into()
        };

        let check_status_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                text(" Check Status").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::CheckGamingMetaStatus)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(12.0));

        // Updated Gaming Meta info card with status
        let gaming_meta_info = container(
            column![
                text("Gaming Meta")
                    .size(title_font_size * 0.71)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                text("Installs a complete gaming setup including:")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(8.0)),
                text("• Steam, Lutris, MangoHUD, Gamescope")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                text("• ProtonPlus, MangoJuice (Flatpak)")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                text("• Heroic Games Launcher (latest release)")
                    .size(body_font_size * 0.93)
                    .style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(20.0)),
                row![
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                            text(" Gaming Meta").size(button_font_size)
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::InstallGamingMeta)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::new(16.0)),
                    Space::with_width(Length::Fixed(12.0)),
                    check_status_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
                Space::with_height(Length::Fixed(16.0)),
                gaming_meta_status_display,
            ]
            .spacing(8)
            .padding(Padding::new(24.0))
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
            radius: settings.border_radius,
        })));

        // Content based on current view
        let content: Element<Message> = match self.current_view {
            TweaksView::GamingMeta => {
                column![
                    gaming_meta_info,
                    Space::with_height(Length::Fixed(16.0)),
                    output_log,
                ]
                .spacing(0)
                .into()
            }
            TweaksView::DnfConfig => {
                dnf_config_info.into()
            }
            TweaksView::CachyosKernel => {
                cachyos_kernel_info.into()
            }
        };

        container(
            column![
                header,
                Space::with_height(Length::Fixed(24.0)),
                sub_tabs,
                Space::with_height(Length::Fixed(16.0)),
                content,
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(32.0))
        .into()
    }
}

async fn check_dnf_package(package: &str) -> bool {
    use tokio::process::Command as TokioCommand;
    let mut cmd = TokioCommand::new("rpm");
    cmd.arg("-q");
    cmd.arg(package);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let output = cmd.output().await.ok();
    output.map(|o| o.status.success()).unwrap_or(false)
}

async fn check_flatpak_package(package: &str) -> bool {
    use tokio::process::Command as TokioCommand;
    let mut cmd = TokioCommand::new("flatpak");
    cmd.arg("list");
    cmd.arg("--app");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let output = cmd.output().await.ok();
    if let Some(o) = output {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            return stdout.contains(package);
        }
    }
    false
}

async fn check_gaming_meta_status() -> Result<GamingMetaStatus, String> {
    let steam = check_dnf_package("steam").await;
    let lutris = check_dnf_package("lutris").await;
    let mangohud = check_dnf_package("mangohud").await;
    let gamescope = check_dnf_package("gamescope").await;
    let mangojuice = check_flatpak_package("io.github.radiolamp.mangojuice").await;
    let protonplus = check_flatpak_package("com.vysp3r.ProtonPlus").await;
    let heroic = check_dnf_package("heroic").await;
    
    Ok(GamingMetaStatus {
        steam,
        lutris,
        mangohud,
        gamescope,
        mangojuice,
        protonplus,
        heroic,
    })
}

async fn check_cachyos_kernel_status() -> Result<CachyosKernelStatus, String> {
    let kernel_cachyos = check_dnf_package("kernel-cachyos").await;
    let cachyos_settings = check_dnf_package("cachyos-settings").await;
    let ananicy_cpp = check_dnf_package("ananicy-cpp").await;
    let cachyos_ananicy_rules = check_dnf_package("cachyos-ananicy-rules").await;
    let scx_manager = check_dnf_package("scx-manager").await;
    let scx_scheds_git = check_dnf_package("scx-scheds-git").await;
    let scx_tools = check_dnf_package("scx-tools").await;
    
    Ok(CachyosKernelStatus {
        kernel_cachyos,
        cachyos_settings,
        ananicy_cpp,
        cachyos_ananicy_rules,
        scx_manager,
        scx_scheds_git,
        scx_tools,
    })
}

// Load DNF configuration from /etc/dnf/dnf.conf
// Requires sudo privileges via pkexec
async fn load_dnf_config() -> Result<DnfConfig, String> {
    use tokio::process::Command as TokioCommand;
    let config_path = PathBuf::from("/etc/dnf/dnf.conf");
    
    // Always use pkexec to read the file (requires root)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("cat");
    cmd.arg(&config_path);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    let output = cmd.output().await
        .map_err(|e| format!("Failed to read dnf.conf: {}. Make sure polkit is installed.", e))?;
    
    // Check if user cancelled the password dialog (exit code 126 or 127)
    if !output.status.success() {
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to read dnf.conf: {}", stderr));
    }
    
    let content = String::from_utf8_lossy(&output.stdout).to_string();
    
    // Parse the INI-style config file
    let mut max_parallel_downloads = None;
    let mut fastestmirror = false;
    
    let mut in_main_section = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Check for section headers
        if line.starts_with('[') && line.ends_with(']') {
            in_main_section = line == "[main]";
            continue;
        }
        
        // Only process lines in [main] section
        if !in_main_section {
            continue;
        }
        
        // Parse key=value pairs
        if let Some(equal_pos) = line.find('=') {
            let key = line[..equal_pos].trim().to_lowercase();
            let value = line[equal_pos+1..].trim();
            
            match key.as_str() {
                "max_parallel_downloads" => {
                    if let Ok(val) = value.parse::<u32>() {
                        max_parallel_downloads = Some(val);
                    }
                }
                "fastestmirror" => {
                    fastestmirror = value.eq_ignore_ascii_case("true") || value == "1";
                }
                _ => {}
            }
        }
    }
    
    Ok(DnfConfig {
        max_parallel_downloads,
        fastestmirror,
    })
}

// Save DNF configuration to /etc/dnf/dnf.conf
async fn save_dnf_config(config: DnfConfig) -> Result<(), String> {
    use tokio::process::Command as TokioCommand;
    use std::io::Write;
    
    let config_path = PathBuf::from("/etc/dnf/dnf.conf");
    
    // Read existing config using pkexec (requires root)
    let mut read_cmd = TokioCommand::new("pkexec");
    read_cmd.arg("cat");
    read_cmd.arg(&config_path);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        read_cmd.env("DISPLAY", display);
    }
    
    let read_output = read_cmd.output().await
        .map_err(|e| format!("Failed to read dnf.conf: {}", e))?;
    
    let existing_content = if !read_output.status.success() {
        // If file doesn't exist or can't be read, create default content
        if read_output.status.code() == Some(126) || read_output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        "[main]\n".to_string()
    } else {
        String::from_utf8_lossy(&read_output.stdout).to_string()
    };
    
    // Parse and update the config
    let mut lines: Vec<String> = existing_content.lines().map(|s| s.to_string()).collect();
    let mut in_main_section = false;
    let mut main_section_start = None;
    let mut main_section_end = None;
    
    // Find [main] section boundaries
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[main]" {
            in_main_section = true;
            main_section_start = Some(i);
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') && in_main_section {
            main_section_end = Some(i);
            break;
        }
    }
    
    // If no [main] section found, add it
    if main_section_start.is_none() {
        lines.insert(0, "[main]".to_string());
        main_section_start = Some(0);
        main_section_end = Some(1);
    }
    
    let start = main_section_start.unwrap();
    let end = main_section_end.unwrap_or(lines.len());
    
    // Remove existing max_parallel_downloads and fastestmirror from [main] section
    // But preserve comments and other settings
    let mut new_main_section: Vec<String> = lines[start..end]
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            // Keep comments, empty lines, and other settings
            // Only remove max_parallel_downloads and fastestmirror lines
            if trimmed.starts_with('#') || trimmed.is_empty() {
                true
            } else if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_lowercase();
                key != "max_parallel_downloads" && key != "fastestmirror"
            } else {
                true
            }
        })
        .cloned()
        .collect();
    
    // Add new settings
    if let Some(parallel) = config.max_parallel_downloads {
        new_main_section.push(format!("max_parallel_downloads={}", parallel));
    }
    new_main_section.push(format!("fastestmirror={}", if config.fastestmirror { "true" } else { "false" }));
    
    // Rebuild the file content
    let mut new_content = if start > 0 {
        lines[..start].join("\n") + "\n"
    } else {
        String::new()
    };
    
    new_content += &new_main_section.join("\n");
    new_content += "\n";
    
    if end < lines.len() {
        new_content += &lines[end..].join("\n");
    }
    
    // Write to temp file first
    let temp_file = std::env::temp_dir().join(format!("dnf_conf_{}.tmp", std::process::id()));
    {
        let mut file = std::fs::File::create(&temp_file)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(new_content.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
    }
    
    // Use pkexec to copy temp file to /etc/dnf/dnf.conf
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("cp");
    cmd.arg(temp_file.to_str().ok_or("Invalid temp file path")?);
    cmd.arg(config_path.to_str().ok_or("Invalid config path")?);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    let output = cmd.output().await
        .map_err(|e| format!("Failed to save dnf.conf: {}. Make sure polkit is installed.", e))?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);
    
    if !output.status.success() {
        // Check if user cancelled the password dialog
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to save dnf.conf: {}", stderr));
    }
    
    Ok(())
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
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        if !self.is_active {
            let palette = style.palette();
            let is_dark = palette.background.r < 0.5;
            appearance.background = Some(if is_dark {
                iced::Color::from_rgba(0.25, 0.25, 0.25, 1.0).into()
            } else {
                iced::Color::from_rgba(0.85, 0.85, 0.86, 1.0).into()
            });
        }
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

