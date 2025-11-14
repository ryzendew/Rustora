use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding, Border, Color};
use crate::gui::app::CustomScrollableStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelView {
    Kernels,
    Scheduler,
}
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::text_input::Appearance as TextInputAppearance;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use tokio::process::Command as TokioCommand;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
pub enum Message {
    LoadBranches,
    BranchesLoaded(Vec<KernelBranch>),
    BranchSelected(String),
    KernelsLoaded(Vec<EnhancedKernelInfo>),
    SearchQueryChanged(String),
    KernelSelected(String),
    KernelDetailsLoaded(KernelDetails),
    ClosePanel,
    InstallKernel(String),
    InstallKernelComplete(Result<String, String>),
    RemoveKernel(String),
    RemoveKernelComplete(Result<String, String>),
    RunningKernelInfoLoaded(RunningKernelInfo, Option<String>, u32),
    StoreBranchDbAndLoadKernels(String, String, Vec<EnhancedKernelInfo>),
    SwitchView(KernelView),
    #[allow(dead_code)]
    OpenSchedulerConfig,
    SchedulersLoaded(Vec<ScxScheduler>, String),
    SchedulerSelected(String),
    SchedulerModeSelected(String),
    SchedulerFlagsChanged(String),
    ApplyScheduler,
    Error(()),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelBranch {
    pub name: String,
    pub db_url: String,
    pub init_script: String,
    #[serde(skip)]
    pub db: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelBranchDb {
    #[serde(rename = "latest_kernel_version_deter_pkg")]
    pub latest_kernel_version_deter_pkg: Option<String>,
    pub kernels: Vec<KernelPackageEntry>,
}

// Custom deserializer for min_x86_march that handles both string and number
fn deserialize_min_x86_march<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct MinX86MarchVisitor;

    impl<'de> Visitor<'de> for MinX86MarchVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number representing x86 micro-architecture level")
        }

        fn visit_str<E>(self, value: &str) -> Result<u32, E>
        where
            E: de::Error,
        {
            value.parse::<u32>().map_err(|_| {
                E::custom(format!("failed to parse '{}' as u32", value))
            })
        }

        fn visit_u64<E>(self, value: u64) -> Result<u32, E>
        where
            E: de::Error,
        {
            Ok(value as u32)
        }

