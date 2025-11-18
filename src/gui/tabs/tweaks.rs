use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, checkbox, progress_bar};
use iced::{Alignment, Element, Length, Padding, Border, Color};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use iced::widget::pick_list;
use crate::gui::app::CustomScrollableStyle;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TweaksView {
    GamingMeta,
    DnfConfig,
    CachyosKernel,
    Hyprland,
    Proton,
    SteamGames,
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
    CheckHyprlandStatus,
    HyprlandStatusChecked(Result<HyprlandStatus, String>),
    InstallHyprland,
    InstallHyprlandDotfiles,
    LoadProtonBuilds,
    ProtonBuildsLoaded(Result<Vec<ProtonRunner>, String>),
    #[allow(dead_code)]
    DetectLaunchers,
    LaunchersDetected(Result<Vec<DetectedLauncher>, String>),
    SelectProtonRunner(String), // runner title
    SelectLauncher(String), // launcher title
    ToggleFilterInstalled,
    ToggleFilterUsed,
    ToggleFilterUnused,
    #[allow(dead_code)]
    CheckProtonUsage, // Check which builds are being used by games
    ProtonUsageChecked(Result<Vec<ProtonRunner>, String>),
    DownloadProtonBuild(String, String, String), // runner_title, title, download_url
    #[allow(dead_code)]
    DownloadProgress(String, f32, String), // title, progress (0.0-1.0), message
    ProtonBuildDownloaded(Result<(String, String, String), String>), // runner_title, title, path
    #[allow(dead_code)]
    InstallProgress(String, f32, String), // title, progress (0.0-1.0), message
    ProtonBuildInstalled(Result<(String, String), String>), // runner_title, title
    #[allow(dead_code)]
    CloseProgressDialog,
    CloseCompletionDialog,
    RemoveProtonBuild(String, String), // runner_title, title
    ProtonBuildRemoved(Result<(String, String), String>), // runner_title, title
    UpdateProtonBuild(String, String), // runner_title, title (for "Latest" builds)
    ProtonBuildUpdated(Result<(String, String), String>), // runner_title, title
    #[allow(dead_code)]
    UpdateAllProtonBuilds, // Update all "Latest" builds
    OpenProtonBuildDirectory(String, String), // runner_title, title
    ShowProtonBuildInfo(String, String, String, String), // runner_title, title, description, page_url
    LoadMoreProtonBuilds(String), // runner_title
    MoreProtonBuildsLoaded(Result<(String, Vec<ProtonBuild>), String>), // runner_title, new_builds
    LoadSteamGames,
    SteamGamesLoaded(Result<Vec<SteamGame>, String>),
    ChangeSteamGameCompatibilityTool(u32, String), // appid, compatibility_tool_name
    SteamGameCompatibilityToolChanged(Result<(u32, String), String>), // appid, compatibility_tool_name
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
    pub repo_kernel_cachyos: bool,
    pub repo_kernel_cachyos_addons: bool,
}

