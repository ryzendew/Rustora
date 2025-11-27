use iced::widget::{button, column, container, row, scrollable, text, Space, progress_bar};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use tokio::process::Command as TokioCommand;
use crate::gui::app::CustomScrollableStyle;
use crate::gui::settings::AppSettings;

#[derive(Debug, Clone)]
pub enum Message {
    StartInstallation,
    StepProgress(String, f32),
    InstallationComplete(Result<(), String>),
    CopyToClipboard,
    Close,
}

#[derive(Debug)]
pub struct HyprlandDialog {
    is_running: bool,
    is_complete: bool,
    has_error: bool,
    progress_text: String,
    terminal_output: String,
    progress: f32,
    current_step_num: usize,
}

impl HyprlandDialog {
    pub fn new() -> Self {
        Self {
            is_running: true,
            is_complete: false,
            has_error: false,
            progress_text: "Installing Hyprland & Dependencies...".to_string(),
            terminal_output: String::new(),
            progress: 0.0,
            current_step_num: 0,
        }
    }

    pub fn run_separate_window() -> Result<(), iced::Error> {
        let dialog = Self::new();

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(1000.0, 700.0);
        window_settings.min_size = Some(iced::Size::new(800.0, 500.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <HyprlandDialog as Application>::run(iced::Settings {
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

impl Application for HyprlandDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::StartInstallation);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        "Hyprland Installation - Rustora".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartInstallation => {
                self.is_running = true;
                self.is_complete = false;
                self.has_error = false;
                self.progress = 0.0;
                self.current_step_num = 0;
                self.terminal_output.clear();
                self.terminal_output.push_str("Starting Hyprland installation...\n");
                self.terminal_output.push_str("=====================================\n\n");

                Command::perform(run_installation_step(0), |result| {
                    match result {
                        Ok((output, step_num, progress)) => {
                            if step_num >= 2 {
                                Message::InstallationComplete(Ok(()))
                            } else {
                                Message::StepProgress(output, progress)
                            }
                        }
                        Err(e) => Message::InstallationComplete(Err(e)),
                    }
                })
            }
            Message::StepProgress(output, progress) => {
                if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);
                self.progress = progress;

                self.progress_text = get_step_progress_text(self.current_step_num);

                let step_complete = output.contains("completed") || output.contains("failed");

                if step_complete {
                    if output.contains("[OK] ALL STEPS COMPLETED SUCCESSFULLY!") {
                        self.is_running = false;
                        self.is_complete = true;
                        self.progress = 1.0;
                        self.progress_text = "Installation completed successfully!".to_string();
                        return Command::none();
                    }

                    self.current_step_num += 1;

                    if self.current_step_num < 11 {
                        Command::perform(run_installation_step(self.current_step_num), |result| {
                            match result {
                                Ok((output, step_num, progress)) => {
                                    if step_num >= 11 {
                                        Message::InstallationComplete(Ok(()))
                                    } else {
                                        Message::StepProgress(output, progress)
                                    }
                                }
                                Err(e) => Message::InstallationComplete(Err(e)),
                            }
                        })
                    } else {
                        self.is_running = false;
                        self.is_complete = true;
                        self.progress = 1.0;
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::InstallationComplete(result) => {
                self.is_running = false;
                match result {
                    Ok(_) => {
                        self.is_complete = true;
                        self.progress = 1.0;
                        self.progress_text = "Installation completed successfully!".to_string();
                        if !self.terminal_output.contains("[OK] ALL STEPS COMPLETED SUCCESSFULLY!") {
                            self.terminal_output.push_str("\n[OK] All steps completed successfully!\n");
                        }
                    }
                    Err(e) => {
                        self.has_error = true;
                        self.progress_text = format!("Installation failed: {}", e);
                        self.terminal_output.push_str(&format!("\n[FAIL] Error: {}\n", e));
                    }
                }
                Command::none()
            }
            Message::CopyToClipboard => {
                let output = self.terminal_output.clone();
                Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            use std::process::Command;
                            if let Ok(mut child) = Command::new("wl-copy")
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                            {
                                if let Some(mut stdin) = child.stdin.take() {
                                    use std::io::Write;
                                    let _ = stdin.write_all(output.as_bytes());
                                    let _ = stdin.flush();
                                }
                                let _ = child.wait();
                            } else if let Ok(mut child) = Command::new("xclip")
                                .args(["-selection", "clipboard"])
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                            {
                                if let Some(mut stdin) = child.stdin.take() {
                                    use std::io::Write;
                                    let _ = stdin.write_all(output.as_bytes());
                                    let _ = stdin.flush();
                                }
                                let _ = child.wait();
                            }
                        }).await.ok();
                    },
                    |_| Message::Close
                )
            }
            Message::Close => {
                window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = crate::gui::Theme::Dark;
        let settings = AppSettings::load();
        self.view_impl(&theme, &settings)
    }

    fn theme(&self) -> IcedTheme {
        crate::gui::Theme::Dark.iced_theme()
    }
}

impl HyprlandDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme, settings: &AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let title_text = if self.is_complete {
            if self.has_error {
                "Installation Failed"
            } else {
                "Installation Complete"
            }
        } else {
            "Installing Hyprland & Dependencies"
        };

        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let icon_font_size = (settings.font_size_icons * settings.scale_icons).round();

        let progress_display = text(&self.progress_text).size(body_font_size);

        let terminal_output = scrollable(
            text(&self.terminal_output)
                .font(iced::Font::MONOSPACE)
                .size(body_font_size * 0.86)
        )
        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
            theme.background(),
            settings.border_radius,
        ))))
        .width(Length::Fill)
        .height(Length::Fill);

        let close_button: Element<Message> = button(
            row![
                text(crate::gui::fonts::glyphs::CLOSE_SYMBOL).font(material_font).size(icon_font_size),
                text(" Close").size(body_font_size)
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        )
        .on_press(Message::Close)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: !self.has_error && self.is_complete,
            radius: settings.border_radius,
            theme: *theme,
        })))
        .padding(Padding::new(12.0))
        .into();

        container(
            column![
                row![
                    text(title_text).size(title_font_size).style(iced::theme::Text::Color(
                        if self.has_error {
                            theme.danger()
                        } else if self.is_complete {
                            iced::Color::from_rgb(0.1, 0.5, 0.1)
                        } else {
                            theme.primary()
                        }
                    )),
                    Space::with_width(Length::Fill),
                    if self.is_complete || self.has_error {
                        close_button
                    } else {
                        Space::with_width(Length::Fixed(0.0)).into()
                    },
                ]
                .spacing(12)
                .align_items(Alignment::Center)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(16.0)),
                progress_bar(0.0..=1.0, self.progress)
                    .width(Length::Fill)
                    .height(Length::Fixed(8.0)),
                Space::with_height(Length::Fixed(8.0)),
                progress_display.style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(16.0)),
                column![
                    row![
                        text("Terminal Output").size(body_font_size * 0.9).style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_width(Length::Fill),
                        {
                            if !self.terminal_output.is_empty() {
                                let copy_btn: Element<Message> = button(
                                    row![
                                        text(crate::gui::fonts::glyphs::COPY_SYMBOL).font(material_font).size(icon_font_size),
                                        text(" Copy").size(body_font_size * 0.9)
                                    ]
                                    .spacing(8)
                                    .align_items(Alignment::Center)
                                )
                                .on_press(Message::CopyToClipboard)
                                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                    is_primary: false,
                                    radius: settings.border_radius,
                                    theme: *theme,
                                })))
                                .padding(Padding::new(8.0))
                                .into();
                                copy_btn
                            } else {
                                Space::with_width(Length::Fixed(0.0)).into()
                            }
                        },
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(8.0)),
                    container(terminal_output)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(12)
                        .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle {
                            radius: settings.border_radius,
                        }))),
                ]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill),
            ]
            .spacing(0)
            .padding(24)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(DialogContainerStyle {
            theme: *theme,
        })))
        .into()
    }
}

