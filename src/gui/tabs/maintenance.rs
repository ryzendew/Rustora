use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone)]
pub enum Message {
    RebuildKernelModules,
    RebuildKernelModulesComplete,
    RegenerateInitramfs,
    RegenerateInitramfsComplete,
    RemoveOrphanedPackages,
    RemoveOrphanedPackagesComplete,
    CleanPackageCache,
    CleanPackageCacheComplete,
    RunAllMaintenance,
    AllMaintenanceComplete(Result<String, String>),
}

#[derive(Debug)]
pub struct MaintenanceTab {
    is_rebuilding_modules: bool,
    is_regenerating_initramfs: bool,
    is_removing_orphaned: bool,
    is_cleaning_cache: bool,
    is_running_all: bool,
    output_log: Vec<String>,
}

impl MaintenanceTab {
    pub fn new() -> Self {
        Self {
            is_rebuilding_modules: false,
            is_regenerating_initramfs: false,
            is_removing_orphaned: false,
            is_cleaning_cache: false,
            is_running_all: false,
            output_log: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::RebuildKernelModules => {
                // Spawn a separate window for maintenance task
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("maintenance-dialog")
                            .arg("rebuild-kernel-modules")
                            .spawn()
                            .ok();
                    },
                    |_| Message::RebuildKernelModulesComplete,
                )
            }
            Message::RebuildKernelModulesComplete => {
                self.is_rebuilding_modules = false;
                iced::Command::none()
            }
            Message::RegenerateInitramfs => {
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("maintenance-dialog")
                            .arg("regenerate-initramfs")
                            .spawn()
                            .ok();
                    },
                    |_| Message::RegenerateInitramfsComplete,
                )
            }
            Message::RegenerateInitramfsComplete => {
                self.is_regenerating_initramfs = false;
                iced::Command::none()
            }
            Message::RemoveOrphanedPackages => {
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("maintenance-dialog")
                            .arg("remove-orphaned-packages")
                            .spawn()
                            .ok();
                    },
                    |_| Message::RemoveOrphanedPackagesComplete,
                )
            }
            Message::RemoveOrphanedPackagesComplete => {
                self.is_removing_orphaned = false;
                iced::Command::none()
            }
            Message::CleanPackageCache => {
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        TokioCommand::new(&exe_path)
                            .arg("maintenance-dialog")
                            .arg("clean-package-cache")
                            .spawn()
                            .ok();
                    },
                    |_| Message::CleanPackageCacheComplete,
                )
            }
            Message::CleanPackageCacheComplete => {
                self.is_cleaning_cache = false;
                iced::Command::none()
            }
            Message::RunAllMaintenance => {
                self.is_running_all = true;
                self.output_log.clear();
                self.output_log.push("Starting all maintenance tasks...".to_string());
                iced::Command::perform(run_all_maintenance(), Message::AllMaintenanceComplete)
            }
            Message::AllMaintenanceComplete(result) => {
                self.is_running_all = false;
                match &result {
                    Ok(msg) => {
                        self.output_log.push(format!("✓ {}", msg));
                    }
                    Err(e) => {
                        self.output_log.push(format!("✗ Error: {}", e));
                    }
                }
                iced::Command::none()
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
                text("System Maintenance")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                Space::with_height(Length::Fixed(8.0)),
                text("Perform system maintenance tasks to keep your Fedora system running smoothly")
                    .size(body_font_size)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(0.0));

        // Helper function to create action cards
        let create_action_card = {
            let material_font = material_font;
            let icon_size = icon_size;
            let button_font_size = button_font_size;
            let body_font_size = body_font_size;
            let theme = theme;
            let settings = settings;
            move |icon: &str, title: &str, description: &str, is_running: bool, message: Message| -> Element<'_, Message> {
                let button_widget = if is_running {
                    button(
                        row![
                            text(icon).font(material_font).size(icon_size),
                            text(" Running...").size(button_font_size)
                        ]
                        .spacing(8)
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
                            text(icon).font(material_font).size(icon_size),
                            text(title).size(button_font_size)
                        ]
                        .spacing(8)
                        .align_items(Alignment::Center)
                    )
                    .on_press(message)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::new(12.0))
                };

                container(
                    column![
                        button_widget,
                        Space::with_height(Length::Fixed(8.0)),
                        text(description)
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                            .width(Length::Fill),
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .padding(Padding::new(20.0))
                .style(iced::theme::Container::Custom(Box::new(ActionCardStyle {
                    radius: settings.border_radius,
                })))
                .into()
            }
        };

        // Kernel maintenance section
        let kernel_section = container(
            column![
                text("Kernel Maintenance")
                    .size(title_font_size * 0.6)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                    .width(Length::Fill),
                Space::with_height(Length::Fixed(16.0)),
                create_action_card(
                    crate::gui::fonts::glyphs::REFRESH_SYMBOL,
                    "Rebuild Kernel Modules",
                    "Rebuilds all kernel modules using akmods. Use this after kernel updates.",
                    self.is_rebuilding_modules,
                    Message::RebuildKernelModules
                ),
                Space::with_height(Length::Fixed(12.0)),
                create_action_card(
                    crate::gui::fonts::glyphs::REFRESH_SYMBOL,
                    "Regenerate Initramfs",
                    "Regenerates all initramfs images using dracut. Ensures proper boot configuration.",
                    self.is_regenerating_initramfs,
                    Message::RegenerateInitramfs
                ),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(24.0))
        .style(iced::theme::Container::Custom(Box::new(SectionCardStyle {
            radius: settings.border_radius,
        })));

        // Package maintenance section
        let package_section = container(
            column![
                text("Package Maintenance")
                    .size(title_font_size * 0.6)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                    .width(Length::Fill),
                Space::with_height(Length::Fixed(16.0)),
                create_action_card(
                    crate::gui::fonts::glyphs::DELETE_SYMBOL,
                    "Remove Orphaned Packages",
                    "Removes packages that are no longer needed by any installed software.",
                    self.is_removing_orphaned,
                    Message::RemoveOrphanedPackages
                ),
                Space::with_height(Length::Fixed(12.0)),
                create_action_card(
                    crate::gui::fonts::glyphs::SETTINGS_SYMBOL,
                    "Clean Package Cache",
                    "Removes cached package files to free up disk space.",
                    self.is_cleaning_cache,
                    Message::CleanPackageCache
                ),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(24.0))
        .style(iced::theme::Container::Custom(Box::new(SectionCardStyle {
            radius: settings.border_radius,
        })));

        // Run all button
        let run_all_button = if self.is_running_all {
            button(
                row![
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                    text(" Running All Maintenance Tasks...").size(button_font_size)
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(16.0))
        } else {
            button(
                row![
                    text(crate::gui::fonts::glyphs::REFRESH_SYMBOL).font(material_font).size(icon_size),
                    text(" Run All Maintenance Tasks").size(button_font_size)
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .on_press(Message::RunAllMaintenance)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: true,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(16.0))
        };

        let run_all_section = container(
            column![
                run_all_button,
                Space::with_height(Length::Fixed(8.0)),
                text("Execute all maintenance tasks in sequence")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0)))
                    .width(Length::Fill),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(24.0))
        .style(iced::theme::Container::Custom(Box::new(SectionCardStyle {
            radius: settings.border_radius,
        })));

        // Actions column
        let actions_column = scrollable(
            column![
                kernel_section,
                Space::with_height(Length::Fixed(20.0)),
                package_section,
                Space::with_height(Length::Fixed(20.0)),
                run_all_section,
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill);

        // Log section
        let log_header = container(
            row![
                text("Activity Log")
                    .size(title_font_size * 0.65)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                Space::with_width(Length::Fill),
            ]
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(0.0);

        let log_content: Element<Message> = if self.output_log.is_empty() {
            container(
                column![
                    Space::with_height(Length::Fill),
                    text("No operations performed yet")
                        .size(body_font_size * 1.15)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fixed(8.0)),
                    text("Select a maintenance task to begin")
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    Space::with_height(Length::Fill),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            scrollable(
                column(
                    self.output_log
                        .iter()
                        .map(|line| {
                            container(
                                row![
                                    text(if line.starts_with("✓") { "✓" } else if line.starts_with("✗") { "✗" } else { "•" })
                                        .size(icon_size)
                                        .style(iced::theme::Text::Color(
                                            if line.starts_with("✓") {
                                                iced::Color::from_rgb(0.1, 0.5, 0.1) // Darker green
                                            } else if line.starts_with("✗") {
                                                iced::Color::from_rgb(0.9, 0.2, 0.2)
                                            } else {
                                                theme.primary_with_settings(Some(settings))
                                            }
                                        ))
                                        .width(Length::Fixed(20.0)),
                                    text(if line.starts_with("✓") || line.starts_with("✗") {
                                        &line[2..]
                                    } else {
                                        line
                                    })
                                        .size(body_font_size)
                                        .width(Length::Fill),
                                ]
                                .spacing(12)
                                .align_items(Alignment::Start)
                            )
                            .width(Length::Fill)
                            .padding(Padding::new(12.0))
                            .style(iced::theme::Container::Custom(Box::new(LogItemStyle {
                                radius: settings.border_radius,
                            })))
                            .into()
                        })
                        .collect::<Vec<_>>(),
                )
                .spacing(6)
                .padding(Padding::new(16.0)),
            )
            .into()
        };

        let log_section = container(
            column![
                log_header,
                Space::with_height(Length::Fixed(16.0)),
                log_content,
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(24.0))
        .style(iced::theme::Container::Custom(Box::new(SectionCardStyle {
            radius: settings.border_radius,
        })));

        // Main layout
        let content = row![
            container(actions_column)
                .width(Length::FillPortion(1))
                .height(Length::Fill),
            Space::with_width(Length::Fixed(24.0)),
            container(log_section)
                .width(Length::FillPortion(2))
                .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            column![
                header,
                Space::with_height(Length::Fixed(24.0)),
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

async fn rebuild_kernel_modules_streaming() -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(["akmods", "--force", "--rebuild"]);
    
    // Use spawn to get streaming output and prevent blocking
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute akmods: {}", e))?;

    // Read output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str("Running: akmods --force --rebuild\n");
    combined_output.push_str("--- Output ---\n");
    
    // Read both stdout and stderr
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stdout: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stderr: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status.success() {
        return Err(format!("Kernel module rebuild failed (exit code: {}):\n{}", 
            status.code().unwrap_or(-1), combined_output));
    }

    Ok(format!("Kernel modules rebuilt successfully\n\n{}", combined_output))
}

async fn regenerate_initramfs_streaming() -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(["dracut", "-f", "regenerate-all"]);
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute dracut: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str("Running: dracut -f regenerate-all\n");
    combined_output.push_str("--- Output ---\n");
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stdout: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stderr: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status.success() {
        return Err(format!("Initramfs regeneration failed (exit code: {}):\n{}", 
            status.code().unwrap_or(-1), combined_output));
    }

    Ok(format!("Initramfs regenerated successfully\n\n{}", combined_output))
}

async fn remove_orphaned_packages_streaming() -> Result<String, String> {
    // First, check for orphaned packages
    let check_output = TokioCommand::new("dnf")
        .args(["repoquery", "--unneeded", "-q"])
        .output()
        .await
        .map_err(|e| format!("Failed to check for orphaned packages: {}", e))?;

    let orphaned = String::from_utf8_lossy(&check_output.stdout);
    let orphaned_count = orphaned.lines().filter(|l| !l.trim().is_empty()).count();

    if orphaned_count == 0 {
        return Ok("No orphaned packages found".to_string());
    }

    // Remove orphaned packages with streaming
    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(["dnf", "autoremove", "-y", "--assumeyes"]);
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to remove orphaned packages: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str(&format!("Found {} orphaned package(s)\n", orphaned_count));
    combined_output.push_str("Running: dnf autoremove -y\n");
    combined_output.push_str("--- Output ---\n");
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stdout: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stderr: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status.success() {
        return Err(format!("Failed to remove orphaned packages (exit code: {}):\n{}", 
            status.code().unwrap_or(-1), combined_output));
    }

    Ok(format!("Removed {} orphaned package(s)\n\n{}", orphaned_count, combined_output))
}

async fn clean_package_cache_streaming() -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.args(["dnf", "clean", "all"]);
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to clean package cache: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str("Running: dnf clean all\n");
    combined_output.push_str("--- Output ---\n");
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stdout: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            combined_output.push_str(&line);
                            combined_output.push('\n');
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stderr: {}", e);
                        combined_output.push_str(&error_msg);
                        combined_output.push('\n');
                        return Err(error_msg);
                    }
                }
            }
        }
    }
    
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status.success() {
        return Err(format!("Package cache cleanup failed (exit code: {}):\n{}", 
            status.code().unwrap_or(-1), combined_output));
    }

    Ok(format!("Package cache cleaned successfully\n\n{}", combined_output))
}