#[derive(Debug, Clone)]
pub struct HyprlandStatus {
    pub hyprland: bool,
    pub hyprpicker: bool,
    pub swww: bool,
    pub quickshell_git: bool,
    pub fuzzel: bool,
    pub wlogout: bool,
    pub cliphist: bool,
    pub brightnessctl: bool,
    pub grim: bool,
    pub slurp: bool,
    pub swappy: bool,
    pub repo_rpmfusion_free: bool,
    pub repo_rpmfusion_nonfree: bool,
    pub repo_quickshell: bool,
    pub repo_hyprland: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtonBuild {
    pub title: String,
    pub description: String,
    pub release_date: String,
    pub download_url: String,
    pub page_url: String,
    pub download_size: u64,
    pub runner_title: String, // e.g., "Proton-GE"
    #[serde(skip)]
    pub is_installed: bool, // Not cached, checked at runtime
    pub directory_name_formats: Vec<DirectoryNameFormat>,
    #[serde(skip)]
    pub usage_count: u32, // Not cached, checked at runtime
    pub is_latest: bool, // True if this is the "Latest" auto-updating version
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirectoryNameFormat {
    pub launcher: String,
    pub directory_name_format: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtonRunner {
    pub title: String,
    pub description: String,
    pub endpoint: String,
    pub asset_position: usize,
    pub directory_name_formats: Vec<DirectoryNameFormat>,
    pub builds: Vec<ProtonBuild>,
    pub has_latest_support: bool, // Whether this runner supports "Latest" auto-update
    pub compat_layer_type: String, // "Proton" or "Wine" or "DXVK" or "VKD3D"
}

#[derive(Debug, Clone)]
pub struct DetectedLauncher {
    pub title: String,
    pub directory: String,
    pub installation_type: String, // "system", "flatpak", "snap"
    #[allow(dead_code)]
    pub is_installed: bool,
}

#[derive(Debug, Clone)]
pub struct SteamGame {
    pub name: String,
    pub appid: u32,
    pub compatibility_tool: String, // Current compatibility tool name, or "Undefined"
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
    // Hyprland status
    hyprland_status: Option<HyprlandStatus>,
    is_checking_hyprland: bool,
    // Proton builds
    proton_runners: Vec<ProtonRunner>,
    selected_proton_runner: Option<String>, // runner title
    detected_launchers: Vec<DetectedLauncher>,
    selected_launcher: Option<String>, // launcher title (e.g., "Steam", "Lutris", "Heroic Games Launcher")
    is_loading_proton_builds: bool,
    is_detecting_launchers: bool,
    proton_builds_error: Option<String>,
    downloading_build: Option<String>, // title being downloaded
    installing_build: Option<String>, // title being installed
    download_progress: f32, // 0.0 to 1.0
    install_progress: f32, // 0.0 to 1.0
    progress_text: String, // Current progress message
    show_progress_dialog: bool, // Whether to show progress dialog
    show_completion_dialog: bool, // Whether to show completion dialog
    completion_message: String, // Completion message
    completion_success: bool, // Whether completion was successful
    // Filters
    show_installed_only: bool,
    show_used_only: bool,
    show_unused_only: bool,
    // Steam Games
    steam_games: Vec<SteamGame>,
    is_loading_steam_games: bool,
    steam_games_error: Option<String>,
    steam_directory: Option<String>, // Detected Steam directory
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
            hyprland_status: None,
            is_checking_hyprland: false,
            proton_runners: Vec::new(),
            selected_proton_runner: None,
            detected_launchers: Vec::new(),
            selected_launcher: None,
            is_loading_proton_builds: false,
            is_detecting_launchers: false,
            proton_builds_error: None,
            downloading_build: None,
            installing_build: None,
            download_progress: 0.0,
            install_progress: 0.0,
            progress_text: String::new(),
            show_progress_dialog: false,
            show_completion_dialog: false,
            completion_message: String::new(),
            completion_success: false,
            show_installed_only: false,
            show_used_only: false,
            show_unused_only: false,
            steam_games: Vec::new(),
            is_loading_steam_games: false,
            steam_games_error: None,
            steam_directory: None,
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
                // Check Hyprland status when switching to that tab
                if view == TweaksView::Hyprland && self.hyprland_status.is_none() {
                    self.is_checking_hyprland = true;
                    return iced::Command::perform(check_hyprland_status(), Message::HyprlandStatusChecked);
                }
                // Load Proton builds and detect launchers when switching to that tab
                if view == TweaksView::Proton {
                    // Always reload if not currently loading and either empty or error exists
                    if !self.is_loading_proton_builds {
                        if self.proton_runners.is_empty() || self.proton_builds_error.is_some() {
                            eprintln!("[DEBUG] Loading Proton builds (empty: {}, has_error: {})", 
                                self.proton_runners.is_empty(), 
                                self.proton_builds_error.is_some());
                            self.is_loading_proton_builds = true;
                            self.proton_builds_error = None;
                            return iced::Command::batch(vec![
                                iced::Command::perform(detect_launchers(), Message::LaunchersDetected),
                                iced::Command::perform(load_proton_builds(), Message::ProtonBuildsLoaded),
                            ]);
                        }
                    }
                    if self.detected_launchers.is_empty() && !self.is_detecting_launchers {
                        self.is_detecting_launchers = true;
                        return iced::Command::perform(detect_launchers(), Message::LaunchersDetected);
                    }
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
                            repo_kernel_cachyos: false,
                            repo_kernel_cachyos_addons: false,
                        });
                    }
                }
                iced::Command::none()
            }
            Message::CheckHyprlandStatus => {
                self.is_checking_hyprland = true;
                iced::Command::perform(check_hyprland_status(), Message::HyprlandStatusChecked)
            }
            Message::HyprlandStatusChecked(result) => {
                self.is_checking_hyprland = false;
                match result {
                    Ok(status) => {
                        self.hyprland_status = Some(status);
                    }
                    Err(_) => {
                        // On error, just set all to false
                        self.hyprland_status = Some(HyprlandStatus {
                            hyprland: false,
                            hyprpicker: false,
                            swww: false,
                            quickshell_git: false,
                            fuzzel: false,
                            wlogout: false,
                            cliphist: false,
                            brightnessctl: false,
                            grim: false,
                            slurp: false,
                            swappy: false,
                            repo_rpmfusion_free: false,
                            repo_rpmfusion_nonfree: false,
                            repo_quickshell: false,
                            repo_hyprland: false,
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
            Message::InstallHyprland => {
                // Spawn a separate window for Hyprland installation
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("hyprland-dialog")
                            .spawn()
                            .ok();
                    },
                    |_| Message::GamingMetaComplete(Ok("Dialog opened".to_string())),
                )
            }
            Message::InstallHyprlandDotfiles => {
                // Spawn a separate window for dotfiles installation
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("hyprland-dotfiles-dialog")
                            .spawn()
                            .ok();
                    },
                    |_| Message::GamingMetaComplete(Ok("Dialog opened".to_string())),
                )
            }
            Message::LoadProtonBuilds => {
                self.is_loading_proton_builds = true;
                self.proton_builds_error = None;
                iced::Command::perform(load_proton_builds(), Message::ProtonBuildsLoaded)
            }
            Message::ProtonBuildsLoaded(result) => {
                self.is_loading_proton_builds = false;
                eprintln!("[DEBUG] ========== ProtonBuildsLoaded START ==========");
                match result {
                    Ok(runners) => {
                        eprintln!("[DEBUG] ProtonBuildsLoaded: received {} runners", runners.len());
                        for (idx, runner) in runners.iter().enumerate() {
                            eprintln!("[DEBUG] Runner {}: {} (type: {})", idx, runner.title, runner.compat_layer_type);
                            eprintln!("[DEBUG]   Description: {}", runner.description);
                            eprintln!("[DEBUG]   Endpoint: {}", runner.endpoint);
                            eprintln!("[DEBUG]   Has {} builds", runner.builds.len());
                            for (build_idx, build) in runner.builds.iter().enumerate() {
                                eprintln!("[DEBUG]     Build {}: {} (installed: {}, latest: {})", 
                                    build_idx, build.title, build.is_installed, build.is_latest);
                            }
                        }
                        self.proton_runners = runners;
                        self.proton_builds_error = None;
                        // Auto-select first runner if none selected
                        if self.selected_proton_runner.is_none() && !self.proton_runners.is_empty() {
                            self.selected_proton_runner = Some(self.proton_runners[0].title.clone());
                            eprintln!("[DEBUG] Auto-selected first runner: {}", self.proton_runners[0].title);
                        }
                        // Update installation status if launchers are already detected
                        if !self.detected_launchers.is_empty() {
                            eprintln!("[DEBUG] Launchers detected ({}), updating installation status", self.detected_launchers.len());
                            self.update_proton_installation_status();
                            // Check usage counts
                            eprintln!("[DEBUG] Checking Proton usage counts...");
                            return iced::Command::perform(check_proton_usage(self.proton_runners.clone(), self.detected_launchers.clone()), Message::ProtonUsageChecked);
                        } else {
                            eprintln!("[DEBUG] No launchers detected yet, installation status will be updated when launchers are detected");
                        }
                        eprintln!("[DEBUG] UI should now show {} runners", self.proton_runners.len());
                        eprintln!("[DEBUG] ========== ProtonBuildsLoaded END ==========");
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] ProtonBuildsLoaded error: {}", e);
                        self.proton_builds_error = Some(e);
                        eprintln!("[DEBUG] ========== ProtonBuildsLoaded END (ERROR) ==========");
                    }
                }
                iced::Command::none()
            }
            Message::DetectLaunchers => {
                self.is_detecting_launchers = true;
                iced::Command::perform(detect_launchers(), Message::LaunchersDetected)
            }
            Message::LaunchersDetected(result) => {
                self.is_detecting_launchers = false;
                match result {
                    Ok(launchers) => {
                        self.detected_launchers = launchers;
                        eprintln!("[DEBUG] Detected {} launchers", self.detected_launchers.len());
                        // Auto-select first launcher if none selected
                        if self.selected_launcher.is_none() && !self.detected_launchers.is_empty() {
                            self.selected_launcher = Some(self.detected_launchers[0].title.clone());
                        }
                        // Update installation status for all builds with the detected launchers
                        self.update_proton_installation_status();
                        // Check usage counts
                        return iced::Command::perform(check_proton_usage(self.proton_runners.clone(), self.detected_launchers.clone()), Message::ProtonUsageChecked);
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to detect launchers: {}", e);
                        // Don't set error, just log it
                    }
                }
                iced::Command::none()
            }
            Message::SelectProtonRunner(runner_title) => {
                self.selected_proton_runner = Some(runner_title);
                iced::Command::none()
            }
            Message::SelectLauncher(launcher_title) => {
                self.selected_launcher = Some(launcher_title);
                // Update installation status when launcher changes
                self.update_proton_installation_status();
                // Check usage counts
                iced::Command::perform(check_proton_usage(self.proton_runners.clone(), self.detected_launchers.clone()), Message::ProtonUsageChecked)
            }
            Message::ToggleFilterInstalled => {
                self.show_installed_only = !self.show_installed_only;
                if self.show_installed_only {
                    self.show_used_only = false;
                    self.show_unused_only = false;
                }
                iced::Command::none()
            }
            Message::ToggleFilterUsed => {
                self.show_used_only = !self.show_used_only;
                if self.show_used_only {
                    self.show_installed_only = false;
                    self.show_unused_only = false;
                }
                iced::Command::none()
            }
            Message::ToggleFilterUnused => {
                self.show_unused_only = !self.show_unused_only;
                if self.show_unused_only {
                    self.show_installed_only = false;
                    self.show_used_only = false;
                }
                iced::Command::none()
            }
            Message::CheckProtonUsage => {
                iced::Command::perform(check_proton_usage(self.proton_runners.clone(), self.detected_launchers.clone()), Message::ProtonUsageChecked)
            }
            Message::ProtonUsageChecked(result) => {
                match result {
                    Ok(updated_runners) => {
                        eprintln!("[DEBUG] ProtonUsageChecked: Updated {} runners with usage counts", updated_runners.len());
                        self.proton_runners = updated_runners;
                        eprintln!("[DEBUG] UI should now refresh with updated usage counts");
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to check usage: {}", e);
                    }
                }
                iced::Command::none()
            }
            Message::DownloadProtonBuild(runner_title, title, download_url) => {
                // Launch separate installation window as a separate process
                let runner_info = self.proton_runners.iter()
                    .find(|r| r.title == runner_title)
                    .and_then(|r| serde_json::to_string(r).ok());
                
                let exe_path = std::env::current_exe()
                    .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                
                let runner_title_clone = runner_title.clone();
                let title_clone = title.clone();
                let download_url_clone = download_url.clone();
                let selected_launcher_clone = self.selected_launcher.clone();
                let runner_info_clone = runner_info.clone();
                
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("proton-install-dialog");
                        cmd.arg(&runner_title_clone);
                        cmd.arg(&title_clone);
                        cmd.arg(&download_url_clone);
                        if let Some(launcher) = &selected_launcher_clone {
                            cmd.arg("--launcher").arg(launcher);
                        }
                        if let Some(runner_info) = &runner_info_clone {
                            cmd.arg("--runner-info").arg(runner_info);
                        }
                        let _ = cmd.spawn();
                    },
                    |_| Message::GamingMetaComplete(Ok("Dialog opened".to_string())),
                )
            }
            Message::DownloadProgress(_title, progress, message) => {
                self.download_progress = progress;
                self.progress_text = message;
                iced::Command::none()
            }
            Message::InstallProgress(_title, progress, message) => {
                self.install_progress = progress;
                self.progress_text = message;
                iced::Command::none()
            }
            Message::CloseProgressDialog => {
                self.show_progress_dialog = false;
                self.download_progress = 0.0;
                self.install_progress = 0.0;
                self.progress_text.clear();
                iced::Command::none()
            }
            Message::CloseCompletionDialog => {
                self.show_completion_dialog = false;
                self.completion_message.clear();
                iced::Command::none()
            }
            Message::ProtonBuildDownloaded(result) => {
                match result {
                    Ok((runner_title, title, path)) => {
                        self.downloading_build = None;
                        self.download_progress = 1.0;
                        self.installing_build = Some(title.clone());
                        self.install_progress = 0.0;
                        self.progress_text = format!("Installing {}...", title);
                        // Get the selected launcher and runner info for installation
                        let selected_launcher = self.selected_launcher.clone();
                        let runner = self.proton_runners.iter().find(|r| r.title == runner_title).cloned();
                        // Check if this is an update (for "Latest" builds)
                        let is_update = self.proton_runners.iter()
                            .any(|r| r.title == runner_title && r.builds.iter().any(|b| b.title == title && b.is_latest && b.is_installed));
                        iced::Command::perform(
                            install_proton_build_with_launcher(runner_title, title, path, selected_launcher, runner),
                            if is_update {
                                Message::ProtonBuildUpdated
                            } else {
                                Message::ProtonBuildInstalled
                            }
                        )
                    }
                    Err(e) => {
                        self.downloading_build = None;
                        self.show_progress_dialog = false;
                        self.show_completion_dialog = true;
                        self.completion_message = format!("Download failed: {}", e);
                        self.completion_success = false;
                        self.proton_builds_error = Some(e);
                        iced::Command::none()
                    }
                }
            }
            Message::ProtonBuildInstalled(result) => {
                self.installing_build = None;
                self.install_progress = 1.0;
                match result {
                    Ok((runner_title, title)) => {
                        eprintln!("[DEBUG] Installation complete, updating installation status for {} {}", runner_title, title);
                        // Update installation status in the runner's builds
                        if let Some(runner) = self.proton_runners.iter_mut().find(|r| r.title == runner_title) {
                            if let Some(build) = runner.builds.iter_mut().find(|b| b.title == title) {
                                build.is_installed = true;
                            }
                        }
                        // Update installation status
                        self.update_proton_installation_status();
                        self.show_progress_dialog = false;
                        self.show_completion_dialog = true;
                        self.completion_message = format!("Successfully installed {} {}", runner_title, title);
                        self.completion_success = true;
                        iced::Command::none()
                    }
                    Err(e) => {
                        self.show_progress_dialog = false;
                        self.show_completion_dialog = true;
                        self.completion_message = format!("Installation failed: {}", e);
                        self.completion_success = false;
                        self.proton_builds_error = Some(e);
                        iced::Command::none()
                    }
                }
            }
            Message::RemoveProtonBuild(runner_title, title) => {
                iced::Command::perform(remove_proton_build(runner_title, title, self.selected_launcher.clone(), self.proton_runners.clone(), self.detected_launchers.clone()), Message::ProtonBuildRemoved)
            }
            Message::ProtonBuildRemoved(result) => {
                match result {
                    Ok((runner_title, title)) => {
                        eprintln!("[DEBUG] Removal complete, updating installation status for {} {}", runner_title, title);
                        // Update installation status in the runner's builds
                        if let Some(runner) = self.proton_runners.iter_mut().find(|r| r.title == runner_title) {
                            if let Some(build) = runner.builds.iter_mut().find(|b| b.title == title) {
                                build.is_installed = false;
                            }
                        }
                        iced::Command::none()
                    }
                    Err(e) => {
                        self.proton_builds_error = Some(e);
                        iced::Command::none()
                    }
                }
            }
            Message::UpdateProtonBuild(runner_title, title) => {
                eprintln!("[DEBUG] UpdateProtonBuild requested: {} {}", runner_title, title);
                // For "Latest" builds, update means re-downloading the latest release
                if let Some(runner) = self.proton_runners.iter().find(|r| r.title == runner_title) {
                    eprintln!("[DEBUG] Found runner: {}", runner_title);
                    if let Some(_build) = runner.builds.iter().find(|b| b.title == title && b.is_latest) {
                        eprintln!("[DEBUG] Found Latest build to update: {}", title);
                        // Find the actual latest release (first non-latest build)
                        if let Some(latest_release) = runner.builds.iter().find(|b| !b.is_latest) {
                            eprintln!("[DEBUG] Updating {} Latest to release: {} from {}", title, latest_release.title, latest_release.download_url);
                            self.downloading_build = Some(title.clone());
                            iced::Command::perform(download_proton_build(runner_title, latest_release.title.clone(), latest_release.download_url.clone()), Message::ProtonBuildDownloaded)
                        } else {
                            eprintln!("[DEBUG] No non-latest release found for {}", runner_title);
                            iced::Command::none()
                        }
                    } else {
                        eprintln!("[DEBUG] Build {} not found or not a Latest build", title);
                        iced::Command::none()
                    }
                } else {
                    eprintln!("[DEBUG] Runner {} not found", runner_title);
                    iced::Command::none()
                }
            }
            Message::ProtonBuildUpdated(result) => {
                self.install_progress = 1.0;
                match result {
                    Ok((runner_title, title)) => {
                        eprintln!("[DEBUG] Update complete for {} {}", runner_title, title);
                        self.downloading_build = None;
                        self.installing_build = None;
                        // Update installation status
                        self.update_proton_installation_status();
                        self.show_progress_dialog = false;
                        self.show_completion_dialog = true;
                        self.completion_message = format!("Successfully updated {} {}", runner_title, title);
                        self.completion_success = true;
                        iced::Command::none()
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Update failed: {}", e);
                        self.downloading_build = None;
                        self.installing_build = None;
                        self.show_progress_dialog = false;
                        self.show_completion_dialog = true;
                        self.completion_message = format!("Update failed: {}", e);
                        self.completion_success = false;
                        self.proton_builds_error = Some(e);
                        iced::Command::none()
                    }
                }
            }
            Message::UpdateAllProtonBuilds => {
                eprintln!("[DEBUG] UpdateAllProtonBuilds requested");
                // Find all installed "Latest" builds and update them
                let mut update_commands = Vec::new();
                for runner in &self.proton_runners {
                    for build in &runner.builds {
                        if build.is_installed && build.is_latest {
                            if let Some(latest_release) = runner.builds.iter().find(|b| !b.is_latest) {
                                eprintln!("[DEBUG] Found Latest build to update: {} {} -> {}", runner.title, build.title, latest_release.title);
                                update_commands.push((runner.title.clone(), build.title.clone(), latest_release.download_url.clone()));
                            } else {
                                eprintln!("[DEBUG] No release found for Latest build: {} {}", runner.title, build.title);
                            }
                        }
                    }
                }
                if !update_commands.is_empty() {
                    eprintln!("[DEBUG] Updating {} Latest builds", update_commands.len());
                    for (runner, title, url) in &update_commands {
                        eprintln!("[DEBUG]   - {} {} from {}", runner, title, url);
                    }
                    // Start first update
                    let (runner_title, title, download_url) = update_commands[0].clone();
                    eprintln!("[DEBUG] Starting update for: {} {}", runner_title, title);
                    self.downloading_build = Some(title.clone());
                    iced::Command::perform(download_proton_build(runner_title, title, download_url), Message::ProtonBuildDownloaded)
                } else {
                    eprintln!("[DEBUG] No Latest builds found to update");
                    iced::Command::none()
                }
            }
            Message::OpenProtonBuildDirectory(runner_title, title) => {
                // Open the installation directory in file manager
                let launcher = self.selected_launcher.clone();
                let runners = self.proton_runners.clone();
                let launchers = self.detected_launchers.clone();
                iced::Command::perform(open_proton_directory(runner_title, title, launcher, runners, launchers), |_| Message::LoadProtonBuilds)
            }
            Message::ShowProtonBuildInfo(runner_title, title, description, page_url) => {
                // Launch separate changelog window as a separate process
                let exe_path = std::env::current_exe()
                    .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                
                let runner_title_clone = runner_title.clone();
                let title_clone = title.clone();
                let description_clone = description.clone();
                let page_url_clone = page_url.clone();
                
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("proton-changelog-dialog");
                        cmd.arg(&runner_title_clone);
                        cmd.arg(&title_clone);
                        cmd.arg(&description_clone);
                        cmd.arg(&page_url_clone);
                        let _ = cmd.spawn();
                    },
                    |_| Message::GamingMetaComplete(Ok("Dialog opened".to_string())),
                )
            }
            Message::LoadMoreProtonBuilds(runner_title) => {
                if let Some(runner) = self.proton_runners.iter().find(|r| r.title == runner_title) {
                    let current_count = runner.builds.iter().filter(|b| !b.is_latest).count();
                    let page = (current_count / 25) + 1;
                    iced::Command::perform(load_more_proton_builds(runner_title, runner.endpoint.clone(), runner.asset_position, page, runner.directory_name_formats.clone()), Message::MoreProtonBuildsLoaded)
                } else {
                    iced::Command::none()
                }
            }
            Message::MoreProtonBuildsLoaded(result) => {
                match result {
                    Ok((runner_title, new_builds)) => {
                        if let Some(runner) = self.proton_runners.iter_mut().find(|r| r.title == runner_title) {
                            // Insert new builds after the "Latest" build (if exists) but before other builds
                            let latest_count = runner.builds.iter().filter(|b| b.is_latest).count();
                            for build in new_builds {
                                runner.builds.insert(latest_count, build);
                            }
                            eprintln!("[DEBUG] Loaded more builds for {}, total: {}", runner_title, runner.builds.len());
                        }
                        iced::Command::none()
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to load more builds: {}", e);
                        iced::Command::none()
                    }
                }
            }
            Message::LoadSteamGames => {
                self.is_loading_steam_games = true;
                iced::Command::perform(load_steam_games(), Message::SteamGamesLoaded)
            }
            Message::SteamGamesLoaded(result) => {
                self.is_loading_steam_games = false;
                match result {
                    Ok(games) => {
                        self.steam_games = games;
                        self.steam_games_error = None;
                    }
                    Err(e) => {
                        self.steam_games_error = Some(e);
                    }
                }
                iced::Command::none()
            }
            Message::ChangeSteamGameCompatibilityTool(appid, compatibility_tool) => {
                let steam_dir = self.steam_directory.clone();
                iced::Command::perform(change_steam_game_compatibility_tool(appid, compatibility_tool, steam_dir), Message::SteamGameCompatibilityToolChanged)
            }
            Message::SteamGameCompatibilityToolChanged(result) => {
                match result {
                    Ok((appid, compatibility_tool)) => {
                        // Update the game's compatibility tool in the list
                        if let Some(game) = self.steam_games.iter_mut().find(|g| g.appid == appid) {
                            game.compatibility_tool = compatibility_tool;
                        }
                        iced::Command::none()
                    }
                    Err(e) => {
                        self.steam_games_error = Some(e);
                        iced::Command::none()
                    }
                }
            }
        }
    }

    fn update_proton_installation_status(&mut self) {
        eprintln!("[DEBUG] ========== update_proton_installation_status START ==========");
        eprintln!("[DEBUG] Total runners: {}", self.proton_runners.len());
        eprintln!("[DEBUG] Detected launchers: {}", self.detected_launchers.len());
        for launcher in &self.detected_launchers {
            eprintln!("[DEBUG]   - {} at {} ({})", launcher.title, launcher.directory, launcher.installation_type);
        }
        
        // Update installation status for all builds based on detected launchers
        for runner in &mut self.proton_runners {
            eprintln!("[DEBUG] Checking runner: {} (type: {})", runner.title, runner.compat_layer_type);
            eprintln!("[DEBUG]   Has {} builds", runner.builds.len());
            eprintln!("[DEBUG]   Has {} directory_name_formats", runner.directory_name_formats.len());
            for format in &runner.directory_name_formats {
                eprintln!("[DEBUG]     Format for {}: {}", format.launcher, format.directory_name_format);
            }
            
            for build in &mut runner.builds {
                let was_installed = build.is_installed;
                build.is_installed = check_proton_installed(
                    &runner.title,
                    &build.title,
                    &runner.directory_name_formats,
                    &self.detected_launchers,
                    &runner.compat_layer_type,
                );
                if was_installed != build.is_installed {
                    eprintln!("[DEBUG]   Build {} {}: {} -> {}", 
                        runner.title, build.title, 
                        if was_installed { "installed" } else { "not installed" },
                        if build.is_installed { "installed" } else { "not installed" });
                } else {
                    eprintln!("[DEBUG]   Build {} {}: {} (unchanged)", 
                        runner.title, build.title,
                        if build.is_installed { "installed" } else { "not installed" });
                }
            }
        }
        eprintln!("[DEBUG] ========== update_proton_installation_status END ==========");
    }

    pub fn view(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        // Calculate font sizes from settings - larger for better readability
        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.2).round();
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();
        
        // Header section removed - sub-tabs moved up

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

        // DNF Configuration section - larger fonts
        let input_font_size = (settings.font_size_inputs * settings.scale_inputs * 1.15).round();
        
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
        .size(body_font_size * 1.05)
        .spacing(10);
        
        let load_dnf_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size * 0.95),
                text(" Load Current Config").size(button_font_size)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .on_press(Message::LoadDnfConfig)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));
        
        let save_dnf_button = button(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                text(" Save DNF Config").size(button_font_size)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .on_press(Message::SaveDnfConfig)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));
        
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
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(10.0)),
                text("Configure DNF to speed up package downloads")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(6.0)),
                text("Note: Saving changes requires administrator privileges (sudo)")
                    .size(body_font_size * 0.9)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.7, 0.1))),
                Space::with_height(Length::Fixed(20.0)),
                // Current config display
                current_config_display,
                Space::with_height(Length::Fixed(20.0)),
                text("Edit Configuration:")
                    .size(body_font_size * 1.1)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                text("Max Parallel Downloads (1-25):")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(10.0)),
                parallel_downloads_input,
                Space::with_height(Length::Fixed(10.0)),
                text("Allows DNF to download multiple packages simultaneously. Recommended: 20")
                    .size(body_font_size * 0.9)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(18.0)),
                fastest_mirror_checkbox,
                Space::with_height(Length::Fixed(10.0)),
                text("Automatically selects the fastest mirror for your location")
                    .size(body_font_size * 0.9)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(24.0)),
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
            .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
            radius: settings.border_radius,
        })));

        // Sub-tabs for Gaming Meta, DNF Config, and Cachyos Kernel - larger and better spaced
        let tab_font_size = (settings.font_size_tabs * settings.scale_tabs * 1.15).round();
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
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
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
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
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
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                button(
                    text("Hyprland")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == TweaksView::Hyprland {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == TweaksView::Hyprland,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(TweaksView::Hyprland))
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                button(
                    text("Proton")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == TweaksView::Proton {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == TweaksView::Proton,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(TweaksView::Proton))
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
            ]
            .spacing(10)
        )
        .width(Length::Fill)
        .padding(Padding::from([0.0, 20.0, 16.0, 20.0]));

        // Cachyos Kernel status display
        let cachyos_kernel_status_display: Element<Message> = if self.is_checking_cachyos_kernel {
            container(
                column![
                    text("Installation Status")
                        .size(title_font_size * 0.85)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Checking installation status...")
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        } else if let Some(ref status) = self.cachyos_kernel_status {
            // Packages section
            let packages_section = container(
                column![
                    text("Installation Status")
                        .size(title_font_size * 0.85)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Packages")
                        .size(body_font_size * 1.05)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        container(
                            text(if status.kernel_cachyos { "✓ kernel-cachyos" } else { "✗ kernel-cachyos" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.kernel_cachyos {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.kernel_cachyos,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.cachyos_settings { "✓ cachyos-settings" } else { "✗ cachyos-settings" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.cachyos_settings {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.cachyos_settings,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.ananicy_cpp { "✓ ananicy-cpp" } else { "✗ ananicy-cpp" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.ananicy_cpp {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.ananicy_cpp,
                            radius: settings.border_radius * 0.5,
                        }))),
                    ]
                    .spacing(0),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        container(
                            text(if status.cachyos_ananicy_rules { "✓ cachyos-ananicy-rules" } else { "✗ cachyos-ananicy-rules" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.cachyos_ananicy_rules {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.cachyos_ananicy_rules,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.scx_manager { "✓ scx-manager" } else { "✗ scx-manager" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.scx_manager {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.scx_manager,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.scx_scheds_git { "✓ scx-scheds-git" } else { "✗ scx-scheds-git" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.scx_scheds_git {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.scx_scheds_git,
                            radius: settings.border_radius * 0.5,
                        }))),
                    ]
                    .spacing(0),
                    Space::with_height(Length::Fixed(8.0)),
                    container(
                        text(if status.scx_tools { "✓ scx-tools" } else { "✗ scx-tools" })
                            .size(body_font_size * 0.9)
                            .style(iced::theme::Text::Color(if status.scx_tools {
                                iced::Color::from_rgb(0.1, 0.7, 0.1)
                            } else {
                                iced::Color::from_rgb(0.6, 0.6, 0.6)
                            }))
                    )
                    .width(Length::Fill)
                    .padding(Padding::from([6.0, 12.0, 6.0, 12.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                        is_installed: status.scx_tools,
                        radius: settings.border_radius * 0.5,
                    }))),
                ]
                .spacing(0)
            )
            .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })));

            // Repositories section
            let repos_section = container(
                column![
                    text("Repositories")
                        .size(body_font_size * 1.05)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        container(
                            text(if status.repo_kernel_cachyos { "✓ kernel-cachyos" } else { "✗ kernel-cachyos" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.repo_kernel_cachyos {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.repo_kernel_cachyos,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.repo_kernel_cachyos_addons { "✓ kernel-cachyos-addons" } else { "✗ kernel-cachyos-addons" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.repo_kernel_cachyos_addons {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.repo_kernel_cachyos_addons,
                            radius: settings.border_radius * 0.5,
                        }))),
                    ]
                    .spacing(0),
                ]
                .spacing(0)
            )
            .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })));

            container(
                column![
                    packages_section,
                    Space::with_height(Length::Fixed(20.0)),
                    repos_section,
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([0.0, 0.0, 0.0, 0.0]))
            .into()
        } else {
            container(
                column![
                    text("Installation Status")
                        .size(title_font_size * 0.85)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Click 'Check Status' to see installed packages")
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        };

        let check_cachyos_status_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size * 0.95),
                text(" Check Status").size(button_font_size)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .on_press(Message::CheckCachyosKernelStatus)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));

        // Cachyos Kernel button
        let cachyos_kernel_button = button(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                text(" Install Cachyos Kernel").size(button_font_size)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .on_press(Message::InstallCachyosKernel)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));

        // Cachyos Kernel left side (info and buttons) - redesigned
        let cachyos_kernel_left = container(
            column![
                text("Cachyos Kernel")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(12.0)),
                text("Installs Cachyos kernel with scheduler extensions:")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(10.0)),
                column![
                    text("• kernel-cachyos + cachyos-settings")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                    text("• ananicy-cpp, cachyos-ananicy-rules")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                    text("• scx-manager, scx-scheds-git, scx-tools")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                    text("• Auto-configures GRUB and regenerates initramfs")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                ]
                .spacing(8),
                Space::with_height(Length::Fixed(20.0)),
                row![
                    cachyos_kernel_button,
                    Space::with_width(Length::Fixed(12.0)),
                    check_cachyos_status_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
            ]
            .spacing(0)
            .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
            radius: settings.border_radius,
        })));

        // Gaming Meta status display
        let gaming_meta_status_display: Element<Message> = if self.is_checking_gaming_meta {
            container(
                column![
                    text("Installation Status")
                        .size(title_font_size * 0.85)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Checking installation status...")
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        } else if let Some(ref status) = self.gaming_meta_status {
            container(
                column![
                    text("Installation Status")
                        .size(title_font_size * 0.85)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Packages")
                        .size(body_font_size * 1.05)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        container(
                            text(if status.steam { "✓ Steam" } else { "✗ Steam" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.steam {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.steam,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.lutris { "✓ Lutris" } else { "✗ Lutris" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.lutris {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.lutris,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.mangohud { "✓ MangoHUD" } else { "✗ MangoHUD" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.mangohud {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.mangohud,
                            radius: settings.border_radius * 0.5,
                        }))),
                    ]
                    .spacing(0),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        container(
                            text(if status.gamescope { "✓ Gamescope" } else { "✗ Gamescope" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.gamescope {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.gamescope,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.mangojuice { "✓ MangoJuice" } else { "✗ MangoJuice" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.mangojuice {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.mangojuice,
                            radius: settings.border_radius * 0.5,
                        }))),
                        Space::with_width(Length::Fixed(8.0)),
                        container(
                            text(if status.protonplus { "✓ ProtonPlus" } else { "✗ ProtonPlus" })
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(if status.protonplus {
                                    iced::Color::from_rgb(0.1, 0.7, 0.1)
                                } else {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                }))
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: status.protonplus,
                            radius: settings.border_radius * 0.5,
                        }))),
                    ]
                    .spacing(0),
                    Space::with_height(Length::Fixed(8.0)),
                    container(
                        text(if status.heroic { "✓ Heroic Games Launcher" } else { "✗ Heroic Games Launcher" })
                            .size(body_font_size * 0.9)
                            .style(iced::theme::Text::Color(if status.heroic {
                                iced::Color::from_rgb(0.1, 0.7, 0.1)
                            } else {
                                iced::Color::from_rgb(0.6, 0.6, 0.6)
                            }))
                    )
                    .width(Length::Fill)
                    .padding(Padding::from([6.0, 12.0, 6.0, 12.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                        is_installed: status.heroic,
                        radius: settings.border_radius * 0.5,
                    }))),
                ]
                .spacing(0)
            )
            .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        } else {
            container(
                column![
                    text("Installation Status")
                        .size(title_font_size * 0.85)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Click 'Check Status' to see installed packages")
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        };

        let check_status_button = button(
            row![
                text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size * 0.95),
                text(" Check Status").size(button_font_size)
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .on_press(Message::CheckGamingMetaStatus)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));

        // Gaming Meta left side (info and buttons) - redesigned for better space usage
        let gaming_meta_left = container(
            column![
                text("Gaming Meta")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(16.0)),
                text("Installs a complete gaming setup including:")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(14.0)),
                column![
                    text("• Steam, Lutris, MangoHUD, Gamescope")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                    Space::with_height(Length::Fixed(8.0)),
                    text("• ProtonPlus, MangoJuice (Flatpak)")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                    Space::with_height(Length::Fixed(8.0)),
                    text("• Heroic Games Launcher (latest release)")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text())),
                ]
                .spacing(0),
                Space::with_height(Length::Fixed(32.0)),
                row![
                    button(
                        row![
                            text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                            text(" Install Gaming Meta").size(button_font_size)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::InstallGamingMeta)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                    Space::with_width(Length::Fixed(12.0)),
                    check_status_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
            ]
            .spacing(0)
            .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
            radius: settings.border_radius,
        })));

        // Content based on current view
        let content: Element<Message> = match self.current_view {
            TweaksView::GamingMeta => {
                // Only show output_log if it has content
                let left_column: Element<Message> = if self.output_log.is_empty() {
                    gaming_meta_left.into()
                } else {
                    column![
                        gaming_meta_left,
                        Space::with_height(Length::Fixed(12.0)),
                        output_log,
                    ]
                    .spacing(0)
                    .into()
                };
                row![
                    container(left_column)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill),
                    Space::with_width(Length::Fixed(16.0)),
                    container(gaming_meta_status_display)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .padding(Padding::from([0.0, 0.0, 0.0, 0.0])),
                ]
                .spacing(0)
                .align_items(Alignment::Start)
                .into()
            }
            TweaksView::DnfConfig => {
                dnf_config_info.into()
            }
            TweaksView::CachyosKernel => {
                row![
                    cachyos_kernel_left
                        .width(Length::FillPortion(1)),
                    Space::with_width(Length::Fixed(16.0)),
                    container(cachyos_kernel_status_display)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .padding(Padding::from([0.0, 0.0, 0.0, 0.0])),
                ]
                .spacing(0)
                .align_items(Alignment::Start)
                .into()
            }
            TweaksView::Hyprland => {
                // Hyprland status display
                let hyprland_status_display: Element<Message> = if self.is_checking_hyprland {
                    container(
                        column![
                            text("Installation Status")
                                .size(title_font_size * 0.65)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_height(Length::Fixed(20.0)),
                            text("Checking installation status...")
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    })))
                    .into()
                } else if let Some(ref status) = self.hyprland_status {
                    // Packages section
                    let packages_section = container(
                        column![
                            text("Installation Status")
                                .size(title_font_size * 0.65)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_height(Length::Fixed(20.0)),
                            text("Packages")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            Space::with_height(Length::Fixed(12.0)),
                            row![
                                container(
                                    text(if status.hyprland { "✓ hyprland" } else { "✗ hyprland" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.hyprland {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.hyprland,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.hyprpicker { "✓ hyprpicker" } else { "✗ hyprpicker" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.hyprpicker {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.hyprpicker,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.swww { "✓ swww" } else { "✗ swww" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.swww {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.swww,
                                    radius: settings.border_radius * 0.5,
                                }))),
                            ]
                            .spacing(0),
                            Space::with_height(Length::Fixed(8.0)),
                            row![
                                container(
                                    text(if status.quickshell_git { "✓ quickshell-git" } else { "✗ quickshell-git" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.quickshell_git {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.quickshell_git,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.fuzzel { "✓ fuzzel" } else { "✗ fuzzel" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.fuzzel {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.fuzzel,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.wlogout { "✓ wlogout" } else { "✗ wlogout" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.wlogout {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.wlogout,
                                    radius: settings.border_radius * 0.5,
                                }))),
                            ]
                            .spacing(0),
                            Space::with_height(Length::Fixed(8.0)),
                            row![
                                container(
                                    text(if status.cliphist { "✓ cliphist" } else { "✗ cliphist" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.cliphist {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.cliphist,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.brightnessctl { "✓ brightnessctl" } else { "✗ brightnessctl" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.brightnessctl {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.brightnessctl,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.grim { "✓ grim" } else { "✗ grim" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.grim {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.grim,
                                    radius: settings.border_radius * 0.5,
                                }))),
                            ]
                            .spacing(0),
                            Space::with_height(Length::Fixed(8.0)),
                            row![
                                container(
                                    text(if status.slurp { "✓ slurp" } else { "✗ slurp" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.slurp {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.slurp,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.swappy { "✓ swappy" } else { "✗ swappy" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.swappy {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.swappy,
                                    radius: settings.border_radius * 0.5,
                                }))),
                            ]
                            .spacing(0),
                        ]
                        .spacing(0)
                    )
                    .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    })));

                    // Repositories section - only show RPM Fusion and quickshell
                    let repos_section = container(
                        column![
                            text("Repositories")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            Space::with_height(Length::Fixed(12.0)),
                            row![
                                container(
                                    text(if status.repo_rpmfusion_free { "✓ RPM Fusion Free" } else { "✗ RPM Fusion Free" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.repo_rpmfusion_free {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.repo_rpmfusion_free,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.repo_rpmfusion_nonfree { "✓ RPM Fusion Nonfree" } else { "✗ RPM Fusion Nonfree" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.repo_rpmfusion_nonfree {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.repo_rpmfusion_nonfree,
                                    radius: settings.border_radius * 0.5,
                                }))),
                            ]
                            .spacing(0),
                            Space::with_height(Length::Fixed(8.0)),
                            row![
                                container(
                                    text(if status.repo_quickshell { "✓ errornointernet/quickshell" } else { "✗ errornointernet/quickshell" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.repo_quickshell {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.repo_quickshell,
                                    radius: settings.border_radius * 0.5,
                                }))),
                                Space::with_width(Length::Fixed(8.0)),
                                container(
                                    text(if status.repo_hyprland { "✓ solopasha/hyprland" } else { "✗ solopasha/hyprland" })
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(if status.repo_hyprland {
                                            iced::Color::from_rgb(0.1, 0.7, 0.1)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }))
                                )
                                .width(Length::Fill)
                                .padding(Padding::from([8.0, 14.0, 8.0, 14.0]))
                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                    is_installed: status.repo_hyprland,
                                    radius: settings.border_radius * 0.5,
                                }))),
                            ]
                            .spacing(0),
                        ]
                        .spacing(0)
                    )
                    .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    })));

                    container(
                        column![
                            packages_section,
                            Space::with_height(Length::Fixed(20.0)),
                            repos_section,
                        ]
                        .spacing(0)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([0.0, 0.0, 0.0, 0.0]))
                    .into()
                } else {
                    container(
                        column![
                            text("Installation Status")
                                .size(title_font_size * 0.65)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_height(Length::Fixed(20.0)),
                            text("Click 'Check Status' to see installed packages")
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    })))
                    .into()
                };

                let check_hyprland_status_button = button(
                    row![
                        text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size * 0.95),
                        text(" Check Status").size(button_font_size)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::CheckHyprlandStatus)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));

                // Hyprland info card
                let hyprland_install_button = button(
                    row![
                        text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                        text(" Install Hyprland & Dependencies").size(button_font_size)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::InstallHyprland)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));

                let hyprland_dotfiles_button = button(
                    row![
                        text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                        text(" Install Dotfiles").size(button_font_size)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::InstallHyprlandDotfiles)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));

                // Hyprland left side (info and buttons) - redesigned
                let hyprland_left = container(
                    column![
                        text("Hyprland Setup")
                            .size(title_font_size)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(12.0)),
                        text("Installs Hyprland window manager and dependencies:")
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_height(Length::Fixed(10.0)),
                        column![
                            text("• Enables COPR repositories (solopasha/hyprland, errornointernet/quickshell)")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.text())),
                            text("• Installs Hyprland, hyprpicker, swww, quickshell-git")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.text())),
                            text("• Installs essential utilities (fuzzel, wlogout, cliphist, etc.)")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.text())),
                        ]
                        .spacing(8),
                        Space::with_height(Length::Fixed(20.0)),
                        row![
                            hyprland_install_button,
                            Space::with_width(Length::Fixed(12.0)),
                            check_hyprland_status_button,
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center),
                        Space::with_height(Length::Fixed(20.0)),
                        text("Install Dotfiles")
                            .size(body_font_size * 1.1)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(10.0)),
                        text("Installs configuration files from Dark Material Shell:")
                            .size(body_font_size * 0.95)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_height(Length::Fixed(10.0)),
                        column![
                            text("• hypr folder → ~/.config/hypr")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.text())),
                            text("• quickshell folder → ~/.config/quickshell")
                                .size(body_font_size * 0.95)
                                .style(iced::theme::Text::Color(theme.text())),
                        ]
                        .spacing(8),
                        Space::with_height(Length::Fixed(20.0)),
                        hyprland_dotfiles_button,
                    ]
                    .spacing(0)
                    .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                    radius: settings.border_radius,
                })));

                row![
                    hyprland_left
                        .width(Length::FillPortion(1)),
                    Space::with_width(Length::Fixed(16.0)),
                    container(hyprland_status_display)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .padding(Padding::from([0.0, 0.0, 0.0, 0.0])),
                ]
                .spacing(0)
                .align_items(Alignment::Start)
                .into()
            }
            TweaksView::Proton => {
                // Sidebar with Proton runners
                let sidebar_items: Vec<Element<Message>> = self.proton_runners.iter().map(|runner| {
                    let is_selected = self.selected_proton_runner.as_ref().map(|s| s == &runner.title).unwrap_or(false);
                    button(
                        text(&runner.title)
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(if is_selected {
                                iced::Color::WHITE
                            } else {
                                theme.text()
                            }))
                    )
                    .on_press(Message::SelectProtonRunner(runner.title.clone()))
                    .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                        is_active: is_selected,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::from([12.0, 16.0, 12.0, 16.0]))
                    .width(Length::Fill)
                    .into()
                }).collect();
                
                let sidebar: Element<Message> = if self.proton_runners.is_empty() {
                    container(
                        text("No runners available")
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([16.0, 12.0, 16.0, 12.0]))
                    .style(iced::theme::Container::Custom(Box::new(SidebarStyle {
                        radius: settings.border_radius,
                    })))
                    .into()
                } else {
                    container(
                        scrollable(
                            container(column(sidebar_items).spacing(8))
                                .width(Length::Fill)
                                .padding(Padding::from([16.0, 12.0, 16.0, 23.0]))
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
                    .padding(Padding::from([0.0, 0.0, 0.0, 0.0]))
                    .style(iced::theme::Container::Custom(Box::new(SidebarStyle {
                        radius: settings.border_radius,
                    })))
                    .into()
                };
                
                // Main content area
                let proton_content: Element<Message> = if self.is_loading_proton_builds {
                    container(
                        column![
                            text("Loading Proton Builds...")
                                .size(title_font_size * 0.8)
                                .style(iced::theme::Text::Color(theme.primary())),
                            Space::with_height(Length::Fixed(20.0)),
                            text("Fetching available builds from GitHub...")
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    })))
                    .into()
                } else if let Some(ref error) = self.proton_builds_error {
                    container(
                        column![
                            text("Error")
                                .size(title_font_size * 0.8)
                                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
                            Space::with_height(Length::Fixed(12.0)),
                            text(error)
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            Space::with_height(Length::Fixed(20.0)),
                            button(
                                text("Retry")
                                    .size(button_font_size)
                            )
                            .on_press(Message::LoadProtonBuilds)
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
                    .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                        radius: settings.border_radius,
                        theme: *theme,
                    })))
                    .into()
                } else if let Some(selected_runner_title) = &self.selected_proton_runner {
                    if let Some(runner) = self.proton_runners.iter().find(|r| r.title == *selected_runner_title) {
                        // Launcher selector buttons
                        let launcher_buttons: Vec<Element<Message>> = self.detected_launchers.iter().map(|launcher| {
                            let is_selected = self.selected_launcher.as_ref().map(|s| s == &launcher.title).unwrap_or(false);
                            button(
                                text(format!("{} ({})", launcher.title, launcher.installation_type))
                                    .size(body_font_size * 0.9)
                                    .style(iced::theme::Text::Color(if is_selected {
                                        iced::Color::WHITE
                                    } else {
                                        theme.text()
                                    }))
                            )
                            .on_press(Message::SelectLauncher(launcher.title.clone()))
                            .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                                is_active: is_selected,
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::from([10.0, 14.0, 10.0, 14.0]))
                            .into()
                        }).collect();
                        
                        // Filter builds based on selected filters
                        let filtered_builds: Vec<&ProtonBuild> = runner.builds.iter()
                            .filter(|build| {
                                if self.show_installed_only && !build.is_installed {
                                    return false;
                                }
                                if self.show_used_only && build.usage_count == 0 {
                                    return false;
                                }
                                if self.show_unused_only && build.usage_count > 0 {
                                    return false;
                                }
                                true
                            })
                            .collect();
                        
                        let builds_list: Vec<Element<Message>> = filtered_builds.iter().map(|build| {
                        let size_mb = build.download_size as f64 / 1_048_576.0;
                        let is_downloading = self.downloading_build.as_ref().map(|t| t == &build.title).unwrap_or(false);
                        let is_installing = self.installing_build.as_ref().map(|t| t == &build.title).unwrap_or(false);
                        
                        let action_button: Element<Message> = if build.is_installed {
                            {
                                let mut buttons = Vec::new();
                                // Only show Update button if there's actually an update available
                                if build.is_latest && build.is_installed && has_proton_update(runner, build, &self.detected_launchers) {
                                    buttons.push(
                                        button(
                                            row![
                                                text(crate::gui::fonts::glyphs::SYNC_SYMBOL).font(material_font).size(icon_size * 0.7),
                                                text(" Update").size(button_font_size * 0.85)
                                            ]
                                            .spacing(6)
                                            .align_items(Alignment::Center)
                                        )
                                        .on_press(Message::UpdateProtonBuild(runner.title.clone(), build.title.clone()))
                                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                            is_primary: true,
                                            radius: settings.border_radius,
                                        })))
                                        .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
                                        .into()
                                    );
                                }
                                buttons.push(
                                    button(
                                        row![
                                            text(crate::gui::fonts::glyphs::FOLDER_SYMBOL).font(material_font).size(icon_size * 0.7),
                                            text(" Open").size(button_font_size * 0.85)
                                        ]
                                        .spacing(6)
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::OpenProtonBuildDirectory(runner.title.clone(), build.title.clone()))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: settings.border_radius,
                                    })))
                                    .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
                                    .into()
                                );
                                buttons.push(
                                    button(
                                        row![
                                            text(crate::gui::fonts::glyphs::INFO_SYMBOL).font(material_font).size(icon_size * 0.7),
                                        ]
                                        .spacing(6)
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::ShowProtonBuildInfo(
                                        runner.title.clone(),
                                        build.title.clone(),
                                        build.description.clone(),
                                        build.page_url.clone()
                                    ))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: settings.border_radius,
                                    })))
                                    .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
                                    .into()
                                );
                                buttons.push(
                                    button(
                                        row![
                                            text(crate::gui::fonts::glyphs::DELETE_SYMBOL).font(material_font).size(icon_size * 0.7),
                                            text(" Remove").size(button_font_size * 0.85)
                                        ]
                                        .spacing(6)
                                        .align_items(Alignment::Center)
                                    )
                                    .on_press(Message::RemoveProtonBuild(runner.title.clone(), build.title.clone()))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: settings.border_radius,
                                    })))
                                    .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
                                    .into()
                                );
                                row(buttons)
                                    .spacing(8)
                                    .align_items(Alignment::Center)
                                    .into()
                            }
                        } else if is_downloading {
                            button(
                                text("Downloading...")
                                    .size(button_font_size * 0.9)
                            )
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: false,
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::from([10.0, 16.0, 10.0, 16.0]))
                            .into()
                        } else if is_installing {
                            button(
                                text("Installing...")
                                    .size(button_font_size * 0.9)
                            )
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: false,
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::from([10.0, 16.0, 10.0, 16.0]))
                            .into()
                        } else {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size * 0.8),
                                    text(" Install").size(button_font_size * 0.9)
                                ]
                                .spacing(8)
                                .align_items(Alignment::Center)
                            )
                            .on_press(Message::DownloadProtonBuild(runner.title.clone(), build.title.clone(), build.download_url.clone()))
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::from([10.0, 16.0, 10.0, 16.0]))
                            .into()
                        };
                        
                        let title_text = if build.usage_count > 0 {
                            format!("{} (Used by {} game{})", build.title, build.usage_count, if build.usage_count > 1 { "s" } else { "" })
                        } else {
                            build.title.clone()
                        };
                        
                        container(
                            column![
                                row![
                                    column![
                                        text(&title_text)
                                            .size(body_font_size)
                                            .style(iced::theme::Text::Color(theme.text())),
                                        Space::with_height(Length::Fixed(4.0)),
                                        {
                                            let mut size_row = row![
                                                text(&format!("{:.2} MB", size_mb))
                                                    .size(body_font_size * 0.9)
                                                    .style(iced::theme::Text::Color(theme.secondary_text())),
                                            ];
                                            if build.is_latest {
                                                size_row = size_row.push(Space::with_width(Length::Fixed(8.0)));
                                                size_row = size_row.push(
                                                    container(
                                                        text("Latest")
                                                            .size(body_font_size * 0.85)
                                                    )
                                                    .padding(Padding::from([4.0, 8.0, 4.0, 8.0]))
                                                    .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                                        is_installed: true,
                                                        radius: settings.border_radius * 0.3,
                                                    })))
                                                );
                                            }
                                            size_row
                                        }
                                        .spacing(0)
                                        .align_items(Alignment::Center),
                                    ]
                                    .spacing(0)
                                    .width(Length::Fill),
                                    action_button,
                                ]
                                .spacing(12)
                                .align_items(Alignment::Center),
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .padding(Padding::from([16.0, 20.0, 16.0, 20.0]))
                        .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                            is_installed: build.is_installed,
                            radius: settings.border_radius * 0.5,
                        })))
                        .into()
                    }).collect();
                    
                        container(
                            column![
                                Space::with_height(Length::Fixed(8.0)),
                                {
                                    let launcher_selector: Element<Message> = if launcher_buttons.is_empty() {
                                        container(
                                            text("No launchers detected")
                                                .size(body_font_size * 0.9)
                                                .style(iced::theme::Text::Color(theme.secondary_text()))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                                            radius: settings.border_radius,
                                        })))
                                        .into()
                                    } else {
                                        row(launcher_buttons)
                                            .spacing(8)
                                            .into()
                                    };
                                    launcher_selector
                                },
                                Space::with_height(Length::Fixed(20.0)),
                                // Filter buttons
                                row![
                                    checkbox(
                                        "Installed",
                                        self.show_installed_only
                                    )
                                    .on_toggle(|_| Message::ToggleFilterInstalled)
                                    .text_size(body_font_size * 0.9),
                                    Space::with_width(Length::Fixed(16.0)),
                                    checkbox(
                                        "Used",
                                        self.show_used_only
                                    )
                                    .on_toggle(|_| Message::ToggleFilterUsed)
                                    .text_size(body_font_size * 0.9),
                                    Space::with_width(Length::Fixed(16.0)),
                                    checkbox(
                                        "Unused",
                                        self.show_unused_only
                                    )
                                    .on_toggle(|_| Message::ToggleFilterUnused)
                                    .text_size(body_font_size * 0.9),
                                ]
                                .spacing(0)
                                .align_items(Alignment::Center),
                                Space::with_height(Length::Fixed(20.0)),
                                row![
                                    button(
                                        text("Refresh")
                                            .size(button_font_size)
                                    )
                                    .on_press(Message::LoadProtonBuilds)
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: settings.border_radius,
                                    })))
                                    .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                                ]
                                .spacing(0),
                                Space::with_height(Length::Fixed(20.0)),
                                {
                                    let builds_content: Element<Message> = if builds_list.is_empty() {
                                        container(
                                            text("No builds available")
                                                .size(body_font_size)
                                                .style(iced::theme::Text::Color(theme.secondary_text()))
                                        )
                                        .width(Length::Fill)
                                        .height(Length::Fill)
                                        .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
                                        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                                            radius: settings.border_radius,
                                        })))
                                        .into()
                                    } else {
                                        let mut build_items = builds_list;
                                        // Add "Load More" button if we have 25 builds (indicating there might be more)
                                        if filtered_builds.len() >= 25 {
                                            build_items.push(
                                                container(
                                                    button(
                                                        row![
                                                            text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size * 0.8),
                                                            text(" Load More").size(button_font_size * 0.9)
                                                        ]
                                                        .spacing(8)
                                                        .align_items(Alignment::Center)
                                                    )
                                                    .on_press(Message::LoadMoreProtonBuilds(runner.title.clone()))
                                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                        is_primary: false,
                                                        radius: settings.border_radius,
                                                    })))
                                                    .padding(Padding::from([14.0, 20.0, 14.0, 20.0]))
                                                    .width(Length::Fill)
                                                )
                                                .width(Length::Fill)
                                                .padding(Padding::from([16.0, 20.0, 16.0, 20.0]))
                                                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                                                    is_installed: false,
                                                    radius: settings.border_radius * 0.5,
                                                })))
                                                .into()
                                            );
                                        }
                                        container(
                                            scrollable(
                                                column(build_items)
                                                    .spacing(12)
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
                                        .padding(Padding::from([0.0, 0.0, 0.0, 0.0]))
                                        .into()
                                    };
                                    builds_content
                                },
                            ]
                            .spacing(0)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
                        .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                            radius: settings.border_radius,
                        })))
                        .into()
                    } else {
                        container(
                            text("Select a runner from the sidebar")
                                .size(body_font_size)
                                .style(iced::theme::Text::Color(theme.secondary_text()))
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
                        .into()
                    }
                } else {
                    container(
                        text("Select a runner from the sidebar")
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
                    .into()
                };
                
                // Layout with sidebar and content
                container(
                    row![
                        container(sidebar)
                            .width(Length::Fixed(280.0))
                            .height(Length::Fill),
                        container(proton_content)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            TweaksView::SteamGames => {
                self.view_steam_games(theme, settings)
            }
        };

        let main_content = container(
            column![
                sub_tabs,
                Space::with_height(Length::Fixed(16.0)),
                content,
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(32.0));
        
        // Overlay progress dialog if active
        let final_content = if self.show_progress_dialog {
            container(
                column![
                    main_content,
                    self.view_progress_dialog(theme, settings),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.show_completion_dialog {
            container(
                column![
                    main_content,
                    self.view_completion_dialog(theme, settings),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            main_content.into()
        };
        
        final_content
    }
    
    fn view_progress_dialog(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let _button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        
        // Calculate overall progress (download is 0-0.5, install is 0.5-1.0)
        // For download, show a pulsing progress if we don't have exact progress
        let overall_progress = if self.downloading_build.is_some() {
            if self.download_progress > 0.0 {
                self.download_progress * 0.5
            } else {
                // Show pulsing animation (will update as download progresses)
                // We'll estimate based on time or use a simple animation
                0.1
            }
        } else if self.installing_build.is_some() {
            if self.install_progress > 0.0 {
                0.5 + (self.install_progress * 0.5)
            } else {
                // Starting install
                0.5
            }
        } else {
            0.0
        };
        
        // Background overlay
        let overlay = container(
            container(
                column![
                    text(&self.progress_text)
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.text()))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fixed(16.0)),
                    progress_bar(0.0..=1.0, overall_progress)
                        .width(Length::Fixed(400.0))
                        .height(Length::Fixed(8.0)),
                    Space::with_height(Length::Fixed(8.0)),
                    // Show download progress if downloading
                    if self.downloading_build.is_some() {
                        let download_text: Element<Message> = text(format!("Downloading: {:.1}%", self.download_progress * 100.0))
                            .size(body_font_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                            .into();
                        let download_bar: Element<Message> = progress_bar(0.0..=1.0, self.download_progress)
                            .width(Length::Fixed(400.0))
                            .height(Length::Fixed(6.0))
                            .into();
                        column![
                            download_text,
                            download_bar,
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                    } else {
                        let empty: Element<Message> = Space::with_height(Length::Fixed(0.0)).into();
                        empty
                    },
                    // Show install progress if installing
                    if self.installing_build.is_some() {
                        let install_text: Element<Message> = text(format!("Installing: {:.1}%", self.install_progress * 100.0))
                            .size(body_font_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                            .into();
                        let install_bar: Element<Message> = progress_bar(0.0..=1.0, self.install_progress)
                            .width(Length::Fixed(400.0))
                            .height(Length::Fixed(6.0))
                            .into();
                        column![
                            Space::with_height(Length::Fixed(8.0)),
                            install_text,
                            install_bar,
                        ]
                        .spacing(4)
                        .align_items(Alignment::Center)
                        .into()
                    } else {
                        let empty: Element<Message> = Space::with_height(Length::Fixed(0.0)).into();
                        empty
                    },
                ]
                .spacing(0)
                .align_items(Alignment::Center)
                .padding(Padding::from([24.0, 32.0, 24.0, 32.0]))
            )
            .width(Length::Fixed(500.0))
            .style(iced::theme::Container::Custom(Box::new(ProgressDialogStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(OverlayStyle {
            background: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7),
        })));
        
        overlay.into()
    }
    
    fn view_completion_dialog(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        
        let close_button = button(
            text("Close")
                .size(button_font_size)
                .style(iced::theme::Text::Color(iced::Color::WHITE))
        )
        .on_press(Message::CloseCompletionDialog)
        .padding(Padding::from([12.0, 24.0, 12.0, 24.0]))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })));
        
        // Background overlay
        let overlay = container(
            container(
                column![
                    text(if self.completion_success { "Success" } else { "Error" })
                        .size(body_font_size * 1.2)
                        .style(iced::theme::Text::Color(
                            if self.completion_success {
                                iced::Color::from_rgb(0.1, 0.5, 0.1)
                            } else {
                                theme.danger()
                            }
                        ))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fixed(16.0)),
                    text(&self.completion_message)
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.text()))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fixed(24.0)),
                    close_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center)
                .padding(Padding::from([24.0, 32.0, 24.0, 32.0]))
            )
            .width(Length::Fixed(500.0))
            .style(iced::theme::Container::Custom(Box::new(ProgressDialogStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(OverlayStyle {
            background: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7),
        })));
        
        overlay.into()
    }

    fn view_steam_games(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let _material_font = crate::gui::fonts::get_material_symbols_font();
        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.2).round();
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let _icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();
        
        if self.is_loading_steam_games {
            container(
                column![
                    text("Loading Steam Games...")
                        .size(title_font_size * 0.8)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(20.0)),
                    text("Reading Steam configuration...")
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        } else if let Some(ref error) = self.steam_games_error {
            container(
                column![
                    text("Error")
                        .size(title_font_size * 0.8)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
                    Space::with_height(Length::Fixed(12.0)),
                    text(error)
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(20.0)),
                    button(
                        text("Retry")
                            .size(button_font_size)
                    )
                    .on_press(Message::LoadSteamGames)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        } else if self.steam_games.is_empty() {
            container(
                column![
                    text("No Steam Games Found")
                        .size(title_font_size * 0.8)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(12.0)),
                    text("Make sure Steam is installed and you have games in your library.")
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    Space::with_height(Length::Fixed(20.0)),
                    button(
                        text("Refresh")
                            .size(button_font_size)
                    )
                    .on_press(Message::LoadSteamGames)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })))
            .into()
        } else {
            // Get all available Proton builds for dropdown
            let mut available_tools = vec!["Undefined".to_string()];
            for runner in &self.proton_runners {
                for build in &runner.builds {
                    if build.is_installed {
                        // Format the tool name as it appears in Steam config
                        let tool_name = if build.is_latest {
                            // For "Latest" builds, use the actual release name
                            if let Some(actual_release) = runner.builds.iter().find(|b| !b.is_latest) {
                                format_directory_name_for_steam(&runner.title, &actual_release.title, &runner.directory_name_formats)
                            } else {
                                continue;
                            }
                        } else {
                            format_directory_name_for_steam(&runner.title, &build.title, &runner.directory_name_formats)
                        };
                        if !available_tools.contains(&tool_name) {
                            available_tools.push(tool_name);
                        }
                    }
                }
            }
            
            // Warning message
            let warning = container(
                text("Close the Steam client beforehand so that the changes can be applied.")
                    .size(body_font_size * 0.9)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.65, 0.0)))
            )
            .width(Length::Fill)
            .padding(Padding::from([12.0, 16.0, 12.0, 16.0]))
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })));
            
            // Table header
            let header = container(
                row![
                    text("Name")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.primary()))
                        .width(Length::FillPortion(3)),
                    text("App ID")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.primary()))
                        .width(Length::FillPortion(1)),
                    text("Compatibility Tool")
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.primary()))
                        .width(Length::FillPortion(2)),
                ]
                .spacing(16)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding(Padding::from([16.0, 20.0, 16.0, 20.0]))
            .style(iced::theme::Container::Custom(Box::new(StatusSectionStyle {
                radius: settings.border_radius,
                theme: *theme,
            })));
            
            // Game rows
            let available_tools_clone = available_tools.clone();
            let game_rows: Vec<Element<Message>> = self.steam_games.iter().map(|game| {
                let current_tool = if game.compatibility_tool == "Undefined" {
                    "Undefined".to_string()
                } else {
                    game.compatibility_tool.clone()
                };
                
                let selected_option = if available_tools_clone.contains(&current_tool) {
                    Some(current_tool.clone())
                } else {
                    Some("Undefined".to_string())
                };
                
                let appid = game.appid;
                let game_name = game.name.clone();
                let compat_tool = game.compatibility_tool.clone();
                
                container(
                    row![
                        text(&game_name)
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.text()))
                            .width(Length::FillPortion(3)),
                        text(&appid.to_string())
                            .size(body_font_size * 0.9)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
                            .width(Length::FillPortion(1)),
                        pick_list(
                            available_tools_clone.clone(),
                            selected_option,
                            move |tool| Message::ChangeSteamGameCompatibilityTool(appid, tool)
                        )
                        .width(Length::FillPortion(2))
                        .text_size(body_font_size * 0.9)
                        .padding(Padding::from([8.0, 12.0, 8.0, 12.0])),
                    ]
                    .spacing(16)
                    .align_items(Alignment::Center)
                )
                .width(Length::Fill)
                .padding(Padding::from([12.0, 20.0, 12.0, 20.0]))
                .style(iced::theme::Container::Custom(Box::new(StatusItemStyle {
                    is_installed: compat_tool != "Undefined",
                    radius: settings.border_radius * 0.5,
                })))
                .into()
            }).collect();
            
            container(
                column![
                    warning,
                    Space::with_height(Length::Fixed(16.0)),
                    header,
                    scrollable(
                        column(game_rows)
                            .spacing(8)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                        Color::from(settings.background_color.clone()),
                        settings.border_radius,
                    )))),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([24.0, 28.0, 24.0, 28.0]))
            .style(iced::theme::Container::Custom(Box::new(RoundedMessageStyle {
                radius: settings.border_radius,
            })))
            .into()
        }
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