        fn visit_i64<E>(self, value: i64) -> Result<u32, E>
        where
            E: de::Error,
        {
            if value < 0 {
                return Err(E::custom("min_x86_march cannot be negative"));
            }
            Ok(value as u32)
        }
    }

    deserializer.deserialize_any(MinX86MarchVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelPackageEntry {
    pub name: String,
    #[serde(rename = "main_package")]
    pub main_package: String,
    pub packages: String,
    #[serde(rename = "min_x86_march", deserialize_with = "deserialize_min_x86_march")]
    pub min_x86_march: u32,
}

#[derive(Debug, Clone)]
pub struct EnhancedKernelInfo {
    pub name: String,
    pub main_package: String,
    pub packages: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
    #[allow(dead_code)]
    pub branch: String,
    pub min_x86_march: u32,
}

#[derive(Debug, Clone)]
pub struct KernelDetails {
    pub name: String,
    pub version: String,
    #[allow(dead_code)]
    pub release: String,
    #[allow(dead_code)]
    pub arch: String,
    #[allow(dead_code)]
    pub repository: String,
    #[allow(dead_code)]
    pub installed: bool,
    pub summary: String,
    pub description: String,
    #[allow(dead_code)]
    pub size: String,
    #[allow(dead_code)]
    pub build_date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunningKernelInfo {
    #[allow(dead_code)]
    pub kernel: String,
    #[allow(dead_code)]
    pub version: String,
    pub scheduler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScxScheduler {
    pub name: String,
    pub modes: Vec<ScxSchedulerMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScxSchedulerMode {
    pub name: String,
    pub flags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScxSchedulers {
    pub scx_schedulers: Vec<ScxScheduler>,
}

#[derive(Debug)]
pub struct KernelTab {
    branches: Vec<KernelBranch>,
    selected_branch: Option<String>,
    kernels: Vec<EnhancedKernelInfo>,
    filtered_kernels: Vec<EnhancedKernelInfo>,
    search_query: String,
    is_loading: bool,
    is_loading_branches: bool,
    selected_kernel: Option<String>,
    kernel_details: Option<KernelDetails>,
    panel_open: bool,
    installing_kernels: std::collections::HashSet<String>,
    removing_kernels: std::collections::HashSet<String>,
    running_kernel_info: Option<RunningKernelInfo>,
    latest_version: Option<String>,
    cpu_feature_level: u32,
    // View state
    current_view: KernelView,
    // SCX Scheduler state
    scx_schedulers: Vec<ScxScheduler>,
    selected_scheduler: Option<String>,
    selected_scheduler_mode: Option<String>,
    scheduler_extra_flags: String,
    current_scheduler: Option<String>,
}

impl KernelTab {
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            selected_branch: None,
            kernels: Vec::new(),
            filtered_kernels: Vec::new(),
            search_query: String::new(),
            is_loading: false,
            is_loading_branches: false,
            selected_kernel: None,
            kernel_details: None,
            panel_open: false,
            installing_kernels: std::collections::HashSet::new(),
            removing_kernels: std::collections::HashSet::new(),
            running_kernel_info: None,
            latest_version: None,
            cpu_feature_level: 1,
            current_view: KernelView::Kernels,
            scx_schedulers: Vec::new(),
            selected_scheduler: None,
            selected_scheduler_mode: None,
            scheduler_extra_flags: String::new(),
            current_scheduler: None,
        }
    }

    fn filter_kernels(&mut self) {
        if self.search_query.trim().is_empty() {
            self.filtered_kernels = self.kernels.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_kernels = self.kernels
                .iter()
                .filter(|kernel| {
                    kernel.name.to_lowercase().contains(&query_lower) ||
                    kernel.main_package.to_lowercase().contains(&query_lower) ||
                    kernel.version.to_lowercase().contains(&query_lower) ||
                    kernel.description.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect();
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::LoadBranches => {
                self.is_loading_branches = true;
                iced::Command::perform(load_kernel_branches(), |result| {
                    match result {
                        Ok(branches) => {
                            Message::BranchesLoaded(branches)
                        }
                        Err(_e) => {
                            Message::Error(())
                        }
                    }
                })
            }
            Message::BranchesLoaded(branches) => {
                self.is_loading_branches = false;
                self.branches = branches.clone();
                // Detect which branch the running kernel belongs to and auto-select it
                if self.selected_branch.is_none() {
                    let branches_clone = branches.clone();
                    let first_branch_name = branches.first().map(|b| b.name.clone());
                    return iced::Command::perform(
                        detect_running_kernel_branch(branches_clone),
                        move |branch_name| {
                            if let Some(branch_name) = branch_name {
                                Message::BranchSelected(branch_name)
                            } else {
                                // Fallback to first branch if detection fails
                                if let Some(first_name) = first_branch_name {
                                    Message::BranchSelected(first_name)
                                } else {
                                    Message::Error(())
                                }
                            }
                        }
                    );
                }
                iced::Command::none()
            }
            Message::BranchSelected(branch_name) => {
                self.selected_branch = Some(branch_name.clone());
                self.is_loading = true;
                let branches_clone = self.branches.clone();
                let branch_name_for_db = branch_name.clone();
                iced::Command::perform(
                    select_branch_and_load_kernels(branch_name.clone(), branches_clone),
                    move |result| match result {
                        Ok((_branch, kernels, _info, _latest, db_content)) => {
                            Message::StoreBranchDbAndLoadKernels(branch_name_for_db.clone(), db_content, kernels)
                        }
                        Err(_e) => {
                            Message::Error(())
                        }
                    }
                )
            }
            Message::StoreBranchDbAndLoadKernels(branch_name, db_content, kernels) => {
                if let Some(branch) = self.branches.iter_mut().find(|b| b.name == branch_name) {
                    branch.db = Some(db_content);
                }
                self.is_loading = false;
                self.kernels = kernels;
                self.filter_kernels();
                // Load running kernel info and latest version
                let branch_name_for_info = self.selected_branch.clone();
                let branches_clone = self.branches.clone();
                iced::Command::perform(
                    async move {
                        let (info, cpu_level) = tokio::join!(
                            get_running_kernel_info(),
                            get_cpu_feature_level()
                        );
                        // Get latest version from selected branch
                        let latest = if let Some(ref branch_name) = branch_name_for_info {
                            if let Some(branch) = branches_clone.iter().find(|b| b.name == *branch_name) {
                                if let Some(ref db) = branch.db {
                                    if let Ok(db_json) = serde_json::from_str::<KernelBranchDb>(db) {
                                        if let Some(ref pkg) = db_json.latest_kernel_version_deter_pkg {
                                            get_package_version(pkg).await.ok()
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        (info, latest, cpu_level)
                    },
                    |(info, latest, cpu_level)| Message::RunningKernelInfoLoaded(info, latest, cpu_level),
                )
            }
            Message::KernelsLoaded(kernels) => {
                self.is_loading = false;
                self.kernels = kernels;
                self.filter_kernels();
                // Load running kernel info and latest version
                let branch_name = self.selected_branch.clone();
                let branches = self.branches.clone();
                iced::Command::perform(
                    async move {
                        let (info, cpu_level) = tokio::join!(
                            get_running_kernel_info(),
                            get_cpu_feature_level()
                        );
                        // Get latest version from selected branch
                        let latest = if let Some(ref branch_name) = branch_name {
                            if let Some(branch) = branches.iter().find(|b| b.name == *branch_name) {
                                if let Some(ref db) = branch.db {
                                    if let Ok(db_json) = serde_json::from_str::<KernelBranchDb>(db) {
                                        if let Some(ref pkg) = db_json.latest_kernel_version_deter_pkg {
                                            get_package_version(pkg).await.ok()
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        (info, latest, cpu_level)
                    },
                    |(info, latest, cpu_level)| Message::RunningKernelInfoLoaded(info, latest, cpu_level),
                )
            }
            Message::RunningKernelInfoLoaded(info, latest, cpu_level) => {
                self.running_kernel_info = Some(info);
                self.latest_version = latest;
                self.cpu_feature_level = cpu_level;
                // Reload details if panel is open
                if let Some(ref selected) = self.selected_kernel {
                    return iced::Command::perform(
                        load_kernel_details(selected.clone()),
                        Message::KernelDetailsLoaded
                    );
                }
                iced::Command::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.filter_kernels();
                iced::Command::none()
            }
            Message::KernelSelected(name) => {
                self.selected_kernel = Some(name.clone());
                self.panel_open = true;
                iced::Command::perform(load_kernel_details(name), Message::KernelDetailsLoaded)
            }
            Message::KernelDetailsLoaded(details) => {
                self.kernel_details = Some(details);
                iced::Command::none()
            }
            Message::ClosePanel => {
                self.panel_open = false;
                self.selected_kernel = None;
                self.kernel_details = None;
                iced::Command::none()
            }
            Message::InstallKernel(kernel_name) => {
                self.installing_kernels.insert(kernel_name.clone());
                // Find the kernel to get its packages
                if let Some(kernel) = self.kernels.iter().find(|k| k.name == kernel_name || k.main_package == kernel_name) {
                    let packages = kernel.packages.clone();
                    iced::Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                            TokioCommand::new(&exe_path)
                                .arg("kernel-install-dialog")
                                .arg(&packages)
                                .spawn()
                                .ok();
                        },
                        |_| Message::InstallKernelComplete(Ok("Dialog opened".to_string())),
                    )
                } else {
                    // Fallback to old behavior
                    iced::Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                            TokioCommand::new(&exe_path)
                                .arg("kernel-install-dialog")
                                .arg(&kernel_name)
                                .spawn()
                                .ok();
                        },
                        |_| Message::InstallKernelComplete(Ok("Dialog opened".to_string())),
                    )
                }
            }
            Message::InstallKernelComplete(result) => {
                    if let Ok(_) = result {
                    self.installing_kernels.clear();
                    if let Some(ref branch) = self.selected_branch {
                        let branch_name = branch.clone();
                        let branches_clone = self.branches.clone();
                        return iced::Command::perform(
                            select_branch_and_load_kernels(branch_name, branches_clone),
                            |result| match result {
                                Ok((_, kernels, _, _, _)) => Message::KernelsLoaded(kernels),
                                Err(_) => Message::Error(()),
                            }
                        );
                    }
                }
                self.installing_kernels.clear();
                iced::Command::none()
            }
            Message::RemoveKernel(kernel_name) => {
                self.removing_kernels.insert(kernel_name.clone());
                // Find the kernel to get its packages
                if let Some(kernel) = self.kernels.iter().find(|k| k.name == kernel_name || k.main_package == kernel_name) {
                    let packages = kernel.packages.clone();
                    iced::Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                            TokioCommand::new(&exe_path)
                                .arg("kernel-remove-dialog")
                                .arg(&packages)
                                .spawn()
                                .ok();
                        },
                        |_| Message::RemoveKernelComplete(Ok("Dialog opened".to_string())),
                    )
                } else {
                    iced::Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                            TokioCommand::new(&exe_path)
                                .arg("kernel-remove-dialog")
                                .arg(&kernel_name)
                                .spawn()
                                .ok();
                        },
                        |_| Message::RemoveKernelComplete(Ok("Dialog opened".to_string())),
                    )
                }
            }
            Message::RemoveKernelComplete(result) => {
                    if let Ok(_) = result {
                    self.removing_kernels.clear();
                    if let Some(ref branch) = self.selected_branch {
                        let branch_name = branch.clone();
                        let branches_clone = self.branches.clone();
                        return iced::Command::perform(
                            select_branch_and_load_kernels(branch_name, branches_clone),
                            |result| match result {
                                Ok((_, kernels, _, _, _)) => Message::KernelsLoaded(kernels),
                                Err(_) => Message::Error(()),
                            }
                        );
                    }
                }
                self.removing_kernels.clear();
                iced::Command::none()
            }
            Message::SwitchView(view) => {
                self.current_view = view;
                if view == KernelView::Scheduler {
                    // Load schedulers when switching to scheduler view
                    iced::Command::perform(
                        async {
                            let (schedulers, current_info) = tokio::join!(
                                load_scx_schedulers(),
                                get_running_kernel_info()
                            );
                            (schedulers, current_info.scheduler)
                        },
                        |result| match result {
                            (Ok(schedulers), current) => {
                                Message::SchedulersLoaded(schedulers, current)
                            }
                            _ => Message::Error(()),
                        }
                    )
                } else {
                    iced::Command::none()
                }
            }
            Message::OpenSchedulerConfig => {
                self.current_view = KernelView::Scheduler;
                iced::Command::perform(
                    async {
                        let (schedulers, current_info) = tokio::join!(
                            load_scx_schedulers(),
                            get_running_kernel_info()
                        );
                        (schedulers, current_info.scheduler)
                    },
                    |result| match result {
                        (Ok(schedulers), current) => {
                            Message::SchedulersLoaded(schedulers, current)
                        }
                        _ => Message::Error(()),
                    }
                )
            }
            Message::SchedulersLoaded(schedulers, current) => {
                self.scx_schedulers = schedulers;
                // Store the full current scheduler text for display (includes full info)
                self.current_scheduler = Some(current.clone());
                // Extract scheduler name from "sched_ext: scheduler_name" format for selection
                let current_clean = if current.starts_with("sched_ext: ") {
                    current.strip_prefix("sched_ext: ").unwrap_or(&current).to_string()
                } else {
                    current.clone()
                };
                self.selected_scheduler = Some(current_clean);
                iced::Command::none()
            }
            Message::SchedulerSelected(scheduler_name) => {
                self.selected_scheduler = Some(scheduler_name);
                iced::Command::none()
            }
            Message::SchedulerModeSelected(mode_name) => {
                let mode_name_clone = mode_name.clone();
                self.selected_scheduler_mode = Some(mode_name);
                // When a mode is selected, update the flags from the mode
                if let Some(ref scheduler) = self.selected_scheduler {
                    if let Some(sched) = self.scx_schedulers.iter().find(|s| &s.name == scheduler) {
                        if let Some(mode) = sched.modes.iter().find(|m| m.name == mode_name_clone) {
                            self.scheduler_extra_flags = mode.flags.clone();
                        }
                    }
                }
                iced::Command::none()
            }
            Message::SchedulerFlagsChanged(flags) => {
                self.scheduler_extra_flags = flags;
                iced::Command::none()
            }
            Message::ApplyScheduler => {
                // Apply scheduler change
                if let Some(ref scheduler) = self.selected_scheduler {
                    let scheduler_name = scheduler.clone();
                    let flags = self.scheduler_extra_flags.clone();
                    let schedulers_clone = self.scx_schedulers.clone();
                    iced::Command::perform(
                        async move {
                            let result = apply_scx_scheduler(scheduler_name.clone(), flags).await;
                            if result.is_ok() {
                                // Wait a bit longer for the scheduler to actually apply
                                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                // Try reading the scheduler multiple times in case it takes a moment
                                let mut info = get_running_kernel_info().await;
                                // Retry a few times to ensure we get the updated scheduler
                                for _ in 0..3 {
                                    if info.scheduler.contains("sched_ext") {
                                        break;
                                    }
                                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                    info = get_running_kernel_info().await;
                                }
                                Ok((schedulers_clone, info.scheduler))
                            } else {
                                let err_msg = result.as_ref().unwrap_err().clone();
                                Err(format!("Failed to apply scheduler: {}", err_msg))
                            }
                        },
                        |result| match result {
                            Ok((schedulers, current)) => Message::SchedulersLoaded(schedulers, current),
                            Err(msg) => {
                                eprintln!("SCX apply error: {}", msg);
                                Message::Error(())
                            },
                        }
                    )
                } else {
                    iced::Command::none()
                }
            }
            Message::Error(()) => {
                self.is_loading = false;
                iced::Command::none()
            }
        }
    }

    pub fn view<'a>(&'a self, theme: &'a crate::gui::Theme, settings: &'a crate::gui::settings::AppSettings) -> Element<'a, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        
        // Calculate font sizes from settings
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let _button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let input_font_size = (settings.font_size_inputs * settings.scale_inputs).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let tab_font_size = (settings.font_size_tabs * settings.scale_tabs).round();
        
        let header = container(
            row![
                text("Kernel Manager")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary()))
                    .width(Length::Fill),
                button(
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL)
                        .font(material_font)
                        .size(icon_size)
                )
                .style(iced::theme::Button::Custom(Box::new(RefreshButtonStyle {
                    radius: settings.border_radius,
                })))
                .on_press(Message::LoadBranches)
                .padding(Padding::new(12.0)),
            ]
            .align_items(Alignment::Center)
            .spacing(16)
        )
        .width(Length::Fill)
        .padding(Padding::new(32.0));

        // Sub-tabs for Kernels and Scheduler
        let sub_tabs = container(
            row![
                button(
                    text("Kernels")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == KernelView::Kernels {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == KernelView::Kernels,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(KernelView::Kernels))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
                button(
                    text("Scheduler")
                        .size(tab_font_size)
                        .style(iced::theme::Text::Color(if self.current_view == KernelView::Scheduler {
                            iced::Color::WHITE
                        } else {
                            theme.text()
                        }))
                )
                .style(iced::theme::Button::Custom(Box::new(SubTabButtonStyle {
                    is_active: self.current_view == KernelView::Scheduler,
                    radius: settings.border_radius,
                })))
                .on_press(Message::SwitchView(KernelView::Scheduler))
                .padding(Padding::from([12.0, 24.0, 12.0, 24.0])),
            ]
            .spacing(12)
        )
        .width(Length::Fill)
        .padding(Padding::from([0.0, 32.0, 20.0, 32.0]));

        // Running kernel info - removed from header to save space

        // Branch selector
        let branch_selector = if !self.branches.is_empty() {
            container(
                row![
                    text("Branch:")
                        .size(body_font_size * 1.29)
                        .style(iced::theme::Text::Color(theme.primary())),
                    Space::with_width(Length::Fixed(16.0)),
                    scrollable(
                        row(
                            self.branches
                                .iter()
                                .map(|branch| {
                                    let branch_name = branch.name.clone();
                                    let is_selected = self.selected_branch.as_ref().map(|s| s == &branch.name).unwrap_or(false);
                                    button(
                                        text(&branch.name)
                                            .size(body_font_size * 1.07)
                                    )
                                    .style(iced::theme::Button::Custom(Box::new(BranchButtonStyle {
                                        is_selected,
                                        radius: settings.border_radius,
                                    })))
                                    .on_press(Message::BranchSelected(branch_name))
                                    .padding(Padding::from([10.0, 20.0, 10.0, 20.0]))
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(12)
                    )
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                        Color::from(settings.background_color.clone()),
                        settings.border_radius,
                    ))))
                ]
                .align_items(Alignment::Center)
                .spacing(0)
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            })))
        } else {
            container(Space::with_height(Length::Shrink))
        };

        let search_bar = container(
            row![
                text(crate::gui::fonts::glyphs::SEARCH_SYMBOL)
                    .font(material_font)
                    .size(icon_size * 1.22)
                    .style(iced::theme::Text::Color(theme.text())),
                text_input("Search kernels...", &self.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .style(iced::theme::TextInput::Custom(Box::new(SearchInputStyle {
                        radius: settings.border_radius,
                    })))
                    .width(Length::Fill)
                    .padding(Padding::new(16.0))
                    .size(input_font_size),
            ]
            .align_items(Alignment::Center)
            .spacing(16)
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0))
        .style(iced::theme::Container::Custom(Box::new(SearchContainerStyle {
            radius: settings.border_radius,
        })));

        let content: Element<Message> = if self.is_loading || self.is_loading_branches {
            container(
                column![
                    Space::with_height(Length::Fill),
                    text(if self.is_loading_branches { "Loading branches..." } else { "Loading kernels..." })
                        .size(body_font_size * 1.29)
                        .style(iced::theme::Text::Color(theme.text()))
                        .width(Length::Shrink),
                    Space::with_height(Length::Fill),
                ]
                .align_items(Alignment::Center)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.filtered_kernels.is_empty() {
            let message = if self.search_query.is_empty() {
                "No kernels found".to_string()
            } else {
                format!("No kernels found matching '{}'", self.search_query)
            };
            container(
                column![
                    Space::with_height(Length::Fill),
                    text(message)
                        .size(body_font_size * 1.29)
                        .style(iced::theme::Text::Color(theme.text()))
                        .width(Length::Shrink),
                    Space::with_height(Length::Fill),
                ]
                .align_items(Alignment::Center)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            let kernel_list: Element<Message> = scrollable(
                column(
                    self.filtered_kernels
                        .iter()
                        .map(|kernel| {
                            self.view_kernel_item(kernel, theme, &material_font, settings)
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(12)
                .padding(Padding::new(16.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                Color::from(settings.background_color.clone()),
                settings.border_radius,
            ))))
            .into();

            if self.panel_open {
                row![
                    container(kernel_list)
                        .width(Length::FillPortion(2))
                        .height(Length::Fill),
                    container(self.view_panel(theme, &material_font, settings))
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                ]
                .spacing(12)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                container(kernel_list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        };

        let main_content: Element<Message> = match self.current_view {
            KernelView::Kernels => {
                column![
                    branch_selector,
                    search_bar,
                    content,
                ]
                .spacing(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            KernelView::Scheduler => {
                self.view_scheduler(theme, &material_font, settings)
            }
        };

        container(
            column![
                header,
                sub_tabs,
                main_content,
            ]
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0))
        .into()
    }

    fn view_kernel_item(&self, kernel: &EnhancedKernelInfo, theme: &crate::gui::Theme, material_font: &iced::Font, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        // Calculate font sizes from settings
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let package_name_size = (settings.font_size_package_names * settings.scale_package_cards).round();
        let package_detail_size = (settings.font_size_package_details * settings.scale_package_cards).round();
        // Status badge
        let status_badge = if kernel.installed {
            container(
                text("INSTALLED")
                    .size(body_font_size * 0.71)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.7, 0.0)))
            )
            .padding(Padding::from([4.0, 8.0, 4.0, 8.0]))
            .style(iced::theme::Container::Custom(Box::new(InstalledBadgeStyle {
                radius: settings.border_radius,
            })))
        } else {
            container(
                text("AVAILABLE")
                    .size(body_font_size * 0.71)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
            )
            .padding(Padding::from([4.0, 8.0, 4.0, 8.0]))
            .style(iced::theme::Container::Custom(Box::new(AvailableBadgeStyle {
                radius: settings.border_radius,
            })))
        };

        // CPU feature requirement badge
        let cpu_badge: Element<Message> = if kernel.min_x86_march > 1 {
            let level_text = match kernel.min_x86_march {
                2 => "x86-64-v2",
                3 => "x86-64-v3",
                4 => "x86-64-v4",
                _ => "",
            };
            container(
                text(level_text)
                    .size(body_font_size * 0.64)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.6, 0.2)))
            )
            .padding(Padding::from([3.0, 7.0, 3.0, 7.0]))
            .style(iced::theme::Container::Custom(Box::new(CpuBadgeStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else {
            Space::with_width(Length::Shrink).into()
        };

        // Action button
        let action_button = if kernel.installed {
            button(
                row![
                    text(crate::gui::fonts::glyphs::DELETE_SYMBOL)
                        .font(*material_font)
                        .size(icon_size * 0.78),
                    text("Remove")
                        .size(button_font_size)
                ]
                .spacing(6)
                .align_items(Alignment::Center)
            )
            .style(iced::theme::Button::Custom(Box::new(RemoveButtonStyle {
                radius: settings.border_radius,
            })))
            .on_press(Message::RemoveKernel(kernel.main_package.clone()))
            .padding(Padding::from([10.0, 16.0, 10.0, 16.0]))
        } else {
            let is_installing = self.installing_kernels.contains(&kernel.main_package);
            button(
                row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL)
                        .font(*material_font)
                        .size(icon_size * 0.78),
                    text(if is_installing { "Installing..." } else { "Install" })
                        .size(button_font_size)
                ]
                .spacing(6)
                .align_items(Alignment::Center)
            )
            .style(iced::theme::Button::Custom(Box::new(InstallButtonStyle {
                radius: settings.border_radius,
            })))
            .on_press(if is_installing {
                Message::Error(())
            } else {
                Message::InstallKernel(kernel.main_package.clone())
            })
            .padding(Padding::from([10.0, 16.0, 10.0, 16.0]))
        };

        // Extract vendor/repository info from branch
        let vendor_text = if kernel.branch.contains("Copr") || kernel.branch.contains("copr") {
            // Try to extract user/repo from branch string
            if let Some(copr_part) = kernel.branch.split("Copr").nth(1) {
                format!("Fedora Copr{}", copr_part.split(')').next().unwrap_or(""))
            } else if kernel.branch.contains("bieszczaders") {
                "Fedora Copr - user bieszczaders".to_string()
            } else {
                kernel.branch.clone()
            }
        } else {
            kernel.branch.clone()
        };

        // Professional card layout - wrapped in button for click-to-view-details
        button(
            container(
                column![
                    // Header row: Title (left), Badges + Action button (right)
                    row![
                        // Left: Title
                        text(&kernel.name)
                            .size(package_name_size)
                            .style(iced::theme::Text::Color(theme.primary()))
                            .width(Length::Fill),
                        // Right: Badges and action button
                        row![
                            status_badge,
                            cpu_badge,
                            action_button,
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                    ]
                    .align_items(Alignment::Center)
                    .width(Length::Fill)
                    .spacing(12),
                    Space::with_height(Length::Fixed(12.0)),
                    // Description
                    text(&kernel.description)
                        .size(package_detail_size)
                        .style(iced::theme::Text::Color(theme.text()))
                        .width(Length::Fill)
                        .shaping(iced::widget::text::Shaping::Advanced),
                    Space::with_height(Length::Fixed(12.0)),
                    // Metadata row: Package, Version, Vendor (aligned with fixed widths)
                    row![
                        // Package
                        column![
                            text("Package")
                                .size(package_detail_size * 0.77)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            Space::with_height(Length::Fixed(2.0)),
                            text(&kernel.main_package)
                                .size(package_detail_size)
                                .style(iced::theme::Text::Color(theme.text())),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(2)),
                        // Version
                        column![
                            text("Version")
                                .size(package_detail_size * 0.77)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            Space::with_height(Length::Fixed(2.0)),
                            text(&kernel.version)
                                .size(package_detail_size)
                                .style(iced::theme::Text::Color(theme.text())),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(2)),
                        // Vendor/Repository
                        column![
                            text("Vendor")
                                .size(package_detail_size * 0.77)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                            Space::with_height(Length::Fixed(2.0)),
                            text(&vendor_text)
                                .size(package_detail_size)
                                .style(iced::theme::Text::Color(theme.text()))
                                .shaping(iced::widget::text::Shaping::Advanced),
                        ]
                        .spacing(0)
                        .width(Length::FillPortion(3)),
                    ]
                    .spacing(16)
                    .align_items(Alignment::Start)
                    .width(Length::Fill),
                ]
                .spacing(0)
                .padding(Padding::new(20.0))
            )
            .style(iced::theme::Container::Custom(Box::new(KernelItemStyle {
                radius: settings.border_radius,
            })))
            .width(Length::Fill)
        )
        .on_press(Message::KernelSelected(kernel.name.clone()))
        .style(iced::theme::Button::Text)
        .padding(0)
        .width(Length::Fill)
        .into()
    }

    fn view_panel(&self, theme: &crate::gui::Theme, material_font: &iced::Font, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        // Calculate font sizes from settings
        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let _button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        let _package_name_size = (settings.font_size_package_names * settings.scale_package_cards).round();
        let _package_detail_size = (settings.font_size_package_details * settings.scale_package_cards).round();
        if let Some(ref details) = self.kernel_details {
            container(
                column![
                    row![
                        text("Kernel Details")
                            .size(title_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.primary()))
                            .width(Length::Fill),
                        button(
                            text(crate::gui::fonts::glyphs::CLOSE_SYMBOL)
                                .font(*material_font)
                                .size(icon_size * 1.33)
                        )
                        .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle {
                            radius: settings.border_radius,
                        })))
                        .on_press(Message::ClosePanel)
                        .padding(Padding::new(12.0)),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(16),
                    Space::with_height(Length::Fixed(20.0)),
                    scrollable(
                        column![
                            container(
                                column![
                                    text("Name")
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                    Space::with_height(Length::Fixed(6.0)),
                                    text(&details.name)
                                        .size(body_font_size * 1.29),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(20.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                            container(
                                column![
                                    text("Version")
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(theme.text())),
                                    Space::with_height(Length::Fixed(6.0)),
                                    text(&details.version)
                                        .size(body_font_size * 1.29)
                                        .style(iced::theme::Text::Color(theme.text())),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(20.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                            container(
                                column![
                                    text("Summary")
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(theme.text())),
                                    Space::with_height(Length::Fixed(6.0)),
                                    text(&details.summary)
                                        .size(body_font_size * 1.14)
                                        .style(iced::theme::Text::Color(theme.text())),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(20.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                            container(
                                column![
                                    text("Description")
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(theme.text())),
                                    Space::with_height(Length::Fixed(6.0)),
                                    text(&details.description)
                                        .size(body_font_size)
                                        .style(iced::theme::Text::Color(theme.text())),
                                ]
                                .spacing(0)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(20.0))
                            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                        ]
                        .spacing(16)
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
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(PanelStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else {
            container(Space::with_height(Length::Shrink))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn view_scheduler<'a>(&'a self, theme: &'a crate::gui::Theme, material_font: &iced::Font, settings: &'a crate::gui::settings::AppSettings) -> Element<'a, Message> {
        // Calculate font sizes from settings
        let _title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let input_font_size = (settings.font_size_inputs * settings.scale_inputs).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();
        // Get current scheduler info - prefer stored current_scheduler, fallback to running_kernel_info
        let current_sched_text = if let Some(ref current) = self.current_scheduler {
            current.clone()
        } else if let Some(ref info) = self.running_kernel_info {
            info.scheduler.clone()
        } else {
            "Unknown".to_string()
        };
        
        // For comparison, extract just the scheduler name (without "sched_ext: " prefix and arguments)
        // Normalize by removing "scx_" prefix for comparison
        let current_sched_name_for_comparison = if current_sched_text.starts_with("sched_ext: ") {
            let name = current_sched_text
                .strip_prefix("sched_ext: ")
                .unwrap_or(&current_sched_text)
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();
            // Remove "scx_" prefix if present for comparison
            name.strip_prefix("scx_").unwrap_or(&name).to_string()
        } else {
            let name = current_sched_text.split_whitespace().next().unwrap_or("").to_lowercase();
            name.strip_prefix("scx_").unwrap_or(&name).to_string()
        };
        
        // Extract current flags from the scheduler text for mode matching
        // Format: "sched_ext: scx_bpfland with arguments "-k -s 5000 -l 5000""
        let current_flags = if current_sched_text.contains("with arguments") {
            // Extract the flags part after "with arguments"
            let flags_part = current_sched_text
                .split("with arguments")
                .nth(1)
                .unwrap_or("")
                .trim();
            
            // Remove quotes (both single and double) and trim
            flags_part
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| flags_part.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(flags_part)
                .trim()
                .to_string()
        } else {
            String::new()
        };

        let scheduler_list: Element<Message> = if self.scx_schedulers.is_empty() {
            container(
                column![
                    Space::with_height(Length::Fixed(100.0)),
                    text("Loading schedulers...")
                        .size(body_font_size * 1.14)
                        .style(iced::theme::Text::Color(theme.secondary_text()))
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            })))
            .into()
        } else {
            scrollable(
                column(
                    self.scx_schedulers
                        .iter()
                        .map(|scheduler| {
                            let scheduler_name = scheduler.name.clone();
                            let is_selected = self.selected_scheduler.as_ref().map(|s| s == &scheduler_name).unwrap_or(false);
                            // Check if this scheduler is currently active
                            // Normalize scheduler names for comparison (handle both "scx_bpfland" and "bpfland")
                            let scheduler_base = scheduler_name.strip_prefix("scx_").unwrap_or(&scheduler_name).to_lowercase();
                            
                            let is_current = if current_sched_text.contains("sched_ext") {
                                // SCX scheduler is running - compare base names
                                // current_sched_name_for_comparison is already normalized (lowercase, no prefix)
                                let current_base = current_sched_name_for_comparison.strip_prefix("scx_").unwrap_or(&current_sched_name_for_comparison);
                                current_base == scheduler_base
                            } else {
                                // No SCX scheduler running - only scx_disabled should be marked as current
                                scheduler_name == "scx_disabled"
                            };
                            
                            button(
                                container(
                                    row![
                                        column![
                                            row![
                                                text(&scheduler.name)
                                                    .size(body_font_size)
                                                    .style(iced::theme::Text::Color(theme.primary()))
                                                    .width(Length::Fill),
                                                if is_current {
                                                    container(
                                                        text("CURRENT")
                                                            .size(body_font_size * 0.71)
                                                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.5, 0.1))) // Darker green
                                                    )
                                                    .padding(Padding::from([4.0, 8.0, 4.0, 8.0]))
                                                    .style(iced::theme::Container::Custom(Box::new(InstalledBadgeStyle {
                radius: settings.border_radius,
            })))
                                                } else {
                                                    container(Space::with_height(Length::Shrink))
                                                },
                                            ]
                                            .align_items(Alignment::Center)
                                            .spacing(12)
                                            .width(Length::Fill),
                                            if !scheduler.modes.is_empty() {
                                                column(
                                                    scheduler.modes
                                                        .iter()
                                                        .map(|mode| {
                                                            let mode_name = mode.name.clone();
                                                            let is_mode_selected = self.selected_scheduler_mode.as_ref().map(|s| s == &mode_name).unwrap_or(false);
                                                            // Check if this mode matches the current flags
                                                            // Normalize scheduler names for comparison
                                                            let scheduler_base = scheduler_name.strip_prefix("scx_").unwrap_or(&scheduler_name).to_lowercase();
                                                            let is_mode_current = !current_flags.is_empty() && 
                                                                current_sched_name_for_comparison == scheduler_base &&
                                                                current_flags.trim() == mode.flags.trim();
                                                            button(
                                                                container(
                                                                    row![
                                                                        text(&mode.name)
                                                                            .size(body_font_size * 0.86)
                                                                            .style(iced::theme::Text::Color(if is_mode_current {
                                                                                iced::Color::from_rgb(1.0, 1.0, 1.0) // White text for current mode
                                                                            } else {
                                                                                theme.primary()
                                                                            })),
                                                                        Space::with_width(Length::Fill),
                                                                        text(&mode.flags)
                                                                            .size(body_font_size * 0.79)
                                                                            .style(iced::theme::Text::Color(if is_mode_current {
                                                                                iced::Color::from_rgb(1.0, 1.0, 1.0) // White text for current mode
                                                                            } else {
                                                                                iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0)
                                                                            })),
                                                                    ]
                                                                    .align_items(Alignment::Center)
                                                                    .spacing(12)
                                                                )
                                                                .width(Length::Fill)
                                                                .padding(Padding::new(12.0))
                                                                .style(iced::theme::Container::Custom(if is_mode_current {
                                                                    // Red highlight for currently active mode
                                                                    Box::new(CurrentModeStyle { radius: settings.border_radius }) as Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>>
                                                                } else if is_mode_selected {
                                                                    // Blue highlight for selected mode
                                                                    Box::new(SelectedModeStyle { radius: settings.border_radius }) as Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>>
                                                                } else {
                                                                    Box::new(ModeStyle { radius: settings.border_radius }) as Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>>
                                                                }))
                                                            )
                                                            .on_press(Message::SchedulerModeSelected(mode_name.clone()))
                                                            .style(iced::theme::Button::Text)
                                                            .padding(0)
                                                            .into()
                                                        })
                                                        .collect::<Vec<_>>(),
                                                )
                                                .spacing(4)
                                                .padding(Padding::from([8.0, 0.0, 0.0, 0.0]))
                                            } else {
                                                column![].spacing(0)
                                            },
                                        ]
                                        .spacing(8)
                                        .width(Length::Fill),
                                    ]
                                    .align_items(Alignment::Center)
                                    .spacing(12)
                                )
                                .width(Length::Fill)
                                .padding(Padding::new(16.0))
                                .style(iced::theme::Container::Custom(if is_current {
                                    // Green highlight for currently running scheduler
                                    Box::new(CurrentSchedulerStyle { radius: settings.border_radius }) as Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>>
                                } else if is_selected {
                                    // Blue highlight for selected scheduler
                                    Box::new(SelectedSchedulerStyle { radius: settings.border_radius }) as Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>>
                                } else {
                                    Box::new(SchedulerItemStyle { radius: settings.border_radius }) as Box<dyn iced::widget::container::StyleSheet<Style = iced::Theme>>
                                }))
                            )
                            .on_press(Message::SchedulerSelected(scheduler_name))
                            .style(iced::theme::Button::Text)
                            .padding(0)
                            .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(8)
                .padding(Padding::new(8.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                Color::from(settings.background_color.clone()),
                settings.border_radius,
            ))))
            .into()
        };

        let extra_flags_input = text_input(
            "Extra flags (optional)",
            &self.scheduler_extra_flags,
        )
        .on_input(Message::SchedulerFlagsChanged)
        .padding(12)
        .size(input_font_size)
        .style(iced::theme::TextInput::Custom(Box::new(SearchInputStyle {
            radius: settings.border_radius,
        })))
        .width(Length::Fill);

        let apply_button: Element<Message> = button(
            row![
                text(crate::gui::fonts::glyphs::CHECK_SYMBOL)
                    .font(*material_font)
                    .size(icon_size),
                text(" Apply Scheduler").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .on_press(Message::ApplyScheduler)
        .padding(Padding::new(12.0))
        .into();

        // Enable apply button if scheduler changed or flags changed
        let can_apply = self.selected_scheduler.is_some() && 
                       (self.selected_scheduler.as_ref() != self.current_scheduler.as_ref() ||
                        !self.scheduler_extra_flags.trim().is_empty());

        container(
            column![
                container(
                    column![
                        text("Current Scheduler")
                            .size(body_font_size * 1.14)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        container(
                            text(&current_sched_text)
                                .size(body_font_size)
                        )
                        .width(Length::Fill)
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .padding(Padding::new(16.0))
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                Space::with_height(Length::Fixed(16.0)),
                container(
                    column![
                        text("Available Schedulers")
                            .size(body_font_size * 1.14)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        scheduler_list,
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding::new(16.0))
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                Space::with_height(Length::Fixed(16.0)),
                container(
                    column![
                        text("Extra Flags")
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.primary())),
                        Space::with_height(Length::Fixed(8.0)),
                        extra_flags_input,
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .padding(Padding::new(16.0))
                .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            }))),
                Space::with_height(Length::Fixed(16.0)),
                container(
                    row![
                        Space::with_width(Length::Fill),
                        if can_apply {
                            apply_button
                        } else {
                            button(
                                row![
                                    text(crate::gui::fonts::glyphs::CHECK_SYMBOL)
                                        .font(*material_font)
                                        .size(icon_size),
                                    text(" Apply Scheduler").size(button_font_size)
                                ]
                                .spacing(8)
                                .align_items(Alignment::Center)
                            )
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: false,
                                radius: settings.border_radius,
                            })))
                            .padding(Padding::new(12.0))
                            .into()
                        },
                    ]
                    .align_items(Alignment::Center)
                )
                .width(Length::Fill)
                .padding(Padding::new(16.0)),
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(24.0))
        .into()
    }
}

#[allow(dead_code)]
fn create_info_badge<'a>(label: &'a str, value: &'a str, theme: &'a crate::gui::Theme) -> Element<'a, Message> {
    container(
        column![
            text(label)
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0))),
            text(value)
                .size(16)
                .style(iced::theme::Text::Color(theme.primary())),
        ]
        .spacing(4)
    )
    .padding(Padding::from([10.0, 16.0, 10.0, 16.0]))
    .style(iced::theme::Container::Custom(Box::new(InfoBadgeStyle)))
    .into()
}

// Load kernel branches from JSON files (same way as fedora-kernel-manager)
async fn load_kernel_branches() -> Result<Vec<KernelBranch>, String> {
    // Try system directory first (same as original)
    let kernel_branches_dir = PathBuf::from("/usr/lib/fedora-kernel-manager/kernel_branches");
    let mut branches = Vec::new();

    if kernel_branches_dir.exists() {
        let entries = fs::read_dir(&kernel_branches_dir)
            .map_err(|e| format!("Failed to read kernel branches directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                
                // Parse as JSON value first (same as original)
                let branch_json: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
                
                let branch = KernelBranch {
                    name: branch_json["name"]
                        .as_str()
                        .ok_or_else(|| format!("Missing 'name' in {}", path.display()))?
                        .to_string(),
                    db_url: branch_json["db_url"]
                        .as_str()
                        .ok_or_else(|| format!("Missing 'db_url' in {}", path.display()))?
                        .to_string(),
                    init_script: branch_json["init_script"]
                        .as_str()
                        .ok_or_else(|| format!("Missing 'init_script' in {}", path.display()))?
                        .to_string(),
                    db: None,
                };
                
                branches.push(branch);
            }
        }
    } else {
        // Try local fallback directory
        let local_branches_dir = PathBuf::from("fedora-kernel-manager-main/data/kernel_branches");
        if local_branches_dir.exists() {
            let entries = fs::read_dir(&local_branches_dir)
                .map_err(|e| format!("Failed to read local kernel branches directory: {}", e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    
                    let branch_json: serde_json::Value = serde_json::from_str(&content)
                        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
                    
                    let branch = KernelBranch {
                        name: branch_json["name"]
                            .as_str()
                            .ok_or_else(|| format!("Missing 'name' in {}", path.display()))?
                            .to_string(),
                        db_url: branch_json["db_url"]
                            .as_str()
                            .ok_or_else(|| format!("Missing 'db_url' in {}", path.display()))?
                            .to_string(),
                        init_script: branch_json["init_script"]
                            .as_str()
                            .ok_or_else(|| format!("Missing 'init_script' in {}", path.display()))?
                            .to_string(),
                        db: None,
                    };
                    
                    branches.push(branch);
                }
            }
        } else {
            // Create default branch if no branches directory exists
            branches.push(KernelBranch {
                name: "kernel (RPM Default)".to_string(),
                db_url: "https://raw.githubusercontent.com/CosmicFusion/fedora-kernel-manager/main/data/db_kernel.json".to_string(),
                init_script: "true".to_string(),
                db: None,
            });
        }
    }

    Ok(branches)
}

// Detect which branch the running kernel belongs to
async fn detect_running_kernel_branch(branches: Vec<KernelBranch>) -> Option<String> {
    // Get running kernel version from uname
    let kernel_version = TokioCommand::new("uname")
        .arg("-r")
        .output()
        .await
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let kernel_version = match kernel_version {
        Some(v) => v,
        None => return branches.first().map(|b| b.name.clone()),
    };

    // Try to get the package name that provides the running kernel
    let kernel_package = TokioCommand::new("rpm")
        .args(["-qf", &format!("/boot/vmlinuz-{}", kernel_version)])
        .output()
        .await
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Check kernel version string for branch identifiers
    let version_lower = kernel_version.to_lowercase();
    
    // Check each branch to see if it matches the running kernel
    for branch in &branches {
        let branch_lower = branch.name.to_lowercase();
        
        // Check for CachyOS kernel
        if branch_lower.contains("cachyos") {
            if version_lower.contains("cachyos") || 
               kernel_package.as_ref().map(|p| p.to_lowercase().contains("cachyos")).unwrap_or(false) {
                return Some(branch.name.clone());
            }
        }
        
        // Check for other specific kernel types
        // Add more patterns here as needed
    }

    // If no specific match, check if it's the default kernel
    // Default kernel typically doesn't have special identifiers
    if !version_lower.contains("cachyos") && 
       !kernel_package.as_ref().map(|p| p.to_lowercase().contains("cachyos")).unwrap_or(false) {
        // Look for default/RPM branch
        for branch in &branches {
            if branch.name.contains("RPM Default") || 
               branch.name.contains("kernel") && !branch.name.to_lowercase().contains("cachyos") {
                return Some(branch.name.clone());
            }
        }
    }

    // Default to first branch if no match found
    branches.first().map(|b| b.name.clone())
}

async fn select_branch_and_load_kernels(branch_name: String, branches: Vec<KernelBranch>) -> Result<(String, Vec<EnhancedKernelInfo>, RunningKernelInfo, Option<String>, String), String> {
    // Find the branch
    let branch = branches.iter()
        .find(|b| b.name == branch_name)
        .ok_or_else(|| {
            format!("Branch not found: {}", branch_name)
        })?;
    

    // Run init script
    if branch.init_script != "true" {
        let output = TokioCommand::new("bash")
            .arg("-c")
            .arg(&branch.init_script)
            .output()
            .await
            .map_err(|e| {
                format!("Failed to run init script: {}", e)
            })?;
        
        if !output.status.success() {
            return Err(format!("Init script failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else {
    }

    // Download branch database (same way as original)
    let db_content = match reqwest::get(&branch.db_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        text
                    }
                    Err(e) => {
                        return Err(format!("Failed to read branch database: {}", e));
                    }
                }
            } else {
                return Err(format!("HTTP error downloading database: {}", response.status()));
            }
        }
        Err(e) => {
            return Err(format!("Failed to download branch database: {}", e));
        }
    };

    // Parse branch database
    let db: KernelBranchDb = match serde_json::from_str::<KernelBranchDb>(&db_content) {
        Ok(db) => {
            db
        }
        Err(e) => {
            return Err(format!("Failed to parse branch database: {}", e));
        }
    };

    // Get CPU feature level and running kernel info in parallel for speed
    let (cpu_feature_level, running_info) = tokio::join!(
        get_cpu_feature_level(),
        get_running_kernel_info()
    );

    // Get latest version using the script (same way as original)
    let latest_version = if let Some(pkg) = &db.latest_kernel_version_deter_pkg {
        let script_path = PathBuf::from("/usr/lib/fedora-kernel-manager/scripts/generate_package_info.sh");
        let script = if script_path.exists() {
            script_path
        } else {
            PathBuf::from("fedora-kernel-manager-main/data/scripts/generate_package_info.sh")
        };
        
        if script.exists() {
            TokioCommand::new(&script)
                .args(["version", pkg])
                .output()
                .await
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
        } else {
            get_package_version(pkg).await.ok()
        }
    } else {
        None
    };

    // Load kernels from branch - optimized for speed
    let mut kernels = Vec::new();
    let mut branch_package_names = std::collections::HashSet::new();

    // Pre-filter entries by CPU feature level
    let compatible_kernels: Vec<_> = db.kernels.into_iter()
        .filter(|e| e.min_x86_march <= cpu_feature_level)
        .collect();

    // Collect package names first
    for entry in &compatible_kernels {
        branch_package_names.insert(entry.main_package.clone());
    }

    // Batch check installed packages in parallel
    let installed_futures: Vec<_> = compatible_kernels.iter()
        .map(|entry| {
            let pkg = entry.main_package.clone();
            (entry.main_package.clone(), tokio::spawn(async move {
                let check_output = TokioCommand::new("rpm")
                    .args(["-q", &pkg])
                    .output()
                    .await;
                check_output.map(|o| o.status.success()).unwrap_or(false)
            }))
        })
        .collect();

    // Collect installed status
    let mut installed_map = std::collections::HashMap::new();
    for (pkg_name, check_future) in installed_futures {
        if let Ok(installed) = check_future.await {
            installed_map.insert(pkg_name, installed);
        }
    }

    // Build kernel list with parallel version/description fetching
    let kernel_futures: Vec<_> = compatible_kernels.into_iter()
        .map(|entry| {
            let pkg = entry.main_package.clone();
            let installed = installed_map.get(&pkg).copied().unwrap_or(false);
            let branch_name = branch.name.clone();
            tokio::spawn(async move {
                let (version, description) = tokio::join!(
                    get_package_version(&pkg),
                    get_package_description(&pkg)
                );
                EnhancedKernelInfo {
                    name: entry.name,
                    main_package: pkg,
                    packages: entry.packages,
                    version: version.unwrap_or_else(|_| "Unknown".to_string()),
                    description: description.unwrap_or_else(|_| "No description".to_string()),
                    installed,
                    branch: branch_name,
                    min_x86_march: entry.min_x86_march,
                }
            })
        })
        .collect();

    // Wait for all kernel info to load
    for future in kernel_futures {
        if let Ok(kernel) = future.await {
            kernels.push(kernel);
        }
    }

    // Skip repository search for now to keep loading fast
    // Repository kernels can be added later if needed

    Ok((branch_name, kernels, running_info, latest_version, db_content))
}

