mod gui;

use clap::{Parser, Subcommand};
use colored::*;
use std::process::Command;
use std::path::Path;
use anyhow::Result;
use iced::Application;

#[derive(Parser)]
#[command(name = "rustora", about = "Rustora - A modern package manager for Fedora", version)]
struct Cli {
    #[arg(value_name = "RPM_FILE")]
    rpm_file: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        query: String,
        #[arg(short, long)]
        details: bool,
    },
    Install {
        packages: Vec<String>,
        #[arg(short, long)]
        yes: bool,
    },
    List {
        #[arg(short, long)]
        details: bool,
    },
    Info {
        package: String,
    },
    Update {
        #[arg(short, long)]
        all: bool,
    },
    Gui {
        #[arg(value_name = "RPM_FILE")]
        rpm_file: Option<String>,
    },
    RemoveDialog {
        packages: Vec<String>,
    },
    InstallDialog {
        packages: Vec<String>,
    },
    FlatpakInstallDialog {
        application_id: String,
        #[arg(long)]
        remote: Option<String>,
    },
    FlatpakRemoveDialog {
        application_ids: Vec<String>,
    },
    FlatpakUpdateDialog {
        #[arg(long)]
        packages_b64: String,
    },
    UpdateDialog {
        #[arg(value_name = "PACKAGES_B64")]
        packages_b64: Option<String>,
    },
    UpdateSettingsDialog,
    #[command(name = "proton-install-dialog")]
    ProtonInstallDialog {
        runner_title: String,
        build_title: String,
        download_url: String,
        #[arg(long)]
        launcher: Option<String>,
        #[arg(long)]
        runner_info: Option<String>,
    },
    #[command(name = "proton-changelog-dialog")]
    ProtonChangelogDialog {
        runner_title: String,
        build_title: String,
        description: String,
        page_url: String,
    },
    MaintenanceDialog {
        task: String,
    },
    KernelInstallDialog {
        kernel_name: String,
    },
    DeviceInstallDialog {
        #[arg(long)]
        profile_name: String,
        #[arg(long)]
        install_script: String,
        #[arg(long)]
        vendor_name: String,
        #[arg(long)]
        device_name: String,
        #[arg(long)]
        driver: String,
        #[arg(long)]
        driver_version: String,
        #[arg(long)]
        bus_id: String,
        #[arg(long)]
        vendor_id: String,
        #[arg(long)]
        device_id: String,
        #[arg(long)]
        repositories: String,
    },
    DeviceRemoveDialog {
        #[arg(long)]
        profile_name: String,
        #[arg(long)]
        remove_script: String,
        #[arg(long)]
        vendor_name: String,
        #[arg(long)]
        device_name: String,
        #[arg(long)]
        driver: String,
        #[arg(long)]
        driver_version: String,
        #[arg(long)]
        bus_id: String,
        #[arg(long)]
        vendor_id: String,
        #[arg(long)]
        device_id: String,
        #[arg(long)]
        repositories: String,
    },
    KernelRemoveDialog {
        kernel_name: String,
    },
    Settings,
    GamingMetaDialog,
    CachyosKernelDialog,
    HyprlandDialog,
    HyprlandDotfilesDialog,
}