async fn check_copr_repo_enabled(repo: &str) -> bool {
    use tokio::process::Command as TokioCommand;
    let mut cmd = TokioCommand::new("dnf");
    cmd.arg("copr");
    cmd.arg("list");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let output = cmd.output().await.ok();
    if let Some(o) = output {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // COPR repos are listed as "copr.fedorainfracloud.org/owner/repo"
            // We need to check if the repo name appears in the output
            // The format is: copr.fedorainfracloud.org/owner/repo
            // We check for both the full format and just owner/repo
            if stdout.contains(repo) {
                return true;
            }
            // Also check for the full URL format
            if stdout.contains(&format!("copr.fedorainfracloud.org/{}", repo)) {
                return true;
            }
        }
    }
    false
}

async fn check_rpmfusion_repo_enabled(repo: &str) -> bool {
    use tokio::process::Command as TokioCommand;
    let mut cmd = TokioCommand::new("dnf");
    cmd.arg("repoinfo");
    cmd.arg(repo);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let output = cmd.output().await.ok();
    if let Some(o) = output {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Check if repo status is "enabled"
            return stdout.contains("Status") && stdout.contains("enabled");
        }
    }
    false
}

async fn check_cachyos_kernel_status() -> Result<CachyosKernelStatus, String> {
    let kernel_cachyos = check_dnf_package("kernel-cachyos").await;
    let cachyos_settings = check_dnf_package("cachyos-settings").await;
    let ananicy_cpp = check_dnf_package("ananicy-cpp").await;
    let cachyos_ananicy_rules = check_dnf_package("cachyos-ananicy-rules").await;
    let scx_manager = check_dnf_package("scx-manager").await;
    let scx_scheds_git = check_dnf_package("scx-scheds-git").await;
    let scx_tools = check_dnf_package("scx-tools").await;
    let repo_kernel_cachyos = check_copr_repo_enabled("bieszczaders/kernel-cachyos").await;
    let repo_kernel_cachyos_addons = check_copr_repo_enabled("bieszczaders/kernel-cachyos-addons").await;
    
    Ok(CachyosKernelStatus {
        kernel_cachyos,
        cachyos_settings,
        ananicy_cpp,
        cachyos_ananicy_rules,
        scx_manager,
        scx_scheds_git,
        scx_tools,
        repo_kernel_cachyos,
        repo_kernel_cachyos_addons,
    })
}