fn get_step_progress_text(step: usize) -> String {
    match step {
        0 => "Step 1/11: Updating system...".to_string(),
        1 => "Step 2/11: Enabling COPR repositories...".to_string(),
        2 => "Step 3/11: Updating package cache...".to_string(),
        3 => "Step 4/11: Installing development tools...".to_string(),
        4 => "Step 5/11: Installing desktop environment components...".to_string(),
        5 => "Step 6/11: Installing system utilities...".to_string(),
        6 => "Step 7/11: Installing applications...".to_string(),
        7 => "Step 8/11: Installing GUI tools...".to_string(),
        8 => "Step 9/11: Building and installing Mission Center...".to_string(),
        9 => "Step 10/11: Building and installing dgop...".to_string(),
        10 => "Step 11/11: Installing matugen...".to_string(),
        _ => "Installation complete!".to_string(),
    }
}

async fn run_installation_step(step: usize) -> Result<(String, usize, f32), String> {
    let total_steps = 11.0;
    let progress = (step as f32 + 1.0) / total_steps;

    match step {
        0 => {
            let output = update_system().await?;
            Ok((format!("{}\n[OK] Step 1 completed: System updated\n", output), 1, progress))
        }
        1 => {
            let output = enable_copr_repos().await?;
            Ok((format!("{}\n[OK] Step 2 completed: COPR repositories enabled\n", output), 2, progress))
        }
        2 => {
            let output = update_cache().await?;
            Ok((format!("{}\n[OK] Step 3 completed: Package cache updated\n", output), 3, progress))
        }
        3 => {
            let output = install_dev_tools().await?;
            Ok((format!("{}\n[OK] Step 4 completed: Development tools installed\n", output), 4, progress))
        }
        4 => {
            let output = install_desktop_components().await?;
            Ok((format!("{}\n[OK] Step 5 completed: Desktop environment components installed\n", output), 5, progress))
        }
        5 => {
            let output = install_system_utils().await?;
            Ok((format!("{}\n[OK] Step 6 completed: System utilities installed\n", output), 6, progress))
        }
        6 => {
            let output = install_applications().await?;
            Ok((format!("{}\n[OK] Step 7 completed: Applications installed\n", output), 7, progress))
        }
        7 => {
            let output = install_gui_tools().await?;
            Ok((format!("{}\n[OK] Step 8 completed: GUI tools installed\n", output), 8, progress))
        }
        8 => {
            let output = build_and_install_mission_center().await?;
            Ok((format!("{}\n[OK] Step 9 completed: Mission Center built and installed\n", output), 9, progress))
        }
        9 => {
            let output = build_and_install_dgop().await?;
            Ok((format!("{}\n[OK] Step 10 completed: dgop built and installed\n", output), 10, progress))
        }
        10 => {
            let output = install_matugen().await?;
            Ok((format!("{}\n[OK] Step 11 completed: matugen installed\n\n[OK] ALL STEPS COMPLETED SUCCESSFULLY!\n", output), 11, progress))
        }
        _ => Err("Invalid step number".to_string()),
    }
}

