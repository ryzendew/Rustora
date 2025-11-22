use iced::widget::{button, column, container, row, scrollable, text, Space, progress_bar};
use iced::{Alignment, Application, Command, Element, Length, Padding, Border, Theme as IcedTheme};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::window;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum Message {
    StartDownload,
    #[allow(dead_code)]
    DownloadProgress(f32, String), // progress (0.0-1.0), message
    DownloadComplete(Result<String, String>), // tar_path or error
    StartExtraction,
    #[allow(dead_code)]
    ExtractionProgress(f32, String), // progress (0.0-1.0), message
    ExtractionComplete(Result<String, String>), // extracted_dir or error
    StartInstallation,
    #[allow(dead_code)]
    InstallationProgress(f32, String), // progress (0.0-1.0), message
    InstallationComplete(Result<(), String>),
    Close,
}

#[derive(Debug, Clone)]
pub struct ProgressState {
    pub download_progress: f32,
    pub download_message: String,
    pub extraction_progress: f32,
    pub extraction_message: String,
    pub installation_progress: f32,
    pub installation_message: String,
}

impl ProgressState {
    pub fn new() -> Self {
        Self {
            download_progress: 0.0,
            download_message: String::new(),
            extraction_progress: 0.0,
            extraction_message: String::new(),
            installation_progress: 0.0,
            installation_message: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct ProtonInstallDialog {
    runner_title: String,
    build_title: String,
    download_url: String,
    selected_launcher: Option<String>,
    runner_info: Option<String>, // JSON serialized ProtonRunner info

    is_downloading: bool,
    is_extracting: bool,
    is_installing: bool,
    is_complete: bool,
    has_error: bool,

    progress_state: Arc<Mutex<ProgressState>>,

    terminal_output: String,
}

impl ProtonInstallDialog {
    pub fn new(
        runner_title: String,
        build_title: String,
        download_url: String,
        selected_launcher: Option<String>,
        runner_info: Option<String>,
    ) -> Self {
        Self {
            runner_title,
            build_title,
            download_url,
            selected_launcher,
            runner_info,
            is_downloading: false,
            is_extracting: false,
            is_installing: false,
            is_complete: false,
            has_error: false,
            progress_state: Arc::new(Mutex::new(ProgressState::new())),
            terminal_output: String::new(),
        }
    }

    pub fn run_separate_window(
        runner_title: String,
        build_title: String,
        download_url: String,
        selected_launcher: Option<String>,
        runner_info: Option<String>,
    ) -> Result<(), iced::Error> {
        let dialog = Self::new(runner_title, build_title, download_url, selected_launcher, runner_info);

        let mut window_settings = iced::window::Settings::default();
        window_settings.size = iced::Size::new(800.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(600.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        let default_font = crate::gui::fonts::get_inter_font();

        <ProtonInstallDialog as Application>::run(iced::Settings {
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

impl Application for ProtonInstallDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = iced::executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::StartDownload);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        format!("Installing {} {} - Rustora", self.runner_title, self.build_title)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartDownload => {
                self.is_downloading = true;
                self.terminal_output.clear();
                self.terminal_output.push_str(&format!("Starting download of {} {}...\n", self.runner_title, self.build_title));
                self.terminal_output.push_str("=====================================\n\n");

                let download_url = self.download_url.clone();
                let build_title = self.build_title.clone();
                let progress_state = Arc::clone(&self.progress_state);

                Command::perform(
                    download_with_progress(download_url, build_title, progress_state),
                    |result| Message::DownloadComplete(result),
                )
            }
            Message::DownloadProgress(progress, message) => {
                if let Ok(mut state) = self.progress_state.lock() {
                    state.download_progress = progress;
                    state.download_message = message;
                }
                Command::none()
            }
            Message::DownloadComplete(result) => {
                self.is_downloading = false;
                match result {
                    Ok(tar_path) => {
                        self.terminal_output.push_str(&format!("Download complete: {}\n\n", tar_path));
                        Command::perform(async {}, |_| Message::StartExtraction)
                    }
                    Err(e) => {
                        self.has_error = true;
                        self.terminal_output.push_str(&format!("Download failed: {}\n", e));
                        Command::none()
                    }
                }
            }
            Message::StartExtraction => {
                self.is_extracting = true;
                self.terminal_output.push_str("Starting extraction...\n");

                // Get tar_path from download
                let temp_dir = std::env::temp_dir();
                let tar_path = temp_dir.join(format!("{}.tar.gz", self.build_title));
                let build_title = self.build_title.clone();
                let progress_state = Arc::clone(&self.progress_state);

                Command::perform(
                    extract_with_progress(tar_path.to_string_lossy().to_string(), build_title, progress_state),
                    |result| Message::ExtractionComplete(result),
                )
            }
            Message::ExtractionProgress(progress, message) => {
                if let Ok(mut state) = self.progress_state.lock() {
                    state.extraction_progress = progress;
                    state.extraction_message = message;
                }
                Command::none()
            }
            Message::ExtractionComplete(result) => {
                self.is_extracting = false;
                match result {
                    Ok(extracted_dir) => {
                        self.terminal_output.push_str(&format!("Extraction complete: {}\n\n", extracted_dir));
                        Command::perform(async {}, |_| Message::StartInstallation)
                    }
                    Err(e) => {
                        self.has_error = true;
                        self.terminal_output.push_str(&format!("Extraction failed: {}\n", e));
                        Command::none()
                    }
                }
            }
            Message::StartInstallation => {
                self.is_installing = true;
                self.terminal_output.push_str("Starting installation...\n");

                // Get paths
                let temp_dir = std::env::temp_dir();
                let tar_path = temp_dir.join(format!("{}.tar.gz", self.build_title));
                let runner_title = self.runner_title.clone();
                let build_title = self.build_title.clone();
                let selected_launcher = self.selected_launcher.clone();
                let runner_info = self.runner_info.clone();
                let progress_state = Arc::clone(&self.progress_state);

                Command::perform(
                    install_with_progress(
                        runner_title,
                        build_title,
                        tar_path.to_string_lossy().to_string(),
                        selected_launcher,
                        runner_info,
                        progress_state,
                    ),
                    |result| Message::InstallationComplete(result),
                )
            }
            Message::InstallationProgress(progress, message) => {
                if let Ok(mut state) = self.progress_state.lock() {
                    state.installation_progress = progress;
                    state.installation_message = message;
                }
                Command::none()
            }
            Message::InstallationComplete(result) => {
                self.is_installing = false;
                match result {
                    Ok(_) => {
                        self.is_complete = true;
                        self.terminal_output.push_str("Installation complete!\n");
                    }
                    Err(e) => {
                        self.has_error = true;
                        self.terminal_output.push_str(&format!("Installation failed: {}\n", e));
                    }
                }
                Command::none()
            }
            Message::Close => {
                iced::window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = crate::gui::Theme::Dark;
        self.view_impl(&theme)
    }

    fn theme(&self) -> IcedTheme {
        crate::gui::Theme::Dark.iced_theme()
    }
}

impl ProtonInstallDialog {
    pub fn view_impl(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let progress_state = self.progress_state.lock().unwrap();

        let close_button: Element<Message> = if self.is_complete || self.has_error {
            button(
                text("Close")
                    .size(16.0)
                    .style(iced::theme::Text::Color(iced::Color::WHITE))
            )
            .on_press(Message::Close)
            .padding(Padding::from([12.0, 24.0, 12.0, 24.0]))
            .style(iced::theme::Button::Custom(Box::new(CloseButtonStyle)))
            .into()
        } else {
            Space::with_width(Length::Fixed(0.0)).into()
        };

        // Download progress
        let download_section: Element<Message> = if self.is_downloading || progress_state.download_progress > 0.0 {
            let download_text: Element<Message> = text("Downloading...")
                .size(14.0)
                .style(iced::theme::Text::Color(theme.text()))
                .into();
            let download_bar: Element<Message> = progress_bar(0.0..=1.0, progress_state.download_progress)
                .width(Length::Fill)
                .height(Length::Fixed(8.0))
                .into();
            column![
                download_text,
                Space::with_height(Length::Fixed(8.0)),
                download_bar,
                text(format!("{:.1}%", progress_state.download_progress * 100.0))
                    .size(12.0)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                if !progress_state.download_message.is_empty() {
                    let msg_text: Element<Message> = text(&progress_state.download_message)
                        .size(12.0)
                        .style(iced::theme::Text::Color(theme.secondary_text()))
                        .into();
                    msg_text
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                },
            ]
            .spacing(4)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };

        // Extraction progress
        let extraction_section: Element<Message> = if self.is_extracting || progress_state.extraction_progress > 0.0 {
            let extract_text: Element<Message> = text("Extracting...")
                .size(14.0)
                .style(iced::theme::Text::Color(theme.text()))
                .into();
            let extract_bar: Element<Message> = progress_bar(0.0..=1.0, progress_state.extraction_progress)
                .width(Length::Fill)
                .height(Length::Fixed(8.0))
                .into();
            column![
                Space::with_height(Length::Fixed(16.0)),
                extract_text,
                Space::with_height(Length::Fixed(8.0)),
                extract_bar,
                text(format!("{:.1}%", progress_state.extraction_progress * 100.0))
                    .size(12.0)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                if !progress_state.extraction_message.is_empty() {
                    let msg_text: Element<Message> = text(&progress_state.extraction_message)
                        .size(12.0)
                        .style(iced::theme::Text::Color(theme.secondary_text()))
                        .into();
                    msg_text
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                },
            ]
            .spacing(4)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };

        // Installation progress
        let installation_section: Element<Message> = if self.is_installing || progress_state.installation_progress > 0.0 {
            let install_text: Element<Message> = text("Installing...")
                .size(14.0)
                .style(iced::theme::Text::Color(theme.text()))
                .into();
            let install_bar: Element<Message> = progress_bar(0.0..=1.0, progress_state.installation_progress)
                .width(Length::Fill)
                .height(Length::Fixed(8.0))
                .into();
            column![
                Space::with_height(Length::Fixed(16.0)),
                install_text,
                Space::with_height(Length::Fixed(8.0)),
                install_bar,
                text(format!("{:.1}%", progress_state.installation_progress * 100.0))
                    .size(12.0)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                if !progress_state.installation_message.is_empty() {
                    let msg_text: Element<Message> = text(&progress_state.installation_message)
                        .size(12.0)
                        .style(iced::theme::Text::Color(theme.secondary_text()))
                        .into();
                    msg_text
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                },
            ]
            .spacing(4)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };

        let terminal_display = scrollable(
            text(&self.terminal_output)
                .size(12.0)
                .font(iced::Font::MONOSPACE)
                .style(iced::theme::Text::Color(theme.text()))
        )
        .style(iced::theme::Scrollable::Custom(Box::new(TerminalScrollableStyle)))
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            column![
                // Header (minimal - just close button)
                row![
                    Space::with_width(Length::Fill),
                    close_button,
                ]
                .spacing(12)
                .align_items(Alignment::Center)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(8.0)),
                // Progress bars
                download_section,
                extraction_section,
                installation_section,
                Space::with_height(Length::Fixed(16.0)),
                // Terminal output
                text("Output:")
                    .size(14.0)
                    .style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(8.0)),
                container(terminal_display)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(12)
                    .style(iced::theme::Container::Custom(Box::new(TerminalContainerStyle))),
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

// Async functions with progress reporting
async fn download_with_progress(
    download_url: String,
    build_title: String,
    progress_state: Arc<Mutex<ProgressState>>,
) -> Result<String, String> {
    use reqwest::Client;

    let client = Client::new();
    let temp_dir = std::env::temp_dir();
    let tar_path = temp_dir.join(format!("{}.tar.gz", build_title));

    // Update progress
    if let Ok(mut state) = progress_state.lock() {
        state.download_progress = 0.0;
        state.download_message = "Connecting...".to_string();
    }

    let response = client
        .get(&download_url)
        .header("User-Agent", "Rustora/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);

    if let Ok(mut state) = progress_state.lock() {
        state.download_progress = 0.1;
        state.download_message = format!("Downloading... ({} MB)", total_size as f64 / 1_048_576.0);
    }

    use tokio::fs::File as TokioFile;
    use tokio::io::{AsyncWriteExt, BufWriter};

    let mut file = TokioFile::create(&tar_path).await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut writer = BufWriter::new(&mut file);
    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;

    use futures::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
        writer.write_all(&chunk).await
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = downloaded as f32 / total_size as f32;
            if let Ok(mut state) = progress_state.lock() {
                state.download_progress = progress.min(0.99);
                state.download_message = format!(
                    "Downloading... {:.1} MB / {:.1} MB ({:.1}%)",
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0,
                    progress * 100.0
                );
            }
        }
    }

    writer.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(writer); // Ensure writer is dropped and file is closed
    drop(file); // Ensure file handle is closed

    // Verify file was written correctly
    let final_metadata = std::fs::metadata(&tar_path)
        .map_err(|e| format!("Failed to verify downloaded file: {}", e))?;
    if final_metadata.len() == 0 {
        return Err("Downloaded file is empty".to_string());
    }
    if total_size > 0 && final_metadata.len() != total_size {
        return Err(format!("Downloaded file size mismatch: expected {} bytes, got {} bytes", total_size, final_metadata.len()));
    }

    if let Ok(mut state) = progress_state.lock() {
        state.download_progress = 1.0;
        state.download_message = "Download complete".to_string();
    }

    Ok(tar_path.to_string_lossy().to_string())
}

async fn extract_with_progress(
    tar_path: String,
    build_title: String,
    progress_state: Arc<Mutex<ProgressState>>,
) -> Result<String, String> {
    use std::fs::File;
    use flate2::read::GzDecoder;
    use tar::Archive;

    if let Ok(mut state) = progress_state.lock() {
        state.extraction_progress = 0.0;
        state.extraction_message = "Validating archive...".to_string();
    }

    // Validate file exists and is not empty
    let tar_path_buf = std::path::Path::new(&tar_path);
    if !tar_path_buf.exists() {
        return Err(format!("Archive file does not exist: {}", tar_path));
    }

    let metadata = std::fs::metadata(tar_path_buf)
        .map_err(|e| format!("Failed to read archive metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err("Archive file is empty".to_string());
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let home_tmp = std::path::Path::new(&home).join(".tmp");
    std::fs::create_dir_all(&home_tmp)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let temp_extract = home_tmp.join(format!("proton_extract_{}", build_title));
    if temp_extract.exists() {
        std::fs::remove_dir_all(&temp_extract)
            .map_err(|e| format!("Failed to clean temp extract: {}", e))?;
    }
    std::fs::create_dir_all(&temp_extract)
        .map_err(|e| format!("Failed to create temp extract: {}", e))?;

    if let Ok(mut state) = progress_state.lock() {
        state.extraction_progress = 0.1;
        state.extraction_message = "Opening archive...".to_string();
    }

    // Extract with progress tracking
    // Use unpack which doesn't require holding Archive across await
    // We'll simulate progress since tar crate doesn't support async extraction easily
    let tar_path_clone = tar_path.clone();
    let temp_extract_clone = temp_extract.clone();
    let progress_state_clone = Arc::clone(&progress_state);

    // Start extraction in a blocking task with progress simulation
    let extract_handle = tokio::task::spawn_blocking(move || {
        // Small delay to ensure file is fully written and closed
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Verify file exists again in blocking context
        if !std::path::Path::new(&tar_path_clone).exists() {
            return Err(format!("Archive file does not exist: {}", tar_path_clone));
        }

        // Verify file is not empty and has reasonable size
        let file_metadata = std::fs::metadata(&tar_path_clone)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        if file_metadata.len() < 100 {
            return Err("Archive file is too small to be valid".to_string());
        }

        let file = File::open(&tar_path_clone)
            .map_err(|e| format!("Failed to open archive: {} ({})", e, tar_path_clone))?;

        // Try to read first few bytes to detect file format
        use std::io::{Read, Seek, SeekFrom};
        let mut peek_buf = [0u8; 6];
        let mut peek_file = file.try_clone()
            .map_err(|e| format!("Failed to clone file handle: {}", e))?;

        if peek_file.read_exact(&mut peek_buf).is_err() {
            return Err("Archive file appears to be corrupted or incomplete".to_string());
        }

        // Check file format by magic bytes
        let is_gzip = peek_buf[0] == 0x1f && peek_buf[1] == 0x8b;
        // Zstd format signature: 28 B5 2F FD (bytes 0-3)
        let is_zstd = peek_buf[0] == 0x28 && peek_buf[1] == 0xb5 && peek_buf[2] == 0x2f && peek_buf[3] == 0xfd;
        // XZ format signature: FD 37 7A 58 5A 00 (bytes 0-5)
        let is_xz = peek_buf[0] == 0xfd && peek_buf[1] == 0x37 && peek_buf[2] == 0x7a && peek_buf[3] == 0x58 && peek_buf[4] == 0x5a && peek_buf[5] == 0x00;
        // 7z format signature: 37 7A BC AF 27 1C (bytes 0-5)
        let is_7z = peek_buf[0] == 0x37 && peek_buf[1] == 0x7a && peek_buf[2] == 0xbc && peek_buf[3] == 0xaf && peek_buf[4] == 0x27 && peek_buf[5] == 0x1c;
        // ZIP format signature: 50 4B (PK) followed by 03, 05, or 07
        let is_zip = peek_buf[0] == 0x50 && peek_buf[1] == 0x4b && (peek_buf[2] == 0x03 || peek_buf[2] == 0x05 || peek_buf[2] == 0x07);

        eprintln!("[DEBUG] File magic bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            peek_buf[0], peek_buf[1], peek_buf[2], peek_buf[3], peek_buf[4], peek_buf[5]);
        eprintln!("[DEBUG] Format detection: gzip={}, zstd={}, xz={}, 7z={}, zip={}", is_gzip, is_zstd, is_xz, is_7z, is_zip);

        if is_7z {
            // Use system's 7z command to extract
            eprintln!("[DEBUG] Detected 7z archive format");
            let output = std::process::Command::new("7z")
                .arg("x")
                .arg(&tar_path_clone)
                .arg(format!("-o{}", temp_extract_clone.to_string_lossy()))
                .arg("-y") // Assume yes to all prompts
                .output()
                .map_err(|e| format!("Failed to execute 7z command. Is p7zip installed? Error: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("7z extraction failed: {}", stderr));
            }

            return Ok(());
        } else if is_zip {
            // Use system's unzip command
            eprintln!("[DEBUG] Detected ZIP archive format");
            let output = std::process::Command::new("unzip")
                .arg("-q") // Quiet mode
                .arg("-o") // Overwrite files
                .arg(&tar_path_clone)
                .arg("-d")
                .arg(&temp_extract_clone)
                .output()
                .map_err(|e| format!("Failed to execute unzip command. Is unzip installed? Error: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("unzip extraction failed: {}", stderr));
            }

            return Ok(());
        } else if !is_gzip && !is_zstd && !is_xz {
            return Err(format!("Unsupported archive format (magic bytes: {:02x} {:02x} {:02x} {:02x}). Expected gzip (.tar.gz), zstd (.tar.zst), xz (.tar.xz), 7z, or zip format.",
                peek_buf[0], peek_buf[1], peek_buf[2], peek_buf[3]));
        }

        // Reset file position for actual extraction
        let mut file = File::open(&tar_path_clone)
            .map_err(|e| format!("Failed to reopen archive: {}", e))?;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("Failed to seek file: {}", e))?;

        // Create appropriate decoder and extract based on format
        if is_gzip {
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);
        archive.unpack(&temp_extract_clone)
            .map_err(|e| {
                    let error_msg = format!("Failed to extract gzip archive: {}", e);
                if error_msg.contains("failed to iterate") {
                    format!("Archive appears to be corrupted or in an unsupported format. Please try downloading again. Original error: {}", e)
                } else {
                    error_msg
                }
                })?;
        } else if is_zstd {
            // zstd
            use zstd::stream::Decoder;
            let decoder = Decoder::new(file)
                .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;
            let mut archive = Archive::new(decoder);
            archive.unpack(&temp_extract_clone)
                .map_err(|e| {
                    let error_msg = format!("Failed to extract zstd archive: {}", e);
                    if error_msg.contains("failed to iterate") {
                        format!("Archive appears to be corrupted or in an unsupported format. Please try downloading again. Original error: {}", e)
                    } else {
                        error_msg
                    }
                })?;
        } else {
            // xz
            use xz2::read::XzDecoder;
            let xz = XzDecoder::new(file);
            let mut archive = Archive::new(xz);
            archive.unpack(&temp_extract_clone)
                .map_err(|e| {
                    let error_msg = format!("Failed to extract xz archive: {}", e);
                    if error_msg.contains("failed to iterate") {
                        format!("Archive appears to be corrupted or in an unsupported format. Please try downloading again. Original error: {}", e)
                    } else {
                        error_msg
                    }
                })?;
        }

        Ok(())
    });

    // Simulate progress while extraction happens
    let mut current_progress = 0.1;
    while !extract_handle.is_finished() {
        if let Ok(mut state) = progress_state_clone.lock() {
            state.extraction_progress = current_progress;
            state.extraction_message = format!("Extracting... {:.0}%", current_progress * 100.0);
        }
        current_progress = (current_progress + 0.05).min(0.95);
        sleep(Duration::from_millis(200)).await;
    }

    // Wait for extraction to complete
    extract_handle.await
        .map_err(|e| format!("Extraction task error: {}", e))??;

    // Update progress to complete
    if let Ok(mut state) = progress_state.lock() {
        state.extraction_progress = 1.0;
        state.extraction_message = "Extraction complete".to_string();
    }

    // Find extracted directory
    let entries: Vec<_> = std::fs::read_dir(&temp_extract)
        .map_err(|e| format!("Failed to read extract dir: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read entry: {}", e))?;

    let extracted_dir = entries
        .iter()
        .find(|entry| {
            if let Ok(metadata) = entry.metadata() {
                metadata.is_dir()
            } else {
                false
            }
        })
        .ok_or_else(|| "No directory found in archive".to_string())?
        .path();

    Ok(extracted_dir.to_string_lossy().to_string())
}

async fn install_with_progress(
    _runner_title: String,
    build_title: String,
    tar_path: String,
    selected_launcher: Option<String>,
    _runner_info: Option<String>,
    progress_state: Arc<Mutex<ProgressState>>,
) -> Result<(), String> {

    if let Ok(mut state) = progress_state.lock() {
        state.installation_progress = 0.0;
        state.installation_message = "Preparing installation...".to_string();
    }

    // Get launcher directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let launcher_title = selected_launcher.as_ref().map(|s| s.as_str()).unwrap_or("Steam");

    let compat_dir = match launcher_title {
        "Steam" => {
            let paths = vec![
                format!("{}/.steam/root/compatibilitytools.d", home),
                format!("{}/.local/share/Steam/compatibilitytools.d", home),
                format!("{}/.steam/steam/compatibilitytools.d", home),
            ];
            paths.iter()
                .find(|p| std::path::Path::new(p).exists())
                .cloned()
                .unwrap_or_else(|| paths[1].clone())
        }
        _ => format!("{}/.local/share/{}/compatibilitytools.d", home, launcher_title),
    };

    std::fs::create_dir_all(&compat_dir)
        .map_err(|e| format!("Failed to create compat directory: {}", e))?;

    if let Ok(mut state) = progress_state.lock() {
        state.installation_progress = 0.2;
        state.installation_message = "Copying files...".to_string();
    }

    // The extracted directory should already exist from extraction step
    // For now, we'll assume it's in the temp extract location
    let home_tmp = std::path::Path::new(&home).join(".tmp");
    let temp_extract = home_tmp.join(format!("proton_extract_{}", build_title));

    let entries: Vec<_> = std::fs::read_dir(&temp_extract)
        .map_err(|e| format!("Failed to read extract dir: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read entry: {}", e))?;

    let extracted_dir = entries
        .iter()
        .find(|entry| {
            if let Ok(metadata) = entry.metadata() {
                metadata.is_dir()
            } else {
                false
            }
        })
        .ok_or_else(|| "No directory found in archive".to_string())?
        .path();

    let dest_path = std::path::Path::new(&compat_dir).join(&build_title);

    if dest_path.exists() {
        std::fs::remove_dir_all(&dest_path)
            .map_err(|e| format!("Failed to remove existing: {}", e))?;
    }

    if let Ok(mut state) = progress_state.lock() {
        state.installation_progress = 0.5;
        state.installation_message = "Installing files...".to_string();
    }

    // Copy directory
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
        .map_err(|e| format!("Failed to copy: {}", e))?;

    if let Ok(mut state) = progress_state.lock() {
        state.installation_progress = 0.9;
        state.installation_message = "Finalizing...".to_string();
    }

    // Cleanup
    let _ = std::fs::remove_file(&tar_path);
    let _ = std::fs::remove_dir_all(&temp_extract);

    if let Ok(mut state) = progress_state.lock() {
        state.installation_progress = 1.0;
        state.installation_message = "Installation complete".to_string();
    }

    Ok(())
}

// Style structs
struct DialogContainerStyle {
    theme: crate::gui::Theme,
}

impl iced::widget::container::StyleSheet for DialogContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(self.theme.background().into()),
            border: Border::default(),
            shadow: Default::default(),
        }
    }
}

struct TerminalContainerStyle;

impl iced::widget::container::StyleSheet for TerminalContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            text_color: None,
            background: Some(iced::Color::from_rgb(0.1, 0.1, 0.1).into()),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            },
            shadow: Default::default(),
        }
    }
}

struct TerminalScrollableStyle;

impl iced::widget::scrollable::StyleSheet for TerminalScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: None,
                border: iced::Border::default(),
                scroller: iced::widget::scrollable::Scroller {
                    color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                    border: iced::Border::default(),
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_mouse_over: bool) -> iced::widget::scrollable::Appearance {
        self.active(_style)
    }
}

struct CloseButtonStyle;

impl ButtonStyleSheet for CloseButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        ButtonAppearance {
            background: Some(iced::Color::from_rgb(0.2, 0.5, 0.8).into()),
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            text_color: iced::Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Color::from_rgb(0.25, 0.6, 0.9).into());
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Color::from_rgb(0.15, 0.4, 0.7).into());
        appearance
    }
}