async fn check_hyprland_status() -> Result<HyprlandStatus, String> {
    let hyprland = check_dnf_package("hyprland").await;
    let hyprpicker = check_dnf_package("hyprpicker").await;
    let swww = check_dnf_package("swww").await;
    let quickshell_git = check_dnf_package("quickshell-git").await;
    let fuzzel = check_dnf_package("fuzzel").await;
    let wlogout = check_dnf_package("wlogout").await;
    let cliphist = check_dnf_package("cliphist").await;
    let brightnessctl = check_dnf_package("brightnessctl").await;
    let grim = check_dnf_package("grim").await;
    let slurp = check_dnf_package("slurp").await;
    let swappy = check_dnf_package("swappy").await;
    // Hyprland needs RPM Fusion, quickshell, and hyprland repos
    let repo_rpmfusion_free = check_rpmfusion_repo_enabled("rpmfusion-free").await;
    let repo_rpmfusion_nonfree = check_rpmfusion_repo_enabled("rpmfusion-nonfree").await;
    let repo_quickshell = check_copr_repo_enabled("errornointernet/quickshell").await;
    let repo_hyprland = check_copr_repo_enabled("solopasha/hyprland").await;
    
    Ok(HyprlandStatus {
        hyprland,
        hyprpicker,
        swww,
        quickshell_git,
        fuzzel,
        wlogout,
        cliphist,
        brightnessctl,
        grim,
        slurp,
        swappy,
        repo_rpmfusion_free,
        repo_rpmfusion_nonfree,
        repo_quickshell,
        repo_hyprland,
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
struct ProgressDialogStyle {
    radius: f32,
    theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for ProgressDialogStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(iced::Color::from_rgb(0.15, 0.15, 0.15).into()),
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: self.theme.primary(),
            },
            shadow: Default::default(),
        }
    }
}

struct OverlayStyle {
    background: iced::Color,
}

impl iced::widget::container::StyleSheet for OverlayStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(self.background.into()),
            border: Border::default(),
            shadow: Default::default(),
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

struct StatusItemStyle {
    is_installed: bool,
    radius: f32,
}

impl iced::widget::container::StyleSheet for StatusItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(if self.is_installed {
                iced::Color::from_rgba(0.1, 0.7, 0.1, 0.15)
            } else {
                iced::Color::from_rgba(0.6, 0.6, 0.6, 0.1)
            })),
            border: Border {
                color: if self.is_installed {
                    iced::Color::from_rgba(0.1, 0.7, 0.1, 0.3)
                } else {
                    iced::Color::from_rgba(0.6, 0.6, 0.6, 0.2)
                },
                width: 1.0,
                radius: self.radius.into(),
            },
            ..Default::default()
        }
    }
}

struct StatusSectionStyle {
    radius: f32,
    theme: crate::gui::Theme,
}

struct SidebarStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for SidebarStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: None,
            border: Border {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0),
                width: 0.0,
                radius: self.radius.into(),
            },
            ..Default::default()
        }
    }
}

impl iced::widget::container::StyleSheet for StatusSectionStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.theme.surface())),
            border: Border {
                color: match self.theme {
                    crate::gui::Theme::Light => iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                    crate::gui::Theme::Dark => iced::Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                },
                width: 1.0,
                radius: self.radius.into(),
            },
            ..Default::default()
        }
    }
}


// Proton builds async functions
fn get_proton_cache_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let cache_dir = std::path::Path::new(&home).join(".cache").join("rustora");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    Ok(cache_dir.join("proton_builds.json"))
}

fn load_proton_cache() -> Option<Vec<ProtonRunner>> {
    let cache_path = get_proton_cache_path().ok()?;
    eprintln!("[DEBUG] Checking cache at: {}", cache_path.display());
    
    if !cache_path.exists() {
        eprintln!("[DEBUG] Cache file does not exist");
        return None;
    }
    
    // Check cache age (1 hour = 3600 seconds)
    let cache_age = cache_path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|modified| {
            modified.duration_since(std::time::UNIX_EPOCH).ok()
        })
        .and_then(|dur| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|now| now.as_secs().saturating_sub(dur.as_secs()))
        });
    
    if let Some(age) = cache_age {
        eprintln!("[DEBUG] Cache age: {} seconds", age);
        if age > 3600 {
            eprintln!("[DEBUG] Cache is older than 1 hour, will refresh");
            return None;
        }
    }
    
    eprintln!("[DEBUG] Loading from cache...");
    match std::fs::read_to_string(&cache_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<ProtonRunner>>(&content) {
                Ok(mut runners) => {
                    // Reset runtime-only fields and ensure compat_layer_type exists
                    for runner in &mut runners {
                        // Set default compat_layer_type if missing (for old cache entries)
                        if runner.compat_layer_type.is_empty() {
                            runner.compat_layer_type = "Proton".to_string();
                        }
                        for build in &mut runner.builds {
                            build.is_installed = false;
                            build.usage_count = 0;
                        }
                    }
                    eprintln!("[DEBUG] Successfully loaded {} runners from cache", runners.len());
                    Some(runners)
                }
                Err(e) => {
                    eprintln!("[DEBUG] Failed to parse cache: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to read cache: {}", e);
            None
        }
    }
}

fn save_proton_cache(runners: &[ProtonRunner]) -> Result<(), String> {
    let cache_path = get_proton_cache_path()?;
    eprintln!("[DEBUG] Saving cache to: {}", cache_path.display());
    
    let json = serde_json::to_string_pretty(runners)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;
    
    std::fs::write(&cache_path, json)
        .map_err(|e| format!("Failed to write cache: {}", e))?;
    
    eprintln!("[DEBUG] Cache saved successfully");
    Ok(())
}

