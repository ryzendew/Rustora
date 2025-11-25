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
    ConvertRpmToRpm,
    ConvertDirToRpm,
    ConvertTarToRpm,
    ConvertTgzToRpm,
    ConvertGemToRpm,
    ConvertPythonToRpm,
    ConvertNpmToRpm,
    ConvertCpanToRpm,
    ConvertZipToRpm,
    // Convert FROM RPM
    ConvertRpmToDeb,
    ConvertRpmToTar,
    ConvertRpmToDir,
    ConvertRpmToZip,
    FileSelected(Option<PathBuf>, ConversionType),
    ConversionProgress(String),
    ConversionComplete(PathBuf),
    ConversionError(String),
    CancelConversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionType {
    // Source types for RPM output
    DebToRpm,
    RpmToRpm,
    DirToRpm,
    TarToRpm,
    TgzToRpm,
    GemToRpm,
    PythonToRpm,
    NpmToRpm,
    CpanToRpm,
    ZipToRpm,
    // Source types for other outputs
    RpmToDeb,
    RpmToTar,
    RpmToDir,
    RpmToZip,
}

#[derive(Debug)]
pub struct FpmTab {
    is_converting: bool,
    conversion_progress: String,
    conversion_output: Vec<String>,
    converted_file: Option<PathBuf>,
    error: Option<String>,
}