async fn execute_command_with_output(cmd: &mut TokioCommand, description: &str) -> Result<String, String> {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute {}: {}", description, e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output = String::new();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stdout: {}", e)),
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stderr: {}", e)),
                }
            }
        }
    }

    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for {}: {}", description, e))?;

    if status.code() == Some(126) || status.code() == Some(127) {
        return Err("Authentication cancelled or polkit not available. Please try again.".to_string());
    }

    if !status.success() {
        let output_lower = output.to_lowercase();
        if output_lower.contains("already installed") || output_lower.contains("is already installed") {
            output.push_str("\n[INFO] Note: Repository was already enabled. Continuing...\n");
            return Ok(output);
        }
        return Err(format!("Command failed with exit code: {:?}\n\nOutput:\n{}", status.code(), output));
    }

    Ok(output)
}

async fn update_system() -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("update");
    cmd.arg("-y");

    let mut output = String::new();
    output.push_str("$ pkexec dnf update -y\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "system update").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn check_copr_repo_enabled(repo: &str) -> bool {
    let mut cmd = TokioCommand::new("dnf");
    cmd.arg("copr");
    cmd.arg("list");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    if let Ok(o) = cmd.output().await {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout).to_lowercase();
            let repo_lower = repo.to_lowercase();
            let full_url = format!("copr.fedorainfracloud.org/{}", repo_lower);
            
            if stdout.contains(&repo_lower) || stdout.contains(&full_url) {
                return true;
            }
        }
    }
    false
}