fn has_new_builds(cached: &[ProtonRunner], new: &[ProtonRunner]) -> bool {
    // Compare by checking if any runner has more builds or different build titles
    for new_runner in new {
        if let Some(cached_runner) = cached.iter().find(|r| r.title == new_runner.title) {
            // Check if number of builds changed
            if new_runner.builds.len() != cached_runner.builds.len() {
                eprintln!("[DEBUG] Runner {} has different number of builds (cached: {}, new: {})", 
                    new_runner.title, cached_runner.builds.len(), new_runner.builds.len());
                return true;
            }
            
            // Check if any build titles are different
            let cached_titles: std::collections::HashSet<&String> = cached_runner.builds.iter()
                .map(|b| &b.title)
                .collect();
            let new_titles: std::collections::HashSet<&String> = new_runner.builds.iter()
                .map(|b| &b.title)
                .collect();
            
            if cached_titles != new_titles {
                eprintln!("[DEBUG] Runner {} has different builds", new_runner.title);
                return true;
            }
        } else {
            // New runner found
            eprintln!("[DEBUG] New runner found: {}", new_runner.title);
            return true;
        }
    }
    
    // Check if any cached runners were removed
    if cached.len() != new.len() {
        eprintln!("[DEBUG] Number of runners changed (cached: {}, new: {})", cached.len(), new.len());
        return true;
    }
    
    false
}

async fn load_proton_builds() -> Result<Vec<ProtonRunner>, String> {
    eprintln!("[DEBUG] load_proton_builds() called");
    
    // Try to load from cache first
    if let Some(cached_runners) = load_proton_cache() {
        eprintln!("[DEBUG] Using cached Proton builds ({} runners)", cached_runners.len());
        
        // If cache is empty, force a fresh fetch
        if cached_runners.is_empty() {
            eprintln!("[DEBUG] Cache is empty, forcing fresh fetch");
        } else {
            // Still fetch in background to check for updates, but return cached immediately
            // We'll update the cache if new builds are found, and the UI will refresh on next load
            let cached_runners_clone = cached_runners.clone();
            tokio::spawn(async move {
                eprintln!("[DEBUG] Background: Checking for new builds...");
                if let Ok(new_runners) = fetch_proton_builds_from_github().await {
                    eprintln!("[DEBUG] Background: Fetched {} runners from GitHub", new_runners.len());
                    if has_new_builds(&cached_runners_clone, &new_runners) {
                        eprintln!("[DEBUG] Background: New builds detected, updating cache");
                        if let Err(e) = save_proton_cache(&new_runners) {
                            eprintln!("[DEBUG] Background: Failed to save cache: {}", e);
                        } else {
                            eprintln!("[DEBUG] Background: Cache updated successfully");
                        }
                    } else {
                        eprintln!("[DEBUG] Background: No new builds found");
                    }
                } else {
                    eprintln!("[DEBUG] Background: Failed to fetch from GitHub");
                }
            });
            eprintln!("[DEBUG] Returning {} cached runners immediately", cached_runners.len());
            return Ok(cached_runners);
        }
    }
    
    // Cache miss, expired, or empty - fetch from GitHub
    eprintln!("[DEBUG] Cache miss/expired/empty, fetching from GitHub");
    let runners = fetch_proton_builds_from_github().await?;
    eprintln!("[DEBUG] Fetched {} runners from GitHub", runners.len());
    
    // Only save to cache if we got some runners
    if !runners.is_empty() {
        if let Err(e) = save_proton_cache(&runners) {
            eprintln!("[DEBUG] Warning: Failed to save cache: {}", e);
        } else {
            eprintln!("[DEBUG] Saved {} runners to cache", runners.len());
        }
    } else {
        eprintln!("[DEBUG] Warning: Fetched 0 runners from GitHub");
    }
    
    Ok(runners)
}

async fn fetch_proton_builds_from_github() -> Result<Vec<ProtonRunner>, String> {
    eprintln!("[DEBUG] fetch_proton_builds_from_github() called");
    
    // First, try to load from local data directory (for development and to avoid rate limits)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Try data/runners.json relative to executable
            let local_path = exe_dir.join("data").join("runners.json");
            if local_path.exists() {
                eprintln!("[DEBUG] Trying local file: {}", local_path.display());
                    if let Ok(content) = std::fs::read_to_string(&local_path) {
                    eprintln!("[DEBUG] Successfully read local runners.json ({} bytes)", content.len());
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        eprintln!("[DEBUG] Successfully parsed local runners.json");
                        return process_runners_json(json).await;
                    } else {
                        eprintln!("[DEBUG] Failed to parse local runners.json, falling back to remote");
                    }
                }
            }
            
            // Also try in project root (for development)
            let project_path = exe_dir.join("..").join("data").join("runners.json");
            if let Ok(canonical_path) = project_path.canonicalize() {
                if canonical_path.exists() {
                    eprintln!("[DEBUG] Trying project file: {}", canonical_path.display());
                    if let Ok(content) = std::fs::read_to_string(&canonical_path) {
                        eprintln!("[DEBUG] Successfully read project runners.json ({} bytes)", content.len());
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            eprintln!("[DEBUG] Successfully parsed project runners.json");
                            return process_runners_json(json).await;
                        }
                    }
                }
            }
        }
    }
    
    // Try current working directory (for development)
    let cwd_path = std::path::Path::new("data").join("runners.json");
    if cwd_path.exists() {
        eprintln!("[DEBUG] Trying CWD file: {}", cwd_path.display());
        if let Ok(content) = std::fs::read_to_string(&cwd_path) {
            eprintln!("[DEBUG] Successfully read CWD runners.json ({} bytes)", content.len());
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                eprintln!("[DEBUG] Successfully parsed CWD runners.json");
                return process_runners_json(json).await;
            }
        }
    }
    
    eprintln!("[DEBUG] No local runners.json found, trying remote URLs...");
    
    // Try multiple URL formats (GitHub raw content URLs can vary)
    // TODO: Add our own GitHub repo URL here once we host it
    let urls = vec![
        "https://raw.githubusercontent.com/Vysp3r/ProtonPlus/main/data/runners.json",
        "https://github.com/Vysp3r/ProtonPlus/raw/main/data/runners.json",
        "https://raw.githubusercontent.com/Vysp3r/ProtonPlus/refs/heads/main/data/runners.json",
    ];
    
    let client = reqwest::Client::new();
    let mut last_error = None;
    let mut json_content = None;
    
    for url in &urls {
        eprintln!("[DEBUG] Trying URL: {}", url);
        match client
            .get(*url)
            .header("User-Agent", "Rustora/1.0")
            .send()
            .await
        {
            Ok(response) => {
                eprintln!("[DEBUG] Response status: {}", response.status());
                if response.status().is_success() {
                    match response.text().await {
                        Ok(content) => {
                            eprintln!("[DEBUG] Received {} bytes of JSON content from {}", content.len(), url);
                            if content.trim().is_empty() {
                                eprintln!("[DEBUG] Empty response from {}, trying next URL...", url);
                                last_error = Some(format!("Empty response from {}", url));
                                continue;
                            }
                            // Check if it's valid JSON by trying to parse it
                            match serde_json::from_str::<serde_json::Value>(&content) {
                                Ok(_) => {
                                    json_content = Some(content);
                                    eprintln!("[DEBUG] Successfully fetched and validated JSON from {}", url);
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("[DEBUG] Invalid JSON from {}: {}, trying next URL...", url, e);
                                    last_error = Some(format!("Invalid JSON from {}: {}", url, e));
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[DEBUG] Failed to read response body from {}: {}, trying next URL...", url, e);
                            last_error = Some(format!("Failed to read response from {}: {}", url, e));
                        }
                    }
                } else {
                    let status = response.status();
                    eprintln!("[DEBUG] HTTP {} from {}, trying next URL...", status, url);
                    // Try to read error body for debugging
                    if let Ok(error_body) = response.text().await {
                        eprintln!("[DEBUG] Error response body: {}", error_body);
                    }
                    last_error = Some(format!("HTTP {} from {}", status, url));
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to fetch from {}: {}, trying next URL...", url, e);
                last_error = Some(format!("Failed to fetch from {}: {}", url, e));
            }
        }
    }
    
    let json_content = json_content.ok_or_else(|| {
        last_error.unwrap_or_else(|| "All URL attempts failed".to_string())
    })?;
    
    let json: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to parse JSON: {}", e);
            eprintln!("[DEBUG] JSON content preview: {}", &json_content.chars().take(200).collect::<String>());
            format!("Failed to parse runners.json: {}", e)
        })?;
    
    eprintln!("[DEBUG] Successfully parsed runners.json");
    process_runners_json(json).await
}