async fn get_cpu_feature_level() -> u32 {
    // Detect CPU feature level using ld-linux-x86-64.so.2 (same way as original)
    // Use blocking spawn for the pipe chain
    let result = tokio::task::spawn_blocking(|| {
        use std::process::{Command, Stdio};
        let base_process = match Command::new("/lib64/ld-linux-x86-64.so.2")
            .arg("--help")
            .env("LANG", "en_US")
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(p) => p,
            Err(_) => return 1,
        };

        let grep_output = match Command::new("grep")
            .arg("(supported, searched)")
            .stdin(base_process.stdout.unwrap())
            .output()
        {
            Ok(o) => o,
            Err(_) => return 1,
        };

        if let Ok(stdout) = String::from_utf8(grep_output.stdout) {
            if let Some(line) = stdout.lines().next() {
                let level = line.trim_end_matches("(supported, searched)").trim();
                return match level {
                    "x86-64-v4" => 4,
                    "x86-64-v3" => 3,
                    "x86-64-v2" => 2,
                    _ => 1,
                };
            }
        }

        1
    }).await;

    result.unwrap_or(1)
}

async fn get_running_kernel_info() -> RunningKernelInfo {
    let kernel = TokioCommand::new("uname")
        .arg("-r")
        .output()
        .await
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let version = kernel.split('-').next().unwrap_or(&kernel).to_string();
    let scheduler = get_current_scheduler(&version).await;

    RunningKernelInfo {
        kernel,
        version,
        scheduler,
    }
}