fn ensure_fonts_async() {
    if !gui::fonts::fonts_exist() {
        tokio::spawn(async {
            let _ = gui::fonts::ensure_fonts().await;
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(rpm_file) = cli.rpm_file {
        let rpm_path = Path::new(&rpm_file).to_path_buf();
        if !rpm_path.exists() {
            return Err(anyhow::anyhow!("RPM file not found: {}", rpm_file));
        }
        if let Some(ext) = rpm_path.extension() {
            if ext.to_string_lossy().to_lowercase() != "rpm" {
                return Err(anyhow::anyhow!("File is not an RPM file: {}", rpm_file));
            }
        } else {
            return Err(anyhow::anyhow!("File does not have an extension: {}", rpm_file));
        }
        ensure_fonts_async();
        use crate::gui::rpm_dialog::RpmDialog;
        RpmDialog::run_separate_window(rpm_path)?;
        return Ok(());
    }

    match cli.command {
        None => {
            ensure_fonts_async();
            let default_font = gui::fonts::get_inter_font();

            gui::RustoraApp::run(iced::Settings {
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
            if let Some(rpm_file_str) = rpm_file {
                let rpm_path = Path::new(&rpm_file_str).to_path_buf();
                if !rpm_path.exists() {
                    return Err(anyhow::anyhow!("RPM file not found: {}", rpm_file_str));
                }
                ensure_fonts_async();
                use crate::gui::rpm_dialog::RpmDialog;
                RpmDialog::run_separate_window(rpm_path)?;
                } else {
                ensure_fonts_async();
                let default_font = gui::fonts::get_inter_font();

                gui::RustoraApp::run(iced::Settings {
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
            ensure_fonts_async();
            use crate::gui::package_dialog::PackageDialog;
            PackageDialog::run_separate_window(packages)?;
            Ok(())
        }
        Some(Commands::InstallDialog { packages }) => {
            ensure_fonts_async();
            use crate::gui::install_dialog::InstallDialog;
            InstallDialog::run_separate_window(packages)?;
            Ok(())
        }
        Some(Commands::FlatpakInstallDialog { application_id, remote }) => {
            ensure_fonts_async();
            use crate::gui::flatpak_dialog::FlatpakDialog;
            FlatpakDialog::run_separate_window(application_id, remote)?;
            Ok(())
        }
        Some(Commands::FlatpakRemoveDialog { application_ids }) => {
            ensure_fonts_async();
            use crate::gui::flatpak_remove_dialog::FlatpakRemoveDialog;
            FlatpakRemoveDialog::run_separate_window(application_ids)?;
            Ok(())
        }
        Some(Commands::FlatpakUpdateDialog { packages_b64 }) => {
            ensure_fonts_async();
            use base64::{Engine as _, engine::general_purpose};
            let decoded = general_purpose::STANDARD.decode(&packages_b64)
                .map_err(|e| anyhow::anyhow!("Failed to decode packages: {}", e))?;
            let packages: Vec<crate::gui::flatpak_update_dialog::FlatpakUpdateInfo> =
                serde_json::from_slice(&decoded)
                    .map_err(|e| anyhow::anyhow!("Failed to parse packages JSON: {}", e))?;
            use crate::gui::flatpak_update_dialog::FlatpakUpdateDialog;
            FlatpakUpdateDialog::run_separate_window(packages)?;
            Ok(())
        }
        Some(Commands::UpdateDialog { packages_b64 }) => {
            ensure_fonts_async();
            use crate::gui::update_dialog::UpdateDialog;
            UpdateDialog::run_separate_window(packages_b64)?;
            Ok(())
        }
        Some(Commands::UpdateSettingsDialog) => {
            ensure_fonts_async();
            use crate::gui::update_settings_dialog::UpdateSettingsDialog;
            UpdateSettingsDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::Settings) => {
            ensure_fonts_async();
            use crate::gui::settings_dialog::SettingsDialog;
            SettingsDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::GamingMetaDialog) => {
            ensure_fonts_async();
            use crate::gui::gaming_meta_dialog::GamingMetaDialog;
            GamingMetaDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::CachyosKernelDialog) => {
            ensure_fonts_async();
            use crate::gui::cachyos_kernel_dialog::CachyosKernelDialog;
            CachyosKernelDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::HyprlandDialog) => {
            ensure_fonts_async();
            use crate::gui::hyprland_dialog::HyprlandDialog;
            HyprlandDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::HyprlandDotfilesDialog) => {
            ensure_fonts_async();
            use crate::gui::hyprland_dotfiles_dialog::HyprlandDotfilesDialog;
            HyprlandDotfilesDialog::run_separate_window()?;
            Ok(())
        }
        Some(Commands::ProtonInstallDialog { runner_title, build_title, download_url, launcher, runner_info }) => {
            ensure_fonts_async();
            use crate::gui::proton_install_dialog::ProtonInstallDialog;
            ProtonInstallDialog::run_separate_window(runner_title, build_title, download_url, launcher, runner_info)?;
            Ok(())
        }
        Some(Commands::ProtonChangelogDialog { runner_title, build_title, description, page_url }) => {
            ensure_fonts_async();
            use crate::gui::proton_changelog_dialog::ProtonChangelogDialog;
            ProtonChangelogDialog::run_separate_window(runner_title, build_title, description, page_url)?;
            Ok(())
        }
        Some(Commands::MaintenanceDialog { task }) => {
            ensure_fonts_async();
            use crate::gui::maintenance_dialog::{MaintenanceDialog, MaintenanceTask};
            let maintenance_task = match task.as_str() {
                "rebuild-kernel-modules" => MaintenanceTask::RebuildKernelModules,
                "regenerate-initramfs" => MaintenanceTask::RegenerateInitramfs,
                "remove-orphaned-packages" => MaintenanceTask::RemoveOrphanedPackages,
                "clean-package-cache" => MaintenanceTask::CleanPackageCache,
                _ => return Err(anyhow::anyhow!("Unknown maintenance task: {}", task)),
            };
            MaintenanceDialog::run_separate_window(maintenance_task)?;
            Ok(())
        }
        Some(Commands::KernelInstallDialog { kernel_name }) => {
            ensure_fonts_async();
            use crate::gui::kernel_install_dialog::KernelInstallDialog;
            KernelInstallDialog::run_separate_window(kernel_name)?;
            Ok(())
        }
        Some(Commands::DeviceInstallDialog {
            profile_name,
            install_script,
            vendor_name,
            device_name,
            driver,
            driver_version,
            bus_id,
            vendor_id,
            device_id,
            repositories,
        }) => {
            ensure_fonts_async();
            use base64::{Engine as _, engine::general_purpose};
            let decoded_script = general_purpose::STANDARD.decode(&install_script)
                .map_err(|e| anyhow::anyhow!("Failed to decode install script: {}", e))?;
            let script = String::from_utf8(decoded_script)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in install script: {}", e))?;

            let decode = |s: &str| String::from_utf8(general_purpose::STANDARD.decode(s).unwrap_or_default()).unwrap_or_default();
            let vendor = decode(&vendor_name);
            let device = decode(&device_name);
            let drv = decode(&driver);
            let drv_ver = decode(&driver_version);
            let bus = decode(&bus_id);
            let vid = decode(&vendor_id);
            let did = decode(&device_id);
            let repos_json = decode(&repositories);
            let repos: Vec<String> = serde_json::from_str(&repos_json).unwrap_or_default();

            use crate::gui::device_install_dialog::{DeviceInstallDialog, DeviceInfo};
            DeviceInstallDialog::run_separate_window(profile_name, script, DeviceInfo {
                vendor_name: vendor,
                device_name: device,
                driver: drv,
                driver_version: drv_ver,
                bus_id: bus,
                vendor_id: vid,
                device_id: did,
                repositories: repos,
            }, false)?;
            Ok(())
        }
        Some(Commands::DeviceRemoveDialog {
            profile_name,
            remove_script,
            vendor_name,
            device_name,
            driver,
            driver_version,
            bus_id,
            vendor_id,
            device_id,
            repositories,
        }) => {
            ensure_fonts_async();
            use base64::{Engine as _, engine::general_purpose};
            let decoded_script = general_purpose::STANDARD.decode(&remove_script)
                .map_err(|e| anyhow::anyhow!("Failed to decode remove script: {}", e))?;
            let script = String::from_utf8(decoded_script)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in remove script: {}", e))?;

            let decode = |s: &str| String::from_utf8(general_purpose::STANDARD.decode(s).unwrap_or_default()).unwrap_or_default();
            let vendor = decode(&vendor_name);
            let device = decode(&device_name);
            let drv = decode(&driver);
            let drv_ver = decode(&driver_version);
            let bus = decode(&bus_id);
            let vid = decode(&vendor_id);
            let did = decode(&device_id);
            let repos_json = decode(&repositories);
            let repos: Vec<String> = serde_json::from_str(&repos_json).unwrap_or_default();

            use crate::gui::device_install_dialog::{DeviceInstallDialog, DeviceInfo};
            DeviceInstallDialog::run_separate_window(profile_name, script, DeviceInfo {
                vendor_name: vendor,
                device_name: device,
                driver: drv,
                driver_version: drv_ver,
                bus_id: bus,
                vendor_id: vid,
                device_id: did,
                repositories: repos,
            }, true)?;
            Ok(())
        }
        Some(Commands::KernelRemoveDialog { kernel_name }) => {
            ensure_fonts_async();
            use crate::gui::kernel_install_dialog::KernelInstallDialog;
            KernelInstallDialog::run_separate_window(kernel_name)?;
            Ok(())
        }
        Some(cmd) => {
            if let Err(_e) = match cmd {
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
                Commands::FlatpakUpdateDialog { .. } => unreachable!(),
                Commands::UpdateDialog { .. } => unreachable!(),
                Commands::UpdateSettingsDialog => unreachable!(),
                Commands::Settings => unreachable!(),
                Commands::GamingMetaDialog => unreachable!(),
                Commands::CachyosKernelDialog => unreachable!(),
                Commands::HyprlandDialog => unreachable!(),
                Commands::HyprlandDotfilesDialog => unreachable!(),
                Commands::ProtonInstallDialog { .. } => unreachable!(),
                Commands::ProtonChangelogDialog { .. } => unreachable!(),
                Commands::MaintenanceDialog { .. } => unreachable!(),
                Commands::KernelInstallDialog { .. } => unreachable!(),
                Commands::KernelRemoveDialog { .. } => unreachable!(),
                Commands::DeviceInstallDialog { .. } => unreachable!(),
                Commands::DeviceRemoveDialog { .. } => unreachable!(),
            } {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

fn search_packages(query: &str, details: bool) -> Result<()> {
    println!("{} Searching for: {}\n", "[SEARCH]".green(), query.bright_white().bold());
    let mut cmd = Command::new("dnf");
    cmd.arg("search").arg("--quiet");
    if details { cmd.arg("--showduplicates"); }
    cmd.arg(query);
    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!("DNF search failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        println!("{} No packages found matching '{}'", "[WARN]".yellow(), query);
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
        for (name, desc) in &results {
            println!("{} {}", name.bright_cyan().bold(), desc.bright_white());
        }
        println!("\n{} Found {} package(s)", "[OK]".green(), results.len().to_string().bright_white().bold());
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
    println!("{} Installing package(s): {}\n", "[PKG]".green(), packages.join(", ").bright_white().bold());
    check_sudo();
    let mut cmd = Command::new("sudo");
    cmd.arg("dnf").arg("install");
    if yes { cmd.arg("-y"); }
    cmd.args(packages);
    let status = cmd.spawn()?.wait()?;
    if !status.success() {
        anyhow::bail!("Package installation failed");
    }
    println!("\n{} Successfully installed package(s)", "[OK]".green().bold());
    Ok(())
}

fn list_packages(details: bool) -> Result<()> {
    println!("{} Listing installed packages...\n", "[LIST]".green());
    let output = Command::new("dnf").args(["list", "--installed", "--quiet"]).output()?;
    if !output.status.success() {
        anyhow::bail!("DNF list failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<&str> = stdout.lines().skip(1).filter(|l| !l.trim().is_empty()).collect();
    let count = packages.len();
    if count == 0 {
        println!("{} No packages found", "[WARN]".yellow());
        return Ok(());
    }
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
    println!("\n{} Total: {} package(s)", "[OK]".green(), count.to_string().bright_white().bold());
    Ok(())
}

fn show_package_info(package: &str) -> Result<()> {
    println!("{} Package information: {}\n", "[INFO]".blue(), package.bright_white().bold());
    let output = Command::new("dnf").args(["info", package]).output()?;
    if !output.status.success() {
        anyhow::bail!("DNF info failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        println!("{} Package '{}' not found", "[WARN]".yellow(), package);
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
        println!("{} Updating all packages...\n", "[UPDATE]".green());
        check_sudo();
        let status = Command::new("sudo").args(["dnf", "upgrade", "-y"]).spawn()?.wait()?;
        if !status.success() {
            anyhow::bail!("Package update failed");
        }
        println!("\n{} Successfully updated packages", "[OK]".green().bold());
    } else {
        println!("{} Updating package database...\n", "[UPDATE]".green());
        let output = Command::new("sudo").args(["dnf", "makecache"]).output()?;
        if !output.status.success() {
            anyhow::bail!("Failed to update package database: {}", String::from_utf8_lossy(&output.stderr));
        }
        println!("\n{} Package database updated", "[OK]".green().bold());
    }
    Ok(())
}

fn check_sudo() {
    if Command::new("sudo").args(["-n", "true"]).status().is_err() {
        println!("{} This operation requires sudo privileges", "[WARN]".yellow());
        println!("{} You may be prompted for your password", "[INFO]".blue());
    }
}