impl FpmTab {
    pub fn new() -> Self {
        Self {
            is_converting: false,
            conversion_progress: String::new(),
            conversion_output: Vec::new(),
            converted_file: None,
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::ConvertDebToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::DebToRpm), |path| {
                    Message::FileSelected(path, ConversionType::DebToRpm)
                })
            }
            Message::ConvertRpmToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::RpmToRpm), |path| {
                    Message::FileSelected(path, ConversionType::RpmToRpm)
                })
            }
            Message::ConvertDirToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::DirToRpm), |path| {
                    Message::FileSelected(path, ConversionType::DirToRpm)
                })
            }
            Message::ConvertTarToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::TarToRpm), |path| {
                    Message::FileSelected(path, ConversionType::TarToRpm)
                })
            }
            Message::ConvertTgzToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::TgzToRpm), |path| {
                    Message::FileSelected(path, ConversionType::TgzToRpm)
                })
            }
            Message::ConvertGemToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::GemToRpm), |path| {
                    Message::FileSelected(path, ConversionType::GemToRpm)
                })
            }
            Message::ConvertPythonToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::PythonToRpm), |path| {
                    Message::FileSelected(path, ConversionType::PythonToRpm)
                })
            }
            Message::ConvertNpmToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::NpmToRpm), |path| {
                    Message::FileSelected(path, ConversionType::NpmToRpm)
                })
            }
            Message::ConvertCpanToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::CpanToRpm), |path| {
                    Message::FileSelected(path, ConversionType::CpanToRpm)
                })
            }
            Message::ConvertZipToRpm => {
                iced::Command::perform(open_file_picker(ConversionType::ZipToRpm), |path| {
                    Message::FileSelected(path, ConversionType::ZipToRpm)
                })
            }
            Message::ConvertRpmToDeb => {
                iced::Command::perform(open_file_picker(ConversionType::RpmToDeb), |path| {
                    Message::FileSelected(path, ConversionType::RpmToDeb)
                })
            }
            Message::ConvertRpmToTar => {
                iced::Command::perform(open_file_picker(ConversionType::RpmToTar), |path| {
                    Message::FileSelected(path, ConversionType::RpmToTar)
                })
            }
            Message::ConvertRpmToDir => {
                iced::Command::perform(open_file_picker(ConversionType::RpmToDir), |path| {
                    Message::FileSelected(path, ConversionType::RpmToDir)
                })
            }
            Message::ConvertRpmToZip => {
                iced::Command::perform(open_file_picker(ConversionType::RpmToZip), |path| {
                    Message::FileSelected(path, ConversionType::RpmToZip)
                })
            }
            Message::FileSelected(Some(path), conv_type) => {
                self.is_converting = true;
                self.conversion_progress = "Starting conversion...".to_string();
                self.conversion_output.clear();
                self.error = None;
                self.converted_file = None;

                self.conversion_output.push(format!("Selected file: {}", path.display()));
                self.conversion_output.push("Starting conversion with fpm...".to_string());

                let path_str = path.to_string_lossy().into_owned();
                iced::Command::perform(convert_package_streaming(path_str, conv_type), |result| {
                    match result {
                        Ok((output_lines, output_path)) => {
                            let output_text = output_lines.join("\n");
                            Message::ConversionProgress(format!("{}\nCOMPLETE_PATH:{}", output_text, output_path))
                        }
                        Err(e) => Message::ConversionError(e),
                    }
                })
            }
            Message::ConversionProgress(output) => {
                for line in output.lines() {
                    if !line.trim().is_empty() {
                        self.conversion_output.push(line.to_string());
                    }
                }
                if self.conversion_output.len() > 100 {
                    let remove_count = self.conversion_output.len() - 100;
                    for _ in 0..remove_count {
                        self.conversion_output.remove(0);
                    }
                }
                self.conversion_progress = "Conversion in progress...".to_string();

                if output.contains("COMPLETE_PATH:") {
                    if let Some(start) = output.find("COMPLETE_PATH:") {
                        let path_line = output[start..].lines().next()
                            .and_then(|line| line.strip_prefix("COMPLETE_PATH:"))
                            .map(|s| s.trim().to_string());

                        if let Some(path) = path_line {
                            self.conversion_output.retain(|line| !line.contains("COMPLETE_PATH:"));

                            return iced::Command::perform(async {}, |_| Message::ConversionComplete(PathBuf::from(path)));
                        }
                    }
                }

                let file_path = if output.contains("[OK] Successfully found converted file:") {
                    output.lines()
                        .find(|line| line.contains("[OK] Successfully found converted file:"))
                        .and_then(|line| line.strip_prefix("[OK] Successfully found converted file: "))
                        .map(|s| s.trim().to_string())
                } else if output.contains("[OK] Found file:") {
                    output.lines()
                        .find(|line| line.contains("[OK] Found file:"))
                        .and_then(|line| line.strip_prefix("[OK] Found file: "))
                        .map(|s| s.trim().to_string())
                } else {
                    None
                };

                if let Some(path) = file_path {
                    iced::Command::perform(async {}, |_| Message::ConversionComplete(PathBuf::from(path)))
                } else {
                    iced::Command::none()
                }
            }
            Message::FileSelected(None, _) => {
                // User cancelled file selection
                iced::Command::none()
            }
            Message::ConversionComplete(file_path) => {
                self.is_converting = false;
                self.converted_file = Some(file_path.clone());
                self.conversion_progress = "Conversion completed successfully!".to_string();

                if file_path.extension().and_then(|s| s.to_str()) == Some("rpm") {
                    let file_path_str = file_path.to_string_lossy().to_string();
                iced::Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let _ = TokioCommand::new(&exe_path)
                                .arg(&file_path_str)
                            .spawn();
                    },
                    |_| Message::CancelConversion,
                )
                } else {
                    iced::Command::none()
                }
            }
            Message::ConversionError(error) => {
                self.is_converting = false;
                self.error = Some(error);
                self.conversion_progress = "Conversion failed!".to_string();
                iced::Command::none()
            }
            Message::CancelConversion => {
                self.converted_file = None;
                self.conversion_output.clear();
                self.conversion_progress.clear();
                iced::Command::none()
            }
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons).round();

        // Header section
        let header = container(
            column![
                text("Package Converter")
                    .size(title_font_size * 1.2)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                Space::with_height(Length::Fixed(8.0)),
                text("Convert packages between different formats using FPM (Effing Package Manager)")
                    .size(body_font_size * 0.95)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
            .padding(Padding::from([0.0, 0.0, 24.0, 0.0]));

        let info_card = container(
            column![
                row![
                    text("[INFO]").size(body_font_size * 1.1),
                    Space::with_width(Length::Fixed(8.0)),
                    text("About FPM")
                        .size(body_font_size * 1.0)
                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                ]
                .align_items(Alignment::Center),
                Space::with_height(Length::Fixed(12.0)),
                text("FPM (Effing Package Manager) is a modern tool for creating packages across multiple formats. It supports conversion between Debian, RPM, and many other package formats without requiring root privileges.")
                    .size(body_font_size * 0.9)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                Space::with_height(Length::Fixed(8.0)),
                container(
                    text("[WARN] Note: When converting DEB to RPM, dependency names are not automatically mapped. You may need to install Fedora equivalents manually (e.g., libc6 → glibc, libgtk-3-0 → gtk3).")
                        .size(body_font_size * 0.85)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.65, 0.0)))
                )
                .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
                .style(iced::theme::Container::Custom(Box::new(WarningContainerStyle {
                    radius: settings.border_radius,
                }))),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::from([16.0, 20.0, 16.0, 20.0]))
        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
            radius: settings.border_radius,
        })));

        let create_button = |label: &str, msg: Message| -> Element<Message> {
            button(
            row![
                    text(crate::gui::fonts::glyphs::DOWNLOAD_SYMBOL).font(material_font).size(icon_size * 0.9),
                    Space::with_width(Length::Fixed(8.0)),
                    text(label).size(button_font_size * 0.95)
            ]
                .spacing(0)
            .align_items(Alignment::Center)
        )
            .on_press(msg)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
            .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
            .into()
        };

        let to_rpm_section = container(
            column![
            row![
                    text("Convert TO RPM")
                        .size(body_font_size * 1.15)
                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                ]
                .width(Length::Fill),
                Space::with_height(Length::Fixed(16.0)),
                row![
                    create_button("DEB → RPM", Message::ConvertDebToRpm),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("RPM → RPM", Message::ConvertRpmToRpm),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("Directory → RPM", Message::ConvertDirToRpm),
                ]
                .spacing(0)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(12.0)),
                row![
                    create_button("TAR → RPM", Message::ConvertTarToRpm),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("TGZ → RPM", Message::ConvertTgzToRpm),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("ZIP → RPM", Message::ConvertZipToRpm),
            ]
                .spacing(0)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(12.0)),
                row![
                    create_button("Gem → RPM", Message::ConvertGemToRpm),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("Python → RPM", Message::ConvertPythonToRpm),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("NPM → RPM", Message::ConvertNpmToRpm),
                ]
                .spacing(0)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(12.0)),
                row![
                    create_button("CPAN → RPM", Message::ConvertCpanToRpm),
                ]
                .spacing(0)
                .width(Length::Fill),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
        .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
            radius: settings.border_radius,
        })));

        let from_rpm_section = container(
            column![
                row![
                    text("Convert FROM RPM")
                        .size(body_font_size * 1.15)
                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                ]
                .width(Length::Fill),
                Space::with_height(Length::Fixed(16.0)),
                row![
                    create_button("RPM → DEB", Message::ConvertRpmToDeb),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("RPM → TAR", Message::ConvertRpmToTar),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("RPM → Directory", Message::ConvertRpmToDir),
                    Space::with_width(Length::Fixed(12.0)),
                    create_button("RPM → ZIP", Message::ConvertRpmToZip),
                ]
                .spacing(0)
                .width(Length::Fill),
            ]
            .spacing(0)
        )
            .width(Length::Fill)
        .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
            radius: settings.border_radius,
        })));

        let status_section = if self.is_converting {
            container(
                column![
                    row![
                        text("[RUN]").size(body_font_size * 1.2),
                        Space::with_width(Length::Fixed(12.0)),
                    text("Conversion in Progress")
                            .size(body_font_size * 1.1)
                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(16.0)),
                    progress_bar(0.0..=1.0, 0.7).width(Length::Fill).height(Length::Fixed(8.0)),
                    Space::with_height(Length::Fixed(16.0)),
                    text(&self.conversion_progress)
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(16.0)),
                    container(
                        scrollable(
                            column(
                                if !self.conversion_output.is_empty() {
                            self.conversion_output
                                .iter()
                                .map(|line| {
                                    text(line)
                                                .size(body_font_size * 0.85)
                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                        .shaping(iced::widget::text::Shaping::Advanced)
                                        .into()
                                })
                                .collect()
                        } else {
                            vec![text("Waiting for conversion output...")
                                        .size(body_font_size * 0.85)
                                .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                .into()]
                                }
                            )
                            .spacing(6)
                        )
                        .height(Length::Fixed(240.0))
                    )
                    .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                        radius: settings.border_radius,
                    }))),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
            .style(iced::theme::Container::Custom(Box::new(InfoContainerStyle {
                radius: settings.border_radius,
            })))
        } else if let Some(ref error) = self.error {
            container(
                column![
                    row![
                        text("[FAIL]").size(body_font_size * 1.2),
                        Space::with_width(Length::Fixed(12.0)),
                    text("Conversion Failed")
                            .size(body_font_size * 1.1)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3))),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(16.0)),
                    container(
                    text(error)
                            .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3)))
                            .shaping(iced::widget::text::Shaping::Advanced)
                    )
                    .width(Length::Fill),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
            .style(iced::theme::Container::Custom(Box::new(ErrorContainerStyle {
                radius: settings.border_radius,
            })))
        } else if let Some(ref file_path) = self.converted_file {
            container(
                column![
                    row![
                        text("[OK]").size(body_font_size * 1.2)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.8, 0.0))),
                        Space::with_width(Length::Fixed(12.0)),
                        text("Conversion Successful")
                            .size(body_font_size * 1.1)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.8, 0.0))),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_height(Length::Fixed(16.0)),
                    text(format!("Package: {}", file_path.file_name().unwrap_or_default().to_string_lossy()))
                        .size(body_font_size * 0.95)
                        .style(iced::theme::Text::Color(theme.text_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(8.0)),
                    text(if file_path.extension().and_then(|s| s.to_str()) == Some("rpm") {
                        "The installation dialog will open automatically."
                    } else {
                        "File saved in the same directory as the source file."
                    })
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
            .style(iced::theme::Container::Custom(Box::new(SuccessContainerStyle {
                radius: settings.border_radius,
            })))
        } else {
            container(Space::with_height(Length::Shrink))
        };

        container(
            scrollable(
                column![
                    header,
                    Space::with_height(Length::Fixed(24.0)),
                    info_card,
                    Space::with_height(Length::Fixed(24.0)),
                    to_rpm_section,
                    Space::with_height(Length::Fixed(20.0)),
                    from_rpm_section,
                    Space::with_height(Length::Fixed(20.0)),
                    status_section,
                ]
                .spacing(0)
                .width(Length::Fill)
            )
            .style(iced::theme::Scrollable::Custom(Box::new(
                crate::gui::app::CustomScrollableStyle::new(
                    iced::Color::from(settings.background_color.clone()),
                    settings.border_radius,
                )
            )))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::from([32.0, 32.0, 32.0, 32.0]))
        .into()
    }
}

