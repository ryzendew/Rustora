use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use chrono::Local;

static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

pub struct Logger {
    log_file: File,
    log_path: PathBuf,
}

impl Logger {
    fn new() -> Result<Self, std::io::Error> {
        // Get home directory
        let home_dir = match std::env::var("HOME") {
            Ok(home) => PathBuf::from(home),
            Err(_) => {
                // Fallback to user's home directory using users crate
                if let Some(name) = users::get_current_username() {
                    if let Some(name_str) = name.to_str() {
                        PathBuf::from(format!("/home/{}", name_str))
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "Could not determine home directory"
                        ));
                    }
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not determine home directory"
                    ));
                }
            }
        };

        // Create .rustora directory
        let rustora_dir = home_dir.join(".rustora");
        fs::create_dir_all(&rustora_dir)?;

        // Create log file with date and time
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_filename = format!("rustora_{}.log", timestamp);
        let log_path = rustora_dir.join(log_filename);

        // Open log file in append mode
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // Write initial log entry
        let mut logger = Logger {
            log_file,
            log_path: log_path.clone(),
        };
        
        logger.write_log(&format!(
            "[{}] Rustora started - Log file: {}",
            now.format("%Y-%m-%d %H:%M:%S"),
            log_path.display()
        ))?;

        Ok(logger)
    }

    fn write_log(&mut self, message: &str) -> Result<(), std::io::Error> {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(self.log_file, "[{}] {}", timestamp, message)?;
        self.log_file.flush()?;
        Ok(())
    }

    pub fn log_debug(message: &str) {
        let mut logger_guard = LOGGER.lock().unwrap();
        
        // Initialize logger if not already initialized
        if logger_guard.is_none() {
            match Logger::new() {
                Ok(logger) => *logger_guard = Some(logger),
                Err(e) => {
                    eprintln!("Failed to initialize logger: {}", e);
                    return;
                }
            }
        }

        // Write log message
        if let Some(ref mut logger) = *logger_guard {
            if let Err(e) = logger.write_log(message) {
                eprintln!("Failed to write log: {}", e);
            }
        }
    }

    pub fn log_tab_action(tab: &str, action: &str) {
        Self::log_debug(&format!("[TAB: {}] Action: {}", tab, action));
    }

    pub fn log_tab_change(from: Option<&str>, to: &str) {
        if let Some(from_tab) = from {
            Self::log_debug(&format!("[TAB] Changed from '{}' to '{}'", from_tab, to));
        } else {
            Self::log_debug(&format!("[TAB] Opened tab '{}'", to));
        }
    }
}

// Initialize logger on first use
pub fn init() {
    let _ = Logger::log_debug("Logger initialized");
}