async fn get_current_scheduler(version: &str) -> String {
    // First try scxctl get for accurate SCX scheduler detection
    if let Ok(output) = TokioCommand::new("scxctl")
        .arg("get")
        .output()
        .await
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            let sched = stdout.trim();
            if !sched.is_empty() && sched != "no scx scheduler running" {
                // scxctl returns format like "running Bpfland with arguments "-k -s 5000 -l 5000""
                // Return the full string for display, but format it consistently
                if sched.starts_with("running ") {
                    // Extract scheduler name and format as "sched_ext: scx_bpfland" with full info
                    let scheduler_name = sched
                        .strip_prefix("running ")
                        .and_then(|s| s.split_whitespace().next())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    if !scheduler_name.is_empty() {
                        // Format with full info: "sched_ext: scx_bpfland with arguments..."
                        let full_info = sched.strip_prefix("running ").unwrap_or(&sched);
                        if scheduler_name.starts_with("scx_") {
                            return format!("sched_ext: {}", full_info);
                        } else {
                            return format!("sched_ext: scx_{}", full_info);
                        }
                    }
                } else {
                    // Fallback: just format the string
                    return format!("sched_ext: {}", sched);
                }
            }
        }
    }

    // Fallback: Check if SCX kernel and read from /sys directly
    if std::path::Path::new("/sys/kernel/sched_ext").exists() {
        if let Ok(scx_sched) = fs::read_to_string("/sys/kernel/sched_ext/root/ops") {
            let sched = scx_sched.trim();
            if !sched.is_empty() {
                if sched.starts_with("scx_") {
                    return format!("sched_ext: {}", sched);
                } else {
                    return format!("sched_ext: scx_{}", sched);
                }
            }
        }
    }

    // Check for BORE
    let bore_check = TokioCommand::new("sysctl")
        .args(["-n", "kernel.sched_bore"])
        .output()
        .await;
    
    if let Ok(output) = bore_check {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if stdout.trim() == "1" {
                return "BORE".to_string();
            }
        }
    }

    // Check version for EEVDF (6.6+)
    if let Ok(version_num) = version.parse::<f32>() {
        if version_num >= 6.6 {
            return "EEVDF?".to_string();
        }
    }

    "CFS?".to_string()
}