async fn process_runners_json(json: serde_json::Value) -> Result<Vec<ProtonRunner>, String> {
    let mut proton_runners = Vec::new();
    
    // Get compat_layers array
    if let Some(compat_layers) = json.get("compat_layers").and_then(|v| v.as_array()) {
        eprintln!("[DEBUG] Found {} compat_layers", compat_layers.len());
        for (layer_idx, layer) in compat_layers.iter().enumerate() {
            if let Some(title) = layer.get("title").and_then(|v| v.as_str()) {
                eprintln!("[DEBUG] Layer {}: {}", layer_idx, title);
                if title == "Proton" || title == "Wine" {
                    eprintln!("[DEBUG] Found {} layer, processing runners...", title);
                    if let Some(runners) = layer.get("runners").and_then(|v| v.as_array()) {
                        eprintln!("[DEBUG] Found {} runners in {} layer", runners.len(), title);
                        for (runner_idx, runner) in runners.iter().enumerate() {
                            if let Some(runner_title) = runner.get("title").and_then(|v| v.as_str()) {
                                eprintln!("[DEBUG] Processing runner {}: {}", runner_idx, runner_title);
                                if let Some(endpoint) = runner.get("endpoint").and_then(|v| v.as_str()) {
                                    eprintln!("[DEBUG] Runner {} endpoint: {}", runner_title, endpoint);
                                    
                                    // Parse directory_name_formats
                                    let mut directory_name_formats = Vec::new();
                                    if let Some(formats) = runner.get("directory_name_formats").and_then(|v| v.as_array()) {
                                        eprintln!("[DEBUG] Runner {} has {} directory_name_formats", runner_title, formats.len());
                                        for format in formats {
                                            if let Some(launcher) = format.get("launcher").and_then(|v| v.as_str()) {
                                                if let Some(format_str) = format.get("directory_name_format").and_then(|v| v.as_str()) {
                                                    directory_name_formats.push(DirectoryNameFormat {
                                                        launcher: launcher.to_string(),
                                                        directory_name_format: format_str.to_string(),
                                                    });
                                                    eprintln!("[DEBUG] Added format for launcher {}: {}", launcher, format_str);
                                                }
                                            }
                                        }
                                    }
                                    
                                    if runner.get("type").and_then(|v| v.as_str()) == Some("github") {
                                        eprintln!("[DEBUG] Runner {} is GitHub type, fetching releases...", runner_title);
                                        let asset_position = runner.get("asset_position")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0) as usize;
                                        
                                        // Fetch releases from GitHub API
                                        let client = reqwest::Client::new();
                                        let url = format!("{}?per_page=25&page=1", endpoint);
                                        eprintln!("[DEBUG] Fetching from URL: {}", url);
                                        
                                        let mut builds = Vec::new();
                                        
                                        match client.get(&url)
                                            .header("User-Agent", "Rustora/1.0")
                                            .send()
                                            .await
                                        {
                                            Ok(response) => {
                                                eprintln!("[DEBUG] Response status for {}: {}", runner_title, response.status());
                                                if response.status().is_success() {
                                                    match response.json::<serde_json::Value>().await {
                                                        Ok(releases_json) => {
                                                            if let Some(releases_array) = releases_json.as_array() {
                                                                eprintln!("[DEBUG] Found {} releases for {}", releases_array.len(), runner_title);
                                                                eprintln!("[DEBUG] Using asset_position: {}", asset_position);
                                                                
                                                                for (release_idx, release) in releases_array.iter().enumerate() {
                                                                    if let Some(tag_name) = release.get("tag_name").and_then(|v| v.as_str()) {
                                                                        eprintln!("[DEBUG] Release {}: {}", release_idx, tag_name);
                                                                        if let Some(assets) = release.get("assets").and_then(|v| v.as_array()) {
                                                                            eprintln!("[DEBUG] Release {} has {} assets", tag_name, assets.len());
                                                                            
                                                                            // Try to find tar.gz file first, fall back to asset_position
                                                                            let mut selected_asset: Option<&serde_json::Value> = None;
                                                                            
                                                                            // First, try to find a .tar.gz file
                                                                            for asset in assets.iter() {
                                                                                if let Some(download_url) = asset.get("browser_download_url").and_then(|v| v.as_str()) {
                                                                                    if download_url.ends_with(".tar.gz") || download_url.ends_with(".tar.gz?") {
                                                                                        eprintln!("[DEBUG] Found tar.gz asset: {}", download_url);
                                                                                        selected_asset = Some(asset);
                                                                                        break;
                                                                                    }
                                                                                }
                                                                            }
                                                                            
                                                                            // If no tar.gz found, use asset_position
                                                                            if selected_asset.is_none() && assets.len() > asset_position {
                                                                                selected_asset = assets.get(asset_position);
                                                                                eprintln!("[DEBUG] Using asset at position {} (no tar.gz found)", asset_position);
                                                                            }
                                                                            
                                                                            if let Some(asset) = selected_asset {
                                                                                if let Some(download_url) = asset.get("browser_download_url").and_then(|v| v.as_str()) {
                                                                                        let description = release.get("body")
                                                                                            .and_then(|v| v.as_str())
                                                                                            .unwrap_or("")
                                                                                            .to_string();
                                                                                        let release_date = release.get("created_at")
                                                                                            .and_then(|v| v.as_str())
                                                                                            .unwrap_or("")
                                                                                            .to_string();
                                                                                        let page_url = release.get("html_url")
                                                                                            .and_then(|v| v.as_str())
                                                                                            .unwrap_or("")
                                                                                            .to_string();
                                                                                        let download_size = asset.get("size")
                                                                                            .and_then(|v| v.as_u64())
                                                                                            .unwrap_or(0);
                                                                                        
                                                                                        eprintln!("[DEBUG] Found asset for {}: {} ({} bytes)", tag_name, download_url, download_size);
                                                                                        
                                                                                        // Check if installed (will be updated after launcher detection)
                                                                                        let is_installed = false; // Will be checked later with proper directory format
                                                                                        
                                                                                        builds.push(ProtonBuild {
                                                                                            title: tag_name.to_string(),
                                                                                            description,
                                                                                            release_date,
                                                                                            download_url: download_url.to_string(),
                                                                                            page_url,
                                                                                            download_size,
                                                                                            runner_title: runner_title.to_string(),
                                                                                            is_installed,
                                                                                            directory_name_formats: directory_name_formats.clone(),
                                                                                            usage_count: 0, // Will be updated after checking Steam config
                                                                                            is_latest: false, // Regular release, not "Latest"
                                                                                        });
                                                                                        eprintln!("[DEBUG] Added build: {} {}", runner_title, tag_name);
                                                                                    } else {
                                                                                        eprintln!("[DEBUG] Selected asset for {} has no browser_download_url", tag_name);
                                                                                    }
                                                                                } else {
                                                                                    eprintln!("[DEBUG] No suitable asset found for {}", tag_name);
                                                                                }
                                                                        } else {
                                                                            eprintln!("[DEBUG] Release {} has no assets array", tag_name);
                                                                        }
                                                                    } else {
                                                                        eprintln!("[DEBUG] Release {} has no tag_name", release_idx);
                                                                    }
                                                                }
                                                            } else {
                                                                eprintln!("[DEBUG] Releases JSON is not an array for {}", runner_title);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("[DEBUG] Failed to parse releases JSON for {}: {}", runner_title, e);
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("[DEBUG] Response status not success for {}: {}", runner_title, response.status());
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("[DEBUG] Failed to fetch releases for {}: {}", runner_title, e);
                                            }
                                        }
                                        
                                        let runner_description = runner.get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        
                                        let has_latest_support = runner.get("support_latest")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(false);
                                        
                                        // If has_latest_support, prepend a "Latest" build
                                        let mut final_builds = builds;
                                        if has_latest_support && !final_builds.is_empty() {
                                            // Create a "Latest" build that points to the first (newest) release
                                            let latest_build = final_builds[0].clone();
                                            let mut latest = latest_build;
                                            latest.title = format!("{} Latest", runner_title);
                                            latest.is_latest = true;
                                            final_builds.insert(0, latest);
                                            eprintln!("[DEBUG] Added 'Latest' build for {}", runner_title);
                                        }
                                        
                                        proton_runners.push(ProtonRunner {
                                            title: runner_title.to_string(),
                                            description: runner_description,
                                            endpoint: endpoint.to_string(),
                                            asset_position,
                                            directory_name_formats: directory_name_formats.clone(),
                                            builds: final_builds,
                                            has_latest_support,
                                            compat_layer_type: title.to_string(), // "Proton" or "Wine"
                                        });
                                        eprintln!("[DEBUG] Added runner: {} with {} builds", runner_title, proton_runners.last().unwrap().builds.len());
                                    } else {
                                        eprintln!("[DEBUG] Runner {} is not GitHub type, skipping", runner_title);
                                    }
                                } else {
                                    eprintln!("[DEBUG] Runner {} has no endpoint", runner_title);
                                }
                            } else {
                                eprintln!("[DEBUG] Runner {} has no title", runner_idx);
                            }
                        }
                    } else {
                        eprintln!("[DEBUG] Proton layer has no runners array");
                    }
                }
            } else {
                eprintln!("[DEBUG] Layer {} has no title", layer_idx);
            }
        }
    } else {
        eprintln!("[DEBUG] JSON has no compat_layers array");
    }
    
    eprintln!("[DEBUG] ========== process_runners_json COMPLETE ==========");
    eprintln!("[DEBUG] Total runners found: {}", proton_runners.len());
    for (idx, runner) in proton_runners.iter().enumerate() {
        eprintln!("[DEBUG] Final runner {}: {} with {} builds", idx, runner.title, runner.builds.len());
    }
    Ok(proton_runners)
}

async fn detect_launchers() -> Result<Vec<DetectedLauncher>, String> {
    eprintln!("[DEBUG] detect_launchers() called");
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let mut launchers = Vec::new();
    
    // Detect Steam (system, flatpak, snap)
    let steam_paths = vec![
        (format!("{}/.local/share/Steam", home), "system"),
        (format!("{}/.steam/steam", home), "system"),
        (format!("{}/.steam/root", home), "system"),
        (format!("{}/.steam/debian-installation", home), "system"),
        (format!("{}/.var/app/com.valvesoftware.Steam/data/Steam", home), "flatpak"),
        ("/snap/steam/common/.steam/root".to_string(), "snap"),
    ];
    
    for (path, install_type) in steam_paths {
        let full_path = if path.starts_with('/') && !path.starts_with(&home) {
            path.clone()
        } else {
            path.clone()
        };
        if std::path::Path::new(&full_path).exists() {
            // Verify it's actually Steam by checking for steamclient.dll files
            let steamclient = format!("{}/steamclient.dll", full_path);
            let steamclient64 = format!("{}/steamclient64.dll", full_path);
            if std::path::Path::new(&steamclient).exists() && std::path::Path::new(&steamclient64).exists() {
                eprintln!("[DEBUG] Found Steam at: {} ({})", full_path, install_type);
                launchers.push(DetectedLauncher {
                    title: "Steam".to_string(),
                    directory: full_path,
                    installation_type: install_type.to_string(),
                    is_installed: true,
                });
                break; // Only add one Steam installation
            }
        }
    }
    
    // Detect Lutris (system, flatpak)
    let lutris_paths = vec![
        (format!("{}/.local/share/lutris", home), "system"),
        (format!("{}/.var/app/net.lutris.Lutris/data/lutris", home), "flatpak"),
    ];
    
    for (path, install_type) in lutris_paths {
        if std::path::Path::new(&path).exists() {
            eprintln!("[DEBUG] Found Lutris at: {} ({})", path, install_type);
            launchers.push(DetectedLauncher {
                title: "Lutris".to_string(),
                directory: path,
                installation_type: install_type.to_string(),
                is_installed: true,
            });
            break;
        }
    }
    
    // Detect Heroic Games Launcher (system, flatpak)
    let heroic_paths = vec![
        (format!("{}/.config/heroic", home), "system"),
        (format!("{}/.var/app/com.heroicgameslauncher.hgl/config/heroic", home), "flatpak"),
    ];
    
    for (path, install_type) in heroic_paths {
        if std::path::Path::new(&path).exists() {
            eprintln!("[DEBUG] Found Heroic Games Launcher at: {} ({})", path, install_type);
            launchers.push(DetectedLauncher {
                title: "Heroic Games Launcher".to_string(),
                directory: path,
                installation_type: install_type.to_string(),
                is_installed: true,
            });
            break;
        }
    }
    
    eprintln!("[DEBUG] Detected {} launchers total", launchers.len());
    Ok(launchers)
}

fn check_proton_installed(runner_title: &str, release_name: &str, directory_name_formats: &[DirectoryNameFormat], detected_launchers: &[DetectedLauncher], compat_layer_type: &str) -> bool {
    eprintln!("[DEBUG] ========== check_proton_installed START ==========");
    eprintln!("[DEBUG] runner_title: {}", runner_title);
    eprintln!("[DEBUG] release_name: {}", release_name);
    eprintln!("[DEBUG] compat_layer_type: {}", compat_layer_type);
    eprintln!("[DEBUG] directory_name_formats count: {}", directory_name_formats.len());
    for (idx, format) in directory_name_formats.iter().enumerate() {
        eprintln!("[DEBUG]   Format {}: launcher={}, format={}", idx, format.launcher, format.directory_name_format);
    }
    eprintln!("[DEBUG] detected_launchers count: {}", detected_launchers.len());
    for (idx, launcher) in detected_launchers.iter().enumerate() {
        eprintln!("[DEBUG]   Launcher {}: {} at {} ({})", idx, launcher.title, launcher.directory, launcher.installation_type);
    }
    
    // Get directory name format for each detected launcher
    for launcher in detected_launchers {
        eprintln!("[DEBUG] Checking launcher: {}", launcher.title);
        let format = directory_name_formats.iter()
            .find(|f| f.launcher == launcher.title)
            .or_else(|| directory_name_formats.iter().find(|f| f.launcher == "default"));
        
        if let Some(format) = format {
            eprintln!("[DEBUG]   Found format for {}: {}", launcher.title, format.directory_name_format);
            let dir_name = format_directory_name(&format.directory_name_format, runner_title, release_name);
            eprintln!("[DEBUG]   Formatted directory name: {} -> {}", format.directory_name_format, dir_name);
            
            // Get the directory path based on launcher type and compat layer type
            let compat_dir = get_launcher_compat_directory_for_type(&launcher.title, &launcher.directory, compat_layer_type);
            eprintln!("[DEBUG]   Compat directory for {} (type: {}): {}", launcher.title, compat_layer_type, compat_dir);
            let full_path = format!("{}/{}", compat_dir, dir_name);
            
            eprintln!("[DEBUG]   Checking full path: {}", full_path);
            let path_exists = std::path::Path::new(&full_path).exists();
            eprintln!("[DEBUG]   Path exists: {}", path_exists);
            
            if path_exists {
                // Verify it's actually a directory and has some content
                if let Ok(metadata) = std::fs::metadata(&full_path) {
                    eprintln!("[DEBUG]   Path metadata: is_dir={}, is_file={}", metadata.is_dir(), metadata.is_file());
                    if metadata.is_dir() {
                        // Check if directory has content (at least one file/dir)
                        if let Ok(entries) = std::fs::read_dir(&full_path) {
                            let entry_count = entries.count();
                            eprintln!("[DEBUG]   Directory has {} entries", entry_count);
                            if entry_count > 0 {
                                eprintln!("[DEBUG] ========== check_proton_installed END: FOUND ==========");
                                return true;
                            } else {
                                eprintln!("[DEBUG]   Directory exists but is empty, continuing search...");
                            }
                        }
                    }
                }
            } else {
                eprintln!("[DEBUG]   Path does not exist, continuing search...");
            }
        } else {
            eprintln!("[DEBUG]   No format found for launcher {}", launcher.title);
        }
    }
    
    eprintln!("[DEBUG] {} {} not found in any launcher directory", runner_title, release_name);
    eprintln!("[DEBUG] ========== check_proton_installed END: NOT FOUND ==========");
    false
}

fn has_proton_update(runner: &ProtonRunner, build: &ProtonBuild, detected_launchers: &[DetectedLauncher]) -> bool {
    // Only check for updates on "Latest" builds that are installed
    if !build.is_latest || !build.is_installed {
        return false;
    }
    
    // Find the latest available release (first non-latest build)
    let latest_release = runner.builds.iter().find(|b| !b.is_latest);
    if latest_release.is_none() {
        return false;
    }
    let latest_release = latest_release.unwrap();
    
    // Check what version is actually installed by looking at the directory
    // For "Latest" builds, we need to find which actual release is installed
    for launcher in detected_launchers {
        let format = runner.directory_name_formats.iter()
            .find(|f| f.launcher == launcher.title)
            .or_else(|| runner.directory_name_formats.iter().find(|f| f.launcher == "default"));
        
        if let Some(format) = format {
            let compat_dir = get_launcher_compat_directory_for_type(&launcher.title, &launcher.directory, &runner.compat_layer_type);
            
            // Check if the latest release is already installed
            let latest_dir_name = format_directory_name(&format.directory_name_format, &runner.title, &latest_release.title);
            let latest_path = format!("{}/{}", compat_dir, latest_dir_name);
            
            if std::path::Path::new(&latest_path).exists() {
                // Latest release is already installed, no update needed
                eprintln!("[DEBUG] Latest release {} is already installed, no update needed", latest_release.title);
                return false;
            }
            
            // Check if any older release is installed
            for installed_build in &runner.builds {
                if installed_build.is_latest {
                    continue;
                }
                let installed_dir_name = format_directory_name(&format.directory_name_format, &runner.title, &installed_build.title);
                let installed_path = format!("{}/{}", compat_dir, installed_dir_name);
                
                if std::path::Path::new(&installed_path).exists() {
                    // An older version is installed, update is available
                    eprintln!("[DEBUG] Older version {} is installed, update available to {}", installed_build.title, latest_release.title);
                    return true;
                }
            }
        }
    }
    
    // If we can't determine what's installed, assume no update to be safe
    false
}

fn format_directory_name(format: &str, runner_title: &str, release_name: &str) -> String {
    eprintln!("[DEBUG] format_directory_name: format='{}', runner_title='{}', release_name='{}'", format, runner_title, release_name);
    let mut result = format.to_string();
    result = result.replace("$release_name", release_name);
    result = result.replace("$title", runner_title);
    
    // Handle special prefixes like ProtonPlus does
    if result.starts_with('_') {
        result = result[1..].to_lowercase();
        eprintln!("[DEBUG]   Applied '_' prefix, result: '{}'", result);
    } else if result.starts_with('!') {
        // Format: !$release_name:v:vkd3d-lutris-
        // Replace v with vkd3d-lutris- in release_name
        let parts: Vec<&str> = result[1..].split(':').collect();
        if parts.len() == 3 {
            let search = parts[1];
            let replace = parts[2];
            let new_result = parts[0].replace(search, replace);
            eprintln!("[DEBUG]   Applied '!' prefix, search='{}', replace='{}', result: '{}'", search, replace, new_result);
            result = new_result;
        }
    } else if result.starts_with('&') {
        // Format: &$release_name:.:Proton-$release_name:$release_name
        // If release_name contains ".", use "Proton-$release_name", else use "$release_name"
        let parts: Vec<&str> = result[1..].split(':').collect();
        if parts.len() == 4 {
            if release_name.contains(parts[1]) {
                let new_result = parts[2].replace("$release_name", release_name);
                eprintln!("[DEBUG]   Applied '&' prefix (contains '{}'), using parts[2], result: '{}'", parts[1], new_result);
                result = new_result;
            } else {
                let new_result = parts[3].replace("$release_name", release_name);
                eprintln!("[DEBUG]   Applied '&' prefix (doesn't contain '{}'), using parts[3], result: '{}'", parts[1], new_result);
                result = new_result;
            }
        }
    }
    
    eprintln!("[DEBUG] format_directory_name result: '{}'", result);
    result
}

#[allow(dead_code)]
fn get_launcher_compat_directory(launcher_title: &str, launcher_dir: &str) -> String {
    // This function is called with the compat_layer type (Proton/Wine) determined by the runner
    // For now, we'll use a generic approach that works for both
    match launcher_title {
        "Steam" => {
            // Try multiple Steam compatibilitytools.d locations
            let paths = vec![
                format!("{}/compatibilitytools.d", launcher_dir),
                format!("{}/steamapps/common", launcher_dir), // Fallback
            ];
            for path in paths {
                if std::path::Path::new(&path).parent().map(|p| p.exists()).unwrap_or(false) {
                    return path;
                }
            }
            format!("{}/compatibilitytools.d", launcher_dir)
        }
        "Lutris" => format!("{}/runners/proton", launcher_dir), // Will be overridden for Wine
        "Heroic Games Launcher" => format!("{}/tools/proton", launcher_dir), // Will be overridden for Wine
        _ => format!("{}/compatibilitytools.d", launcher_dir),
    }
}

fn get_launcher_compat_directory_for_type(launcher_title: &str, launcher_dir: &str, compat_layer_type: &str) -> String {
    match (launcher_title, compat_layer_type) {
        ("Steam", _) => {
            // Steam only supports Proton in compatibilitytools.d
            let paths = vec![
                format!("{}/compatibilitytools.d", launcher_dir),
                format!("{}/steamapps/common", launcher_dir),
            ];
            for path in paths {
                if std::path::Path::new(&path).parent().map(|p| p.exists()).unwrap_or(false) {
                    return path;
                }
            }
            format!("{}/compatibilitytools.d", launcher_dir)
        }
        ("Lutris", "Proton") => format!("{}/runners/proton", launcher_dir),
        ("Lutris", "Wine") => format!("{}/runners/wine", launcher_dir),
        ("Heroic Games Launcher", "Proton") => format!("{}/tools/proton", launcher_dir),
        ("Heroic Games Launcher", "Wine") => format!("{}/tools/wine", launcher_dir),
        ("Bottles", "Proton") => format!("{}/runners", launcher_dir),
        ("Bottles", "Wine") => format!("{}/runners", launcher_dir),
        ("WineZGUI", "Wine") => format!("{}/Runners", launcher_dir),
        _ => format!("{}/compatibilitytools.d", launcher_dir),
    }
}

// Progress is updated step-by-step during download and installation
// Download progress: 0-50% (overall), 0-100% (download bar)
// Install progress: 50-100% (overall), 0-100% (install bar)

async fn download_proton_build(runner_title: String, title: String, download_url: String) -> Result<(String, String, String), String> {
    eprintln!("[DEBUG] ========== DOWNLOAD START ==========");
    eprintln!("[DEBUG] Runner: {}", runner_title);
    eprintln!("[DEBUG] Build: {}", title);
    eprintln!("[DEBUG] URL: {}", download_url);
    eprintln!("[DEBUG] =====================================");
    
    let client = reqwest::Client::new();
    let temp_dir = std::env::temp_dir();
    let tar_path = temp_dir.join(format!("{}.tar.gz", title));
    eprintln!("[DEBUG] Temp directory: {}", temp_dir.display());
    eprintln!("[DEBUG] Target file: {}", tar_path.display());
    
    // Check if file already exists
    if tar_path.exists() {
        let metadata = std::fs::metadata(&tar_path).ok();
        if let Some(meta) = metadata {
            eprintln!("[DEBUG] Existing file found: {} bytes", meta.len());
        }
    }
    
    eprintln!("[DEBUG] Sending GET request to: {}", download_url);
    let start_time = std::time::Instant::now();
    let response = client
        .get(&download_url)
        .header("User-Agent", "Rustora/1.0")
        .send()
        .await
        .map_err(|e| {
            eprintln!("[DEBUG] Request failed: {}", e);
            eprintln!("[DEBUG] Error type: {:?}", e);
            format!("Failed to download: {}", e)
        })?;
    
    let request_duration = start_time.elapsed();
    eprintln!("[DEBUG] Request completed in {:.2}s", request_duration.as_secs_f64());
    eprintln!("[DEBUG] Response status: {}", response.status());
    eprintln!("[DEBUG] Response headers: {:?}", response.headers());
    
    let status = response.status();
    if !status.is_success() {
        eprintln!("[DEBUG] Response status indicates failure: {}", status);
        // Try to read error body (this consumes response)
        let error_body_result = response.text().await;
        if let Ok(error_body) = error_body_result {
            eprintln!("[DEBUG] Error response body: {}", error_body);
        }
        return Err(format!("Failed to download: HTTP {}", status));
    }
    
    if let Some(content_length) = response.content_length() {
        eprintln!("[DEBUG] Content length: {} bytes ({:.2} MB)", content_length, content_length as f64 / 1_048_576.0);
    } else {
        eprintln!("[DEBUG] Content length: unknown");
    }
    
    eprintln!("[DEBUG] Reading response bytes...");
    let read_start = std::time::Instant::now();
    let bytes = response.bytes().await
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to read bytes: {}", e);
            eprintln!("[DEBUG] Error type: {:?}", e);
            format!("Failed to read download: {}", e)
        })?;
    let read_duration = read_start.elapsed();
    eprintln!("[DEBUG] Read completed in {:.2}s", read_duration.as_secs_f64());
    eprintln!("[DEBUG] Received {} bytes ({:.2} MB)", bytes.len(), bytes.len() as f64 / 1_048_576.0);
    
    eprintln!("[DEBUG] Writing to file: {}", tar_path.display());
    let write_start = std::time::Instant::now();
    std::fs::write(&tar_path, bytes.as_ref())
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to write file: {}", e);
            eprintln!("[DEBUG] Error type: {:?}", e);
            eprintln!("[DEBUG] File path: {}", tar_path.display());
            if let Some(parent) = tar_path.parent() {
                eprintln!("[DEBUG] Parent directory exists: {}", parent.exists());
                if let Ok(metadata) = parent.metadata() {
                    eprintln!("[DEBUG] Parent permissions: {:?}", metadata.permissions());
                }
            }
            format!("Failed to save file: {}", e)
        })?;
    let write_duration = write_start.elapsed();
    eprintln!("[DEBUG] Write completed in {:.2}s", write_duration.as_secs_f64());
    
    // Verify file was written
    if let Ok(metadata) = std::fs::metadata(&tar_path) {
        eprintln!("[DEBUG] File written successfully: {} bytes", metadata.len());
    } else {
        eprintln!("[DEBUG] WARNING: Could not verify file after write");
    }
    
    eprintln!("[DEBUG] ========== DOWNLOAD COMPLETE ==========");
    eprintln!("[DEBUG] File: {}", tar_path.display());
    eprintln!("[DEBUG] ========================================");
    Ok((runner_title, title, tar_path.to_string_lossy().to_string()))
}

#[allow(dead_code)]
async fn install_proton_build(runner_title: String, title: String, tar_path: String) -> Result<(String, String), String> {
    eprintln!("[DEBUG] install_proton_build({}, {})", title, tar_path);
    use std::fs::File;
    use flate2::read::GzDecoder;
    use tar::Archive;
    
    // Determine Steam compatibilitytools.d directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    eprintln!("[DEBUG] HOME directory: {}", home);
    let steam_paths = vec![
        format!("{}/.steam/root/compatibilitytools.d", home),
        format!("{}/.local/share/Steam/compatibilitytools.d", home),
        format!("{}/.steam/steam/compatibilitytools.d", home),
    ];
    
    eprintln!("[DEBUG] Checking Steam compatibilitytools.d directories...");
    let compat_dir = steam_paths.iter()
        .find(|p| {
            let exists = std::path::Path::new(p).exists();
            eprintln!("[DEBUG] Checking {}: {}", p, if exists { "exists" } else { "not found" });
            exists
        })
        .ok_or_else(|| {
            eprintln!("[DEBUG] No existing compatibilitytools.d found, will create: {}", steam_paths[1]);
            // Try to create the first one
            let first = &steam_paths[1]; // Use .local/share/Steam/compatibilitytools.d
            if let Some(parent) = std::path::Path::new(first).parent() {
                eprintln!("[DEBUG] Creating parent directory: {}", parent.display());
                let _ = std::fs::create_dir_all(parent);
            }
            first.clone()
        })?;
    
    eprintln!("[DEBUG] Using compatibilitytools.d directory: {}", compat_dir);
    
    // Extract archive (detect format)
    eprintln!("[DEBUG] Opening archive: {}", tar_path);
    let mut file = File::open(&tar_path)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to open archive: {}", e);
            format!("Failed to open archive: {}", e)
        })?;
    
    // Read first 6 bytes to detect compression format (xz needs 6 bytes)
    use std::io::{Read, Seek, SeekFrom};
    let mut magic_buf = [0u8; 6];
    if file.read_exact(&mut magic_buf).is_err() {
        return Err("Archive file appears to be corrupted or incomplete".to_string());
    }
    
    // Reset file position for actual extraction
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek file: {}", e))?;
    
    // Detect compression format
    let is_gzip = magic_buf[0] == 0x1f && magic_buf[1] == 0x8b;
    let is_zstd = magic_buf[0] == 0x28 && magic_buf[1] == 0xb5 && magic_buf[2] == 0x2f && magic_buf[3] == 0xfd;
    let is_xz = magic_buf[0] == 0xfd && magic_buf[1] == 0x37 && magic_buf[2] == 0x7a && magic_buf[3] == 0x58 && magic_buf[4] == 0x5a && magic_buf[5] == 0x00;
    
    eprintln!("[DEBUG] File magic bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", 
        magic_buf[0], magic_buf[1], magic_buf[2], magic_buf[3], magic_buf[4], magic_buf[5]);
    eprintln!("[DEBUG] Format detection: gzip={}, zstd={}, xz={}", is_gzip, is_zstd, is_xz);
    
    if !is_gzip && !is_zstd && !is_xz {
        return Err(format!("Unsupported archive format (magic bytes: {:02x} {:02x} {:02x} {:02x}). Expected gzip (.tar.gz), zstd (.tar.zst), or xz (.tar.xz).", 
            magic_buf[0], magic_buf[1], magic_buf[2], magic_buf[3]));
    }
    
    // Extract to temp directory first
    let temp_extract = std::env::temp_dir().join(format!("proton_extract_{}", title));
    
    // Create appropriate decoder and extract based on format
    if is_gzip {
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);
        archive.unpack(&temp_extract)
            .map_err(|e| format!("Failed to extract gzip archive: {}", e))?;
    } else if is_zstd {
        // zstd
        use zstd::stream::Decoder;
        let decoder = Decoder::new(file)
            .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;
        let mut archive = Archive::new(decoder);
        archive.unpack(&temp_extract)
            .map_err(|e| format!("Failed to extract zstd archive: {}", e))?;
    } else {
        // xz
        use xz2::read::XzDecoder;
        let xz = XzDecoder::new(file);
        let mut archive = Archive::new(xz);
        archive.unpack(&temp_extract)
            .map_err(|e| format!("Failed to extract xz archive: {}", e))?;
    }
    eprintln!("[DEBUG] Archive extracted successfully");
    
    // Find the extracted directory (usually the first directory in the archive)
    eprintln!("[DEBUG] Reading temp extract directory...");
    let entries: Vec<_> = std::fs::read_dir(&temp_extract)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to read extract dir: {}", e);
            format!("Failed to read extract dir: {}", e)
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to read entry: {}", e);
            format!("Failed to read entry: {}", e)
        })?;
    
    eprintln!("[DEBUG] Found {} entries in extract directory", entries.len());
    let extracted_dir = entries
        .iter()
        .find(|entry| {
            if let Ok(metadata) = entry.metadata() {
                metadata.is_dir()
            } else {
                false
            }
        })
        .ok_or_else(|| {
            eprintln!("[DEBUG] No directory found in archive");
            "No directory found in archive"
        })?
        .path();
    
    eprintln!("[DEBUG] Extracted directory: {}", extracted_dir.display());
    if !extracted_dir.is_dir() {
        eprintln!("[DEBUG] Extracted path is not a directory");
        return Err("Archive does not contain a directory".to_string());
    }
    
    // Move to compatibilitytools.d
    // Use the release name as directory name (will be updated to use directory_name_format)
    let dest_path = std::path::Path::new(&compat_dir).join(&title);
    eprintln!("[DEBUG] Destination path: {}", dest_path.display());
    if dest_path.exists() {
        eprintln!("[DEBUG] Destination exists, removing...");
        std::fs::remove_dir_all(&dest_path)
            .map_err(|e| {
                eprintln!("[DEBUG] Failed to remove existing installation: {}", e);
                format!("Failed to remove existing installation: {}", e)
            })?;
    }
    
    eprintln!("[DEBUG] Moving {} to {}", extracted_dir.display(), dest_path.display());
    std::fs::rename(&extracted_dir, &dest_path)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to move directory: {}", e);
            format!("Failed to move to compatibilitytools.d: {}. You may need to run with sudo.", e)
        })?;
    eprintln!("[DEBUG] Successfully moved to compatibilitytools.d");
    
    // Clean up
    eprintln!("[DEBUG] Cleaning up temp files...");
    let _ = std::fs::remove_file(&tar_path);
    let _ = std::fs::remove_dir_all(&temp_extract);
    eprintln!("[DEBUG] Cleanup complete");
    
    eprintln!("[DEBUG] Installation complete for: {}", title);
    Ok((runner_title, title))
}