// Helper function to extract path from command output
fn extract_path_from_output(output: &[u8]) -> Option<PathBuf> {
    let binding = String::from_utf8_lossy(output);
    let path = binding.trim();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

async fn open_file_picker(conv_type: ConversionType) -> Option<PathBuf> {
    use tokio::process::Command;

    let (title, filter) = match conv_type {
        ConversionType::DebToRpm => ("Select DEB Package to Convert", "*.deb"),
        ConversionType::RpmToRpm | ConversionType::RpmToDeb | ConversionType::RpmToTar | ConversionType::RpmToDir | ConversionType::RpmToZip => ("Select RPM Package to Convert", "*.rpm"),
        ConversionType::DirToRpm => ("Select Directory to Convert", ""),
        ConversionType::TarToRpm => ("Select TAR Archive to Convert", "*.tar"),
        ConversionType::TgzToRpm => ("Select TGZ Archive to Convert", "*.tgz *.tar.gz"),
        ConversionType::GemToRpm => ("Select Ruby Gem to Convert", "*.gem"),
        ConversionType::PythonToRpm => ("Select Python Package", "*.whl *.tar.gz *.zip"),
        ConversionType::NpmToRpm => ("Select NPM Package", "*.tgz"),
        ConversionType::CpanToRpm => ("Select CPAN Package", "*.tar.gz"),
        ConversionType::ZipToRpm => ("Select ZIP Archive to Convert", "*.zip"),
    };

    // For directories, use directory selection
    if conv_type == ConversionType::DirToRpm {
        // Try zenity first
        if let Ok(output) = Command::new("zenity")
            .args(["--file-selection", "--title", title, "--directory"])
        .output()
            .await
        {
        if output.status.success() {
                if let Some(path) = extract_path_from_output(&output.stdout) {
                    return Some(path);
            }
        }
    }

    // Fallback to kdialog
        if let Ok(output) = Command::new("kdialog")
            .args(["--getexistingdirectory", "."])
        .output()
            .await
        {
            if output.status.success() {
                if let Some(path) = extract_path_from_output(&output.stdout) {
                    return Some(path);
                }
            }
        }
        return None;
    }

    // Try zenity first (GNOME)
    if let Ok(output) = Command::new("zenity")
        .args(["--file-selection", "--title", title, "--file-filter", filter])
        .output()
        .await
    {
        if output.status.success() {
            if let Some(path) = extract_path_from_output(&output.stdout) {
                return Some(path);
            }
        }
    }

    // Fallback to kdialog
    if let Ok(output) = Command::new("kdialog")
        .args(["--getopenfilename", ".", filter])
        .output()
        .await
    {
        if output.status.success() {
            if let Some(path) = extract_path_from_output(&output.stdout) {
                return Some(path);
            }
        }
    }

    None
}

async fn convert_package_streaming(file_path: String, conv_type: ConversionType) -> Result<(Vec<String>, String), String> {
    let input_path = PathBuf::from(&file_path);

    if !input_path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Get the absolute directory where the input file is located
    let parent_dir = input_path.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    let parent_dir_abs = parent_dir.canonicalize()
        .unwrap_or_else(|_| parent_dir.clone());
    let parent_dir_str = parent_dir_abs.to_string_lossy().to_string();

    let input_file_abs = input_path.canonicalize()
        .unwrap_or_else(|_| input_path.clone());
    let input_file_str = input_file_abs.to_string_lossy().to_string();

    let mut output_lines = Vec::new();
    output_lines.push(format!("Input file: {}", input_file_str));
    output_lines.push(format!("Working directory: {}", parent_dir_str));

    async fn extract_archive(
        archive_path: &str,
        extract_type: &str,
        output_lines: &mut Vec<String>,
    ) -> Result<PathBuf, String> {
        let temp_dir = std::env::temp_dir().join(format!(
            "fpm_extract_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));

            std::fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        output_lines.push(format!("Extracting {} to: {}", extract_type, temp_dir.display()));

        let temp_dir_str = temp_dir.to_string_lossy();
        let mut extract_cmd = match extract_type {
            "TAR" => {
                let mut cmd = TokioCommand::new("tar");
                cmd.args(["-xf", archive_path, "-C", &temp_dir_str]);
                cmd
            }
            "TGZ" => {
                let mut cmd = TokioCommand::new("tar");
                cmd.args(["-xzf", archive_path, "-C", &temp_dir_str]);
                cmd
            }
            "ZIP" => {
                let mut cmd = TokioCommand::new("unzip");
                cmd.args(["-q", archive_path, "-d", &temp_dir_str]);
                cmd
            }
            _ => return Err(format!("Unknown extract type: {}", extract_type)),
        };

        let result = extract_cmd.output().await
            .map_err(|e| format!("Failed to extract {}: {}", extract_type, e))?;

        if !result.status.success() {
            let error = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Failed to extract {} archive: {}", extract_type, error));
            }

        output_lines.push(format!("[OK] {} extracted successfully", extract_type));
        Ok(temp_dir)
    }

    // Determine source type, target type, and any special handling
    let (source_type, target_type, temp_extract_dir, target_ext) = match conv_type {
        ConversionType::DebToRpm => ("deb", "rpm", None, "rpm"),
        ConversionType::RpmToRpm => ("rpm", "rpm", None, "rpm"),
        ConversionType::DirToRpm => ("dir", "rpm", None, "rpm"),
        ConversionType::TarToRpm => {
            let temp_dir = extract_archive(&input_file_str, "TAR", &mut output_lines).await?;
            ("dir", "rpm", Some(temp_dir), "rpm")
        }
        ConversionType::TgzToRpm => {
            let temp_dir = extract_archive(&input_file_str, "TGZ", &mut output_lines).await?;
            ("dir", "rpm", Some(temp_dir), "rpm")
        }
        ConversionType::GemToRpm => ("gem", "rpm", None, "rpm"),
        ConversionType::PythonToRpm => ("python", "rpm", None, "rpm"),
        ConversionType::NpmToRpm => ("npm", "rpm", None, "rpm"),
        ConversionType::CpanToRpm => ("cpan", "rpm", None, "rpm"),
        ConversionType::ZipToRpm => {
            let temp_dir = extract_archive(&input_file_str, "ZIP", &mut output_lines).await?;
            ("dir", "rpm", Some(temp_dir), "rpm")
        }
        ConversionType::RpmToDeb => ("rpm", "deb", None, "deb"),
        ConversionType::RpmToTar => ("rpm", "tar", None, "tar"),
        ConversionType::RpmToDir => ("rpm", "dir", None, ""),
        ConversionType::RpmToZip => ("rpm", "zip", None, "zip"),
    };

    output_lines.push(format!("Source type: {}, Target type: {}", source_type, target_type));

    let mut cmd = TokioCommand::new("fpm");

    cmd.arg("-s").arg(source_type);

    cmd.arg("-t").arg(target_type);

    cmd.arg("-f");

    cmd.arg("--verbose");

    if source_type == "deb" && target_type == "rpm" {
        cmd.arg("--no-auto-depends");
        output_lines.push("Skipping automatic dependency detection to avoid Debian package name conflicts".to_string());
        output_lines.push("Common Debian → Fedora mappings:".to_string());
        output_lines.push("  • libc6 → glibc (usually already installed)".to_string());
        output_lines.push("  • libgtk-3-0 → gtk3".to_string());
        output_lines.push("  • libwebkit2gtk-4.1-0 → webkit2gtk4.1".to_string());
        output_lines.push("  • libxdo3 → xdotool".to_string());
        output_lines.push("Install Fedora equivalents first, then install the converted RPM".to_string());
    }

    if let Some(ref extract_dir) = temp_extract_dir {
        cmd.arg(extract_dir.to_string_lossy().as_ref());
    } else {
        cmd.arg(&input_file_str);
    }

    output_lines.push(format!("Running: fpm -s {} -t {} -f --verbose {} (in directory: {})",
        source_type,
        target_type,
        if let Some(ref d) = temp_extract_dir { d.display().to_string() } else { input_file_str.clone() },
        parent_dir_str));
    output_lines.push("--- Output ---".to_string());

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.current_dir(&parent_dir_abs);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute fpm: {}. Make sure fpm is installed (run: gem install fpm)", e))?;

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
        .map_err(|e| format!("Failed to wait for fpm process: {}", e))?;

    output_lines.push(format!("--- Process exited with code: {:?} ---", status.code()));

    if !status.success() {
        let error_output = output_lines.join("\n");
        return Err(format!("Conversion failed:\n{}", error_output));
    }

    if let Some(ref temp_dir) = temp_extract_dir {
        let _ = std::fs::remove_dir_all(temp_dir);
        output_lines.push(format!("Cleaned up temp directory: {}", temp_dir.display()));
    }

    fn extract_filename_from_line(line: &str, target_ext: &str) -> Option<String> {
        if let Some(path_start) = line.find(":path=>\"") {
            let path_str = &line[path_start + 7..];
            if let Some(path_end) = path_str.find('"') {
                let full_path = &path_str[..path_end];
                if let Some(filename) = std::path::Path::new(full_path).file_name() {
                    return Some(filename.to_string_lossy().into_owned());
                }
            }
        }

        for part in line.split_whitespace() {
            let cleaned = part.trim_matches(['"', '\'', ',', '}']);
            if !target_ext.is_empty() && cleaned.ends_with(target_ext) {
                if let Some(filename) = std::path::Path::new(cleaned).file_name() {
                    return Some(filename.to_string_lossy().into_owned());
                }
            }
        }
        None
    }

    let generated_file_name = output_lines
        .iter()
        .find_map(|line| {
            if line.contains("Created package") {
                extract_filename_from_line(line, target_ext)
            } else if !target_ext.is_empty()
                && line.contains(target_ext)
                && !line.contains("Searching")
                && !line.contains("Looking") {
                extract_filename_from_line(line, target_ext)
            } else {
                None
        }
        });

    output_lines.push(format!("Searching for output file in: {}", parent_dir_abs.display()));

    let mut found_file: Option<PathBuf> = None;

    if let Some(ref file_name) = generated_file_name {
        output_lines.push(format!("Looking for file: {}", file_name));
        let file_path = parent_dir_abs.join(file_name);
        if file_path.exists() {
            output_lines.push(format!("[OK] Found file: {}", file_path.display()));
            found_file = Some(file_path);
        }
    }

    if found_file.is_none() {
        output_lines.push("Parsed file name not found, searching for recently created files...".to_string());

        if let Ok(entries) = std::fs::read_dir(&parent_dir_abs) {
            found_file = entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let matches = if !target_ext.is_empty() {
                        path.extension().and_then(|s| s.to_str()) == Some(target_ext)
                    } else {
                        path.is_dir()
                    };

                    if !matches {
                        return None;
                    }

                    output_lines.push(format!("Found file: {}", path.display()));

                    entry.metadata().ok()?.modified().ok().and_then(|modified| {
                                let now = std::time::SystemTime::now();
                        let duration = now.duration_since(modified).ok()?;
                                    if duration.as_secs() < 120 {
                            Some((modified, path))
                        } else {
                            None
                        }
                    })
                })
                .max_by_key(|(modified, _)| *modified)
                .map(|(_, path)| path);
                                        }
    }

    if let Some(file) = found_file {
        output_lines.push(format!("[OK] Successfully found converted file: {}", file.display()));
        Ok((output_lines, file.to_string_lossy().to_string()))
    } else {
        let error_output = output_lines.join("\n");
        Err(format!("Output file was not created. Searched in: {}. FPM output:\n{}",
            parent_dir_abs.display(),
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