// Removed get_installed_packages - we now use rpm -q directly for each package (same as original)

#[allow(dead_code)]
async fn search_repo_kernels(
    existing_packages: &std::collections::HashSet<String>,
    cpu_feature_level: u32,
    branch_name: &str,
) -> Vec<EnhancedKernelInfo> {
    // Search repositories for kernel packages
    let output = match TokioCommand::new("dnf")
        .args(["search", "--quiet", "--showduplicates", "kernel"])
        .output()
        .await
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut kernel_packages = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Matched fields:") || line.starts_with("Importing") {
            continue;
        }

        // Parse dnf search output: "package.arch : description" or "package.arch<TAB>description"
        let pkg_name = if let Some(colon_pos) = line.find(" : ") {
            line[..colon_pos].trim()
        } else if let Some(tab_pos) = line.find('\t') {
            line[..tab_pos].trim()
        } else {
            continue;
        };

        // Extract package name (remove .arch suffix)
        let pkg_name = pkg_name.split('.').next().unwrap_or(pkg_name).trim();

        // Skip if already in branch database
        if existing_packages.contains(pkg_name) {
            continue;
        }

        // Skip headers packages
        if pkg_name.contains("kernel-headers") || 
           pkg_name.contains("headers") ||
           pkg_name.contains("kernel-devel") ||
           pkg_name.contains("kernel-debug") ||
           pkg_name.contains("kernel-core") ||
           pkg_name.contains("kernel-modules") ||
           pkg_name.contains("kernel-modules-extra") {
            continue;
        }

        // Only include packages that start with "kernel-" (main kernel packages)
        if !pkg_name.starts_with("kernel-") {
            continue;
        }

        // Check if installed
        let installed = {
            let check_output = TokioCommand::new("rpm")
                .args(["-q", pkg_name])
                .output()
                .await;
            check_output.map(|o| o.status.success()).unwrap_or(false)
        };

        // Get version and description
        let version = get_package_version(pkg_name).await.unwrap_or_else(|_| "Unknown".to_string());
        let description = get_package_description(pkg_name).await.unwrap_or_else(|_| "No description".to_string());

        // Extract kernel name from package (e.g., "kernel-6.8.0" -> "6.8.0")
        let kernel_name = if let Some(stripped) = pkg_name.strip_prefix("kernel-") {
            format!("kernel-{}", stripped)
        } else {
            pkg_name.to_string()
        };

        kernel_packages.push(EnhancedKernelInfo {
            name: kernel_name.clone(),
            main_package: pkg_name.to_string(),
            packages: format!("{} kernel-headers-{}", pkg_name, 
                if let Some(ver) = pkg_name.strip_prefix("kernel-") {
                    ver
                } else {
                    ""
                }),
            version,
            description,
            installed,
            branch: format!("{} (from repos)", branch_name),
            min_x86_march: 1, // Default to v1, will be filtered if needed
        });
    }

    // Filter by CPU feature level
    kernel_packages.retain(|k| k.min_x86_march <= cpu_feature_level);

    kernel_packages
}