async fn install_proton_build_with_launcher(
    runner_title: String,
    title: String,
    tar_path: String,
    selected_launcher: Option<String>,
    runner: Option<ProtonRunner>,
) -> Result<(String, String), String> {
    eprintln!("[DEBUG] ========== INSTALLATION START ==========");
    eprintln!("[DEBUG] Runner: {}", runner_title);
    eprintln!("[DEBUG] Build: {}", title);
    eprintln!("[DEBUG] Archive: {}", tar_path);
    eprintln!("[DEBUG] ========================================");
    use std::fs::File;
    use flate2::read::GzDecoder;
    use tar::Archive;
    
    // Verify archive exists
    if !std::path::Path::new(&tar_path).exists() {
        eprintln!("[DEBUG] ERROR: Archive file does not exist: {}", tar_path);
        return Err(format!("Archive file not found: {}", tar_path));
    }
    
    if let Ok(metadata) = std::fs::metadata(&tar_path) {
        eprintln!("[DEBUG] Archive size: {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1_048_576.0);
    }
    
    // Get the selected launcher or use first detected
    let launcher_title = selected_launcher.as_ref().map(|s| s.as_str()).unwrap_or("Steam");
    eprintln!("[DEBUG] Target launcher: {}", launcher_title);
    
    // Detect launchers to get directory
    let detected_launchers = detect_launchers().await
        .map_err(|e| format!("Failed to detect launchers: {}", e))?;
    
    let launcher = detected_launchers.iter()
        .find(|l| l.title == launcher_title)
        .ok_or_else(|| format!("Launcher '{}' not found", launcher_title))?;
    
    eprintln!("[DEBUG] Using launcher directory: {}", launcher.directory);
    
    // Get directory name format for this launcher
    let directory_name = if let Some(ref runner_data) = runner {
        let format = runner_data.directory_name_formats.iter()
            .find(|f| f.launcher == launcher_title)
            .or_else(|| runner_data.directory_name_formats.iter().find(|f| f.launcher == "default"));
        
        if let Some(format) = format {
            format_directory_name(&format.directory_name_format, &runner_title, &title)
        } else {
            title.clone()
        }
    } else {
        title.clone()
    };
    
    eprintln!("[DEBUG] Directory name: {}", directory_name);
    
    // Get the compatibility directory for this launcher and compat layer type
    let compat_layer_type = runner.as_ref().map(|r| r.compat_layer_type.as_str()).unwrap_or("Proton");
    let compat_dir = get_launcher_compat_directory_for_type(&launcher.title, &launcher.directory, compat_layer_type);
    eprintln!("[DEBUG] Compatibility directory: {} (type: {})", compat_dir, compat_layer_type);
    
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(&compat_dir).parent() {
        eprintln!("[DEBUG] Creating parent directory: {}", parent.display());
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::create_dir_all(&compat_dir)
        .map_err(|e| format!("Failed to create compatibility directory: {}", e))?;
    
    // Extract archive (detect format)
    eprintln!("[DEBUG] Opening archive: {}", tar_path);
    let mut file = File::open(&tar_path)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to open archive: {}", e);
            format!("Failed to open archive: {}", e)
        })?;
    
    // Read first 6 bytes to detect compression format (xz needs 6 bytes)
    use std::io::{Read, Seek, SeekFrom};
    let mut magic_buf = [0u8; 6];
    if file.read_exact(&mut magic_buf).is_err() {
        return Err("Archive file appears to be corrupted or incomplete".to_string());
    }
    
    // Reset file position for actual extraction
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek file: {}", e))?;
    
    // Detect compression format
    let is_gzip = magic_buf[0] == 0x1f && magic_buf[1] == 0x8b;
    let is_zstd = magic_buf[0] == 0x28 && magic_buf[1] == 0xb5 && magic_buf[2] == 0x2f && magic_buf[3] == 0xfd;
    let is_xz = magic_buf[0] == 0xfd && magic_buf[1] == 0x37 && magic_buf[2] == 0x7a && magic_buf[3] == 0x58 && magic_buf[4] == 0x5a && magic_buf[5] == 0x00;
    
    eprintln!("[DEBUG] File magic bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", 
        magic_buf[0], magic_buf[1], magic_buf[2], magic_buf[3], magic_buf[4], magic_buf[5]);
    eprintln!("[DEBUG] Format detection: gzip={}, zstd={}, xz={}", is_gzip, is_zstd, is_xz);
    
    if !is_gzip && !is_zstd && !is_xz {
        return Err(format!("Unsupported archive format (magic bytes: {:02x} {:02x} {:02x} {:02x}). Expected gzip (.tar.gz), zstd (.tar.zst), or xz (.tar.xz).", 
            magic_buf[0], magic_buf[1], magic_buf[2], magic_buf[3]));
    }
    
    // Extract to temp directory in user's home folder (to avoid cross-device issues)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let home_tmp = std::path::Path::new(&home).join(".tmp");
    std::fs::create_dir_all(&home_tmp)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to create home tmp directory: {}", e);
            format!("Failed to create temp directory: {}", e)
        })?;
    let temp_extract = home_tmp.join(format!("proton_extract_{}", title));
    if temp_extract.exists() {
        eprintln!("[DEBUG] Temp extract directory exists, removing...");
        std::fs::remove_dir_all(&temp_extract)
            .map_err(|e| {
                eprintln!("[DEBUG] Failed to remove existing temp extract: {}", e);
                format!("Failed to clean temp extract: {}", e)
            })?;
    }
    std::fs::create_dir_all(&temp_extract)
        .map_err(|e| format!("Failed to create temp extract: {}", e))?;
    
    // Create appropriate decoder and extract based on format
    if is_gzip {
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);
        archive.unpack(&temp_extract)
            .map_err(|e| format!("Failed to extract gzip archive: {}", e))?;
    } else if is_zstd {
        // zstd
        use zstd::stream::Decoder;
        let decoder = Decoder::new(file)
            .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;
        let mut archive = Archive::new(decoder);
        archive.unpack(&temp_extract)
            .map_err(|e| format!("Failed to extract zstd archive: {}", e))?;
    } else {
        // xz
        use xz2::read::XzDecoder;
        let xz = XzDecoder::new(file);
        let mut archive = Archive::new(xz);
        archive.unpack(&temp_extract)
            .map_err(|e| format!("Failed to extract xz archive: {}", e))?;
    }
    
    // Get the temp extract directory (same for both formats)
    let home_tmp = std::path::Path::new(&home).join(".tmp");
    let temp_extract = home_tmp.join(format!("proton_extract_{}", title));
    eprintln!("[DEBUG] Temp extract directory: {}", temp_extract.display());
    eprintln!("[DEBUG] Archive extracted successfully");
    
    // Find the extracted directory (usually the first directory in the archive)
    eprintln!("[DEBUG] Reading temp extract directory...");
    let entries: Vec<_> = std::fs::read_dir(&temp_extract)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to read extract dir: {}", e);
            format!("Failed to read extract dir: {}", e)
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to read entry: {}", e);
            format!("Failed to read entry: {}", e)
        })?;
    
    eprintln!("[DEBUG] Found {} entries in extract directory", entries.len());
    let extracted_dir = entries
        .iter()
        .find(|entry| {
            if let Ok(metadata) = entry.metadata() {
                metadata.is_dir()
            } else {
                false
            }
        })
        .ok_or_else(|| {
            eprintln!("[DEBUG] No directory found in archive");
            "No directory found in archive"
        })?
        .path();
    
    eprintln!("[DEBUG] Extracted directory: {}", extracted_dir.display());
    if !extracted_dir.is_dir() {
        eprintln!("[DEBUG] Extracted path is not a directory");
        return Err("Archive does not contain a directory".to_string());
    }
    
    // Move to compatibility directory with proper directory name
    let dest_path = std::path::Path::new(&compat_dir).join(&directory_name);
    eprintln!("[DEBUG] Destination path: {}", dest_path.display());
    
    // Check if destination exists
    if dest_path.exists() {
        eprintln!("[DEBUG] Destination exists, checking contents...");
        if let Ok(metadata) = dest_path.metadata() {
            eprintln!("[DEBUG] Existing installation is directory: {}", metadata.is_dir());
            if let Ok(entries) = std::fs::read_dir(&dest_path) {
                let count = entries.count();
                eprintln!("[DEBUG] Existing installation has {} entries", count);
            }
        }
        eprintln!("[DEBUG] Removing existing installation...");
        let remove_start = std::time::Instant::now();
        std::fs::remove_dir_all(&dest_path)
            .map_err(|e| {
                eprintln!("[DEBUG] Failed to remove existing installation: {}", e);
                eprintln!("[DEBUG] Error type: {:?}", e);
                eprintln!("[DEBUG] Destination: {}", dest_path.display());
                if let Ok(metadata) = dest_path.metadata() {
                    eprintln!("[DEBUG] Permissions: {:?}", metadata.permissions());
                }
                format!("Failed to remove existing installation: {}", e)
            })?;
        eprintln!("[DEBUG] Existing installation removed in {:.2}s", remove_start.elapsed().as_secs_f64());
    } else {
        eprintln!("[DEBUG] Destination does not exist, will create new installation");
    }
    
    // Verify source exists before moving
    if !extracted_dir.exists() {
        eprintln!("[DEBUG] ERROR: Extracted directory does not exist: {}", extracted_dir.display());
        return Err(format!("Extracted directory not found: {}", extracted_dir.display()));
    }
    
    if let Ok(metadata) = extracted_dir.metadata() {
        eprintln!("[DEBUG] Extracted directory size: {} bytes", metadata.len());
    }
    
    eprintln!("[DEBUG] Copying {} to {}", extracted_dir.display(), dest_path.display());
    let copy_start = std::time::Instant::now();
    
    // Use copy_dir_all to handle cross-device moves
    fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if path.is_dir() {
                copy_dir_all(&path, &dst_path)?;
            } else {
                std::fs::copy(&path, &dst_path)?;
            }
        }
        Ok(())
    }
    
    copy_dir_all(&extracted_dir, &dest_path)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to copy directory: {}", e);
            eprintln!("[DEBUG] Error type: {:?}", e);
            eprintln!("[DEBUG] Source: {}", extracted_dir.display());
            eprintln!("[DEBUG] Destination: {}", dest_path.display());
            eprintln!("[DEBUG] Source exists: {}", extracted_dir.exists());
            eprintln!("[DEBUG] Destination parent exists: {}", dest_path.parent().map(|p| p.exists()).unwrap_or(false));
            if let Some(parent) = dest_path.parent() {
                if let Ok(metadata) = parent.metadata() {
                    eprintln!("[DEBUG] Parent permissions: {:?}", metadata.permissions());
                }
            }
            format!("Failed to copy to {}: {}", compat_dir, e)
        })?;
    let copy_duration = copy_start.elapsed();
    eprintln!("[DEBUG] Copy completed in {:.2}s", copy_duration.as_secs_f64());
    
    // Verify installation
    if dest_path.exists() {
        eprintln!("[DEBUG] Installation verified at: {}", dest_path.display());
        if let Ok(metadata) = dest_path.metadata() {
            eprintln!("[DEBUG] Installation is directory: {}", metadata.is_dir());
        }
    } else {
        eprintln!("[DEBUG] WARNING: Installation path does not exist after move!");
    }
    
    eprintln!("[DEBUG] Successfully moved to {}", compat_dir);
    
    // Clean up
    eprintln!("[DEBUG] Cleaning up temp files...");
    let cleanup_start = std::time::Instant::now();
    
    // Remove temp archive
    if let Err(e) = std::fs::remove_file(&tar_path) {
        eprintln!("[DEBUG] Warning: Failed to remove temp archive: {}", e);
    } else {
        eprintln!("[DEBUG] Temp archive removed: {}", tar_path);
    }
    
    // Remove temp extract directory (now that we've copied it)
    if let Err(e) = std::fs::remove_dir_all(&temp_extract) {
        eprintln!("[DEBUG] Warning: Failed to remove temp extract dir: {}", e);
        eprintln!("[DEBUG] Temp extract path: {}", temp_extract.display());
    } else {
        eprintln!("[DEBUG] Temp extract directory removed: {}", temp_extract.display());
    }
    
    // Also try to remove the parent .tmp directory if it's empty
    if let Some(parent_tmp) = temp_extract.parent() {
        if parent_tmp.exists() {
            // Only remove if it's empty (ignore errors)
            let _ = std::fs::remove_dir(parent_tmp);
        }
    }
    
    let cleanup_duration = cleanup_start.elapsed();
    eprintln!("[DEBUG] Cleanup completed in {:.2}s", cleanup_duration.as_secs_f64());
    
    eprintln!("[DEBUG] ========== INSTALLATION COMPLETE ==========");
    eprintln!("[DEBUG] Build: {} {}", runner_title, title);
    eprintln!("[DEBUG] Location: {}", dest_path.display());
    eprintln!("[DEBUG] Launcher: {}", launcher_title);
    eprintln!("[DEBUG] ============================================");
    Ok((runner_title, title))
}

async fn check_proton_usage(mut runners: Vec<ProtonRunner>, launchers: Vec<DetectedLauncher>) -> Result<Vec<ProtonRunner>, String> {
    eprintln!("[DEBUG] check_proton_usage() called");
    
    // For now, only check Steam usage (other launchers don't have a centralized config like Steam)
    let steam_launcher = launchers.iter().find(|l| l.title == "Steam");
    
    let mut tool_usage: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    
    if let Some(steam) = steam_launcher {
        eprintln!("[DEBUG] Checking Steam usage for {} runners", runners.len());
        
        // Read Steam's config.vdf to get compatibility tool mappings
        let config_path = format!("{}/config/config.vdf", steam.directory);
        eprintln!("[DEBUG] Reading Steam config: {}", config_path);
        
        if let Ok(config_content) = std::fs::read_to_string(&config_path) {
            // Parse CompatToolMapping section
            // Format: "CompatToolMapping"\n\t\t\t\t{\n\t\t\t\t\t"<appid>"\n\t\t\t\t\t{\n\t\t\t\t\t\t"name"\t\t"<tool_name>"\n...
            let compat_tool_mapping_start = "\"CompatToolMapping\"\n\t\t\t\t{";
            let compat_tool_mapping_end = "\n\t\t\t\t}";
            
            if let Some(start_pos) = config_content.find(compat_tool_mapping_start) {
                let start = start_pos + compat_tool_mapping_start.len();
                if let Some(end_pos) = config_content[start..].find(compat_tool_mapping_end) {
                    let mapping_content = &config_content[start..start + end_pos];
                    eprintln!("[DEBUG] Found CompatToolMapping section ({} bytes)", mapping_content.len());
                    
                    // Find all "name"\t\t"<tool_name>" patterns
                    let name_pattern = "\"name\"\t\t\"";
                    let mut search_pos = 0;
                    while let Some(name_start) = mapping_content[search_pos..].find(name_pattern) {
                        let name_start = search_pos + name_start + name_pattern.len();
                        if let Some(name_end) = mapping_content[name_start..].find('"') {
                            let tool_name = mapping_content[name_start..name_start + name_end].to_string();
                            let count = tool_usage.entry(tool_name.clone()).or_insert(0);
                            *count += 1;
                            eprintln!("[DEBUG] Found tool usage: {} (count: {})", tool_name, count);
                            search_pos = name_start + name_end;
                        } else {
                            break;
                        }
                    }
                    
                    eprintln!("[DEBUG] Found {} unique tools in use", tool_usage.len());
                }
            } else {
                eprintln!("[DEBUG] No CompatToolMapping found in Steam config");
            }
        } else {
            eprintln!("[DEBUG] Could not read Steam config file");
        }
    } else {
        eprintln!("[DEBUG] No Steam launcher found, skipping usage check");
    }
    
    // Update usage counts in runners
    for runner in &mut runners {
        // Collect build titles and latest flags first to avoid borrow issues
        let build_titles: Vec<String> = runner.builds.iter().map(|b| b.title.clone()).collect();
        let is_latest_flags: Vec<bool> = runner.builds.iter().map(|b| b.is_latest).collect();
        
        for (build_idx, build) in runner.builds.iter_mut().enumerate() {
            // Check if this build's directory name matches any tool in use
            // We need to check all launchers and directory name formats
            let mut usage = 0u32;
            for launcher in &launchers {
                let format = runner.directory_name_formats.iter()
                    .find(|f| f.launcher == launcher.title)
                    .or_else(|| runner.directory_name_formats.iter().find(|f| f.launcher == "default"));
                
                if let Some(format) = format {
                    let dir_name = format_directory_name(&format.directory_name_format, &runner.title, &build.title);
                    // Check if this directory name (or variations) is in tool_usage
                    // Also check the original title
                    if let Some(&count) = tool_usage.get(&dir_name) {
                        usage += count;
                    }
                    if let Some(&count) = tool_usage.get(&build.title) {
                        usage += count;
                    }
                    // Check for "Latest" builds - they use the actual release name
                    if is_latest_flags[build_idx] && build_idx + 1 < build_titles.len() {
                        let actual_release = &build_titles[build_idx + 1]; // First non-latest build
                        let latest_dir_name = format_directory_name(&format.directory_name_format, &runner.title, actual_release);
                        if let Some(&count) = tool_usage.get(&latest_dir_name) {
                            usage += count;
                        }
                    }
                }
            }
            build.usage_count = usage;
            if usage > 0 {
                eprintln!("[DEBUG] Build {} {} has usage count: {}", runner.title, build.title, usage);
            }
        }
    }
    
    Ok(runners)
}