async fn enable_copr_repos() -> Result<String, String> {
    let mut output = String::new();
    
    let repo = "lions/hyprland";
    if check_copr_repo_enabled(repo).await {
        output.push_str(&format!("$ Repository {} is already enabled\n", repo));
        output.push_str("-------------------------------------------------------------\n");
        output.push_str("[INFO] Skipping - repository already enabled\n");
    } else {
        output.push_str(&format!("$ pkexec dnf copr enable -y {}\n", repo));
        output.push_str("-------------------------------------------------------------\n");

        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("dnf");
        cmd.arg("copr");
        cmd.arg("enable");
        cmd.arg("-y");
        cmd.arg(repo);

        match execute_command_with_output(&mut cmd, "COPR repository lions/hyprland").await {
            Ok(cmd_output) => {
                output.push_str(&cmd_output);
            }
            Err(e) => {
                let error_lower = e.to_lowercase();
                if error_lower.contains("chroot not found") || error_lower.contains("404") {
                    output.push_str("[WARN] Repository lions/hyprland not available for this Fedora version\n");
                    output.push_str("[INFO] This may mean the repository doesn't have a chroot for your Fedora release\n");
                    output.push_str("[INFO] Continuing anyway - quickshell-git may need to be installed manually if needed\n");
                } else {
                    output.push_str(&format!("[WARN] Failed to enable {}: {}\n", repo, e));
                    output.push_str("[INFO] Continuing anyway - packages may be available from other sources\n");
                }
            }
        }
    }

    Ok(output)
}