async fn get_package_version(package: &str) -> Result<String, String> {
    let script_path = PathBuf::from("/usr/lib/fedora-kernel-manager/scripts/generate_package_info.sh");
    let script = if script_path.exists() {
        script_path
    } else {
        PathBuf::from("fedora-kernel-manager-main/data/scripts/generate_package_info.sh")
    };

    if !script.exists() {
        // Fallback to direct rpm command if script doesn't exist
        let output = TokioCommand::new("rpm")
            .args(["-q", "--queryformat", "%{VERSION}-%{RELEASE}", package])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                String::from_utf8(o.stdout)
                    .map(|s| s.trim().to_string())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))
            }
            _ => Err(format!("Package {} not found", package))
        }
    } else {
        let output = TokioCommand::new(&script)
            .args(["version", package])
            .output()
            .await
            .map_err(|e| format!("Failed to execute generate_package_info.sh: {}", e))?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map(|s| s.trim().to_string())
                .map_err(|e| format!("Invalid UTF-8: {}", e))
        } else {
            Err(format!("Package {} not found", package))
        }
    }
}

async fn get_package_description(package: &str) -> Result<String, String> {
    let script_path = PathBuf::from("/usr/lib/fedora-kernel-manager/scripts/generate_package_info.sh");
    let script = if script_path.exists() {
        script_path
    } else {
        PathBuf::from("fedora-kernel-manager-main/data/scripts/generate_package_info.sh")
    };

    if !script.exists() {
        // Fallback to direct dnf command if script doesn't exist
        let output = TokioCommand::new("dnf")
            .args(["info", package])
            .output()
            .await
            .map_err(|e| format!("Failed to execute dnf: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Summary     :") {
                    return Ok(line.split(':').nth(1).unwrap_or("").trim().to_string());
                }
            }
        }
        Ok("No description available".to_string())
    } else {
        let output = TokioCommand::new(&script)
            .args(["description", package])
            .output()
            .await
            .map_err(|e| format!("Failed to execute generate_package_info.sh: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)
                .map_err(|e| format!("Invalid UTF-8: {}", e))?;
            let description = stdout.trim().to_string();
            if description.is_empty() {
                Ok("No description available".to_string())
            } else {
                Ok(description)
            }
        } else {
            Ok("No description available".to_string())
        }
    }
}

