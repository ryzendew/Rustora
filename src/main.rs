mod gui;

use clap::{Parser, Subcommand};
use colored::*;
use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context};
use iced::Application;

#[derive(Parser)]
#[command(name = "fedoraforge", about = "FedoraForge - A modern package manager for Fedora", version)]
struct Cli {
    /// RPM file to open (when opened from file manager)
    #[arg(value_name = "RPM_FILE")]
    rpm_file: Option<String>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for packages
    Search {
        /// Package name or search term
        query: String,
        /// Show detailed information
        #[arg(short, long)]
        details: bool,
    },
    /// Install packages or .rpm files
    Install {
        /// Package name(s) or .rpm file path(s) to install
        packages: Vec<String>,
        /// Assume yes to all prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// List installed packages
    List {
        /// Show detailed information
        #[arg(short, long)]
        details: bool,
    },
    /// Show package information
    Info {
        /// Package name
        package: String,
    },
    /// Update package database
    Update {
        /// Update all packages
        #[arg(short, long)]
        all: bool,
    },
    /// Launch GUI
    Gui {
        /// Optional RPM file to install
        #[arg(value_name = "RPM_FILE")]
        rpm_file: Option<String>,
    },
    /// Show package removal dialog (internal use)
    RemoveDialog {
        /// Package names to remove
        packages: Vec<String>,
    },
    /// Show package installation dialog (internal use)
    InstallDialog {
        /// Package names to install
        packages: Vec<String>,
    },
    /// Show Flatpak installation dialog (internal use)
    FlatpakInstallDialog {
        /// Application ID to install
        application_id: String,
        /// Optional remote name
        #[arg(long)]
        remote: Option<String>,
    },
    /// Show Flatpak removal dialog (internal use)
    FlatpakRemoveDialog {
        /// Application IDs to remove
        application_ids: Vec<String>,
    },
    /// Show update dialog (internal use)
    UpdateDialog,
    /// Show update settings dialog (internal use)
    UpdateSettingsDialog,
    /// Show maintenance dialog (internal use)
    MaintenanceDialog {
        /// Maintenance task to perform
        task: String,
    },
    /// Show kernel install dialog (internal use)
    KernelInstallDialog {
        /// Kernel name to install
        kernel_name: String,
    },
    /// Show kernel remove dialog (internal use)
    KernelRemoveDialog {
        /// Kernel name to remove
        kernel_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check if an RPM file was passed as a positional argument (from file manager)
    if let Some(rpm_file) = cli.rpm_file {
        let rpm_path = Path::new(&rpm_file).to_path_buf();
        if !rpm_path.exists() {
            return Err(anyhow::anyhow!("RPM file not found: {}", rpm_file));
        }
        // Verify it's actually an RPM file
        if let Some(ext) = rpm_path.extension() {
            if ext.to_string_lossy().to_lowercase() != "rpm" {
                return Err(anyhow::anyhow!("File is not an RPM file: {}", rpm_file));
            }
        } else {
            return Err(anyhow::anyhow!("File does not have an extension: {}", rpm_file));
        }
        // Only ensure fonts if they don't exist (fast check)
        if !gui::fonts::fonts_exist() {
            // Spawn font installation in background, don't wait
            tokio::spawn(async {
                let _ = gui::fonts::ensure_fonts().await;
            });
        }
        use crate::gui::rpm_dialog::RpmDialog;
        RpmDialog::run_separate_window(rpm_path)?;
        return Ok(());
    }
    
    match cli.command {
        None => {
            // Default to GUI when no command is provided
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }

            // Use cached InterVariable font (optimized)
            let default_font = gui::fonts::get_inter_font();

            gui::FedoraForgeApp::run(iced::Settings {
                window: iced::window::Settings {
                    size: iced::Size::new(1200.0, 800.0),
                    ..Default::default()
                },
                flags: None,
                default_font,
                ..Default::default()
            })?;
            Ok(())
        }
        Some(Commands::Gui { rpm_file }) => {
            // If an RPM file is provided, show only the RPM dialog window
            if let Some(rpm_file_str) = rpm_file {
                let rpm_path = Path::new(&rpm_file_str).to_path_buf();
                if !rpm_path.exists() {
                    return Err(anyhow::anyhow!("RPM file not found: {}", rpm_file_str));
                }
                // Only ensure fonts if they don't exist (fast check)
                if !gui::fonts::fonts_exist() {
                    // Spawn font installation in background, don't wait
                    tokio::spawn(async {
                        let _ = gui::fonts::ensure_fonts().await;
                    });
                }
                use crate::gui::rpm_dialog::RpmDialog;
                RpmDialog::run_separate_window(rpm_path)?;
            } else {
                // Only ensure fonts if they don't exist (fast check)
                if !gui::fonts::fonts_exist() {
                    // Spawn font installation in background, don't wait
                    tokio::spawn(async {
                        let _ = gui::fonts::ensure_fonts().await;
                    });
                }

                // Use cached InterVariable font (optimized)
                let default_font = gui::fonts::get_inter_font();

                gui::FedoraForgeApp::run(iced::Settings {
                    window: iced::window::Settings {
                        size: iced::Size::new(1200.0, 800.0),
                        ..Default::default()
                    },
                    flags: None,
                    default_font,
                    ..Default::default()
                })?;
            }
            Ok(())
        }
        Some(Commands::RemoveDialog { packages }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::package_dialog::PackageDialog;
            PackageDialog::run_separate_window(packages)?;
            Ok(())
        }
        Some(Commands::InstallDialog { packages }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::install_dialog::InstallDialog;
            InstallDialog::run_separate_window(packages)?;
            Ok(())
        }
        Some(Commands::FlatpakInstallDialog { application_id, remote }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::flatpak_dialog::FlatpakDialog;
            FlatpakDialog::run_separate_window(application_id, remote)?;
            Ok(())
        }
        Some(Commands::FlatpakRemoveDialog { application_ids }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::flatpak_remove_dialog::FlatpakRemoveDialog;
            FlatpakRemoveDialog::run_separate_window(application_ids)?;
            Ok(())
        }
        Some(Commands::UpdateDialog) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::update_dialog::UpdateDialog;
            UpdateDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::UpdateSettingsDialog) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::update_settings_dialog::UpdateSettingsDialog;
            UpdateSettingsDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::MaintenanceDialog { task }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::maintenance_dialog::{MaintenanceDialog, MaintenanceTask};
            let maintenance_task = match task.as_str() {
                "rebuild-kernel-modules" => MaintenanceTask::RebuildKernelModules,
                "regenerate-initramfs" => MaintenanceTask::RegenerateInitramfs,
                "remove-orphaned-packages" => MaintenanceTask::RemoveOrphanedPackages,
                "clean-package-cache" => MaintenanceTask::CleanPackageCache,
                _ => {
                    eprintln!("Unknown maintenance task: {}", task);
                    return Err(anyhow::anyhow!("Unknown maintenance task: {}", task));
                }
            };
            MaintenanceDialog::run_separate_window(maintenance_task)?;
            Ok(())
        }
        Some(Commands::KernelInstallDialog { kernel_name }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            use crate::gui::kernel_install_dialog::KernelInstallDialog;
            KernelInstallDialog::run_separate_window(kernel_name)?;
            Ok(())
        }
        Some(Commands::KernelRemoveDialog { kernel_name }) => {
            // Only ensure fonts if they don't exist (fast check)
            if !gui::fonts::fonts_exist() {
                // Spawn font installation in background, don't wait
                tokio::spawn(async {
                    let _ = gui::fonts::ensure_fonts().await;
                });
            }
            // For now, use the same install dialog but with remove logic
            // TODO: Create separate remove dialog
            use crate::gui::kernel_install_dialog::KernelInstallDialog;
            KernelInstallDialog::run_separate_window(kernel_name)?;
            Ok(())
        }
        Some(cmd) => {
            if let Err(e) = match cmd {
                Commands::Search { query, details } => search_packages(&query, details),
                Commands::Install { packages, yes } => install_packages(&packages, yes),
                Commands::List { details } => list_packages(details),
                Commands::Info { package } => show_package_info(&package),
                Commands::Update { all } => update_packages(all),
                Commands::Gui { .. } => unreachable!(),
                Commands::RemoveDialog { .. } => unreachable!(),
                Commands::InstallDialog { .. } => unreachable!(),
                Commands::FlatpakInstallDialog { .. } => unreachable!(),
                Commands::FlatpakRemoveDialog { .. } => unreachable!(),
                Commands::UpdateDialog => unreachable!(),
                Commands::UpdateSettingsDialog => unreachable!(),
                Commands::MaintenanceDialog { .. } => unreachable!(),
                Commands::KernelInstallDialog { .. } => unreachable!(),
                Commands::KernelRemoveDialog { .. } => unreachable!(),
            } {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

fn search_packages(query: &str, details: bool) -> Result<()> {
    println!("{} Searching for: {}\n", "🔍".green(), query.bright_white().bold());
    let mut cmd = Command::new("dnf");
    cmd.arg("search").arg("--quiet");
    if details { cmd.arg("--showduplicates"); }
    cmd.arg(query);
    let output = cmd.output().context("Failed to execute dnf search")?;
    if !output.status.success() {
        anyhow::bail!("DNF search failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        println!("{} No packages found matching '{}'", "⚠️".yellow(), query);
        return Ok(());
    }
    let mut results: Vec<(String, String)> = stdout.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || !line.contains(" : ") { return None; }
            let parts: Vec<&str> = line.splitn(2, " : ").collect();
            if parts.len() == 2 {
                Some((parts[0].trim().split('.').next().unwrap_or(parts[0].trim()).to_string(), parts[1].trim().to_string()))
            } else { None }
        })
        .collect();
    if results.is_empty() {
        print!("{}", stdout);
    } else {
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results.dedup_by(|a, b| a.0 == b.0);
        let count = results.len();
        for (name, desc) in &results {
            println!("{} {}", name.bright_cyan().bold(), desc.bright_white());
        }
        println!("\n{} Found {} package(s)", "✓".green(), count.to_string().bright_white().bold());
    }
    Ok(())
}

fn install_packages(packages: &[String], yes: bool) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified");
    }
    for pkg in packages {
        if pkg.ends_with(".rpm") && !Path::new(pkg).exists() {
            anyhow::bail!("RPM file not found: {}", pkg);
        }
    }
    println!("{} Installing package(s): {}\n", "📦".green(), packages.join(", ").bright_white().bold());
    check_sudo();
    let mut cmd = Command::new("sudo");
    cmd.arg("dnf").arg("install");
    if yes { cmd.arg("-y"); }
    cmd.args(packages);
    let status = cmd.spawn().context("Failed to execute dnf install")?.wait().context("Failed to wait for process")?;
    if !status.success() {
        anyhow::bail!("Package installation failed");
    }
    println!("\n{} Successfully installed package(s)", "✓".green().bold());
    Ok(())
}