async fn filter_installed_packages(packages: &[&str]) -> Vec<String> {
    let mut to_install = Vec::new();
    
    for pkg in packages {
        let is_installed = tokio::task::spawn_blocking({
            let pkg = pkg.to_string();
            move || {
                std::process::Command::new("rpm")
                    .arg("-q")
                    .arg(&pkg)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
        })
        .await
        .unwrap_or(false);
        
        if !is_installed {
            to_install.push(pkg.to_string());
        }
    }
    
    to_install
}

async fn update_cache() -> Result<String, String> {
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("makecache");

    let mut output = String::new();
    output.push_str("$ pkexec dnf makecache\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "package cache update").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn install_dev_tools() -> Result<String, String> {
    let packages = vec![
        "rust", "cargo",
        "gcc", "gcc-c++", "pkg-config",
        "openssl-devel",
        "libX11-devel", "libXcursor-devel", "libXrandr-devel", "libXi-devel",
        "mesa-libGL-devel",
        "fontconfig-devel", "freetype-devel", "expat-devel",
        "dnf", "rpm", "polkit", "zenity", "curl", "unzip", "fontconfig",
        "cairo-gobject", "cairo-gobject-devel",
        "rust-gdk4-sys+default-devel",
        "gtk4-layer-shell-devel",
        "qt5-qtgraphicaleffects",
        "qt6-qt5compat",
        "python3-pyqt6",
        "python3.11", "python3.11-libs",
        "libxcrypt-compat", "libcurl", "libcurl-devel", "apr", "fuse-libs",
        "golang", "git", "make",
        "meson", "ninja-build",
        "gtk4-devel", "libadwaita-devel",
    ];

    let to_install = filter_installed_packages(&packages).await;
    
    if to_install.is_empty() {
        let mut output = String::new();
        output.push_str("$ All development tools are already installed\n");
        output.push_str("(Checking for updates...)\n");
        output.push_str("-------------------------------------------------------------\n");
        
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("dnf");
        cmd.arg("upgrade");
        cmd.arg("-y");
        for pkg in &packages {
            cmd.arg(pkg);
        }
        
        let cmd_output = execute_command_with_output(&mut cmd, "development tools updates").await?;
        output.push_str(&cmd_output);
        return Ok(output);
    }

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    for pkg in &to_install {
        cmd.arg(pkg);
    }

    let mut output = String::new();
    output.push_str("$ pkexec dnf install -y ");
    output.push_str(&to_install.join(" "));
    output.push_str("\n");
    output.push_str("(Note: Already installed packages will be checked for updates, not reinstalled)\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "development tools").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn install_desktop_components() -> Result<String, String> {
    let packages = vec![
        "hyprland",
        "hyprpicker",
        "awww",
        "xdg-desktop-portal-hyprland",
        "xdg-desktop-portal-wlr",
        "xdg-desktop-portal-gnome",
        "gnome-keyring",
    ];

    let to_install = filter_installed_packages(&packages).await;
    
    if to_install.is_empty() {
        let mut output = String::new();
        output.push_str("$ All desktop components are already installed\n");
        output.push_str("(Checking for updates...)\n");
        output.push_str("-------------------------------------------------------------\n");
        
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("dnf");
        cmd.arg("upgrade");
        cmd.arg("-y");
        for pkg in &packages {
            cmd.arg(pkg);
        }
        
        let cmd_output = execute_command_with_output(&mut cmd, "desktop components updates").await?;
        output.push_str(&cmd_output);
        return Ok(output);
    }

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    for pkg in &to_install {
        cmd.arg(pkg);
    }

    let mut output = String::new();
    output.push_str("$ pkexec dnf install -y ");
    output.push_str(&to_install.join(" "));
    output.push_str("\n");
    output.push_str("(Note: Already installed packages will be checked for updates, not reinstalled)\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "desktop environment components").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn install_system_utils() -> Result<String, String> {
    let packages = vec![
        "brightnessctl",
        "cliphist",
        "fuzzel",
        "gnome-text-editor",
        "grim",
        "nautilus",
        "pavucontrol",
        "ptyxis",
        "slurp",
        "swappy",
        "tesseract",
        "wl-clipboard",
        "wlogout",
        "yad",
        "btop",
        "lm_sensors",
        "fuse", "fuse-libs",
        "gedit",
    ];

    let to_install = filter_installed_packages(&packages).await;
    
    if to_install.is_empty() {
        let mut output = String::new();
        output.push_str("$ All system utilities are already installed\n");
        output.push_str("(Checking for updates...)\n");
        output.push_str("-------------------------------------------------------------\n");
        
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("dnf");
        cmd.arg("upgrade");
        cmd.arg("-y");
        for pkg in &packages {
            cmd.arg(pkg);
        }
        
        let cmd_output = execute_command_with_output(&mut cmd, "system utilities updates").await?;
        output.push_str(&cmd_output);
        return Ok(output);
    }

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    for pkg in &to_install {
        cmd.arg(pkg);
    }

    let mut output = String::new();
    output.push_str("$ pkexec dnf install -y ");
    output.push_str(&to_install.join(" "));
    output.push_str("\n");
    output.push_str("(Note: Already installed packages will be checked for updates, not reinstalled)\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "system utilities").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn install_applications() -> Result<String, String> {
    let packages = vec![
        "firefox",
        "obs-studio",
        "steam", "lutris", "mangohud", "gamescope",
    ];

    let to_install = filter_installed_packages(&packages).await;
    
    if to_install.is_empty() {
        let mut output = String::new();
        output.push_str("$ All applications are already installed\n");
        output.push_str("(Checking for updates...)\n");
        output.push_str("-------------------------------------------------------------\n");
        
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("dnf");
        cmd.arg("upgrade");
        cmd.arg("-y");
        for pkg in &packages {
            cmd.arg(pkg);
        }
        
        let cmd_output = execute_command_with_output(&mut cmd, "applications updates").await?;
        output.push_str(&cmd_output);
        return Ok(output);
    }

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    for pkg in &to_install {
        cmd.arg(pkg);
    }

    let mut output = String::new();
    output.push_str("$ pkexec dnf install -y ");
    output.push_str(&to_install.join(" "));
    output.push_str("\n");
    output.push_str("(Note: Already installed packages will be checked for updates, not reinstalled)\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "applications").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn install_gui_tools() -> Result<String, String> {
    let packages = vec![
        "qt6ct",
        "nwg-look",
        "quickshell-git",
    ];

    let to_install = filter_installed_packages(&packages).await;
    
    if to_install.is_empty() {
        let mut output = String::new();
        output.push_str("$ All GUI tools are already installed\n");
        output.push_str("(Checking for updates...)\n");
        output.push_str("-------------------------------------------------------------\n");
        
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("dnf");
        cmd.arg("upgrade");
        cmd.arg("-y");
        for pkg in &packages {
            cmd.arg(pkg);
        }
        
        let cmd_output = execute_command_with_output(&mut cmd, "GUI tools updates").await?;
        output.push_str(&cmd_output);
        return Ok(output);
    }

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("dnf");
    cmd.arg("install");
    cmd.arg("-y");
    for pkg in &to_install {
        cmd.arg(pkg);
    }

    let mut output = String::new();
    output.push_str("$ pkexec dnf install -y ");
    output.push_str(&to_install.join(" "));
    output.push_str("\n");
    output.push_str("(Note: Already installed packages will be checked for updates, not reinstalled)\n");
    output.push_str("-------------------------------------------------------------\n");

    let cmd_output = execute_command_with_output(&mut cmd, "GUI tools").await?;
    output.push_str(&cmd_output);

    Ok(output)
}

async fn build_and_install_mission_center() -> Result<String, String> {
    let mut output = String::new();
    output.push_str("$ Building and installing Mission Center from source\n");
    output.push_str("-------------------------------------------------------------\n");

    let build_script = r#"
        cd /tmp
        if [ -d mission-center ]; then
            rm -rf mission-center
        fi
        git clone https://gitlab.com/mission-center-devs/mission-center.git mission-center
        cd mission-center
        
        if [ -f meson.build ]; then
            meson setup build --prefix=/usr
            meson compile -C build
            meson install -C build
        elif [ -f CMakeLists.txt ]; then
            mkdir -p build
            cd build
            cmake .. -DCMAKE_INSTALL_PREFIX=/usr
            make
            make install
        elif [ -f Makefile ]; then
            make
            make install
        else
            echo "Unknown build system"
            exit 1
        fi
        
        cd /tmp
        rm -rf mission-center
    "#;

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("sh");
    cmd.arg("-c");
    cmd.arg(build_script);

    let cmd_output = execute_command_with_output(&mut cmd, "Mission Center build and install").await?;
    output.push_str(&cmd_output);
    output.push_str("\n[INFO] Mission Center installed successfully!\n");

    Ok(output)
}

async fn build_and_install_dgop() -> Result<String, String> {
    let mut output = String::new();
    output.push_str("$ Building and installing dgop from source\n");
    output.push_str("-------------------------------------------------------------\n");

    let build_script = r#"
        cd /tmp
        if [ -d dgop ]; then
            rm -rf dgop
        fi
        git clone https://github.com/AvengeMedia/dgop.git
        cd dgop
        make
        make install
        cd ..
        rm -rf dgop
    "#;

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("sh");
    cmd.arg("-c");
    cmd.arg(build_script);

    let cmd_output = execute_command_with_output(&mut cmd, "dgop build and install").await?;
    output.push_str(&cmd_output);
    output.push_str("\n[INFO] dgop installed successfully!\n");
    output.push_str("[INFO] Note: For NVIDIA GPU temperature monitoring, install nvidia-utils (optional)\n");

    Ok(output)
}

async fn install_matugen() -> Result<String, String> {
    let mut output = String::new();
    output.push_str("$ Installing matugen via cargo\n");
    output.push_str("-------------------------------------------------------------\n");

    let check_script = r#"
        if command -v matugen &> /dev/null; then
            echo "matugen is already installed"
            exit 0
        fi
    "#;

    let mut check_cmd = TokioCommand::new("sh");
    check_cmd.arg("-c");
    check_cmd.arg(check_script);

    let check_output = check_cmd.output().await.ok();
    if let Some(o) = check_output {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("already installed") {
                output.push_str("[INFO] matugen is already installed!\n");
                return Ok(output);
            }
        }
    }

    let mut cmd = TokioCommand::new("cargo");
    cmd.arg("install");
    cmd.arg("matugen");

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to install matugen: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stdout: {}", e)),
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Error reading stderr: {}", e)),
                }
            }
        }
    }

    let status = child.wait().await
        .map_err(|e| format!("Failed to wait for matugen installation: {}", e))?;

    if !status.success() {
        return Err(format!("matugen installation failed with exit code: {:?}\n\nOutput:\n{}", status.code(), output));
    }

    output.push_str("\n[INFO] matugen installed successfully!\n");

    Ok(output)
}

struct RoundedButtonStyle {
    is_primary: bool,
    radius: f32,
    theme: crate::gui::Theme,
}

impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Background::Color(if self.is_primary {
                self.theme.primary()
            } else {
                iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1)
            })),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: if self.is_primary {
                    self.theme.primary()
                } else {
                    iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3)
                },
            },
            text_color: self.theme.text(),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(_style);
        appearance.background = Some(iced::Background::Color(if self.is_primary {
            let primary = self.theme.primary();
            iced::Color::from_rgba(primary.r * 0.9, primary.g * 0.9, primary.b * 0.9, 1.0)
        } else {
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2)
        }));
        appearance
    }
}

struct TerminalContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
            border: Border {
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                width: 1.0,
                radius: self.radius.into(),
            },
            ..Default::default()
        }
    }
}

struct DialogContainerStyle {
    theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.theme.background())),
            ..Default::default()
        }
    }
}
