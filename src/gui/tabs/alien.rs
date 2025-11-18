use iced::widget::{button, column, container, row, scrollable, text, Space, progress_bar};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use tokio::process::Command as TokioCommand;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    ConvertDebToRpm,
    ConvertTgzToRpm,
    FileSelected(Option<PathBuf>, ConversionType),
    ConversionProgress(String),
    ConversionComplete(PathBuf),
    ConversionError(String),
    CancelConversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionType {
    Deb,
    Tgz,
}

#[derive(Debug)]
pub struct AlienTab {
    is_converting: bool,
    conversion_progress: String,
    conversion_output: Vec<String>,
    converted_rpm: Option<PathBuf>,
    error: Option<String>,
}

impl AlienTab {
    pub fn new() -> Self {
        Self {
            is_converting: false,
            conversion_progress: String::new(),
            conversion_output: Vec::new(),
            converted_rpm: None,
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::ConvertDebToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::Deb), |path| {
                    Message::FileSelected(path, ConversionType::Deb)
                })
            }
            Message::ConvertTgzToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::Tgz), |path| {
                    Message::FileSelected(path, ConversionType::Tgz)
                })
            }
            Message::FileSelected(Some(path), conv_type) => {
                self.is_converting = true;
                self.conversion_progress = "Starting conversion...".to_string();
                self.conversion_output.clear();
                self.error = None;
                self.converted_rpm = None;
                
                // Add initial output
                self.conversion_output.push(format!("Selected file: {}", path.display()));
                self.conversion_output.push("Starting conversion with alien...".to_string());
                
                let path_str = path.to_string_lossy().to_string();
                iced::Command::perform(convert_package_streaming(path_str, conv_type), |result| {
                    match result {
                        Ok((output_lines, rpm_path)) => {
                            // Send all output at once, then trigger completion with the actual path
                            let output_text = output_lines.join("\n");
                            // Store the output first, then send completion
                            // We need to send both the output and the completion
                            // Use a two-step approach: send output, then completion
                            Message::ConversionProgress(format!("{}\nCOMPLETE_PATH:{}", output_text, rpm_path))
                        }
                        Err(e) => Message::ConversionError(e),
                    }
                })
            }
            Message::ConversionProgress(output) => {
                // Split multi-line output and add each line
                for line in output.lines() {
                    if !line.trim().is_empty() {
                        self.conversion_output.push(line.to_string());
                    }
                }
                // Keep only last 100 lines to avoid memory issues
                if self.conversion_output.len() > 100 {
                    let remove_count = self.conversion_output.len() - 100;
                    for _ in 0..remove_count {
                        self.conversion_output.remove(0);
                    }
                }
                self.conversion_progress = "Conversion in progress...".to_string();
                
                // Check if this output contains the completion marker with the path
                if output.contains("COMPLETE_PATH:") {
                    // Extract the RPM path from the special marker we added
                    if let Some(start) = output.find("COMPLETE_PATH:") {
                        let path_line = output[start..].lines().next()
                            .and_then(|line| line.strip_prefix("COMPLETE_PATH:"))
                            .map(|s| s.trim().to_string());
                        
                        if let Some(path) = path_line {
                            // Remove the marker line from output before displaying
                            let clean_output: Vec<String> = output.lines()
                                .filter(|line| !line.contains("COMPLETE_PATH:"))
                                .map(|s| s.to_string())
                                .collect();
                            // Update output without the marker
                            self.conversion_output = clean_output;
                            
                            return iced::Command::perform(async {}, |_| Message::ConversionComplete(PathBuf::from(path)));
                        }
                    }
                }
                
                // Fallback: try to extract from success messages
                let rpm_path = if output.contains("✓ Successfully found converted RPM:") {
                    output.lines()
                        .find(|line| line.contains("✓ Successfully found converted RPM:"))
                        .and_then(|line| line.strip_prefix("✓ Successfully found converted RPM: "))
                        .map(|s| s.trim().to_string())
                } else if output.contains("✓ Found RPM file:") {
                    output.lines()
                        .find(|line| line.contains("✓ Found RPM file:"))
                        .and_then(|line| line.strip_prefix("✓ Found RPM file: "))
                        .map(|s| s.trim().to_string())
                } else if output.contains("✓ RPM file created successfully:") {
                    output.lines()
                        .find(|line| line.contains("✓ RPM file created successfully:"))
                        .and_then(|line| line.strip_prefix("✓ RPM file created successfully: "))
                        .map(|s| s.trim().to_string())
                } else if output.contains("✓ Moved RPM to:") {
                    output.lines()
                        .find(|line| line.contains("✓ Moved RPM to:"))
                        .and_then(|line| line.strip_prefix("✓ Moved RPM to: "))
                        .map(|s| s.trim().to_string())
                } else {
                    None
                };
                
                if let Some(path) = rpm_path {
                    iced::Command::perform(async {}, |_| Message::ConversionComplete(PathBuf::from(path)))
                } else {
                    iced::Command::none()
                }
            }
            Message::FileSelected(None, _) => {
                // User cancelled file selection
                iced::Command::none()
            }
            Message::ConversionComplete(rpm_path) => {
                self.is_converting = false;
                self.converted_rpm = Some(rpm_path.clone());
                self.conversion_progress = "Conversion completed successfully!".to_string();
                
                // Open RPM dialog with the converted RPM file
                // Use RpmDialog which is designed for RPM files (shows dependencies, etc.)
                let rpm_path_str = rpm_path.to_string_lossy().to_string();
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let _ = TokioCommand::new(&exe_path)
                            .arg(&rpm_path_str)  // RPM file as positional argument opens RpmDialog
                            .spawn();
                    },
                    |_| Message::CancelConversion,
                )
            }
            Message::ConversionError(error) => {
                self.is_converting = false;
                self.error = Some(error);
                self.conversion_progress = "Conversion failed!".to_string();
                iced::Command::none()
            }
            Message::CancelConversion => {
                // Reset state after opening install dialog
                self.converted_rpm = None;
                self.conversion_output.clear();
                self.conversion_progress.clear();
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
        
        // Title
        let title = container(
            text("Package Converter (Alien)")
                .size(title_font_size)
                .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0));

        // Description
        let description = container(
            column![
                text("Convert packages from other Linux distributions to RPM format for Fedora")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                Space::with_height(Length::Fixed(10.0)),
                container(
                    column![
                        text("⚠️ Important Notes:")
                            .size(body_font_size * 0.93)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.65, 0.0))),
                        Space::with_height(Length::Fixed(5.0)),
                        text("• Alien has NO options to prevent file conflicts with system packages")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                        text("• Some packages may not install due to directory conflicts (/usr/bin, /usr/lib, etc.)")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                        text("• The --fixperms option only works for DEB packages, not RPM")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                        text("• Always check for native RPM versions first when possible")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                        text("• Alien version: 8.95 (known limitations with modern packages)")
                            .size(body_font_size * 0.86)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                    ]
                    .spacing(4)
                )
                .padding(Padding::new(12.0))
                .style(iced::theme::Container::Custom(Box::new(WarningContainerStyle {
                    radius: settings.border_radius,
                })))
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0));

        // Conversion buttons
        let deb_button = button(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                text(" Convert DEB to RPM").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::ConvertDebToRpm)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(16.0))
        .width(Length::Fixed(250.0));

        let tgz_button = button(
            row![
                text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size),
                text(" Convert TGZ to RPM").size(button_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::ConvertTgzToRpm)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::new(16.0))
        .width(Length::Fixed(250.0));

        let buttons_row = row![deb_button, tgz_button]
            .spacing(20)
            .align_items(Alignment::Center);

        let buttons_container = container(buttons_row)
            .width(Length::Fill)
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            })));

        // Conversion status/progress
        let status_section = if self.is_converting {
            container(
                column![
                    text("Conversion in Progress")
                        .size(title_font_size * 0.75)
                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(15.0)),
                    progress_bar(0.0..=1.0, 0.7).width(Length::Fill),
                    Space::with_height(Length::Fixed(10.0)),
                    text(&self.conversion_progress)
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(15.0)),
                    {
                        let output_lines: Vec<Element<Message>> = if !self.conversion_output.is_empty() {
                            self.conversion_output
                                .iter()
                                .map(|line| {
                                    text(line)
                                        .size(body_font_size * 0.86)
                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                        .shaping(iced::widget::text::Shaping::Advanced)
                                        .into()
                                })
                                .collect()
                        } else {
                            vec![text("Waiting for conversion output...")
                                .size(body_font_size * 0.86)
                                .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                .into()]
                        };
                        scrollable(
                            column(output_lines)
                                .spacing(4)
                        )
                        .height(Length::Fixed(200.0))
                    },
                ]
                .spacing(10)
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            })))
        } else if let Some(ref error) = self.error {
            container(
                column![
                    text("Conversion Failed")
                        .size(title_font_size * 0.75)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3))),
                    Space::with_height(Length::Fixed(10.0)),
                    text(error)
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3)))
                        .shaping(iced::widget::text::Shaping::Advanced),
                ]
                .spacing(10)
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(ErrorContainerStyle {
                radius: settings.border_radius,
            })))
        } else if let Some(ref rpm_path) = self.converted_rpm {
            container(
                column![
                    text("Conversion Successful!")
                        .size(title_font_size * 0.75)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.8, 0.0))),
                    Space::with_height(Length::Fixed(10.0)),
                    text(format!("Converted package: {}", rpm_path.file_name().unwrap_or_default().to_string_lossy()))
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(10.0)),
                    text("The installation dialog should open automatically.")
                        .size(body_font_size * 0.86)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                ]
                .spacing(10)
            )
            .width(Length::Fill)
            .padding(Padding::new(20.0))
            .style(iced::theme::Container::Custom(Box::new(SuccessContainerStyle {
                radius: settings.border_radius,
            })))
        } else {
            container(Space::with_height(Length::Shrink))
        };

        container(
            scrollable(
                column![
                    title,
                    description,
                    buttons_container,
                    status_section,
                ]
                .spacing(20)
                .padding(Padding::new(20.0))
            )
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

async fn open_file_picker(conv_type: ConversionType) -> Option<PathBuf> {
    use tokio::process::Command;
    
    let (title, filter) = match conv_type {
        ConversionType::Deb => ("Select DEB Package to Convert", "*.deb"),
        ConversionType::Tgz => ("Select TGZ Package to Convert", "*.tgz *.tar.gz"),
    };
    
    // Try zenity first (GNOME)
    let output = Command::new("zenity")
        .args([
            "--file-selection",
            "--title", title,
            "--file-filter", filter,
        ])
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
        .args([
            "--getopenfilename",
            ".",
            filter,
        ])
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

async fn convert_package_streaming(file_path: String, _conv_type: ConversionType) -> Result<(Vec<String>, String), String> {
    let input_path = PathBuf::from(&file_path);
    
    if !input_path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Get the absolute directory where the .deb file is located
    // Alien creates the RPM in the current working directory, so we need to run it from there
    let parent_dir = input_path.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    
    // Get absolute path to ensure we're working with the correct directory
    let parent_dir_abs = parent_dir.canonicalize()
        .unwrap_or_else(|_| parent_dir.clone());
    let parent_dir_str = parent_dir_abs.to_string_lossy().to_string();
    
    // Get just the filename (not full path) for alien command
    let file_name = input_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("package.deb");
    
    // Build alien command with pkexec for elevated privileges
    // Alien requires root to install packages and manipulate system files during conversion
    // Note: Alien has no built-in options to prevent file conflicts with system directories
    // The --fixperms option only works for DEB packages, not RPM
    // Use bash -c with explicit directory change and verify with pwd
    // We need to ensure we're in the right directory because pkexec may start from /root
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("bash");
    cmd.arg("-c");
    // Change to the absolute directory, verify with pwd, then run alien
    // Using bash -c ensures proper directory handling
    let shell_cmd = format!(
        "cd '{}' || exit 1; pwd; alien --scripts -r '{}'",
        parent_dir_str, file_name
    );
    cmd.arg(&shell_cmd);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    
    // Use spawn to get streaming output
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute alien: {}. Make sure alien and polkit are installed.", e))?;
    
    // Read output in real-time
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut output_lines = Vec::new();
    output_lines.push(format!("Running: pkexec bash -c \"cd '{}' && alien --scripts -r '{}'\"", parent_dir_str, file_name));
    output_lines.push(format!("Target directory: {}", parent_dir_str));
    output_lines.push("--- Output ---".to_string());
    
    // Read both stdout and stderr line by line
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            output_lines.push(line);
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stdout: {}", e);
                        output_lines.push(error_msg.clone());
                        return Err(error_msg);
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        if !line.trim().is_empty() {
                            output_lines.push(line);
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let error_msg = format!("Error reading stderr: {}", e);
                        output_lines.push(error_msg.clone());
                        return Err(error_msg);
                    }
                }
            }
        }
    }
    
    // Wait for process to complete
    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for alien process: {}", e))?;
    
    // Check if user cancelled authentication (exit codes 126/127 = auth failure)
    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or failed. Please try again.".to_string());
    }
    
    output_lines.push(format!("--- Process exited with code: {:?} ---", status.code()));
    
    if !status.success() {
        let error_output = output_lines.join("\n");
        return Err(format!("Conversion failed:\n{}", error_output));
    }
    
    // Check the pwd output to see where we actually ran from
    let mut actual_working_dir: Option<String> = None;
    for line in &output_lines {
        if line.starts_with("/") && !line.contains(" ") && std::path::Path::new(line).is_absolute() {
            // This might be the pwd output
            if *line != parent_dir_str && (line.contains("/home") || line.contains("/root")) {
                actual_working_dir = Some(line.clone());
            }
        }
    }
    
    // Parse the actual RPM filename from alien's output
    // Alien outputs something like "discord-0.0.114-2.x86_64.rpm generated"
    let mut generated_rpm_name: Option<String> = None;
    for line in &output_lines {
        if line.contains(".rpm") && (line.contains("generated") || line.contains("created")) {
            // Extract the RPM filename from the line
            // Format is usually: "filename.rpm generated" or "filename.rpm"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.ends_with(".rpm") {
                    generated_rpm_name = Some(part.to_string());
                    break;
                }
            }
            if generated_rpm_name.is_some() {
                break;
            }
        }
    }
    
    // If pwd shows we're in /root, we need to look there too
    let search_dirs = if let Some(ref dir) = actual_working_dir {
        if dir == "/root" {
            output_lines.push(format!("WARNING: Command ran from /root instead of target directory!"));
            output_lines.push(format!("Searching in both {} and /root", parent_dir.display()));
            vec![parent_dir_abs.clone(), PathBuf::from("/root")]
        } else {
            vec![parent_dir_abs.clone()]
        }
    } else {
        vec![parent_dir_abs.clone()]
    };
    
    output_lines.push(format!("Searching for RPM in: {}", parent_dir_abs.display()));
    
    // Look for the RPM file in all search directories
    let mut found_rpm: Option<PathBuf> = None;
    
    // First, try the parsed name in each search directory
    if let Some(ref rpm_name) = generated_rpm_name {
        output_lines.push(format!("Looking for RPM file: {}", rpm_name));
        for search_dir in &search_dirs {
            let rpm_path = search_dir.join(rpm_name);
            if rpm_path.exists() {
                output_lines.push(format!("✓ Found RPM file: {}", rpm_path.display()));
                found_rpm = Some(rpm_path);
                break;
            }
        }
    }
    
    // If we didn't find it by name, search for recently created RPM files in all directories
    if found_rpm.is_none() {
        output_lines.push("Parsed RPM name not found, searching directories for recently created RPMs...".to_string());
        let mut newest_rpm: Option<PathBuf> = None;
        let mut newest_time: Option<std::time::SystemTime> = None;
        
        for search_dir in &search_dirs {
            if let Ok(entries) = std::fs::read_dir(search_dir) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "rpm" {
                            output_lines.push(format!("Found RPM file: {}", entry.path().display()));
                            // Check if it was recently created (within last 2 minutes)
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    let now = std::time::SystemTime::now();
                                    if let Ok(duration) = now.duration_since(modified) {
                                        if duration.as_secs() < 120 {
                                            // Keep track of the newest RPM file
                                            if newest_time.is_none() || modified > newest_time.unwrap() {
                                                newest_time = Some(modified);
                                                newest_rpm = Some(entry.path());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        found_rpm = newest_rpm;
    }
    
    if let Some(rpm) = found_rpm {
        // If the RPM was created in /root but should be in Downloads, move it using pkexec
        if rpm.parent() == Some(std::path::Path::new("/root")) && parent_dir_abs != PathBuf::from("/root") {
            let target_path = parent_dir_abs.join(rpm.file_name().unwrap_or_default());
            output_lines.push(format!("RPM was created in /root, moving to: {}", target_path.display()));
            
            // Use pkexec to move the file (requires root permissions)
            let rpm_str = rpm.to_string_lossy().to_string();
            let target_str = target_path.to_string_lossy().to_string();
            
            let move_cmd = TokioCommand::new("pkexec")
                .arg("mv")
                .arg(&rpm_str)
                .arg(&target_str)
                .output()
                .await;
            
            match move_cmd {
                Ok(output) if output.status.success() => {
                    output_lines.push(format!("✓ Moved RPM to: {}", target_path.display()));
                    Ok((output_lines, target_str))
                }
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr);
                    output_lines.push(format!("Warning: Could not move file: {}. Using original location.", error));
                    Ok((output_lines, rpm.to_string_lossy().to_string()))
                }
                Err(e) => {
                    output_lines.push(format!("Warning: Could not move file: {}. Using original location.", e));
                    Ok((output_lines, rpm.to_string_lossy().to_string()))
                }
            }
        } else {
            output_lines.push(format!("✓ Successfully found converted RPM: {}", rpm.display()));
            Ok((output_lines, rpm.to_string_lossy().to_string()))
        }
    } else {
        let error_output = output_lines.join("\n");
        Err(format!("RPM file was not created. Searched in: {}. Alien output:\n{}", 
            search_dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", "),
            error_output))
    }
}

// Style implementations
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

struct InfoContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for InfoContainerStyle {
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

struct ErrorContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for ErrorContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 0.3, 0.3, 0.1))),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgb(1.0, 0.3, 0.3),
            },
            ..Default::default()
        }
    }
}

struct SuccessContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for SuccessContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.8, 0.0, 0.1))),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgb(0.0, 0.8, 0.0),
            },
            ..Default::default()
        }
    }
}

struct WarningContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for WarningContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 0.65, 0.0, 0.1))),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgb(1.0, 0.65, 0.0),
            },
            ..Default::default()
        }
    }
}