fn list_packages(details: bool) -> Result<()> {
    println!("{} Listing installed packages...\n", "📋".green());
    let output = Command::new("dnf").args(["list", "--installed", "--quiet"]).output().context("Failed to execute dnf list")?;
    if !output.status.success() {
        anyhow::bail!("DNF list failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<&str> = stdout.lines().skip(1).filter(|l| !l.trim().is_empty()).collect();
    if packages.is_empty() {
        println!("{} No packages found", "⚠️".yellow());
        return Ok(());
    }
    let count = packages.len();
    for line in packages {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            if details {
                println!("{} {} {} {}", parts[0].bright_cyan().bold(), parts.get(1).unwrap_or(&"").bright_white(), parts.get(2).unwrap_or(&"").bright_black(), if parts.len() > 3 { parts[3..].join(" ") } else { String::new() });
            } else {
                println!("{}", parts[0].bright_cyan().bold());
            }
        }
    }
    println!("\n{} Total: {} package(s)", "✓".green(), count.to_string().bright_white().bold());
    Ok(())
}

fn show_package_info(package: &str) -> Result<()> {
    println!("{} Package information: {}\n", "ℹ️".blue(), package.bright_white().bold());
    let output = Command::new("dnf").args(["info", package]).output().context("Failed to execute dnf info")?;
    if !output.status.success() {
        anyhow::bail!("DNF info failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        println!("{} Package '{}' not found", "⚠️".yellow(), package);
        return Ok(());
    }
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            println!();
            continue;
        }
        if let Some((key, value)) = line.splitn(2, ':').collect::<Vec<&str>>().split_first().and_then(|(_, rest)| if rest.len() == 1 { Some((line.splitn(2, ':').next().unwrap_or("").trim(), rest[0].trim())) } else { None }) {
            println!("{}: {}", key.bright_cyan().bold(), value.bright_white());
        } else {
            println!("{}", line.bright_white());
        }
    }
    Ok(())
}

fn update_packages(all: bool) -> Result<()> {
    if all {
        println!("{} Updating all packages...\n", "🔄".green());
        check_sudo();
        let status = Command::new("sudo").args(["dnf", "upgrade", "-y"]).spawn().context("Failed to execute dnf upgrade")?.wait().context("Failed to wait for process")?;
        if !status.success() {
            anyhow::bail!("Package update failed");
        }
        println!("\n{} Successfully updated packages", "✓".green().bold());
    } else {
        println!("{} Updating package database...\n", "🔄".green());
        let output = Command::new("sudo").args(["dnf", "makecache"]).output().context("Failed to execute dnf makecache")?;
        if !output.status.success() {
            anyhow::bail!("Failed to update package database: {}", String::from_utf8_lossy(&output.stderr));
        }
        println!("\n{} Package database updated", "✓".green().bold());
    }
    Ok(())
}

fn check_sudo() {
    if Command::new("sudo").args(["-n", "true"]).status().is_err() {
        println!("{} This operation requires sudo privileges", "⚠️".yellow());
        println!("{} You may be prompted for your password", "ℹ️".blue());
    }
}