async fn remove_proton_build(
    runner_title: String,
    title: String,
    selected_launcher: Option<String>,
    runners: Vec<ProtonRunner>,
    launchers: Vec<DetectedLauncher>,
) -> Result<(String, String), String> {
    eprintln!("[DEBUG] ========== REMOVAL START ==========");
    eprintln!("[DEBUG] Runner: {}", runner_title);
    eprintln!("[DEBUG] Build: {}", title);
    eprintln!("[DEBUG] ====================================");
    
    // Get the selected launcher or use first detected
    let launcher_title = selected_launcher.as_ref().map(|s| s.as_str()).unwrap_or("Steam");
    eprintln!("[DEBUG] Removing from launcher: {}", launcher_title);
    
    let launcher = launchers.iter()
        .find(|l| l.title == launcher_title)
        .ok_or_else(|| format!("Launcher '{}' not found", launcher_title))?;
    
    // Get runner info
    let runner = runners.iter()
        .find(|r| r.title == runner_title)
        .ok_or_else(|| format!("Runner '{}' not found", runner_title))?;
    
    // Get directory name format for this launcher
    let format = runner.directory_name_formats.iter()
        .find(|f| f.launcher == launcher_title)
        .or_else(|| runner.directory_name_formats.iter().find(|f| f.launcher == "default"));
    
    let directory_name = if let Some(format) = format {
        format_directory_name(&format.directory_name_format, &runner_title, &title)
    } else {
        title.clone()
    };
    
    eprintln!("[DEBUG] Directory name: {}", directory_name);
    
    // Get the compatibility directory for this launcher and compat layer type
    let compat_dir = get_launcher_compat_directory_for_type(&launcher.title, &launcher.directory, &runner.compat_layer_type);
    let install_path = std::path::Path::new(&compat_dir).join(&directory_name);
    
    eprintln!("[DEBUG] Installation path: {} (type: {})", install_path.display(), runner.compat_layer_type);
    
    if !install_path.exists() {
        eprintln!("[DEBUG] ERROR: Installation not found at: {}", install_path.display());
        eprintln!("[DEBUG] Compat directory: {}", compat_dir);
        eprintln!("[DEBUG] Directory name: {}", directory_name);
        return Err(format!("Installation not found at: {}", install_path.display()));
    }
    
    // Check installation size before removal
    if let Ok(metadata) = install_path.metadata() {
        eprintln!("[DEBUG] Installation is directory: {}", metadata.is_dir());
        if let Ok(entries) = std::fs::read_dir(&install_path) {
            let count = entries.count();
            eprintln!("[DEBUG] Installation contains {} entries", count);
        }
    }
    
    eprintln!("[DEBUG] Removing directory: {}", install_path.display());
    let remove_start = std::time::Instant::now();
    std::fs::remove_dir_all(&install_path)
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to remove directory: {}", e);
            eprintln!("[DEBUG] Error type: {:?}", e);
            eprintln!("[DEBUG] Path: {}", install_path.display());
            if let Ok(metadata) = install_path.metadata() {
                eprintln!("[DEBUG] Permissions: {:?}", metadata.permissions());
            }
            format!("Failed to remove {}: {}. You may need to run with sudo.", directory_name, e)
        })?;
    let remove_duration = remove_start.elapsed();
    eprintln!("[DEBUG] Removal completed in {:.2}s", remove_duration.as_secs_f64());
    
    // Verify removal
    if install_path.exists() {
        eprintln!("[DEBUG] WARNING: Installation still exists after removal!");
    } else {
        eprintln!("[DEBUG] Removal verified: path no longer exists");
    }
    
    eprintln!("[DEBUG] ========== REMOVAL COMPLETE ==========");
    eprintln!("[DEBUG] Removed: {} {}", runner_title, title);
    eprintln!("[DEBUG] ======================================");
    Ok((runner_title, title))
}

async fn open_proton_directory(
    runner_title: String,
    title: String,
    selected_launcher: Option<String>,
    runners: Vec<ProtonRunner>,
    launchers: Vec<DetectedLauncher>,
) -> Result<(), String> {
    eprintln!("[DEBUG] open_proton_directory({}, {})", runner_title, title);
    
    // Get the selected launcher or use first detected
    let launcher_title = selected_launcher.as_ref().map(|s| s.as_str()).unwrap_or("Steam");
    
    let launcher = launchers.iter()
        .find(|l| l.title == launcher_title)
        .ok_or_else(|| format!("Launcher '{}' not found", launcher_title))?;
    
    // Get runner info
    let runner = runners.iter()
        .find(|r| r.title == runner_title)
        .ok_or_else(|| format!("Runner '{}' not found", runner_title))?;
    
    // Get directory name format for this launcher
    let format = runner.directory_name_formats.iter()
        .find(|f| f.launcher == launcher_title)
        .or_else(|| runner.directory_name_formats.iter().find(|f| f.launcher == "default"));
    
    let directory_name = if let Some(format) = format {
        format_directory_name(&format.directory_name_format, &runner_title, &title)
    } else {
        title.clone()
    };
    
    // Get the compatibility directory for this launcher and compat layer type
    let compat_dir = get_launcher_compat_directory_for_type(&launcher.title, &launcher.directory, &runner.compat_layer_type);
    let install_path = std::path::Path::new(&compat_dir).join(&directory_name);
    
    eprintln!("[DEBUG] Opening directory: {} (type: {})", install_path.display(), runner.compat_layer_type);
    
    if !install_path.exists() {
        return Err(format!("Directory not found: {}", install_path.display()));
    }
    
    // Open in file manager
    let path_str = install_path.to_string_lossy().to_string();
    let output = tokio::process::Command::new("xdg-open")
        .arg(&path_str)
        .output()
        .await
        .map_err(|e| format!("Failed to open file manager: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to open file manager".to_string());
    }
    
    Ok(())
}

async fn load_more_proton_builds(
    runner_title: String,
    endpoint: String,
    asset_position: usize,
    page: usize,
    directory_name_formats: Vec<DirectoryNameFormat>,
) -> Result<(String, Vec<ProtonBuild>), String> {
    eprintln!("[DEBUG] load_more_proton_builds({}, page {})", runner_title, page);
    
    let client = reqwest::Client::new();
    let url = format!("{}?per_page=25&page={}", endpoint, page);
    eprintln!("[DEBUG] Fetching from URL: {}", url);
    
    let response = client
        .get(&url)
        .header("User-Agent", "Rustora/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch releases: HTTP {}", response.status()));
    }
    
    let releases_json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse releases JSON: {}", e))?;
    
    let mut new_builds = Vec::new();
    
    if let Some(releases_array) = releases_json.as_array() {
        eprintln!("[DEBUG] Found {} releases on page {}", releases_array.len(), page);
        
        for release in releases_array {
            if let Some(tag_name) = release.get("tag_name").and_then(|v| v.as_str()) {
                if let Some(assets) = release.get("assets").and_then(|v| v.as_array()) {
                    if assets.len() > asset_position {
                        if let Some(asset) = assets.get(asset_position) {
                            if let Some(download_url) = asset.get("browser_download_url").and_then(|v| v.as_str()) {
                                let description = release.get("body")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let release_date = release.get("created_at")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let page_url = release.get("html_url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let download_size = asset.get("size")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                
                                new_builds.push(ProtonBuild {
                                    title: tag_name.to_string(),
                                    description,
                                    release_date,
                                    download_url: download_url.to_string(),
                                    page_url,
                                    download_size,
                                    runner_title: runner_title.clone(),
                                    is_installed: false,
                                    directory_name_formats: directory_name_formats.clone(),
                                    usage_count: 0,
                                    is_latest: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    eprintln!("[DEBUG] Loaded {} new builds", new_builds.len());
    Ok((runner_title, new_builds))
}

fn format_directory_name_for_steam(runner_title: &str, release_name: &str, formats: &[DirectoryNameFormat]) -> String {
    // Find Steam-specific format or use default
    let format = formats.iter()
        .find(|f| f.launcher == "Steam")
        .or_else(|| formats.iter().find(|f| f.launcher == "default"));
    
    if let Some(format) = format {
        format_directory_name(&format.directory_name_format, runner_title, release_name)
    } else {
        release_name.to_string()
    }
}

async fn load_steam_games() -> Result<Vec<SteamGame>, String> {
    eprintln!("[DEBUG] load_steam_games() called");
    
    // Detect Steam installation
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let steam_paths = vec![
        format!("{}/.local/share/Steam", home),
        format!("{}/.steam/steam", home),
        format!("{}/.steam/root", home),
        format!("{}/.steam/debian-installation", home),
        format!("{}/.var/app/com.valvesoftware.Steam/data/Steam", home),
        format!("/snap/steam/common/.steam/root"),
    ];
    
    let steam_dir = steam_paths.iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| "Steam installation not found".to_string())?;
    
    eprintln!("[DEBUG] Found Steam at: {}", steam_dir);
    
    let config_path = format!("{}/config/config.vdf", steam_dir);
    eprintln!("[DEBUG] Reading Steam config: {}", config_path);
    
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read Steam config: {}", e))?;
    
    // Parse games from config.vdf
    // We need to find the CompatToolMapping section and also get game names from libraryfolders.vdf
    let mut games = Vec::new();
    
    // First, get game names from libraryfolders.vdf
    let libraryfolders_path = format!("{}/steamapps/libraryfolders.vdf", steam_dir);
    let mut game_names: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    
    if let Ok(libraryfolders_content) = std::fs::read_to_string(&libraryfolders_path) {
        // Parse libraryfolders.vdf to get game names
        // Format: "apps"\n\t\t\t{\n\t\t\t\t"<appid>"\t\t"<name>"\n...
        let apps_start = "\"apps\"\n\t\t\t{";
        if let Some(start_pos) = libraryfolders_content.find(apps_start) {
            let start = start_pos + apps_start.len();
            let apps_end = "\n\t\t\t}";
            if let Some(end_pos) = libraryfolders_content[start..].find(apps_end) {
                let apps_content = &libraryfolders_content[start..start + end_pos];
                // Parse appid and name pairs
                let mut pos = 0;
                while pos < apps_content.len() {
                    // Find appid
                    if let Some(appid_start) = apps_content[pos..].find('"') {
                        let appid_start = pos + appid_start + 1;
                        if let Some(appid_end) = apps_content[appid_start..].find('"') {
                            let appid_str = &apps_content[appid_start..appid_start + appid_end];
                            if let Ok(appid) = appid_str.parse::<u32>() {
                                // Find name after appid
                                if let Some(name_start) = apps_content[appid_start + appid_end..].find('"') {
                                    let name_start = appid_start + appid_end + name_start + 1;
                                    if let Some(name_end) = apps_content[name_start..].find('"') {
                                        let name = apps_content[name_start..name_start + name_end].to_string();
                                        game_names.insert(appid, name);
                                        pos = name_start + name_end;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    pos += 1;
                }
            }
        }
    }
    
    // Parse CompatToolMapping from config.vdf
    let compat_tool_mapping_start = "\"CompatToolMapping\"\n\t\t\t\t{";
    let compat_tool_mapping_end = "\n\t\t\t\t}";
    
    if let Some(start_pos) = config_content.find(compat_tool_mapping_start) {
        let start = start_pos + compat_tool_mapping_start.len();
        if let Some(end_pos) = config_content[start..].find(&compat_tool_mapping_end) {
            let mapping_content = &config_content[start..start + end_pos];
            eprintln!("[DEBUG] Found CompatToolMapping section ({} bytes)", mapping_content.len());
            
            // Parse each game entry
            // Format: "\n\t\t\t\t\t\"<appid>\"\n\t\t\t\t\t{\n\t\t\t\t\t\t\"name\"\t\t\"<tool_name>\"\n...
            let mut pos = 0;
            while pos < mapping_content.len() {
                // Find appid
                if let Some(appid_start) = mapping_content[pos..].find("\n\t\t\t\t\t\"") {
                    let appid_start = pos + appid_start + "\n\t\t\t\t\t\"".len();
                    if let Some(appid_end) = mapping_content[appid_start..].find('"') {
                        let appid_str = &mapping_content[appid_start..appid_start + appid_end];
                        if let Ok(appid) = appid_str.parse::<u32>() {
                            // Find name field
                            if let Some(name_start) = mapping_content[appid_start + appid_end..].find("\"name\"\t\t\"") {
                                let name_start = appid_start + appid_end + name_start + "\"name\"\t\t\"".len();
                                if let Some(name_end) = mapping_content[name_start..].find('"') {
                                    let tool_name = mapping_content[name_start..name_start + name_end].to_string();
                                    let game_name = game_names.get(&appid)
                                        .cloned()
                                        .unwrap_or_else(|| format!("App {}", appid));
                                    
                                    games.push(SteamGame {
                                        name: game_name,
                                        appid,
                                        compatibility_tool: tool_name,
                                    });
                                    
                                    pos = name_start + name_end;
                                    continue;
                                }
                            }
                        }
                    }
                }
                pos += 1;
            }
        }
    }
    
    // Also add games that don't have compatibility tools set (from libraryfolders)
    for (appid, name) in game_names {
        if !games.iter().any(|g| g.appid == appid) {
            games.push(SteamGame {
                name,
                appid,
                compatibility_tool: "Undefined".to_string(),
            });
        }
    }
    
    // Sort by name
    games.sort_by(|a, b| a.name.cmp(&b.name));
    
    eprintln!("[DEBUG] Loaded {} Steam games", games.len());
    Ok(games)
}

async fn change_steam_game_compatibility_tool(
    appid: u32,
    compatibility_tool: String,
    steam_directory: Option<String>,
) -> Result<(u32, String), String> {
    eprintln!("[DEBUG] change_steam_game_compatibility_tool({}, {})", appid, compatibility_tool);
    
    // Detect Steam installation
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let steam_paths = vec![
        format!("{}/.local/share/Steam", home),
        format!("{}/.steam/steam", home),
        format!("{}/.steam/root", home),
        format!("{}/.steam/debian-installation", home),
        format!("{}/.var/app/com.valvesoftware.Steam/data/Steam", home),
        format!("/snap/steam/common/.steam/root"),
    ];
    
    let steam_dir = steam_directory
        .or_else(|| steam_paths.iter().find(|p| std::path::Path::new(p).exists()).cloned())
        .ok_or_else(|| "Steam installation not found".to_string())?;
    
    let config_path = format!("{}/config/config.vdf", steam_dir);
    eprintln!("[DEBUG] Modifying Steam config: {}", config_path);
    
    let mut config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read Steam config: {}", e))?;
    
    // Ensure CompatToolMapping section exists
    let compat_tool_mapping_start = "\"CompatToolMapping\"\n\t\t\t\t{";
    let _compat_tool_mapping_end = "\n\t\t\t\t}";
    
    if !config_content.contains(compat_tool_mapping_start) {
        // Add CompatToolMapping section
        let steam_start = "\"Steam\"\n\t\t\t{";
        if let Some(start_pos) = config_content.find(&steam_start) {
            let insert_pos = start_pos + steam_start.len();
            config_content.insert_str(insert_pos, "\n\t\t\t\t\"CompatToolMapping\"\n\t\t\t\t{\n\t\t\t\t}");
        } else {
            return Err("Could not find Steam section in config.vdf".to_string());
        }
    }
    
    // Find and modify the entry
    let start_text = "\"CompatToolMapping\"\n\t\t\t\t{";
    let end_text = "\n\t\t\t\t}";
    
    let start_pos = config_content.find(start_text)
        .ok_or_else(|| "CompatToolMapping section not found".to_string())?;
    let mapping_start = start_pos + start_text.len();
    let end_pos = config_content[mapping_start..].find(end_text)
        .ok_or_else(|| "CompatToolMapping section end not found".to_string())? + mapping_start;
    
    let mapping_content = &config_content[mapping_start..end_pos];
    
    let appid_str = appid.to_string();
    let appid_entry_start = format!("\n\t\t\t\t\t\"{}\"", appid_str);
    
    if mapping_content.contains(&appid_entry_start) {
        // Update existing entry
        if compatibility_tool == "Undefined" {
            // Remove the entry
            let entry_start = mapping_content.find(&appid_entry_start)
                .ok_or_else(|| "Game entry not found".to_string())?;
            let entry_end = mapping_content[entry_start + appid_entry_start.len()..].find("\n\t\t\t\t}")
                .ok_or_else(|| "Game entry end not found".to_string())? + entry_start + appid_entry_start.len() + "\n\t\t\t\t}".len();
            
            let full_entry = &mapping_content[entry_start..entry_end];
            config_content = config_content.replace(full_entry, "");
        } else {
            // Update the tool name
            let entry_start = mapping_content.find(&appid_entry_start)
                .ok_or_else(|| "Game entry not found".to_string())?;
            let entry_content = &mapping_content[entry_start..];
            
            let name_pattern = "\"name\"\t\t\"";
            if let Some(name_start) = entry_content.find(name_pattern) {
                let name_start = entry_start + name_start + name_pattern.len();
                if let Some(name_end) = config_content[name_start..].find('"') {
                    let old_tool = &config_content[name_start..name_start + name_end];
                    config_content = config_content.replace(
                        &format!("\"name\"\t\t\"{}\"", old_tool),
                        &format!("\"name\"\t\t\"{}\"", compatibility_tool)
                    );
                } else {
                    return Err("Invalid game entry format".to_string());
                }
            } else {
                return Err("Game entry missing name field".to_string());
            }
        }
    } else {
        // Add new entry
        if compatibility_tool == "Undefined" {
            return Ok((appid, "Undefined".to_string()));
        }
        
        let new_entry = format!(
            "\n\t\t\t\t\t\"{}\"\n\t\t\t\t\t{{\n\t\t\t\t\t\t\"name\"\t\t\"{}\"\n\t\t\t\t\t\t\"config\"\t\t\"\"\n\t\t\t\t\t\t\"priority\"\t\t\"250\"\n\t\t\t\t\t}}",
            appid_str, compatibility_tool
        );
        
        // Insert before the closing brace
        let insert_pos = end_pos - "\n\t\t\t\t}".len();
        config_content.insert_str(insert_pos, &new_entry);
    }
    
    // Write back to file
    std::fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write Steam config: {}. You may need to close Steam first.", e))?;
    
    eprintln!("[DEBUG] Successfully updated compatibility tool for appid {}", appid);
    Ok((appid, compatibility_tool))
}