async fn load_kernel_details(kernel_name: String) -> KernelDetails {
    let output = TokioCommand::new("dnf")
        .args(["info", &kernel_name])
        .output()
        .await;

    let mut name = kernel_name.clone();
    let mut version = String::new();
    let mut release = String::new();
    let mut arch = String::new();
    let mut repository = String::new();
    let mut summary = String::new();
    let mut description = String::new();
    let mut size = String::new();
    let mut build_date = None;
    let mut installed = false;

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Name        :") {
                    name = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Version     :") {
                    version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Release     :") {
                    release = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Architecture:") {
                    arch = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Repository  :") {
                    repository = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    installed = repository == "@System";
                } else if line.starts_with("Summary     :") {
                    summary = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Size        :") {
                    size = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Build Date  :") {
                    build_date = Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("Description :") {
                    description = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("            ") && !description.is_empty() {
                    description.push_str("\n");
                    description.push_str(line.trim());
                }
            }
        }
    }

    KernelDetails {
        name,
        version,
        release,
        arch,
        repository,
        installed,
        summary: if summary.is_empty() { "No summary available".to_string() } else { summary },
        description: if description.is_empty() { "No description available".to_string() } else { description },
        size: if size.is_empty() { "Unknown".to_string() } else { size },
        build_date,
    }
}

async fn load_scx_schedulers() -> Result<Vec<ScxScheduler>, String> {
    let scx_scheds_path = PathBuf::from("/usr/lib/fedora-kernel-manager/scx_scheds.json");
    let path = if scx_scheds_path.exists() {
        scx_scheds_path
    } else {
        PathBuf::from("fedora-kernel-manager-main/data/scx_scheds.json")
    };

    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read SCX schedulers: {}", e))?;
        let schedulers: ScxSchedulers = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse SCX schedulers: {}", e))?;
        Ok(schedulers.scx_schedulers)
    } else {
        Ok(Vec::new())
    }
}