async fn run_all_maintenance() -> Result<String, String> {
    let mut results = Vec::new();

    // Run all tasks in sequence
    match rebuild_kernel_modules_streaming().await {
        Ok(msg) => results.push(format!("Kernel modules: ✓ Success\n{}", msg)),
        Err(e) => results.push(format!("Kernel modules: ✗ Failed\n{}", e)),
    }

    match regenerate_initramfs_streaming().await {
        Ok(msg) => results.push(format!("Initramfs: ✓ Success\n{}", msg)),
        Err(e) => results.push(format!("Initramfs: ✗ Failed\n{}", e)),
    }

    match remove_orphaned_packages_streaming().await {
        Ok(msg) => results.push(format!("Orphaned packages: ✓ Success\n{}", msg)),
        Err(e) => results.push(format!("Orphaned packages: ✗ Failed\n{}", e)),
    }

    match clean_package_cache_streaming().await {
        Ok(msg) => results.push(format!("Cache: ✓ Success\n{}", msg)),
        Err(e) => results.push(format!("Cache: ✗ Failed\n{}", e)),
    }

    Ok(results.join("\n\n"))
}

struct SectionCardStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for SectionCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.98,
                palette.background.g * 0.98,
                palette.background.b * 0.98,
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

struct ActionCardStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for ActionCardStyle {
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
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1),
            },
            ..Default::default()
        }
    }
}

struct LogItemStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for LogItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                palette.background.r * 0.95,
                palette.background.g * 0.95,
                palette.background.b * 0.95,
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