async fn apply_scx_scheduler(scheduler_name: String, flags: String) -> Result<(), String> {
    // Remove "sched_ext: " prefix if present
    let scheduler_name_clean = scheduler_name
        .strip_prefix("sched_ext: ")
        .unwrap_or(&scheduler_name)
        .to_string();

    // Remove "scx_" prefix if present (scxctl expects just the base name, e.g., "bpfland" not "scx_bpfland")
    let scheduler_base = scheduler_name_clean
        .strip_prefix("scx_")
        .unwrap_or(&scheduler_name_clean)
        .to_string();

    // Handle scx_disabled case
    if scheduler_base == "disabled" || scheduler_name_clean == "scx_disabled" {
        let output = TokioCommand::new("pkexec")
            .arg("scxctl")
            .arg("stop")
            .output()
            .await
            .map_err(|e| format!("Failed to execute scxctl stop: {}", e))?;

        if output.status.success() {
            // Also update /etc/default/scx for consistency
            let _ = TokioCommand::new("pkexec")
                .arg("sh")
                .arg("-c")
                .arg("echo 'SCX_SCHEDULER=scx_disabled' > /etc/default/scx")
                .output()
                .await;
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "Failed to disable SCX scheduler (exit code: {:?})\nStderr: {}\nStdout: {}",
                output.status.code(), stderr, stdout
            ));
        }
    }

    // Check if a scheduler is already running - if so, use 'switch', otherwise use 'start'
    let check_output = TokioCommand::new("scxctl")
        .arg("get")
        .output()
        .await
        .ok();
    
    let has_running_scheduler = check_output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| {
            let s = s.trim();
            !s.is_empty() && s != "no scx scheduler running"
        })
        .unwrap_or(false);

    // Use scxctl to start/switch the scheduler
    // Use 'switch' if a scheduler is already running, 'start' otherwise
    let action = if has_running_scheduler { "switch" } else { "start" };
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("scxctl");
    cmd.arg(action);
    cmd.arg("--sched");
    cmd.arg(&scheduler_base);

    // Add flags if provided (use --args=<flags> format to avoid argument parsing issues)
    if !flags.trim().is_empty() {
        cmd.arg(&format!("--args={}", flags.trim()));
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute scxctl: {}", e))?;

    if output.status.success() {
        // Update /etc/default/scx for consistency
        let _ = TokioCommand::new("pkexec")
            .arg("sh")
            .arg("-c")
            .arg(&format!(
                "echo 'SCX_SCHEDULER={}' > /etc/default/scx && echo 'SCX_FLAGS={}' >> /etc/default/scx",
                scheduler_name_clean,
                flags.trim()
            ))
            .output()
            .await;
        
        // Wait a moment for the scheduler to apply
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // scxctl often puts errors in stdout, not stderr
        let error_msg = if !stdout.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        Err(format!(
            "SCX scheduler change failed (exit code: {:?}): {}",
            output.status.code(), error_msg
        ))
    }
}

// Style structs
struct KernelItemStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for KernelItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.15, 0.15, 0.15, 1.0).into()
            } else {
                iced::Color::from_rgba(0.98, 0.98, 0.99, 1.0).into() // Softer, calmer white
            }),
            border: Border::with_radius(self.radius * 0.5),
            ..Default::default()
        }
    }
}

struct InstallButtonStyle {
    radius: f32,
}
impl ButtonStyleSheet for InstallButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.9).into()),
            text_color: iced::Color::WHITE,
            border: Border::with_radius(self.radius * 0.375),
            ..Default::default()
        }
    }
}

struct RemoveButtonStyle {
    radius: f32,
}
impl ButtonStyleSheet for RemoveButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Color::from_rgb(0.9, 0.3, 0.3).into()),
            text_color: iced::Color::WHITE,
            border: Border::with_radius(self.radius * 0.375),
            ..Default::default()
        }
    }
}

struct InstalledBadgeStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for InstalledBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.1, 0.5, 0.1, 0.3).into() // Darker green background
            } else {
                iced::Color::from_rgba(0.1, 0.5, 0.1, 0.15).into() // Darker green for light mode
            }),
            border: Border::with_radius(self.radius * 0.25),
            ..Default::default()
        }
    }
}

struct AvailableBadgeStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for AvailableBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2).into()),
            border: Border::with_radius(self.radius * 0.25),
            ..Default::default()
        }
    }
}

struct CpuBadgeStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for CpuBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.9, 0.6, 0.2, 0.2).into()),
            border: Border::with_radius(self.radius * 0.25),
            ..Default::default()
        }
    }
}

#[allow(dead_code)]
struct InfoBadgeStyle;
impl iced::widget::container::StyleSheet for InfoBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.15, 0.15, 0.15, 1.0).into()
            } else {
                iced::Color::from_rgba(0.97, 0.97, 0.98, 1.0).into() // Softer container
            }),
            border: Border::with_radius(6.0),
            ..Default::default()
        }
    }
}

struct BranchButtonStyle {
    is_selected: bool,
    radius: f32,
}
impl ButtonStyleSheet for BranchButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        ButtonAppearance {
            background: Some(if self.is_selected {
                palette.primary.into()
            } else {
                if is_dark {
                    iced::Color::from_rgba(0.2, 0.2, 0.2, 1.0).into()
                } else {
                    iced::Color::from_rgba(0.9, 0.9, 0.91, 1.0).into() // Softer button background
                }
            }),
            text_color: if self.is_selected { iced::Color::WHITE } else { palette.text },
            border: Border::with_radius(self.radius * 0.375),
            ..Default::default()
        }
    }
}

struct RefreshButtonStyle {
    radius: f32,
}
impl ButtonStyleSheet for RefreshButtonStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> ButtonAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        ButtonAppearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.2, 0.2, 0.2, 1.0).into()
            } else {
                iced::Color::from_rgba(0.85, 0.85, 0.85, 1.0).into()
            }),
            text_color: palette.text,
            border: Border::with_radius(self.radius * 0.375),
            ..Default::default()
        }
    }
}

struct SearchContainerStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for SearchContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.12, 0.12, 0.12, 1.0).into()
            } else {
                iced::Color::from_rgba(0.96, 0.96, 0.97, 1.0).into() // Softer background
            }),
            border: Border::with_radius(self.radius * 0.5),
            ..Default::default()
        }
    }
}

struct SearchInputStyle {
    radius: f32,
}
impl TextInputStyleSheet for SearchInputStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> TextInputAppearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        TextInputAppearance {
            background: if is_dark {
                iced::Color::from_rgba(0.1, 0.1, 0.1, 1.0).into()
            } else {
                iced::Color::from_rgba(0.99, 0.99, 0.99, 1.0).into() // Softer input background
            },
            border: Border::with_radius(self.radius * 0.375),
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
                iced::Color::from_rgba(0.99, 0.99, 0.99, 1.0).into() // Softer input background
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
                radius: self.radius.into(),
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

struct PanelStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for PanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.12, 0.12, 0.12, 1.0).into()
            } else {
                iced::Color::from_rgba(0.96, 0.96, 0.97, 1.0).into() // Softer background
            }),
            border: Border::with_radius(self.radius * 0.5),
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
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.15, 0.15, 0.15, 1.0).into()
            } else {
                iced::Color::from_rgba(0.97, 0.97, 0.98, 1.0).into() // Softer container
            }),
            border: Border::with_radius(self.radius * 0.375),
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
        let is_dark = palette.background.r < 0.5;
        ButtonAppearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.2, 0.2, 0.2, 1.0).into()
            } else {
                iced::Color::from_rgba(0.85, 0.85, 0.85, 1.0).into()
            }),
            text_color: palette.text,
            border: Border::with_radius(self.radius * 0.375),
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
                    iced::Color::from_rgba(0.9, 0.9, 0.91, 1.0).into() // Softer button background
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
                iced::Color::from_rgba(0.3, 0.3, 0.3, 1.0).into()
            } else {
                iced::Color::from_rgba(0.75, 0.75, 0.75, 1.0).into()
            });
        }
        appearance
    }
}

struct SchedulerItemStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for SchedulerItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.15, 0.15, 0.15, 1.0).into()
            } else {
                iced::Color::from_rgba(0.98, 0.98, 0.99, 1.0).into() // Softer, calmer white
            }),
            border: Border::with_radius(self.radius * 0.5),
            ..Default::default()
        }
    }
}

struct SelectedSchedulerStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for SelectedSchedulerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.2, 0.4, 0.6, 0.3).into()),
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: iced::Color::from_rgb(0.2, 0.6, 0.9),
            },
            ..Default::default()
        }
    }
}

struct CurrentSchedulerStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for CurrentSchedulerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.3, 0.5, 0.4, 0.25).into()),
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: iced::Color::from_rgb(0.4, 0.7, 0.5), // Calmer green
            },
            ..Default::default()
        }
    }
}

struct ModeStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for ModeStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        let is_dark = palette.background.r < 0.5;
        Appearance {
            background: Some(if is_dark {
                iced::Color::from_rgba(0.12, 0.12, 0.12, 1.0).into()
            } else {
                iced::Color::from_rgba(0.96, 0.96, 0.97, 1.0).into() // Softer background
            }),
            border: Border::with_radius(self.radius * 0.375),
            ..Default::default()
        }
    }
}

struct SelectedModeStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for SelectedModeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.2, 0.5, 0.7, 0.3).into()),
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: iced::Color::from_rgb(0.2, 0.6, 0.9),
            },
            ..Default::default()
        }
    }
}

struct CurrentModeStyle {
    radius: f32,
}
impl iced::widget::container::StyleSheet for CurrentModeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Color::from_rgba(0.5, 0.3, 0.3, 0.25).into()),
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: iced::Color::from_rgb(0.7, 0.4, 0.4), // Calmer red
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
            border: Border::with_radius(self.radius),
            text_color: if self.is_primary {
                iced::Color::WHITE
            } else {
                palette.text
            },
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
