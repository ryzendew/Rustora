use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Padding, Border, Color};
use crate::gui::app::CustomScrollableStyle;
use iced::widget::container::Appearance;
use iced::widget::scrollable::{Appearance as ScrollableAppearance, StyleSheet as ScrollableStyleSheet};
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

use libcfhdb::pci::{CfhdbPciDevice, CfhdbPciProfile};
use libcfhdb::usb::{CfhdbUsbDevice, CfhdbUsbProfile};
use crate::logger;

#[derive(Debug, Clone)]
pub struct PreCheckedPciDevice {
    pub device: CfhdbPciDevice,
    pub profiles: Vec<Arc<PreCheckedPciProfile>>,
}

#[derive(Debug, Clone)]
pub struct PreCheckedPciProfile {
    profile: CfhdbPciProfile,

    installed: Arc<std::sync::RwLock<bool>>,
    driver_version: Arc<std::sync::RwLock<Option<String>>>,
    repository: Arc<std::sync::RwLock<Option<String>>>,
    package_size: Arc<std::sync::RwLock<Option<String>>>,
    dependencies: Arc<std::sync::RwLock<Option<Vec<String>>>>,
}

impl PreCheckedPciProfile {
    pub fn new(profile: CfhdbPciProfile) -> Self {
        Self {
            profile,
            installed: Arc::new(std::sync::RwLock::new(false)),
            driver_version: Arc::new(std::sync::RwLock::new(None)),
            repository: Arc::new(std::sync::RwLock::new(None)),
            package_size: Arc::new(std::sync::RwLock::new(None)),
            dependencies: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn profile(&self) -> &CfhdbPciProfile {
        &self.profile
    }

    #[allow(dead_code)]
    pub fn installed(&self) -> bool {
        *self.installed.read().unwrap()
    }

    pub fn update_installed(&self) {
        *self.installed.write().unwrap() = self.profile.get_status();
    }

    pub fn driver_version(&self) -> Option<String> {
        self.driver_version.read().unwrap().clone()
    }

    pub fn set_driver_version(&self, version: Option<String>) {
        *self.driver_version.write().unwrap() = version;
    }

    pub fn repository(&self) -> Option<String> {
        self.repository.read().unwrap().clone()
    }

    pub fn set_repository(&self, repo: Option<String>) {
        *self.repository.write().unwrap() = repo;
    }

    pub fn package_size(&self) -> Option<String> {
        self.package_size.read().unwrap().clone()
    }

    pub fn set_package_size(&self, size: Option<String>) {
        *self.package_size.write().unwrap() = size;
    }

    pub fn dependencies(&self) -> Option<Vec<String>> {
        self.dependencies.read().unwrap().clone()
    }

    pub fn set_dependencies(&self, deps: Option<Vec<String>>) {
        *self.dependencies.write().unwrap() = deps;
    }
}

#[derive(Debug, Clone)]
pub struct PreCheckedUsbDevice {
    pub device: CfhdbUsbDevice,
    pub profiles: Vec<Arc<PreCheckedUsbProfile>>,
}

#[derive(Debug, Clone)]
pub struct PreCheckedUsbProfile {
    profile: CfhdbUsbProfile,

    installed: Arc<std::sync::RwLock<bool>>,
    driver_version: Arc<std::sync::RwLock<Option<String>>>,
    #[allow(dead_code)]
    repository: Arc<std::sync::RwLock<Option<String>>>,
    #[allow(dead_code)]
    package_size: Arc<std::sync::RwLock<Option<String>>>,
    #[allow(dead_code)]
    dependencies: Arc<std::sync::RwLock<Option<Vec<String>>>>,
}

impl PreCheckedUsbProfile {
    pub fn new(profile: CfhdbUsbProfile) -> Self {
        Self {
            profile,
            installed: Arc::new(std::sync::RwLock::new(false)),
            driver_version: Arc::new(std::sync::RwLock::new(None)),
            repository: Arc::new(std::sync::RwLock::new(None)),
            package_size: Arc::new(std::sync::RwLock::new(None)),
            dependencies: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn profile(&self) -> &CfhdbUsbProfile {
        &self.profile
    }

    #[allow(dead_code)]
    pub fn installed(&self) -> bool {
        *self.installed.read().unwrap()
    }

    pub fn update_installed(&self) {
        *self.installed.write().unwrap() = self.profile.get_status();
    }

    pub fn driver_version(&self) -> Option<String> {
        self.driver_version.read().unwrap().clone()
    }

    pub fn set_driver_version(&self, version: Option<String>) {
        *self.driver_version.write().unwrap() = version;
    }

    #[allow(dead_code)]
    pub fn repository(&self) -> Option<String> {
        self.repository.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn set_repository(&self, repo: Option<String>) {
        *self.repository.write().unwrap() = repo;
    }

    #[allow(dead_code)]
    pub fn package_size(&self) -> Option<String> {
        self.package_size.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn set_package_size(&self, size: Option<String>) {
        *self.package_size.write().unwrap() = size;
    }

    #[allow(dead_code)]
    pub fn dependencies(&self) -> Option<Vec<String>> {
        self.dependencies.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn set_dependencies(&self, deps: Option<Vec<String>>) {
        *self.dependencies.write().unwrap() = deps;
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    RequestPermissions,
    PermissionsGranted,
    PermissionsDenied(String),
    LoadDevices,
    DevicesLoaded {
        pci_devices: Vec<(String, Vec<PreCheckedPciDevice>)>,
        usb_devices: Vec<(String, Vec<PreCheckedUsbDevice>)>,
        pci_profiles: Vec<Arc<PreCheckedPciProfile>>,
        usb_profiles: Vec<Arc<PreCheckedUsbProfile>>,
    },
    SelectCategory(CategoryType, String),
    SelectDevice(DeviceType, String, usize),
    LoadDevicesAfterCache,
    DownloadProfiles,
    #[allow(dead_code)]
    DownloadProfilesForce,
    ProfilesDownloaded(Result<(), String>),
    BackToDeviceList,
    ToggleProfileSelection(#[allow(dead_code)] DeviceType, #[allow(dead_code)] String, #[allow(dead_code)] usize, String),
    InstallSelectedProfiles(DeviceType, String, usize),
    StartDevice(DeviceType, String, usize),
    StopDevice(DeviceType, String, usize),
    EnableDevice(DeviceType, String, usize),
    DisableDevice(DeviceType, String, usize),
    DeviceControlComplete,
    InstallProfile(DeviceType, String, usize, String),
    RemoveProfile(DeviceType, String, usize, String),
    ProfileOperationComplete,
    Error(String),
    ClearError,
    UpdateStatus,
    ToggleCfhdbProfiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryType {
    Pci,
    Usb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Pci,
    Usb,
}

#[derive(Debug, Clone)]
enum DeviceInfo {
    Pci {
        #[allow(dead_code)]
        vendor_name: String,
        #[allow(dead_code)]
        device_name: String,
        driver: String,
        driver_version: String,
        bus_id: String,
        vendor_id: String,
        device_id: String,
        started: bool,
        enabled: bool,
    },
    Usb {
        #[allow(dead_code)]
        manufacturer: String,
        #[allow(dead_code)]
        product: String,
        driver: String,
        driver_version: String,
        bus_id: String,
        vendor_id: String,
        product_id: String,
        started: bool,
        enabled: bool,
    },
}

#[derive(Debug)]
pub struct DeviceTab {

    is_loading: bool,
    loading_message: String,

    pci_devices: Vec<(String, Vec<PreCheckedPciDevice>)>,
    usb_devices: Vec<(String, Vec<PreCheckedUsbDevice>)>,
    pci_profiles: Vec<Arc<PreCheckedPciProfile>>,
    usb_profiles: Vec<Arc<PreCheckedUsbProfile>>,

    selected_category: Option<(CategoryType, String)>,
    selected_device: Option<(DeviceType, String, usize)>,
    selected_profiles: std::collections::HashSet<String>,

    error: Option<String>,
}

impl DeviceTab {
    pub fn new() -> Self {
        Self {
            is_loading: false,
            loading_message: String::new(),
            pci_devices: Vec::new(),
            usb_devices: Vec::new(),
            pci_profiles: Vec::new(),
            usb_profiles: Vec::new(),
            selected_category: None,
            selected_device: None,
            selected_profiles: std::collections::HashSet::new(),
            error: None,
        }
    }

    // Helper function to format device operation errors with user-friendly messages
    fn format_device_error(operation: &str, error: std::io::Error) -> String {
        let error_msg = format!("{}", error);
        let user_friendly_msg = if error_msg.contains("exited with code 127") {
            if error_msg.contains("sysfs_helper.sh") {
                "The cfhdb system package is not installed. The device control features require the cfhdb package to be installed on your system. Please install it using: sudo dnf install cfhdb"
            } else if error_msg.contains("pkexec") {
                "Authentication failed or polkit is not available. Please ensure polkit is installed and try again."
            } else {
                "Command not found. Please check that required system tools are installed."
            }
        } else if error_msg.contains("Permission denied") || error_msg.contains("permission") {
            "Permission denied. Please ensure you have the necessary permissions to manage devices."
        } else {
            error_msg.as_str()
        };
        format!("Failed to {} device: {}", operation, user_friendly_msg)
    }

    // Check if cfhdb system components are available
    fn check_cfhdb_available() -> bool {
        std::path::Path::new("/usr/lib/cfhdb/scripts/sysfs_helper.sh").exists()
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::RequestPermissions => {
                self.is_loading = true;
                self.loading_message = "Requesting elevated permissions...".into();
                self.error = None;

                iced::Command::perform(request_permissions(), |result| {
                    match result {
                        Ok(_) => Message::PermissionsGranted,
                        Err(e) => Message::PermissionsDenied(e),
                    }
                })
            }
            Message::PermissionsGranted => {
                self.loading_message = "Permissions granted. Loading devices...".into();

                iced::Command::perform(async {}, |_| Message::LoadDevices)
            }
            Message::PermissionsDenied(error) => {
                self.is_loading = false;
                self.loading_message = String::new();
                self.error = Some(format!("Permission denied: {}\n\nDevice management requires elevated permissions. Please try again.", error));
                iced::Command::none()
            }
            Message::LoadDevices => {

                    self.is_loading = true;
                    self.loading_message = "Checking device profiles...".into();
                self.error = None;

                    iced::Command::perform(ensure_profiles_cached(), |result| {
                        match result {
                            Ok(_) => Message::LoadDevicesAfterCache,
                            Err(e) => Message::Error(format!("Failed to cache profiles: {}", e)),
                        }
                    })
            }
            Message::LoadDevicesAfterCache => {
                self.loading_message = "Loading device profiles...".into();
                iced::Command::perform(load_all_devices(), |result| {
                    match result {
                        Ok(data) => Message::DevicesLoaded {
                            pci_devices: data.0,
                            usb_devices: data.1,
                            pci_profiles: data.2,
                            usb_profiles: data.3,
                        },
                        Err(e) => Message::Error(e),
                    }
                })
            }
            Message::DownloadProfiles => {
                self.is_loading = true;
                self.loading_message = "Downloading and caching profiles...".into();
                iced::Command::perform(ensure_profiles_cached_force(), |result| {
                    Message::ProfilesDownloaded(result)
                })
            }
            Message::DownloadProfilesForce => {
                self.is_loading = true;
                self.loading_message = "Downloading and caching profiles...".into();
                iced::Command::perform(ensure_profiles_cached_force(), |result| {
                    Message::ProfilesDownloaded(result)
                })
            }
            Message::ProfilesDownloaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(_) => {
                        self.loading_message = String::new();
                        self.error = None;

                        iced::Command::perform(async {}, |_| Message::LoadDevices)
                    }
                    Err(e) => {
                        self.loading_message = String::new();

                        let error_msg = if e.contains("dns error") || e.contains("failed to lookup") {
                            format!("Network error: Could not reach profile server. Please check your internet connection.\n\nDetails: {}", e)
                        } else if e.contains("HTTP") {
                            format!("Server error: The profile server returned an error.\n\nDetails: {}", e)
                        } else {
                            format!("Failed to download profiles: {}", e)
                        };
                        self.error = Some(error_msg);
                        iced::Command::none()
                    }
                }
            }
            Message::DevicesLoaded {
                pci_devices,
                usb_devices,
                pci_profiles,
                usb_profiles,
            } => {
                self.is_loading = false;
                self.pci_devices = pci_devices;
                self.usb_devices = usb_devices;
                self.pci_profiles = pci_profiles;
                self.usb_profiles = usb_profiles;

                iced::Command::perform(load_profile_versions(self.pci_profiles.clone(), self.usb_profiles.clone()), |_| Message::UpdateStatus)
            }
            Message::SelectCategory(cat_type, class) => {
                self.selected_category = Some((cat_type, class));
                self.selected_device = None;
                iced::Command::none()
            }
            Message::SelectDevice(dev_type, class, index) => {
                self.selected_device = Some((dev_type, class, index));

                iced::Command::perform(async {}, |_| Message::UpdateStatus)
            }
            Message::BackToDeviceList => {
                self.selected_device = None;
                self.selected_profiles.clear();
                iced::Command::none()
            }
            Message::ToggleProfileSelection(_, _, _, profile_codename) => {
                if self.selected_profiles.contains(&profile_codename) {
                    self.selected_profiles.remove(&profile_codename);
                } else {
                    self.selected_profiles.insert(profile_codename);
                }
                iced::Command::none()
            }
            Message::InstallSelectedProfiles(dev_type, class, device_idx) => {
                if self.selected_profiles.is_empty() {
                    return iced::Command::none();
                }

                let first_profile = self.selected_profiles.iter().next().cloned();
                self.selected_profiles.clear();

                if let Some(profile) = first_profile {
                    iced::Command::perform(async {}, move |_| {
                        Message::InstallProfile(dev_type, class, device_idx, profile)
                    })
                } else {
                    iced::Command::none()
                }
            }
            Message::ClearError => {
                self.error = None;

                if self.selected_device.is_some() {
                    iced::Command::none()
                } else {
                    iced::Command::none()
                }
            }
            Message::StartDevice(dev_type, class, device_idx) => {
                let result = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.start_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                    DeviceType::Usb => {
                        if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.start_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                };
                match result {
                    Ok(_) => iced::Command::perform(async {}, |_| Message::DeviceControlComplete),
                    Err(e) => {
                        let final_msg = Self::format_device_error("start", e);
                        iced::Command::perform(async {}, move |_| Message::Error(final_msg))
                    }
                }
            }
            Message::StopDevice(dev_type, class, device_idx) => {
                let result = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.stop_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                    DeviceType::Usb => {
                        if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.stop_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                };
                match result {
                    Ok(_) => iced::Command::perform(async {}, |_| Message::DeviceControlComplete),
                    Err(e) => {
                        let final_msg = Self::format_device_error("stop", e);
                        iced::Command::perform(async {}, move |_| Message::Error(final_msg))
                    }
                }
            }
            Message::EnableDevice(dev_type, class, device_idx) => {
                let result = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.enable_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                    DeviceType::Usb => {
                        if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.enable_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                };
                match result {
                    Ok(_) => iced::Command::perform(async {}, |_| Message::DeviceControlComplete),
                    Err(e) => {
                        let final_msg = Self::format_device_error("enable", e);
                        iced::Command::perform(async {}, move |_| Message::Error(final_msg))
                    }
                }
            }
            Message::DisableDevice(dev_type, class, device_idx) => {
                let result = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.disable_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                    DeviceType::Usb => {
                        if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                device.device.disable_device()
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device not found"))
                            }
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Device class not found"))
                        }
                    }
                };
                match result {
                    Ok(_) => iced::Command::perform(async {}, |_| Message::DeviceControlComplete),
                    Err(e) => {
                        let final_msg = Self::format_device_error("disable", e);
                        iced::Command::perform(async {}, move |_| Message::Error(final_msg))
                    }
                }
            }
            Message::DeviceControlComplete => {

                iced::Command::perform(async {}, |_| Message::UpdateStatus)
            }
            Message::InstallProfile(dev_type, class, device_idx, profile_codename) => {

                let profile_data = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {

                                if let Some(profile) = device.profiles.iter().find(|p| p.profile().codename == profile_codename) {
                                    let p = profile.profile();
                                    let d = &device.device;

                                    let driver_version = profile.driver_version().unwrap_or_default();

                                    let driver_name = if !driver_version.is_empty() {
                                        driver_version.clone()
                                    } else {

                                        p.i18n_desc.split(" (").next()
                                            .unwrap_or_else(|| p.i18n_desc.split(" for ").next().unwrap_or(&p.i18n_desc))
                                            .trim()
                                            .to_string()
                                    };

                                    let repositories = extract_repositories_from_script(&p.install_script);
                                    Some((
                                        p.i18n_desc.clone(),
                                        p.install_script.clone(),
                                        d.vendor_name.clone(),
                                        d.device_name.clone(),
                                        driver_name,
                                        driver_version,
                                        d.sysfs_busid.clone(),
                                        d.vendor_id.clone(),
                                        d.device_id.clone(),
                                        repositories,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    DeviceType::Usb => {
                        if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {

                                if let Some(profile) = device.profiles.iter().find(|p| p.profile().codename == profile_codename) {
                                    let p = profile.profile();
                                    let d = &device.device;

                                    let driver_version = profile.driver_version().unwrap_or_default();

                                    let driver_name = if !driver_version.is_empty() {
                                        driver_version.clone()
                                    } else {

                                        p.i18n_desc.split(" (").next()
                                            .unwrap_or_else(|| p.i18n_desc.split(" for ").next().unwrap_or(&p.i18n_desc))
                                            .trim()
                                            .to_string()
                                    };

                                    let repositories = extract_repositories_from_script(&p.install_script);
                                    Some((
                                        p.i18n_desc.clone(),
                                        p.install_script.clone(),
                                        d.manufacturer_string_index.clone(),
                                        d.product_string_index.clone(),
                                        driver_name,
                                        driver_version,
                                        d.sysfs_busid.clone(),
                                        d.vendor_id.clone(),
                                        d.product_id.clone(),
                                        repositories,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };

                if let Some((profile_name, install_script, vendor_name, device_name, driver, driver_version, bus_id, vendor_id, device_id, repositories)) = profile_data {
                    if let Some(script) = install_script {

                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let exe_str = exe_path.to_string_lossy().into_owned();
                        let profile_name_clone = profile_name.clone();
                        let script_clone = script.clone();

                        iced::Command::perform(
                            async move {
                                use tokio::process::Command as TokioCommand;

                                use base64::{Engine as _, engine::general_purpose};
                                let encoded_script = general_purpose::STANDARD.encode(script_clone.as_bytes());
                                let encoded_vendor = general_purpose::STANDARD.encode(vendor_name.as_bytes());
                                let encoded_device = general_purpose::STANDARD.encode(device_name.as_bytes());
                                let encoded_driver = general_purpose::STANDARD.encode(driver.as_bytes());
                                let encoded_drv_ver = general_purpose::STANDARD.encode(driver_version.as_bytes());
                                let encoded_bus = general_purpose::STANDARD.encode(bus_id.as_bytes());
                                let encoded_vid = general_purpose::STANDARD.encode(vendor_id.as_bytes());
                                let encoded_did = general_purpose::STANDARD.encode(device_id.as_bytes());
                                let encoded_repos = general_purpose::STANDARD.encode(serde_json::to_string(&repositories).unwrap_or_default().as_bytes());

                                TokioCommand::new(&exe_str)
                                    .arg("device-install-dialog")
                                    .arg("--profile-name")
                                    .arg(&profile_name_clone)
                                    .arg("--install-script")
                                    .arg(&encoded_script)
                                    .arg("--vendor-name")
                                    .arg(&encoded_vendor)
                                    .arg("--device-name")
                                    .arg(&encoded_device)
                                    .arg("--driver")
                                    .arg(&encoded_driver)
                                    .arg("--driver-version")
                                    .arg(&encoded_drv_ver)
                                    .arg("--bus-id")
                                    .arg(&encoded_bus)
                                    .arg("--vendor-id")
                                    .arg(&encoded_vid)
                                    .arg("--device-id")
                                    .arg(&encoded_did)
                                    .arg("--repositories")
                                    .arg(&encoded_repos)
                                    .spawn()
                                    .ok();
                            },
                            |_| Message::ProfileOperationComplete,
                        )
                    } else {
                        self.error = Some("This profile does not have an install script.".to_string());
                        iced::Command::none()
                    }
                } else {
                    self.error = Some("Profile not found.".to_string());
                    iced::Command::none()
                }
            }
            Message::RemoveProfile(dev_type, class, device_idx, profile_codename) => {

                let profile_data = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {

                                if let Some(profile) = device.profiles.iter().find(|p| p.profile().codename == profile_codename) {
                                    let p = profile.profile();
                                    let d = &device.device;
                                    let driver_version = profile.driver_version().unwrap_or_default();
                                    let driver_name = if !driver_version.is_empty() {
                                        driver_version.clone()
                                    } else {
                                        p.i18n_desc.split(" (").next()
                                            .unwrap_or_else(|| p.i18n_desc.split(" for ").next().unwrap_or(&p.i18n_desc))
                                            .trim()
                                            .to_string()
                                    };
                                    let repositories = extract_repositories_from_script(&p.remove_script);
                                    Some((
                                        p.i18n_desc.clone(),
                                        p.remove_script.clone(),
                                        d.vendor_name.clone(),
                                        d.device_name.clone(),
                                        driver_name,
                                        driver_version,
                                        d.sysfs_busid.clone(),
                                        d.vendor_id.clone(),
                                        d.device_id.clone(),
                                        repositories,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    DeviceType::Usb => {
                        if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {

                                if let Some(profile) = device.profiles.iter().find(|p| p.profile().codename == profile_codename) {
                                    let p = profile.profile();
                                    let d = &device.device;
                                    let driver_version = profile.driver_version().unwrap_or_default();
                                    let driver_name = if !driver_version.is_empty() {
                                        driver_version.clone()
                                    } else {
                                        p.i18n_desc.split(" (").next()
                                            .unwrap_or_else(|| p.i18n_desc.split(" for ").next().unwrap_or(&p.i18n_desc))
                                            .trim()
                                            .to_string()
                                    };
                                    let repositories = extract_repositories_from_script(&p.remove_script);
                                    Some((
                                        p.i18n_desc.clone(),
                                        p.remove_script.clone(),
                                        d.manufacturer_string_index.clone(),
                                        d.product_string_index.clone(),
                                        driver_name,
                                        driver_version,
                                        d.sysfs_busid.clone(),
                                        d.vendor_id.clone(),
                                        d.product_id.clone(),
                                        repositories,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };

                if let Some((profile_name, remove_script, vendor_name, device_name, driver, driver_version, bus_id, vendor_id, device_id, repositories)) = profile_data {
                    if let Some(script) = remove_script {

                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("rustora"));
                        let exe_str = exe_path.to_string_lossy().into_owned();
                        let profile_name_clone = profile_name.clone();
                        let script_clone = script.clone();

                        iced::Command::perform(
                            async move {
                                use tokio::process::Command as TokioCommand;

                                use base64::{Engine as _, engine::general_purpose};
                                let encoded_script = general_purpose::STANDARD.encode(script_clone.as_bytes());
                                let encoded_vendor = general_purpose::STANDARD.encode(vendor_name.as_bytes());
                                let encoded_device = general_purpose::STANDARD.encode(device_name.as_bytes());
                                let encoded_driver = general_purpose::STANDARD.encode(driver.as_bytes());
                                let encoded_drv_ver = general_purpose::STANDARD.encode(driver_version.as_bytes());
                                let encoded_bus = general_purpose::STANDARD.encode(bus_id.as_bytes());
                                let encoded_vid = general_purpose::STANDARD.encode(vendor_id.as_bytes());
                                let encoded_did = general_purpose::STANDARD.encode(device_id.as_bytes());
                                let encoded_repos = general_purpose::STANDARD.encode(serde_json::to_string(&repositories).unwrap_or_default().as_bytes());

                                TokioCommand::new(&exe_str)
                                    .arg("device-remove-dialog")
                                    .arg("--profile-name")
                                    .arg(&profile_name_clone)
                                    .arg("--remove-script")
                                    .arg(&encoded_script)
                                    .arg("--vendor-name")
                                    .arg(&encoded_vendor)
                                    .arg("--device-name")
                                    .arg(&encoded_device)
                                    .arg("--driver")
                                    .arg(&encoded_driver)
                                    .arg("--driver-version")
                                    .arg(&encoded_drv_ver)
                                    .arg("--bus-id")
                                    .arg(&encoded_bus)
                                    .arg("--vendor-id")
                                    .arg(&encoded_vid)
                                    .arg("--device-id")
                                    .arg(&encoded_did)
                                    .arg("--repositories")
                                    .arg(&encoded_repos)
                                    .spawn()
                                    .ok();
                            },
                            |_| Message::ProfileOperationComplete,
                        )
                    } else {
                        self.error = Some("This profile does not have a remove script.".to_string());
                        iced::Command::none()
                    }
                } else {
                    self.error = Some("Profile not found.".to_string());
                    iced::Command::none()
                }
            }
            Message::ProfileOperationComplete => {

                iced::Command::perform(async {}, |_| Message::UpdateStatus)
            }
            Message::UpdateStatus => {

                if let Some((dev_type, class, device_idx)) = &self.selected_device {

                    match dev_type {
                        DeviceType::Pci => {
                            if let Some((_, devices)) = self.pci_devices.iter_mut().find(|(c, _)| c == class) {
                                if let Some(device) = devices.get_mut(*device_idx) {

                                    if let Ok(updated) = libcfhdb::pci::CfhdbPciDevice::get_device_from_busid(&device.device.sysfs_busid) {
                                        device.device = updated;
                                    }

                                    for profile in &device.profiles {
                                        profile.update_installed();
                                    }
                                }
                            }
                        }
                        DeviceType::Usb => {
                            if let Some((_, devices)) = self.usb_devices.iter_mut().find(|(c, _)| c == class) {
                                if let Some(device) = devices.get_mut(*device_idx) {

                                    if let Ok(updated) = libcfhdb::usb::CfhdbUsbDevice::get_device_from_busid(&device.device.sysfs_busid) {
                                        device.device = updated;
                                    }

                                    for profile in &device.profiles {
                                        profile.update_installed();
                                    }
                                }
                            }
                        }
                    }
                }

                iced::Command::none()
            }
            Message::Error(msg) => {
                self.is_loading = false;
                self.error = Some(msg);
                iced::Command::none()
            }
            Message::ToggleCfhdbProfiles => {
                // Toggle the setting and reload devices
                let mut current_settings = crate::gui::settings::AppSettings::load();
                current_settings.show_cfhdb_profiles = !current_settings.show_cfhdb_profiles;
                current_settings.save();
                logger::Logger::log_debug(&format!("[Device Tab] Toggled cfhdb profiles: {}", current_settings.show_cfhdb_profiles));
                // Reload devices to apply the change
                self.is_loading = true;
                self.loading_message = "Reloading devices...".into();
                iced::Command::perform(ensure_profiles_cached(), |result| {
                    match result {
                        Ok(_) => Message::LoadDevicesAfterCache,
                        Err(e) => Message::Error(format!("Failed to cache profiles: {}", e)),
                    }
                })
            }
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {

        let _title_font_size = (settings.font_size_titles * settings.scale_titles).round();
        let body_font_size = (settings.font_size_body * settings.scale_body).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons).round();
        let _icon_size = (settings.font_size_icons * settings.scale_icons).round();
        if self.is_loading {
            container(
                column![
                    text("Loading device manager...").size(body_font_size * 1.14),
                    Space::with_height(Length::Fixed(10.0)),
                    text(&self.loading_message).size(body_font_size).style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
        } else if let Some(err) = &self.error {
            let back_button = button(
                text("<- Back").size(button_font_size)
            )
            .on_press(Message::ClearError)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
                radius: settings.border_radius,
            })))
            .padding(Padding::new(10.0));

            container(
                column![
                    back_button,
                    Space::with_height(Length::Fixed(20.0)),
                    text("Error").size(body_font_size * 1.29).style(iced::theme::Text::Color(theme.danger())),
                    Space::with_height(Length::Fixed(10.0)),
                    text(err).size(body_font_size),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
        } else if self.pci_devices.is_empty() && self.usb_devices.is_empty() {
            container(
                column![
                    text("No devices found").size(body_font_size * 1.14),
                    Space::with_height(Length::Fixed(10.0)),
                    button("Load Devices")
                        .on_press(Message::LoadDevices)
                        .padding(Padding::new(12.0))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
                            radius: settings.border_radius,
                        }))),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
        } else {

            self.view_main(theme, settings)
        }
    }

    fn view_main(&self, theme: &crate::gui::Theme, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();
        let sidebar = self.view_sidebar(theme, &material_font, settings);
        let content = self.view_content(theme, &material_font, settings);

        container(
            row![
                container(sidebar)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
                Space::with_width(Length::Fixed(20.0)),
                container(content)
                    .width(Length::FillPortion(3))
                    .height(Length::Fill),
            ]
            .spacing(0)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::from([16.0, 16.0, 16.0, 16.0]))
        .into()
    }

    fn view_sidebar(&self, theme: &crate::gui::Theme, material_font: &iced::Font, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.15).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.2).round();
        let section_font_size = (settings.font_size_body * settings.scale_body * 1.05).round();

        let mut sidebar_items = column![].spacing(8);
        let download_button_text = if self.is_loading && self.loading_message.contains("Downloading") {
            row![
                text(glyphs::REFRESH_SYMBOL).font(*material_font).size(icon_size),
                text(" Downloading...").size(button_font_size),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        } else {
            row![
                text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(icon_size),
                text(" Download Profiles").size(button_font_size),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        };

        let download_button = if self.is_loading && self.loading_message.contains("Downloading") {
            button(download_button_text)
                .width(Length::Fill)
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
        } else {
            button(download_button_text)
                .on_press(Message::DownloadProfiles)
                .width(Length::Fill)
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: settings.border_radius,
                })))
        };

        sidebar_items = sidebar_items.push(download_button);
        sidebar_items = sidebar_items.push(Space::with_height(Length::Fixed(16.0)));
        
        // Add toggle for cfhdb profiles
        let cfhdb_toggle_text = if settings.show_cfhdb_profiles {
            row![
                text(glyphs::CHECK_SYMBOL).font(*material_font).size(icon_size * 0.9),
                text(" Show CFHDB Profiles").size(button_font_size * 0.9),
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        } else {
            row![
                text(" ").size(icon_size * 0.9),
                text(" Show CFHDB Profiles").size(button_font_size * 0.9),
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        };
        
        let cfhdb_toggle = button(cfhdb_toggle_text)
            .on_press(Message::ToggleCfhdbProfiles)
            .width(Length::Fill)
            .padding(Padding::from([12.0, 16.0, 12.0, 16.0]))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: settings.show_cfhdb_profiles,
                radius: settings.border_radius,
            })));
        
        sidebar_items = sidebar_items.push(cfhdb_toggle);
        sidebar_items = sidebar_items.push(Space::with_height(Length::Fixed(24.0)));
        sidebar_items = sidebar_items.push(
            container(
                text("PCI Devices")
                    .size(section_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
            )
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
            .width(Length::Fill)
        );

        for (class, _devices) in &self.pci_devices {
            let class_name = get_pci_class_name(class);
            let is_selected = self.selected_category.as_ref()
                .map(|(t, c)| *t == CategoryType::Pci && c == class)
                .unwrap_or(false);

            let class_button = button(
                row![
                    text(glyphs::SETTINGS_SYMBOL).font(*material_font).size(icon_size * 0.9),
                    text(&class_name).size(button_font_size),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .on_press(Message::SelectCategory(CategoryType::Pci, class.clone()))
            .width(Length::Fill)
            .padding(Padding::from([12.0, 16.0, 12.0, 16.0]))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: is_selected,
                radius: settings.border_radius,
            })));

            sidebar_items = sidebar_items.push(class_button);
        }

        sidebar_items = sidebar_items.push(Space::with_height(Length::Fixed(24.0)));
        sidebar_items = sidebar_items.push(
            container(
                text("USB Devices")
                    .size(section_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
            )
            .padding(Padding::from([8.0, 12.0, 8.0, 12.0]))
            .width(Length::Fill)
        );

        for (class, _devices) in &self.usb_devices {
            let class_name = get_usb_class_name(class);
            let is_selected = self.selected_category.as_ref()
                .map(|(t, c)| *t == CategoryType::Usb && c == class)
                .unwrap_or(false);

            let class_button = button(
                row![
                    text(glyphs::SETTINGS_SYMBOL).font(*material_font).size(icon_size * 0.9),
                    text(&class_name).size(button_font_size),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .on_press(Message::SelectCategory(CategoryType::Usb, class.clone()))
            .width(Length::Fill)
            .padding(Padding::from([12.0, 16.0, 12.0, 16.0]))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: is_selected,
                radius: settings.border_radius,
            })));

            sidebar_items = sidebar_items.push(class_button);
        }

        container(
            scrollable(
                container(sidebar_items)
                    .width(Length::Fill)
                    .padding(Padding::from([16.0, 12.0, 16.0, 12.0]))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Scrollable::Custom(Box::new(SidebarScrollableStyle {
                background_color: Color::from(settings.background_color.clone()),
                border_radius: settings.border_radius,
                _theme: *theme,
            })))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(SidebarStyle {
            radius: settings.border_radius,
        })))
        .padding(Padding::from([16.0, 12.0, 16.0, 12.0]))
        .into()
    }

    fn view_content(&self, theme: &crate::gui::Theme, material_font: &iced::Font, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        if let Some((cat_type, class)) = &self.selected_category {
            if let Some((dev_type, _, device_idx)) = &self.selected_device {

                self.view_device_details(theme, material_font, *dev_type, class, *device_idx, settings)
            } else {

                self.view_device_list(theme, material_font, *cat_type, class, settings)
            }
        } else {

            let title_font_size = (settings.font_size_titles * settings.scale_titles).round();
            let body_font_size = (settings.font_size_body * settings.scale_body).round();
            container(
                column![
                    text("Device Manager").size(title_font_size * 0.86).style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Select a device category from the sidebar to get started.")
                        .size(body_font_size)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        }
    }

    fn view_device_list(&self, theme: &crate::gui::Theme, _material_font: &iced::Font, cat_type: CategoryType, class: &str, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        match cat_type {
            CategoryType::Pci => {
                let devices: Vec<_> = self.pci_devices.iter()
                    .find(|(c, _)| c == class)
                    .map(|(_, devices)| devices.iter().enumerate().collect::<Vec<_>>())
                    .unwrap_or_default();

                if devices.is_empty() {
                    return container(
                        text("No devices found in this category")
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into();
                }

                let device_cards: Vec<Element<Message>> = devices.iter()
                    .map(|(idx, device)| {
                        let d = &device.device;
                        let name = format!("{} - {}", d.vendor_name, d.device_name);
                        let status = get_device_status(d);
                        let bus_id = &d.sysfs_busid;

                        let status_color = match status {
                            DeviceStatus::ActiveEnabled => theme.primary_with_settings(Some(settings)),
                            DeviceStatus::ActiveDisabled => iced::Color::from_rgb(0.2, 0.5, 0.9),
                            DeviceStatus::InactiveEnabled => iced::Color::from_rgb(0.2, 0.9, 0.2),
                            DeviceStatus::InactiveDisabled => theme.danger(),
                        };

                        button(
                            container(
                                column![
                                    row![
                                        text(&name).size(body_font_size * 1.25).width(Length::Fill),
                                        container(
                                            Space::with_width(Length::Fixed(14.0))
                                                .height(Length::Fixed(14.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                            color: status_color,
                                            radius: settings.border_radius,
                                        }))),
                                    ]
                                    .spacing(12)
                                    .width(Length::Fill)
                                    .align_items(Alignment::Center),
                                    Space::with_height(Length::Fixed(10.0)),
                                    text(format!("Bus ID: {}", bus_id))
                                        .size(body_font_size * 0.95)
                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                                ]
                                .spacing(6)
                                .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
                                .width(Length::Fill)
                            )
                            .style(iced::theme::Container::Custom(Box::new(DeviceCardStyle {
                                radius: settings.border_radius,
                            })))
                            .width(Length::Fill)
                        )
                        .on_press(Message::SelectDevice(DeviceType::Pci, class.to_string(), *idx))
                        .style(iced::theme::Button::Text)
                        .padding(0)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();

                container(
                    scrollable(
                        column(device_cards).spacing(10).padding(10)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                        Color::from(settings.background_color.clone()),
                        settings.border_radius,
                    ))))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            CategoryType::Usb => {
                let devices: Vec<_> = self.usb_devices.iter()
                    .find(|(c, _)| c == class)
                    .map(|(_, devices)| devices.iter().enumerate().collect::<Vec<_>>())
                    .unwrap_or_default();

                if devices.is_empty() {
                    return container(
                        text("No devices found in this category")
                            .size(body_font_size)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into();
                }

                let device_cards: Vec<Element<Message>> = devices.iter()
                    .map(|(idx, device)| {
                        let d = &device.device;
                        let name = format!("{} - {}", d.manufacturer_string_index, d.product_string_index);
                        let status = get_usb_device_status(d);
                        let bus_id = &d.sysfs_busid;

                        let status_color = match status {
                            DeviceStatus::ActiveEnabled => theme.primary_with_settings(Some(settings)),
                            DeviceStatus::ActiveDisabled => iced::Color::from_rgb(0.2, 0.5, 0.9),
                            DeviceStatus::InactiveEnabled => iced::Color::from_rgb(0.2, 0.9, 0.2),
                            DeviceStatus::InactiveDisabled => theme.danger(),
                        };

                        button(
                            container(
                                column![
                                    row![
                                        text(&name).size(body_font_size * 1.25).width(Length::Fill),
                                        container(
                                            Space::with_width(Length::Fixed(14.0))
                                                .height(Length::Fixed(14.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                            color: status_color,
                                            radius: settings.border_radius,
                                        }))),
                                    ]
                                    .spacing(12)
                                    .width(Length::Fill)
                                    .align_items(Alignment::Center),
                                    Space::with_height(Length::Fixed(10.0)),
                                    text(format!("Bus ID: {}", bus_id))
                                        .size(body_font_size * 0.95)
                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                                ]
                                .spacing(6)
                                .padding(Padding::from([18.0, 20.0, 18.0, 20.0]))
                                .width(Length::Fill)
                            )
                            .style(iced::theme::Container::Custom(Box::new(DeviceCardStyle {
                                radius: settings.border_radius,
                            })))
                            .width(Length::Fill)
                        )
                        .on_press(Message::SelectDevice(DeviceType::Usb, class.to_string(), *idx))
                        .style(iced::theme::Button::Text)
                        .padding(0)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();

                container(
                    scrollable(
                        column(device_cards).spacing(10).padding(10)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                        Color::from(settings.background_color.clone()),
                        settings.border_radius,
                    ))))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    fn view_device_details(&self, theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {

        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.2).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();
        use crate::gui::fonts::glyphs;
        let (device_name, device_info, profiles_pci, profiles_usb, status) = match dev_type {
            DeviceType::Pci => {
                if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == class) {
                    if let Some(device) = devices.get(device_idx) {
                        let d = &device.device;
                        let name = format!("{} - {}", d.vendor_name, d.device_name);
                        let driver_version = get_driver_version(&d.kernel_driver);
                        let info = DeviceInfo::Pci {
                            vendor_name: d.vendor_name.clone(),
                            device_name: d.device_name.clone(),
                            driver: d.kernel_driver.clone(),
                            driver_version,
                            bus_id: d.sysfs_busid.clone(),
                            vendor_id: d.vendor_id.clone(),
                            device_id: d.device_id.clone(),
                            started: d.started.unwrap_or(false),
                            enabled: d.enabled,
                        };
                        (name, info, Some(device.profiles.clone()), None, get_device_status(d))
                    } else {
                        return self.view_error("Device not found");
                    }
                } else {
                    return self.view_error("Device class not found");
                }
            }
            DeviceType::Usb => {
                if let Some((_, devices)) = self.usb_devices.iter().find(|(c, _)| c == class) {
                    if let Some(device) = devices.get(device_idx) {
                        let d = &device.device;
                        let name = format!("{} - {}", d.manufacturer_string_index, d.product_string_index);
                        let driver_version = get_driver_version(&d.kernel_driver);
                        let info = DeviceInfo::Usb {
                            manufacturer: d.manufacturer_string_index.clone(),
                            product: d.product_string_index.clone(),
                            driver: d.kernel_driver.clone(),
                            driver_version,
                            bus_id: d.sysfs_busid.clone(),
                            vendor_id: d.vendor_id.clone(),
                            product_id: d.product_id.clone(),
                            started: d.started.unwrap_or(false),
                            enabled: d.enabled,
                        };
                        (name, info, None, Some(device.profiles.clone()), get_usb_device_status(d))
                    } else {
                        return self.view_error("Device not found");
                    }
                } else {
                    return self.view_error("Device class not found");
                }
            }
        };
        let back_button = button(
            row![
                text(glyphs::CLOSE_SYMBOL).font(*material_font).size(icon_size * 0.85),
                text(" Back").size(button_font_size),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .on_press(Message::BackToDeviceList)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 20.0, 14.0, 20.0]));
        let device_title = text(&device_name)
            .size(title_font_size)
            .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
            .width(Length::Fill);
        let status_color = match status {
            DeviceStatus::ActiveEnabled => theme.primary_with_settings(Some(settings)),
            DeviceStatus::ActiveDisabled => iced::Color::from_rgb(0.2, 0.5, 0.9),
            DeviceStatus::InactiveEnabled => iced::Color::from_rgb(0.2, 0.9, 0.2),
            DeviceStatus::InactiveDisabled => theme.danger(),
        };

        let status_indicator = container(
            Space::with_width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0))
        )
        .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
            color: status_color,
            radius: settings.border_radius,
        })));
        let status_badges = self.view_status_badges(theme, &device_info, settings);
        let control_buttons = self.view_control_buttons(theme, material_font, dev_type, class, device_idx, &device_info, settings);
        let profiles_section = match (profiles_pci, profiles_usb) {
            (Some(p), None) => self.view_profiles_section_pci(theme, material_font, dev_type, class, device_idx, &p, settings),
            (None, Some(p)) => self.view_profiles_section_usb(theme, material_font, dev_type, class, device_idx, &p, settings),
            _ => self.view_error("No profiles available"),
        };

        container(
            scrollable(
                column![

                    container(
                        row![
                            back_button,
                            Space::with_width(Length::Fill),
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center)
                        .width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .padding(Padding::from([0.0, 0.0, 16.0, 0.0])),

                    Space::with_height(Length::Fixed(20.0)),
                    container(
                        row![
                            device_title,
                            Space::with_width(Length::Fixed(12.0)),
                            status_indicator,
                        ]
                        .spacing(0)
                        .align_items(Alignment::Center)
                        .width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .padding(Padding::from([0.0, 0.0, 16.0, 0.0])),

                    Space::with_height(Length::Fixed(20.0)),
                    container(
                        row![

                            container(status_badges)
                                .width(Length::FillPortion(2)),
                            Space::with_width(Length::Fixed(20.0)),

                            container(
                                control_buttons
                            )
                            .width(Length::FillPortion(1))
                            .center_x(),
                        ]
                        .spacing(0)
                        .align_items(Alignment::Start)
                        .width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .padding(Padding::from([0.0, 0.0, 16.0, 0.0])),

                    Space::with_height(Length::Fixed(24.0)),
                    container(profiles_section)
                        .width(Length::Fill)
                        .padding(Padding::from([0.0, 0.0, 16.0, 0.0])),
                ]
                .spacing(0)
                .width(Length::Fill)
                .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
            )
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle::new(
                Color::from(settings.background_color.clone()),
                settings.border_radius,
            ))))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_error(&self, msg: &str) -> Element<'_, Message> {
        let body_font_size = 14.0; // Default size for error messages
        container(
            text(msg).size(body_font_size)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn view_status_badges(&self, theme: &crate::gui::Theme, device_info: &DeviceInfo, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let _body_font_size = (settings.font_size_body * settings.scale_body).round();
        let mut badges = column![].spacing(6);

        match device_info {
            DeviceInfo::Pci { started, enabled, driver, driver_version, bus_id, vendor_id, device_id, .. } => {
                badges = badges.push(self.create_status_badge(theme, "Started", if *started { "Yes" } else { "No" }, *started, settings));
                badges = badges.push(self.create_status_badge(theme, "Enabled", if *enabled { "Yes" } else { "No" }, *enabled, settings));
                badges = badges.push(self.create_info_badge(theme, "Driver", driver, settings));
                if !driver_version.is_empty() {
                    badges = badges.push(self.create_info_badge(theme, "Driver Version", driver_version, settings));
                }
                badges = badges.push(self.create_info_badge(theme, "Bus ID", bus_id, settings));
                badges = badges.push(self.create_info_badge(theme, "Vendor ID", vendor_id, settings));
                badges = badges.push(self.create_info_badge(theme, "Device ID", device_id, settings));
            }
            DeviceInfo::Usb { started, enabled, driver, driver_version, bus_id, vendor_id, product_id, .. } => {
                badges = badges.push(self.create_status_badge(theme, "Started", if *started { "Yes" } else { "No" }, *started, settings));
                badges = badges.push(self.create_status_badge(theme, "Enabled", if *enabled { "Yes" } else { "No" }, *enabled, settings));
                badges = badges.push(self.create_info_badge(theme, "Driver", driver, settings));
                if !driver_version.is_empty() {
                    badges = badges.push(self.create_info_badge(theme, "Driver Version", driver_version, settings));
                }
                badges = badges.push(self.create_info_badge(theme, "Bus ID", bus_id, settings));
                badges = badges.push(self.create_info_badge(theme, "Vendor ID", vendor_id, settings));
                badges = badges.push(self.create_info_badge(theme, "Product ID", product_id, settings));
            }
        }

        container(
            badges
        )
        .width(Length::Fill)
        .padding(Padding::from([10.0, 12.0, 10.0, 12.0]))
        .style(iced::theme::Container::Custom(Box::new(StatusBadgeContainerStyle {
            radius: settings.border_radius,
        })))
        .into()
    }

    fn create_status_badge(&self, theme: &crate::gui::Theme, label: &str, value: &str, is_positive: bool, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        container(
            row![
                text(label)
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                    .width(Length::Fixed(140.0)),
                text(value)
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(if is_positive { theme.primary_with_settings(Some(settings)) } else { theme.danger() })),
            ]
            .spacing(12)
            .align_items(Alignment::Center)
            .width(Length::Fill)
        )
        .padding(Padding::from([12.0, 14.0, 12.0, 14.0]))
        .style(iced::theme::Container::Custom(Box::new(BadgeStyle {
            is_positive,
            radius: settings.border_radius,
        })))
        .width(Length::Fill)
        .into()
    }

    fn create_info_badge(&self, theme: &crate::gui::Theme, label: &str, value: &str, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        container(
            row![
                text(label)
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                    .width(Length::Fixed(140.0)),
                text(value)
                    .size(body_font_size),
            ]
            .spacing(12)
            .align_items(Alignment::Center)
            .width(Length::Fill)
        )
        .padding(Padding::from([12.0, 14.0, 12.0, 14.0]))
        .style(iced::theme::Container::Custom(Box::new(InfoBadgeStyle {
            radius: settings.border_radius,
        })))
        .width(Length::Fill)
        .into()
    }

    fn view_control_buttons(&self, _theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, device_info: &DeviceInfo, settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;

        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.4).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.1).round();
        let (_started, _enabled) = match device_info {
            DeviceInfo::Pci { started, enabled, .. } => (*started, *enabled),
            DeviceInfo::Usb { started, enabled, .. } => (*started, *enabled),
        };

        let start_button = button(
            column![
                text(glyphs::REFRESH_SYMBOL).font(*material_font).size(icon_size),
                text("Start").size(button_font_size * 0.85),
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::StartDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 16.0, 14.0, 16.0]))
        .width(Length::Fixed(80.0));

        let stop_button = button(
            column![
                text(glyphs::CLOSE_SYMBOL).font(*material_font).size(icon_size),
                text("Stop").size(button_font_size * 0.85),
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::StopDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 16.0, 14.0, 16.0]))
        .width(Length::Fixed(80.0));

        let enable_button = button(
            column![
                text(glyphs::CHECK_SYMBOL).font(*material_font).size(icon_size),
                text("Enable").size(button_font_size * 0.85),
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::EnableDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 16.0, 14.0, 16.0]))
        .width(Length::Fixed(80.0));

        let disable_button = button(
            column![
                text(glyphs::CANCEL_SYMBOL).font(*material_font).size(icon_size),
                text("Disable").size(button_font_size * 0.85),
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::DisableDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
            radius: settings.border_radius,
        })))
        .padding(Padding::from([14.0, 16.0, 14.0, 16.0]))
        .width(Length::Fixed(80.0));

        container(
            column![
                row![
                    start_button,
                    Space::with_width(Length::Fixed(10.0)),
                    stop_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
                Space::with_height(Length::Fixed(10.0)),
                row![
                    enable_button,
                    Space::with_width(Length::Fixed(10.0)),
                    disable_button,
                ]
                .spacing(0)
                .align_items(Alignment::Center),
            ]
            .spacing(0)
            .align_items(Alignment::Center)
        )
        .width(Length::Shrink)
        .center_x()
        .into()
    }

    fn view_profiles_section_pci(&self, theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, profiles: &[Arc<PreCheckedPciProfile>], settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();
        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.1).round();

        if profiles.is_empty() {
            return container(
                text("No profiles available for this device")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
            )
            .width(Length::Fill)
            .padding(20)
            .into();
        }

        let mut profile_cards = column![].spacing(10);
        let nvidia_vendor_id = "10de".to_string();
        let mut sorted_profiles: Vec<_> = profiles.iter().cloned().collect();
        sorted_profiles.sort_by(|a, b| {
            let a_profile = a.profile();
            let b_profile = b.profile();

            let a_is_nvidia = a_profile.vendor_ids.contains(&nvidia_vendor_id);
            let b_is_nvidia = b_profile.vendor_ids.contains(&nvidia_vendor_id);
            match (a_is_nvidia, b_is_nvidia) {
                (true, false) => std::cmp::Ordering::Less,  // NVIDIA before non-NVIDIA
                (false, true) => std::cmp::Ordering::Greater, // non-NVIDIA after NVIDIA
                (true, true) => {

                    b_profile.priority.cmp(&a_profile.priority)
                }
                (false, false) => {

                    b_profile.priority.cmp(&a_profile.priority)
                }
            }
        });
        let has_uninstalled = sorted_profiles.iter().any(|p| !p.installed());
        let selected_count = sorted_profiles.iter()
            .filter(|p| self.selected_profiles.contains(&p.profile().codename))
            .count();

        let install_selected_section: Element<Message> = if has_uninstalled && selected_count > 0 {
            container(
                row![
                    Space::with_width(Length::Fill),
                    button(
                        row![
                            text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(icon_size),
                            text(&format!("Install Selected ({})", selected_count)).size(button_font_size),
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center)
                    )
                    .on_press(Message::InstallSelectedProfiles(dev_type, class.to_string(), device_idx))
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: settings.border_radius,
                    })))
                    .padding(Padding::from([14.0, 20.0, 14.0, 20.0])),
                ]
                .spacing(0)
                .align_items(Alignment::Center)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .padding(Padding::from([0.0, 0.0, 0.0, 0.0]))
            .into()
        } else {
            Space::with_width(Length::Shrink).into()
        };

        for (_profile_idx, profile) in sorted_profiles.iter().enumerate() {
            let profile_data = profile.profile();
            let is_installed = profile.installed();
            let is_selected = self.selected_profiles.contains(&profile_data.codename);
            let codename_clone = profile_data.codename.clone();
            let class_clone = class.to_string();
            let checkbox: Element<Message> = if !is_installed {
                use iced::widget::checkbox;
                checkbox("", is_selected)
                    .on_toggle(move |_| {
                        Message::ToggleProfileSelection(dev_type, class_clone.clone(), device_idx, codename_clone.clone())
                    })
                    .size(body_font_size)
                    .spacing(8)
                    .into()
            } else {
                Space::with_width(Length::Fixed(24.0)).into()
            };

            let install_button = if is_installed {
                button(
                    row![
                        text(glyphs::CHECK_SYMBOL).font(*material_font).size(icon_size),
                        text(" Installed").size(button_font_size),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
            } else {
                button(
                    row![
                        text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(icon_size),
                        text(" Install").size(button_font_size),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::InstallProfile(dev_type, class.to_string(), device_idx, profile_data.codename.clone()))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
            };

            let remove_button: Element<Message> = if profile_data.removable && is_installed {
                button(
                    row![
                        text(glyphs::DELETE_SYMBOL).font(*material_font).size(icon_size),
                        text(" Remove").size(button_font_size),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::RemoveProfile(dev_type, class.to_string(), device_idx, profile_data.codename.clone()))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
                .into()
            } else {
                Space::with_width(Length::Shrink).into()
            };

            let experimental_badge: Element<Message> = if profile_data.experimental {
                container(
                    text("Experimental")
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.danger()))
                )
                .padding(Padding::from([6.0, 10.0, 6.0, 10.0]))
                .style(iced::theme::Container::Custom(Box::new(ExperimentalBadgeStyle {
                    radius: settings.border_radius,
                })))
                .into()
            } else {
                Space::with_width(Length::Shrink).into()
            };
            let display_name = if profile_data.i18n_desc.contains("NVIDIA") {
                profile_data.i18n_desc.clone()
            } else if profile_data.codename.starts_with("nvidia-") {

                if let Some(version) = profile_data.codename.strip_prefix("nvidia-") {
                    format!("NVIDIA Graphics Driver {}", version)
                } else {
                    profile_data.i18n_desc.clone()
                }
            } else {
                profile_data.i18n_desc.clone()
            };

            let profile_card = container(
                column![
                    row![
                        checkbox,
                        Space::with_width(Length::Fixed(14.0)),
                        text(&display_name)
                            .size(body_font_size * 1.25)
                            .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                            .width(Length::Fill),
                        Space::with_width(Length::Fixed(14.0)),
                        {
                            let indicator: Element<Message> = if is_installed {
                                container(
                                    Space::with_width(Length::Fixed(12.0))
                                        .height(Length::Fixed(12.0))
                                )
                                .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                    color: theme.primary_with_settings(Some(settings)),
                                    radius: settings.border_radius,
                                })))
                                .into()
                            } else {
                                Space::with_width(Length::Shrink).into()
                            };
                            indicator
                        },
                    ]
                    .spacing(0)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(10.0)),
                    text(&profile_data.codename)
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                    {

                        let driver_version_display = if let Some(driver_version) = profile.driver_version() {
                            if !driver_version.is_empty() {

                                let cleaned = if profile_data.codename.starts_with("nvidia-") {

                                    if let Some(version_from_codename) = profile_data.codename.strip_prefix("nvidia-") {
                                        let clean: String = version_from_codename
                                            .chars()
                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                            .collect();
                                        if !clean.is_empty() && clean.matches('.').count() >= 1 {
                                            clean
                                        } else {

                                            if driver_version.contains("580") && driver_version.contains("390") {

                                                if profile_data.codename.contains("580") {

                                                    if let Some(start) = driver_version.find("580") {
                                                        let rest = &driver_version[start..];
                                                        let clean: String = rest
                                                            .chars()
                                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                            .collect();
                                                        if !clean.is_empty() {
                                                            clean
                                                        } else {
                                                            driver_version
                                                        }
                                                    } else {
                                                        driver_version
                                                    }
                                                } else if profile_data.codename.contains("390") {

                                                    if let Some(start) = driver_version.find("390") {
                                                        let rest = &driver_version[start..];
                                                        let clean: String = rest
                                                            .chars()
                                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                            .collect();
                                                        if !clean.is_empty() {
                                                            clean
                                                        } else {
                                                            driver_version
                                                        }
                                                    } else {
                                                        driver_version
                                                    }
                                                } else {
                                                    driver_version
                                                }
                                            } else {

                                                driver_version
                                                    .chars()
                                                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                    .collect()
                                            }
                                        }
                                    } else {

                                        if driver_version.contains("580") && driver_version.contains("390") {

                                            if profile_data.codename.contains("580") {

                                                if let Some(start) = driver_version.find("580") {
                                                    let rest = &driver_version[start..];
                                                    let clean: String = rest
                                                        .chars()
                                                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                        .collect();
                                                    if !clean.is_empty() {
                                                        clean
                                                    } else {
                                                        driver_version
                                                    }
                                                } else {
                                                    driver_version
                                                }
                                            } else if profile_data.codename.contains("390") {

                                                if let Some(start) = driver_version.find("390") {
                                                    let rest = &driver_version[start..];
                                                    let clean: String = rest
                                                        .chars()
                                                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                        .collect();
                                                    if !clean.is_empty() {
                                                        clean
                                                    } else {
                                                        driver_version
                                                    }
                                                } else {
                                                    driver_version
                                                }
                                            } else {
                                                driver_version
                                            }
                                        } else {

                                            driver_version
                                                .chars()
                                                .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                .collect()
                                        }
                                    }
                                } else if profile_data.codename.starts_with("mesa-") {

                                    if let Some(version_from_codename) = profile_data.codename.strip_prefix("mesa-") {
                                        let clean: String = version_from_codename
                                            .chars()
                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                            .collect();
                                        if !clean.is_empty() && clean.matches('.').count() >= 1 {
                                            clean
                                        } else {

                                            driver_version
                                                .chars()
                                                .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                .collect()
                                        }
                                    } else {

                                        driver_version
                                            .chars()
                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                            .collect()
                                    }
                                } else {

                                    driver_version
                                        .chars()
                                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                                        .collect()
                                };
                                Some(cleaned)
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let mut info_rows = column![].spacing(4);

                        if let Some(drv_ver) = driver_version_display {
                            info_rows = info_rows.push(
                                text(format!("Driver Version: {}", drv_ver))
                                    .size(body_font_size * 0.95)
                                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                            );
                        }
                        if let Some(repo) = profile.repository() {
                            info_rows = info_rows.push(
                                text(format!("Repository: {}", repo))
                                    .size(body_font_size * 0.95)
                                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                            );
                        }
                        if let Some(size) = profile.package_size() {
                            info_rows = info_rows.push(
                                text(format!("Total Size: {}", size))
                                    .size(body_font_size * 0.95)
                                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                            );
                        }
                        if let Some(deps) = profile.dependencies() {
                            if !deps.is_empty() {
                                info_rows = info_rows.push(
                                    text(format!("Dependencies: {} packages", deps.len()))
                                        .size(body_font_size * 0.95)
                                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
                                );
                            }
                        }

                        info_rows.width(Length::Fill)
                    },
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        text(format!("License: {}", profile_data.license))
                            .size(body_font_size * 0.95)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                        Space::with_width(Length::Fill),
                        experimental_badge,
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    {
                        let mut button_row = row![install_button];
                        if profile_data.removable && is_installed {
                            button_row = button_row.push(Space::with_width(Length::Fixed(12.0)));
                            button_row = button_row.push(remove_button);
                        }
                        button_row
                            .spacing(0)
                            .align_items(Alignment::Center)
                            .width(Length::Fill)
                    },
                ]
                .spacing(8)
                .padding(Padding::from([20.0, 24.0, 20.0, 24.0]))
                .width(Length::Fill)
            )
            .style(iced::theme::Container::Custom(Box::new(ProfileCardStyle {
                radius: settings.border_radius,
            })))
            .width(Length::Fill);

            profile_cards = profile_cards.push(profile_card);
        }

        container(
            column![
                row![
                    text("Available Profiles")
                        .size(title_font_size)
                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                        .width(Length::Fill),
                    install_selected_section,
                ]
                .spacing(0)
                .align_items(Alignment::Center)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(20.0)),
                profile_cards,
            ]
            .spacing(0)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .into()
    }

    fn view_profiles_section_usb(&self, theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, profiles: &[Arc<PreCheckedUsbProfile>], settings: &crate::gui::settings::AppSettings) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        let body_font_size = (settings.font_size_body * settings.scale_body * 1.15).round();
        let button_font_size = (settings.font_size_buttons * settings.scale_buttons * 1.2).round();
        let icon_size = (settings.font_size_icons * settings.scale_icons * 1.3).round();

        if profiles.is_empty() {
            return container(
                text("No profiles available for this device")
                    .size(body_font_size)
                    .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings))))
            )
            .width(Length::Fill)
            .padding(20)
            .into();
        }

        let mut profile_cards = column![].spacing(10);

        let mut sorted_profiles: Vec<_> = profiles.iter().cloned().collect();
        sorted_profiles.sort_by_key(|p| p.profile().priority);

        for (_profile_idx, profile) in sorted_profiles.iter().enumerate() {
            let profile_data = profile.profile();
            let is_installed = profile.installed();
            let is_selected = self.selected_profiles.contains(&profile_data.codename);
            let codename_clone = profile_data.codename.clone();
            let class_clone = class.to_string();
            let checkbox: Element<Message> = if !is_installed {
                use iced::widget::checkbox;
                checkbox("", is_selected)
                    .on_toggle(move |_| {
                        Message::ToggleProfileSelection(dev_type, class_clone.clone(), device_idx, codename_clone.clone())
                    })
                    .size(body_font_size)
                    .spacing(8)
                    .into()
            } else {
                Space::with_width(Length::Fixed(24.0)).into()
            };

            let install_button = if is_installed {
                button(
                    row![
                        text(glyphs::CHECK_SYMBOL).font(*material_font).size(icon_size),
                        text(" Installed").size(button_font_size),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
            } else {
                button(
                    row![
                        text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(icon_size),
                        text(" Install").size(button_font_size),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::InstallProfile(dev_type, class.to_string(), device_idx, profile_data.codename.clone()))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
            };

            let remove_button: Element<Message> = if profile_data.removable && is_installed {
                button(
                    row![
                        text(glyphs::DELETE_SYMBOL).font(*material_font).size(icon_size),
                        text(" Remove").size(button_font_size),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::RemoveProfile(dev_type, class.to_string(), device_idx, profile_data.codename.clone()))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: settings.border_radius,
                })))
                .padding(Padding::from([14.0, 18.0, 14.0, 18.0]))
                .into()
            } else {
                Space::with_width(Length::Shrink).into()
            };

            let experimental_badge: Element<Message> = if profile_data.experimental {
                container(
                    text("Experimental")
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.danger()))
                )
                .padding(Padding::from([6.0, 10.0, 6.0, 10.0]))
                .style(iced::theme::Container::Custom(Box::new(ExperimentalBadgeStyle {
                    radius: settings.border_radius,
                })))
                .into()
            } else {
                Space::with_width(Length::Shrink).into()
            };
            let display_name = if profile_data.i18n_desc.contains("NVIDIA") {
                profile_data.i18n_desc.clone()
            } else if profile_data.codename.starts_with("nvidia-") {

                if let Some(version) = profile_data.codename.strip_prefix("nvidia-") {
                    format!("NVIDIA Graphics Driver {}", version)
                } else {
                    profile_data.i18n_desc.clone()
                }
            } else {
                profile_data.i18n_desc.clone()
            };

            let profile_card = container(
                column![
                    row![
                        checkbox,
                        Space::with_width(Length::Fixed(14.0)),
                        text(&display_name)
                            .size(body_font_size * 1.25)
                            .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                            .width(Length::Fill),
                        Space::with_width(Length::Fixed(14.0)),
                        {
                            let indicator: Element<Message> = if is_installed {
                                container(
                                    Space::with_width(Length::Fixed(12.0))
                                        .height(Length::Fixed(12.0))
                                )
                                .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                    color: theme.primary_with_settings(Some(settings)),
                                    radius: settings.border_radius,
                                })))
                                .into()
                            } else {
                                Space::with_width(Length::Shrink).into()
                            };
                            indicator
                        },
                    ]
                    .spacing(0)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(10.0)),
                    text(&profile_data.codename)
                        .size(body_font_size * 0.9)
                        .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                    {

                        if let Some(driver_version) = profile.driver_version() {
                            if !driver_version.is_empty() {
                                row![
                                    text(format!("Driver Version: {}", driver_version))
                                        .size(body_font_size * 0.79)
                                        .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings)))),
                                ]
                                .spacing(8)
                                .align_items(Alignment::Center)
                                .width(Length::Fill)
                            } else {
                                row![Space::with_width(Length::Shrink)]
                            }
                        } else {
                            row![Space::with_width(Length::Shrink)]
                        }
                    },
                    Space::with_height(Length::Fixed(12.0)),
                    row![
                        text(format!("License: {}", profile_data.license))
                            .size(body_font_size * 0.95)
                            .style(iced::theme::Text::Color(theme.secondary_text_with_settings(Some(settings)))),
                        Space::with_width(Length::Fill),
                        experimental_badge,
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(16.0)),
                    row![
                        install_button,
                        remove_button,
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ]
                .spacing(4)
                .padding(16)
                .width(Length::Fill)
            )
            .style(iced::theme::Container::Custom(Box::new(ProfileCardStyle {
                radius: settings.border_radius,
            })))
            .width(Length::Fill);

            profile_cards = profile_cards.push(profile_card);
        }

        let title_font_size = (settings.font_size_titles * settings.scale_titles * 1.1).round();

        container(
            column![
                text("Available Profiles")
                    .size(title_font_size)
                    .style(iced::theme::Text::Color(theme.primary_with_settings(Some(settings))))
                    .width(Length::Fill),
                Space::with_height(Length::Fixed(10.0)),
                profile_cards,
            ]
            .spacing(10)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .into()
    }
}

async fn load_profile_versions(pci_profiles: Vec<Arc<PreCheckedPciProfile>>, usb_profiles: Vec<Arc<PreCheckedUsbProfile>>) -> () {
    use tokio::task;
    let pci_handles: Vec<_> = pci_profiles.into_iter().map(|profile| {
        let profile_clone = profile.clone();
        task::spawn_blocking(move || {

            profile_clone.update_installed();

            let version = extract_driver_version_sync(&profile_clone.profile());
            profile_clone.set_driver_version(Some(version));
        })
    }).collect();
    let usb_handles: Vec<_> = usb_profiles.into_iter().map(|profile| {
        let profile_clone = profile.clone();
        task::spawn_blocking(move || {

            profile_clone.update_installed();

            let version = extract_driver_version_sync_usb(&profile_clone.profile());
            profile_clone.set_driver_version(Some(version));
        })
    }).collect();
    for handle in pci_handles {
        let _ = handle.await;
    }
    for handle in usb_handles {
        let _ = handle.await;
    }
}
fn extract_driver_version_sync(profile: &CfhdbPciProfile) -> String {

    if profile.vendor_ids.contains(&"10de".to_string()) && profile.priority == 100 {

        if let Some(version_part) = profile.codename.strip_prefix("nvidia-") {

            if version_part.matches('.').count() >= 1 && version_part.chars().any(|c| c.is_ascii_digit()) {

                let clean_version: String = version_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if !clean_version.is_empty() {
                    return clean_version;
                }
            }
        }

        if profile.i18n_desc.contains("Driver ") {

            let parts: Vec<&str> = profile.i18n_desc.split("Driver ").collect();
            if parts.len() > 1 {
                let version_part = parts[1].trim();

                let clean_version: String = version_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();

                if clean_version.matches('.').count() >= 1 && clean_version.chars().any(|c| c.is_ascii_digit()) {
                    return clean_version;
                }
            }
        }
    }
    if profile.codename.starts_with("mesa-") && profile.priority == 90 {

        if let Some(version_part) = profile.codename.strip_prefix("mesa-") {

            let clean_version: String = version_part
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                return clean_version;
            }
        }

        if profile.i18n_desc.contains("Driver ") {
            let parts: Vec<&str> = profile.i18n_desc.split("Driver ").collect();
            if parts.len() > 1 {
                let version_part = parts[1].trim();
                let clean_version: String = version_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if clean_version.matches('.').count() >= 1 && clean_version.chars().any(|c| c.is_ascii_digit()) {
                    return clean_version;
                }
            }
        }
    }
    let mut version = extract_driver_version_from_packages_fast(&profile.packages);
    if version.is_empty() {
        version = extract_driver_version_from_install_script_fast(&profile.install_script);
    }
    version
}

fn extract_driver_version_sync_usb(profile: &CfhdbUsbProfile) -> String {
    let mut version = extract_driver_version_from_packages_fast(&profile.packages);
    if version.is_empty() {
        version = extract_driver_version_from_install_script_fast(&profile.install_script);
    }
    version
}

fn extract_driver_version_from_packages_fast(packages: &Option<Vec<String>>) -> String {
    if let Some(pkgs) = packages {
        for pkg in pkgs {

            if pkg.contains("nvidia-driver") {
                if let Some(version_part) = pkg.strip_prefix("nvidia-driver-") {
                    if version_part.chars().any(|c| c.is_ascii_digit()) {
                        if version_part.contains('.') && version_part.matches('.').count() >= 2 {
                            return version_part.to_string();
                        }
                        if let Some(major_version) = version_part.split('.').next() {
                            if major_version.chars().all(|c| c.is_ascii_digit()) {

                                return major_version.to_string();
                            }
                        }
                    }
                }
            }
            if pkg.contains("akmod-nvidia") {

                if let Some(version_part) = pkg.strip_prefix("akmod-nvidia-") {
                    if version_part.chars().any(|c| c.is_ascii_digit()) {
                        if version_part.contains('.') && version_part.matches('.').count() >= 2 {
                            return version_part.to_string();
                        }
                        if let Some(major_version) = version_part.split('.').next() {
                            if major_version.chars().all(|c| c.is_ascii_digit()) {
                                return major_version.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    String::new()
}

fn extract_driver_version_from_install_script_fast(install_script: &Option<String>) -> String {
    if let Some(script) = install_script {
        for line in script.lines() {
            if line.contains("dnf install") && line.contains("nvidia-driver") {
                let words: Vec<&str> = line.split_whitespace().collect();
                for word in &words {
                    if word.starts_with("nvidia-driver") {
                        if let Some(version_part) = word.strip_prefix("nvidia-driver-") {
                            if version_part.chars().any(|c| c.is_ascii_digit()) {
                                if version_part.contains('.') && version_part.matches('.').count() >= 2 {
                                    return version_part.to_string();
                                }
                                if let Some(major_version) = version_part.split('.').next() {
                                    if major_version.chars().all(|c| c.is_ascii_digit()) {

                                        return major_version.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    String::new()
}
#[allow(dead_code)]
fn get_package_version(package_name: &str) -> Result<String, ()> {
    let pkg = package_name.to_string();

    let handle = std::thread::spawn(move || {
        if let Ok(output) = std::process::Command::new("dnf")
            .args(["repoquery", "--qf", "%{VERSION}", "--whatprovides", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(versions) = String::from_utf8(output.stdout) {

                    if let Some(first_line) = versions.lines().next() {
                        let version = first_line.trim();
                        if !version.is_empty() && version.contains('.') {

                            let version = version.split('-').next().unwrap_or(version);
                            if version.chars().any(|c| c.is_ascii_digit()) {
                                return Ok(version.to_string());
                            }
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("dnf")
            .args(["list", "--available", "--quiet", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(info) = String::from_utf8(output.stdout) {
                    for line in info.lines() {

                        if line.contains("Available Packages") || line.contains("Installed Packages") || line.contains("Last metadata") {
                            continue;
                        }

                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {

                            let version_release = parts[1];

                            if let Some(version) = version_release.split('-').next() {
                                if version.contains('.') && version.chars().any(|c| c.is_ascii_digit()) {
                                    return Ok(version.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("dnf")
            .args(["info", "--available", "--quiet", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(info) = String::from_utf8(output.stdout) {
                    for line in info.lines() {
                        if line.starts_with("Version") {
                            if let Some(version) = line.split_whitespace().nth(1) {
                                let version = version.split('-').next().unwrap_or(version);

                                if version.contains('.') && version.chars().any(|c| c.is_ascii_digit()) {
                                    return Ok(version.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("rpm")
            .args(["-q", "--qf", "%{VERSION}", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(version) = String::from_utf8(output.stdout) {
                    let version = version.trim();
                    if !version.is_empty() && version != pkg {
                        return Ok(version.to_string());
                    }
                }
            }
        }

        Err(())
    });

    handle.join().unwrap_or(Err(()))
}
fn query_package_info(profile: &PreCheckedPciProfile, package_names: &[String]) {
    use std::process::Command;
    use std::collections::HashSet;

    if package_names.is_empty() {
        return;
    }
    let mut total_size = 0u64;

    if let Ok(output) = Command::new("dnf")
        .args(&["repoquery", "--available", "--quiet", "--qf", "%{name}|%{SIZE}"])
        .args(package_names)
        .output()
    {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 2 {
                        if let Ok(size) = parts[1].trim().parse::<u64>() {
                            total_size += size;
                        }
                    }
                }
            }
        }
    }
    let mut all_deps = HashSet::new();

    if let Ok(output) = Command::new("dnf")
        .args(&["repoquery", "--available", "--quiet", "--requires", "--resolve"])
        .args(package_names)
        .output()
    {
        if output.status.success() {
            if let Ok(deps_str) = String::from_utf8(output.stdout) {
                for dep in deps_str.lines() {
                    let dep = dep.trim();
                    if !dep.is_empty() {
                        all_deps.insert(dep.to_string());
                    }
                }
            }
        }
    }
    let size_str = if total_size > 0 {
        if total_size > 1024 * 1024 * 1024 {
            format!("{:.2} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if total_size > 1024 * 1024 {
            format!("{:.2} MB", total_size as f64 / (1024.0 * 1024.0))
        } else if total_size > 1024 {
            format!("{:.2} KB", total_size as f64 / 1024.0)
        } else {
            format!("{} B", total_size)
        }
    } else {
        "Unknown".to_string()
    };

    profile.set_package_size(Some(size_str));
    profile.set_dependencies(Some(all_deps.into_iter().collect()));
}
fn extract_repositories_from_script(install_script: &Option<String>) -> Vec<String> {
    let mut repos = Vec::new();
    if let Some(script) = install_script {
        for line in script.lines() {

            if line.contains("dnf config-manager") && line.contains("setopt") {

                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in &parts {
                    if part.contains(".enabled=") {

                        if let Some(repo_name) = part.split(".enabled=").next() {
                            let enabled = part.contains("=1");
                            if enabled {
                                repos.push(repo_name.to_string());
                            }
                        }
                    }
                }
            }

            if line.contains("dnf install") && line.contains("--enablerepo") {

                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if *part == "--enablerepo" && i + 1 < parts.len() {
                        repos.push(parts[i + 1].to_string());
                    }
                }
            }
        }
    }

    repos.sort();
    repos.dedup();
    repos
}
fn get_driver_version(driver: &str) -> String {
    if driver.is_empty() || driver == "none" {
        return String::new();
    }

    if driver == "nvidia" {
        if let Ok(version) = std::fs::read_to_string("/proc/driver/nvidia/version") {
            if let Some(line) = version.lines().next() {

                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in &parts {

                    if part.contains('.') && part.chars().all(|c| c.is_ascii_digit() || c == '.') {

                        if part.matches('.').count() >= 1 {
                            return part.to_string();
                        }
                    }
                }
            }
        }
    }
    if let Ok(output) = std::process::Command::new("modinfo")
        .arg(driver)
        .arg("-F")
        .arg("version")
        .output()
    {
        if output.status.success() {
            if let Ok(version) = String::from_utf8(output.stdout) {
                let version = version.trim();
                if !version.is_empty() {
                    return version.to_string();
                }
            }
        }
    }
    if let Ok(output) = std::process::Command::new("modinfo")
        .arg(driver)
        .arg("-F")
        .arg("vermagic")
        .output()
    {
        if output.status.success() {
            if let Ok(vermagic) = String::from_utf8(output.stdout) {
                let vermagic = vermagic.trim();
                if !vermagic.is_empty() {

                    if let Some(version) = vermagic.split_whitespace().next() {
                        return version.to_string();
                    }
                }
            }
        }
    }

    String::new()
}

fn get_pci_class_name(class: &str) -> String {

    match class {
        "0300" => "VGA Compatible Controller".to_string(),
        "0200" => "Ethernet Controller".to_string(),
        "0403" => "Audio Device".to_string(),
        "0c03" => "USB Controller".to_string(),
        _ => format!("PCI Class {}", class),
    }
}

fn get_usb_class_name(class: &str) -> String {

    match class {
        "01" => "Audio".to_string(),
        "02" => "Communications".to_string(),
        "03" => "HID (Human Interface Device)".to_string(),
        "08" => "Mass Storage".to_string(),
        "0e" => "Video".to_string(),
        _ => format!("USB Class {}", class),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceStatus {
    ActiveEnabled,
    ActiveDisabled,
    InactiveEnabled,
    InactiveDisabled,
}

fn get_device_status(device: &CfhdbPciDevice) -> DeviceStatus {
    let started = device.started.unwrap_or(false);
    let enabled = device.enabled;
    match (enabled, started) {
        (true, true) => DeviceStatus::ActiveEnabled,
        (false, true) => DeviceStatus::ActiveDisabled,
        (true, false) => DeviceStatus::InactiveEnabled,
        (false, false) => DeviceStatus::InactiveDisabled,
    }
}

fn get_usb_device_status(device: &CfhdbUsbDevice) -> DeviceStatus {
    let started = device.started.unwrap_or(false);
    let enabled = device.enabled;
    match (enabled, started) {
        (true, true) => DeviceStatus::ActiveEnabled,
        (false, true) => DeviceStatus::ActiveDisabled,
        (true, false) => DeviceStatus::InactiveEnabled,
        (false, false) => DeviceStatus::InactiveDisabled,
    }
}

impl From<CategoryType> for DeviceType {
    fn from(cat: CategoryType) -> Self {
        match cat {
            CategoryType::Pci => DeviceType::Pci,
            CategoryType::Usb => DeviceType::Usb,
        }
    }
}
async fn query_nvidia_driver_packages() -> Result<Vec<NvidiaDriverPackage>, String> {
    use std::process::Command;
    use tokio::task;
    let result = task::spawn_blocking(move || {
        use rayon::prelude::*;

        let patterns = vec!["nvidia-driver*", "akmod-nvidia*", "xorg-x11-drv-nvidia*"];
        let all_packages: Vec<String> = patterns
            .par_iter()
            .flat_map(|pattern| {

                let mut results = Vec::new();
                if let Ok(output) = Command::new("dnf")
                    .args(&["repoquery", "--available", "--quiet", "--qf", "%{name}|%{version}|%{repoid}", "--enablerepo=rpmfusion-free*"])
                    .arg(pattern)
                    .output()
                {
                    if output.status.success() {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            results.extend(stdout.lines()
                                .map(|l| l.trim().to_string())
                                .filter(|l| !l.is_empty()));
                        }
                    }
                }

                let rpmfusion_nonfree_repos = vec![
                    "rpmfusion-nonfree",
                    "rpmfusion-nonfree-updates",
                    "rpmfusion-nonfree-nvidia-driver",
                ];

                for repo in rpmfusion_nonfree_repos {
                    if let Ok(output) = Command::new("dnf")
                        .args(&["repoquery", "--available", "--quiet", "--qf", "%{name}|%{version}|%{repoid}", "--enablerepo", repo])
                        .arg(pattern)
                        .output()
                    {
                        if output.status.success() {
                            if let Ok(stdout) = String::from_utf8(output.stdout) {
                                results.extend(stdout.lines()
                                    .map(|l| l.trim().to_string())
                                    .filter(|l| !l.is_empty()));
                            }
                        }
                    }
                }
                if let Ok(output) = Command::new("dnf")
                    .args(&["repoquery", "--available", "--quiet", "--qf", "%{name}|%{version}|%{repoid}"])
                    .arg(pattern)
                    .output()
                {
                    if output.status.success() {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            results.extend(stdout.lines()
                                .map(|l| l.trim().to_string())
                                .filter(|l| !l.is_empty() && (l.contains("negativo17") || l.contains("negativo") || l.contains("fedora-nvidia"))));
                        }
                    }
                }

                results
            })
            .collect();

        let mut packages: Vec<NvidiaDriverPackage> = Vec::new();
        let all_text = all_packages.join("");

        let package_patterns = vec!["akmod-nvidia", "nvidia-driver", "xorg-x11-drv-nvidia"];
        let mut positions = Vec::new();
        for pattern in &package_patterns {
            let mut search_pos = 0;
            while let Some(pos) = all_text[search_pos..].find(pattern) {
                let absolute_pos = search_pos + pos;

                if absolute_pos == 0 || all_text.chars().nth(absolute_pos - 1) == Some('|') {
                    positions.push(absolute_pos);
                }
                search_pos = absolute_pos + 1;
            }
        }
        positions.sort();
        positions.dedup();
        for i in 0..positions.len() {
            let start = positions[i];
            let end = if i + 1 < positions.len() {
                positions[i + 1]
            } else {
                all_text.len()
            };

            let entry = &all_text[start..end];
            let parts: Vec<&str> = entry.split('|').collect();

            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let version = parts[1].to_string();

                let repo_full = parts[2];

                let repo = if let Some(next_pkg_pos) = package_patterns.iter()
                    .filter_map(|pattern| repo_full.find(pattern))
                    .min()
                {
                    repo_full[..next_pkg_pos].to_string()
                } else {
                    repo_full.to_string()
                };

                let is_rpmfusion_free = repo.contains("rpmfusion-free");
                let is_rpmfusion_nonfree = repo.contains("rpmfusion-nonfree");
                let is_negativo17 = repo.contains("negativo17") || repo.contains("negativo") || repo.contains("fedora-nvidia");

                if is_rpmfusion_free || is_rpmfusion_nonfree || is_negativo17 {

                    let driver_version = extract_nvidia_version(&name, &version);

                    packages.push(NvidiaDriverPackage {
                        name,
                        version,
                        repo,
                        driver_version,
                    });
                }
            }
        }

        Ok(packages)
    })
    .await
    .map_err(|_| "Task join error".to_string())?;

    result
}

#[derive(Debug, Clone)]
struct NvidiaDriverPackage {
    name: String,
    #[allow(dead_code)]
    version: String,
    repo: String,
    driver_version: String,
}

#[derive(Debug, Clone)]
struct MesaDriverPackage {
    name: String,
    #[allow(dead_code)]
    version: String,
    repo: String,
    driver_version: String,
}

fn extract_nvidia_version(package_name: &str, package_version: &str) -> String {
    if package_name.starts_with("akmod-nvidia-") {

        if let Some(suffix) = package_name.strip_prefix("akmod-nvidia-") {

            if suffix.ends_with("xx") {

                if let Some(major_version) = suffix.strip_suffix("xx") {

                    if let Some(version) = package_version.split('-').next() {

                        if version.starts_with(&format!("{}.", major_version)) {
                            return version.to_string();
                        }

                        if version.matches('.').count() >= 1 && version.chars().any(|c| c.is_ascii_digit()) {
                            return version.to_string();
                        }
                    }

                    return major_version.to_string();
                }
            }
        }
    }
    if package_name == "akmod-nvidia" || (package_name.contains("akmod-nvidia") && !package_name.contains("-")) {
        if let Some(version) = package_version.split('-').next() {

            if version.matches('.').count() >= 2 && version.chars().any(|c| c.is_ascii_digit()) {
                return version.to_string();
            }
        }
    }
    if let Some(version_part) = package_name.strip_prefix("nvidia-driver-") {

        if let Ok(major_version) = version_part.parse::<u32>() {
            if let Ok(output) = std::process::Command::new("dnf")
                .args(&["repoquery", "--available", "--quiet", "--qf", "%{VERSION}"])
                .arg("akmod-nvidia")
                .output()
            {
                if output.status.success() {
                    if let Ok(versions) = String::from_utf8(output.stdout) {

                        for line in versions.lines() {
                            let version = line.trim();
                            if let Some(version_part) = version.split('-').next() {

                                if version_part.starts_with(&format!("{}.", major_version)) {

                                    if version_part.matches('.').count() >= 2 {
                                        return version_part.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            return version_part.to_string();
        }
    }
    if package_name.contains("xorg-x11-drv-nvidia") {
        if let Some(version) = package_version.split('-').next() {
            if version.matches('.').count() >= 2 {
                return version.to_string();
            }
        }
    }
    package_version.split('-').next().unwrap_or(package_version).to_string()
}
fn extract_mesa_version(package_name: &str, package_version: &str) -> String {
    if package_name.contains("mesa-dri-drivers") || package_name.contains("mesa-libGL") || package_name.contains("mesa-vulkan-drivers") {

        if let Some(version) = package_version.split('-').next() {

            if version.matches('.').count() >= 1 && version.chars().any(|c| c.is_ascii_digit()) {
                return version.to_string();
            }
        }
    }
    package_version.split('-').next().unwrap_or(package_version).to_string()
}
async fn query_mesa_driver_packages() -> Result<Vec<MesaDriverPackage>, String> {
    use std::process::Command;
    use tokio::task;
    let result = task::spawn_blocking(move || {
        use rayon::prelude::*;

        let patterns = vec!["mesa-dri-drivers", "mesa-libGL", "mesa-vulkan-drivers"];
        let all_packages: Vec<String> = patterns
            .par_iter()
            .flat_map(|pattern| {
                if let Ok(output) = Command::new("dnf")
                    .args(&["repoquery", "--available", "--quiet", "--qf", "%{name}|%{version}|%{repoid}"])
                    .arg(pattern)
                    .output()
                {
                    if output.status.success() {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            return stdout.lines()
                                .map(|l| l.trim().to_string())
                                .filter(|l| !l.is_empty())
                                .collect::<Vec<String>>();
                        }
                    }
                }
                Vec::new()
            })
            .collect();
        let packages: Vec<MesaDriverPackage> = all_packages
            .into_iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    let repo = parts[2].to_string();

                    let driver_version = extract_mesa_version(&name, &version);

                    Some(MesaDriverPackage {
                        name,
                        version,
                        repo,
                        driver_version,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    })
    .await
    .map_err(|_| "Task join error".to_string())?;

    result
}

fn create_mesa_profiles_from_repos(packages: Vec<MesaDriverPackage>) -> Vec<(CfhdbPciProfile, String, Vec<String>)> {
    let mut profiles = Vec::new();
    let mut version_groups: HashMap<String, Vec<MesaDriverPackage>> = HashMap::new();
    for pkg in packages {
        version_groups.entry(pkg.driver_version.clone())
            .or_insert_with(Vec::new)
            .push(pkg);
    }
    for (driver_version, pkgs) in version_groups {

        let repo = pkgs.iter()
            .find(|p| p.repo.contains("updates"))
            .map(|p| p.repo.clone())
            .or_else(|| pkgs.first().map(|p| p.repo.clone()))
            .unwrap_or_default();
        let package_names: Vec<String> = pkgs.iter()
            .map(|p| p.name.clone())
            .collect();
        let repo_display = if repo.contains("updates") {
            "Fedora Updates".to_string()
        } else if repo.contains("fedora") {
            "Fedora".to_string()
        } else {
            repo.clone()
        };

        let install_script = format!("dnf install -y {}", package_names.join(" "));

        let desc = if driver_version.matches('.').count() >= 2 {

            format!("Mesa Graphics Driver {}", driver_version)
        } else if driver_version.matches('.').count() >= 1 {

            format!("Mesa Graphics Driver {}", driver_version)
        } else {
            format!("Mesa Graphics Driver {}", driver_version)
        };
        let profile = CfhdbPciProfile {
            codename: format!("mesa-{}", driver_version),
            i18n_desc: desc.clone(),
            icon_name: "mesa".to_string(),
            license: "open-source".to_string(),
            class_ids: vec!["0300".to_string()], // VGA controller only
            vendor_ids: vec!["*".to_string()], // Mesa works with AMD, Intel, and some NVIDIA
            device_ids: vec!["*".to_string()], // All devices
            blacklisted_class_ids: Vec::new(),
            blacklisted_vendor_ids: Vec::new(),
            blacklisted_device_ids: Vec::new(),
            packages: Some(package_names.clone()),
            check_script: format!("rpm -q {} > /dev/null 2>&1", pkgs.first().map(|p| &p.name).unwrap_or(&"mesa-dri-drivers".to_string())),
            install_script: Some(install_script),
            remove_script: Some(format!("dnf remove -y {}", pkgs.iter().map(|p| p.name.clone()).collect::<Vec<String>>().join(" "))),
            experimental: false,
            removable: true,
            veiled: false,
            priority: 90,
        };

        profiles.push((profile, repo_display, package_names));
    }

    profiles.sort_by_key(|(p, _, _)| p.priority);
    profiles
}

fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();

    let max_len = a_parts.len().max(b_parts.len());
    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);

        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
}

fn create_nvidia_profiles_from_repos(packages: Vec<NvidiaDriverPackage>) -> Vec<(CfhdbPciProfile, String, Vec<String>)> {
    let mut profiles = Vec::new();

    let mut version_groups: HashMap<String, Vec<NvidiaDriverPackage>> = HashMap::new();
    for pkg in packages {
        version_groups.entry(pkg.driver_version.clone())
            .or_insert_with(Vec::new)
            .push(pkg);
    }

    let mut version_groups_vec: Vec<(String, Vec<NvidiaDriverPackage>)> = version_groups.into_iter().collect();
    version_groups_vec.sort_by(|a, b| {

        version_compare(&b.0, &a.0)
    });

    for (driver_version, pkgs) in version_groups_vec {
        let repo = pkgs.iter()
            .find(|p| p.repo.contains("negativo17") || p.repo.contains("negativo") || p.repo.contains("fedora-nvidia"))
            .map(|p| p.repo.clone())
            .or_else(|| {

                pkgs.iter()
                    .find(|p| p.repo.contains("rpmfusion-nonfree-nvidia-driver"))
                    .map(|p| p.repo.clone())
            })
            .or_else(|| pkgs.first().map(|p| p.repo.clone()))
            .unwrap_or_default();
        let package_names: Vec<String> = pkgs.iter()
            .map(|p| p.name.clone())
            .collect();
        let repo_display = if repo.contains("fedora-nvidia") || repo.contains("negativo17") || repo.contains("negativo") {
            "negativo17.org".to_string()
        } else if repo.contains("rpmfusion-nonfree") {
            "RPM Fusion (Non-Free)".to_string()
        } else if repo.contains("rpmfusion-free") {
            "RPM Fusion (Free)".to_string()
        } else if repo.contains("rpmfusion") {
            "RPM Fusion".to_string()
        } else {
            repo.clone()
        };
        let removal_commands = "(dnf remove -y nvidia* || true) && (dnf remove -y kmod-nvidia* || true) && (dnf remove -y akmod-nvidia* || true) && (dnf remove -y dkms-nvidia || true) && (dnf remove -y xorg-x11-drv-nvidia* || true) && (rm -rf /var/lib/dkms/nvidia* || true)";
        let (nvidia_packages, install_script) = if repo.contains("negativo17") || repo.contains("negativo") || repo.contains("fedora-nvidia") {

            let packages = vec![
                "nvidia-driver".to_string(),
                "nvidia-driver-cuda".to_string(),
                "nvidia-settings".to_string(),
            ];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && (dnf repoinfo fedora-nvidia > /dev/null 2>&1 || dnf config-manager addrepo --from-repofile=https://negativo17.org/repos/fedora-nvidia.repo) && dnf install -y {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        } else if repo.contains("rpmfusion-nonfree") {

            let akmod_pkg = pkgs.iter()
                .find(|p| p.name.starts_with("akmod-nvidia"))
                .map(|p| p.name.as_str())
                .unwrap_or("akmod-nvidia");

            let packages = vec![akmod_pkg.to_string()];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && dnf install -y --enablerepo=rpmfusion-nonfree-updates {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        } else if repo.contains("rpmfusion-free") {
            let akmod_pkg = pkgs.iter()
                .find(|p| p.name.starts_with("akmod-nvidia"))
                .map(|p| p.name.as_str())
                .unwrap_or("akmod-nvidia");

            let packages = vec![akmod_pkg.to_string()];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && dnf install -y --enablerepo=rpmfusion-free-updates {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        } else {
            let akmod_pkg = pkgs.iter()
                .find(|p| p.name.starts_with("akmod-nvidia"))
                .map(|p| p.name.as_str())
                .unwrap_or("akmod-nvidia");

            let packages = vec![akmod_pkg.to_string()];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && dnf install -y --enablerepo=rpmfusion-free-updates --enablerepo=rpmfusion-nonfree-updates {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        };

        let desc = if driver_version.matches('.').count() >= 2 {

            format!("NVIDIA Graphics Driver {}", driver_version)
        } else if driver_version.chars().all(|c| c.is_ascii_digit()) {

            format!("NVIDIA Graphics Driver {} Series", driver_version)
        } else {
            format!("NVIDIA Graphics Driver {}", driver_version)
        };

        let profile = CfhdbPciProfile {
            codename: format!("nvidia-{}", driver_version),
            i18n_desc: desc.clone(),
            icon_name: "nvidia".to_string(),
            license: "proprietary".to_string(),
            class_ids: vec!["0300".to_string()], // VGA controller only
            vendor_ids: vec!["10de".to_string()], // NVIDIA vendor ID only
            device_ids: vec!["*".to_string()], // All NVIDIA devices
            blacklisted_class_ids: Vec::new(),
            blacklisted_vendor_ids: Vec::new(),
            blacklisted_device_ids: Vec::new(),
            packages: Some(nvidia_packages.iter().map(|s| s.to_string()).collect()),
            check_script: format!("rpm -q {} > /dev/null 2>&1", pkgs.first().map(|p| &p.name).unwrap_or(&"akmod-nvidia".to_string())),
            install_script: Some(install_script),
            remove_script: Some(format!("dnf remove -y {}", pkgs.iter().map(|p| p.name.clone()).collect::<Vec<String>>().join(" "))),
            experimental: false,
            removable: true,
            veiled: false,
            priority: 100, // High priority for repository-based profiles
        };

        profiles.push((profile, repo_display, package_names));
    }

    profiles.sort_by_key(|(p, _, _)| p.priority);
    profiles
}
async fn load_all_devices() -> Result<(
    Vec<(String, Vec<PreCheckedPciDevice>)>,
    Vec<(String, Vec<PreCheckedUsbDevice>)>,
    Vec<Arc<PreCheckedPciProfile>>,
    Vec<Arc<PreCheckedUsbProfile>>,
), String> {
    // Check if cfhdb profiles should be loaded
    let settings = crate::gui::settings::AppSettings::load();
    let show_cfhdb_profiles = settings.show_cfhdb_profiles;
    
    logger::Logger::log_debug(&format!("[Device Tab] Loading devices with cfhdb profiles: {}", show_cfhdb_profiles));

    let mut pci_profiles = if show_cfhdb_profiles {
        load_pci_profiles().await?
    } else {
        Vec::new()
    };
    
    // Only filter out Mesa/NVIDIA/CUDA profiles if cfhdb profiles are disabled
    // When enabled, show all cfhdb profiles including Mesa and NVIDIA
    if show_cfhdb_profiles {
        // When showing cfhdb profiles, only filter out Nobara-specific profiles
        // Keep Mesa, NVIDIA, and CUDA profiles from cfhdb
        pci_profiles.retain(|p| {
            // Only filter out Nobara-specific NVIDIA profiles
            let is_nvidia_nobara = p.vendor_ids.contains(&"10de".to_string()) &&
                p.install_script.as_ref().map(|s| s.contains("nobara") || s.contains("Nobara")).unwrap_or(false);
            !is_nvidia_nobara
        });
        logger::Logger::log_debug(&format!("[Device Tab] Loaded {} cfhdb PCI profiles (including Mesa/NVIDIA)", pci_profiles.len()));
    } else {
        // When cfhdb profiles are disabled, filter out Mesa/NVIDIA/CUDA to avoid conflicts with repo profiles
        pci_profiles.retain(|p| {
            let is_nvidia_nobara = p.vendor_ids.contains(&"10de".to_string()) &&
                p.install_script.as_ref().map(|s| s.contains("nobara") || s.contains("Nobara")).unwrap_or(false);
            let is_mesa = p.codename.to_lowercase().contains("mesa") ||
                p.i18n_desc.to_lowercase().contains("mesa") ||
                p.install_script.as_ref().map(|s| s.to_lowercase().contains("mesa")).unwrap_or(false);
            let is_cuda_profile = p.codename.to_lowercase().contains("cuda") ||
                (p.i18n_desc.to_lowercase().contains("cuda") &&
                 !p.i18n_desc.to_lowercase().contains("nvidia graphics driver"));
            !is_nvidia_nobara && !is_mesa && !is_cuda_profile
        });
    }
    // Check if we have NVIDIA driver profiles (not just CUDA) in cfhdb profiles
    // Look for profiles that:
    // - Have NVIDIA vendor ID (10de)
    // - Are NOT CUDA profiles
    // - Have codename starting with "nvidia-" followed by a version number, OR
    // - Have description containing "NVIDIA Graphics Driver" (not CUDA)
    let has_cfhdb_nvidia_driver = pci_profiles.iter().any(|p| {
        if !p.vendor_ids.contains(&"10de".to_string()) {
            return false;
        }
        // Exclude CUDA profiles
        if p.codename.to_lowercase().contains("cuda") || 
           p.i18n_desc.to_lowercase().contains("cuda") {
            return false;
        }
        // Check if it's a driver profile (not compute/CUDA)
        let is_driver = p.codename.starts_with("nvidia-") && 
                       p.codename.len() > 7 && // Has something after "nvidia-"
                       p.codename.chars().skip(7).next().map(|c| c.is_ascii_digit()).unwrap_or(false); // Version starts with digit
        let is_driver_desc = p.i18n_desc.to_lowercase().contains("nvidia graphics driver") &&
                            !p.i18n_desc.to_lowercase().contains("cuda");
        is_driver || is_driver_desc
    });
    
    // Log all NVIDIA profiles found for debugging
    let nvidia_profiles_found: Vec<String> = pci_profiles.iter()
        .filter(|p| p.vendor_ids.contains(&"10de".to_string()))
        .map(|p| format!("{}: {}", p.codename, p.i18n_desc))
        .collect();
    logger::Logger::log_debug(&format!("[Device Tab] Found {} NVIDIA-related profiles: {:?}", 
        nvidia_profiles_found.len(), nvidia_profiles_found));
    logger::Logger::log_debug(&format!("[Device Tab] CFHDB has NVIDIA driver profiles (non-CUDA): {}", has_cfhdb_nvidia_driver));
    
    // Always add repo-based NVIDIA profiles when cfhdb is enabled
    // This ensures users have NVIDIA driver options even if cfhdb only has CUDA
    let mut repo_profiles_with_info: Vec<(CfhdbPciProfile, String, Vec<String>)> = Vec::new();
    
    if !show_cfhdb_profiles || !has_cfhdb_nvidia_driver {
        // Query repo packages
        let (nvidia_result, mesa_result) = tokio::join!(
            query_nvidia_driver_packages(),
            query_mesa_driver_packages()
        );

        // Add NVIDIA profiles if cfhdb is disabled OR if cfhdb doesn't have NVIDIA driver profiles
        if !show_cfhdb_profiles || !has_cfhdb_nvidia_driver {
            match nvidia_result {
                Ok(repo_packages) if !repo_packages.is_empty() => {
                    let nvidia_profiles = create_nvidia_profiles_from_repos(repo_packages);
                    logger::Logger::log_debug(&format!("[Device Tab] Adding {} repo-based NVIDIA driver profiles", nvidia_profiles.len()));
                    repo_profiles_with_info.extend(nvidia_profiles);
                }
                Ok(_) => {
                }
                Err(_e) => {
                }
            }
        }
        
        // Add Mesa profiles only if cfhdb is disabled (cfhdb should have Mesa)
        if !show_cfhdb_profiles {
            match mesa_result {
                Ok(repo_packages) if !repo_packages.is_empty() => {
                    let mesa_profiles = create_mesa_profiles_from_repos(repo_packages);
                    logger::Logger::log_debug(&format!("[Device Tab] Adding {} repo-based Mesa profiles", mesa_profiles.len()));
                    repo_profiles_with_info.extend(mesa_profiles);
                }
                Ok(_) => {
                }
                Err(_e) => {
                }
            }
        }
        
        if !show_cfhdb_profiles {
            logger::Logger::log_debug(&format!("[Device Tab] Using repo-based profiles (cfhdb disabled)"));
        } else {
            logger::Logger::log_debug(&format!("[Device Tab] Using cfhdb profiles + repo NVIDIA drivers (cfhdb missing NVIDIA driver profiles)"));
        }
    } else {
        logger::Logger::log_debug(&format!("[Device Tab] Using cfhdb profiles only"));
    }
    let mut all_pci_profiles = pci_profiles;
    let mut repo_info_map: HashMap<String, (String, Vec<String>)> = HashMap::new();
    for (profile, repo_name, package_names) in repo_profiles_with_info {
        let codename = profile.codename.clone();
        repo_info_map.insert(codename, (repo_name, package_names));
        all_pci_profiles.push(profile);
    }

    // Count NVIDIA and Mesa profiles for logging
    let nvidia_count = all_pci_profiles.iter()
        .filter(|p| p.vendor_ids.contains(&"10de".to_string()))
        .count();
    let mesa_count = all_pci_profiles.iter()
        .filter(|p| p.codename.to_lowercase().contains("mesa") || 
                     p.i18n_desc.to_lowercase().contains("mesa"))
        .count();
    logger::Logger::log_debug(&format!("[Device Tab] Total profiles: {} (NVIDIA: {}, Mesa: {})", 
        all_pci_profiles.len(), nvidia_count, mesa_count));

    let pci_profiles_arc: Vec<Arc<PreCheckedPciProfile>> = all_pci_profiles
        .into_iter()
        .map(|p| {
            let profile = PreCheckedPciProfile::new(p.clone());
            if p.vendor_ids.contains(&"10de".to_string()) {

                if let Some(version_part) = p.codename.strip_prefix("nvidia-") {

                    let clean_version: String = version_part
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                        profile.set_driver_version(Some(clean_version));
                    }
                }

                else if p.i18n_desc.contains("Driver ") {

                    let parts: Vec<&str> = p.i18n_desc.split("Driver ").collect();
                    if parts.len() > 1 {
                        let version_part = parts[1].trim();

                        let clean_version: String = version_part
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();

                        if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                            profile.set_driver_version(Some(clean_version));
                        }
                    }
                }
                if p.priority == 100 && p.packages.is_some() {

                    if let Some((repo_name, _package_names)) = repo_info_map.get(&p.codename) {
                        profile.set_repository(Some(repo_name.clone()));

                    } else {

                        if let Some(pkgs) = &p.packages {
                            if !pkgs.is_empty() {
                                let repo = if p.install_script.as_ref().map(|s| s.contains("negativo17") || s.contains("negativo")).unwrap_or(false) {
                                    Some("negativo17.org".to_string())
                                } else if p.install_script.as_ref().map(|s| s.contains("rpmfusion-nonfree")).unwrap_or(false) {
                                    Some("RPM Fusion (Non-Free)".to_string())
                                } else if p.install_script.as_ref().map(|s| s.contains("rpmfusion-free")).unwrap_or(false) {
                                    Some("RPM Fusion (Free)".to_string())
                                } else if p.install_script.as_ref().map(|s| s.contains("rpmfusion")).unwrap_or(false) {
                                    Some("RPM Fusion".to_string())
                                } else {
                                    None
                                };
                                profile.set_repository(repo);

                            }
                        }
                    }
                }
            }

            else if p.codename.starts_with("mesa-") || p.i18n_desc.contains("Mesa") {

                if let Some(version_part) = p.codename.strip_prefix("mesa-") {

                    let clean_version: String = version_part
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                        profile.set_driver_version(Some(clean_version));
                    }
                }

                else if p.i18n_desc.contains("Driver ") {

                    let parts: Vec<&str> = p.i18n_desc.split("Driver ").collect();
                    if parts.len() > 1 {
                        let version_part = parts[1].trim();

                        let clean_version: String = version_part
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();

                        if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                            profile.set_driver_version(Some(clean_version));
                        }
                    }
                }
                if p.priority == 90 && p.packages.is_some() {

                    if let Some((repo_name, _package_names)) = repo_info_map.get(&p.codename) {
                        profile.set_repository(Some(repo_name.clone()));

                    } else {

                        if let Some(pkgs) = &p.packages {
                            if !pkgs.is_empty() {
                                let repo = if p.install_script.as_ref().map(|s| s.contains("updates")).unwrap_or(false) {
                                    Some("Fedora Updates".to_string())
                                } else if p.install_script.as_ref().map(|s| s.contains("fedora")).unwrap_or(false) {
                                    Some("Fedora".to_string())
                                } else {
                                    None
                                };
                                profile.set_repository(repo);

                            }
                        }
                    }
                }
            }

            Arc::new(profile)
        })
        .collect();
    let profiles_to_query: Vec<_> = pci_profiles_arc.iter()
        .filter_map(|profile_arc| {
            let p = profile_arc.profile();

            if (p.priority == 100 || p.priority == 90) && p.packages.is_some() {
                if let Some(pkgs) = &p.packages {
                    if !pkgs.is_empty() {
                        return Some((profile_arc.clone(), pkgs.clone()));
                    }
                }
            }
            None
        })
        .collect();
    if !profiles_to_query.is_empty() {
        std::thread::spawn(move || {
            for (profile, package_names) in profiles_to_query {
                query_package_info(&profile, &package_names);
            }
        });
    }
    let usb_profiles = if show_cfhdb_profiles {
        load_usb_profiles().await?
    } else {
        Vec::new()
    };
    let usb_profiles_arc: Vec<Arc<PreCheckedUsbProfile>> = usb_profiles
        .into_iter()
        .map(|p| {
            let profile = PreCheckedUsbProfile::new(p);
            Arc::new(profile)
        })
        .collect();
    let pci_devices = get_pci_devices(&pci_profiles_arc)
        .ok_or_else(|| "Failed to get PCI devices".to_string())?;
    let usb_devices = get_usb_devices(&usb_profiles_arc)
        .ok_or_else(|| "Failed to get USB devices".to_string())?;
    let mut pci_vec: Vec<(String, Vec<PreCheckedPciDevice>)> = pci_devices.into_iter().collect();
    let mut usb_vec: Vec<(String, Vec<PreCheckedUsbDevice>)> = usb_devices.into_iter().collect();

    pci_vec.sort_by(|a, b| a.0.cmp(&b.0));
    usb_vec.sort_by(|a, b| a.0.cmp(&b.0));

    Ok((pci_vec, usb_vec, pci_profiles_arc, usb_profiles_arc))
}

async fn load_pci_profiles() -> Result<Vec<CfhdbPciProfile>, String> {
    let cached_db_path = Path::new("/var/cache/cfhdb/pci.json");
    if cached_db_path.exists() {
        match std::fs::read_to_string(cached_db_path) {
            Ok(data) => {
                match parse_pci_profiles(&data) {
                    Ok(profiles) => {
                        return Ok(profiles);
                    }
                    Err(_e) => {
                    }
                }
            }
            Err(_e) => {
            }
        }
    } else {
    }
    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            url
        }
        Err(e) => {
            return Err(format!("Failed to read profile config: {}", e));
        }
    };

    match reqwest::get(&profile_url.pci_json_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        let _ = cache_profile_file(cached_db_path, &text).await;
                        match parse_pci_profiles(&text) {
                            Ok(profiles) => {
                                Ok(profiles)
                            }
                            Err(e) => {
                                Err(format!("Failed to parse PCI profiles: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        Err(format!("Failed to read response: {}", e))
                    }
                }
            } else {
                Err(format!("HTTP error: {}", response.status()))
            }
        }
        Err(e) => {
            Err(format!("Failed to download PCI profiles: {}", e))
        }
    }
}

async fn load_usb_profiles() -> Result<Vec<CfhdbUsbProfile>, String> {
    let cached_db_path = Path::new("/var/cache/cfhdb/usb.json");
    if cached_db_path.exists() {
        match std::fs::read_to_string(cached_db_path) {
            Ok(data) => {
                match parse_usb_profiles(&data) {
                    Ok(profiles) => {
                        return Ok(profiles);
                    }
                    Err(_e) => {
                    }
                }
            }
            Err(_e) => {
            }
        }
    } else {
    }
    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            url
        }
        Err(e) => {
            return Err(format!("Failed to read profile config: {}", e));
        }
    };

    match reqwest::get(&profile_url.usb_json_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        let _ = cache_profile_file(cached_db_path, &text).await;
                        match parse_usb_profiles(&text) {
                            Ok(profiles) => {
                                Ok(profiles)
                            }
                            Err(e) => {
                                Err(format!("Failed to parse USB profiles: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        Err(format!("Failed to read response: {}", e))
                    }
                }
            } else {
                Err(format!("HTTP error: {}", response.status()))
            }
        }
        Err(e) => {
            Err(format!("Failed to download USB profiles: {}", e))
        }
    }
}

fn get_profile_url_config() -> Result<ProfileUrlConfig, std::io::Error> {
    let file_path = "/etc/cfhdb/profile-config.json";
    if let Ok(json_content) = std::fs::read_to_string(file_path) {
        if let Ok(config) = serde_json::from_str::<ProfileUrlConfig>(&json_content) {
            return Ok(config);
        }
    }

    Ok(ProfileUrlConfig {
        pci_json_url: "https://raw.githubusercontent.com/Nobara-Project/cfhdb/refs/heads/master/data/profiles/pci.json".to_string(),
        usb_json_url: "https://raw.githubusercontent.com/Nobara-Project/cfhdb/refs/heads/master/data/profiles/usb.json".to_string(),
    })
}
fn profiles_need_update(cached_path: &Path) -> bool {
    if !cached_path.exists() {
        return true;
    }
    match std::fs::metadata(cached_path) {
        Ok(metadata) => {
            match metadata.modified() {
                Ok(modified) => {
                    match modified.elapsed() {
                        Ok(elapsed) => {
                            let _hours = elapsed.as_secs() / 3600;
                            let needs_update = elapsed.as_secs() > 24 * 60 * 60;
                            needs_update
                        }
                        Err(_) => {
                            true
                        }
                    }
                }
                Err(_) => {
                    true
                }
            }
        }
        Err(_) => {
            true
        }
    }
}
async fn ensure_profiles_cached_force() -> Result<(), String> {

    let pci_path = Path::new("/var/cache/cfhdb/pci.json");
    let usb_path = Path::new("/var/cache/cfhdb/usb.json");

    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            url
        }
        Err(e) => {
            return Err(format!("Failed to read profile config: {}", e));
        }
    };
    match reqwest::get(&profile_url.pci_json_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        if let Err(e) = cache_profile_file(pci_path, &text).await {
                            return Err(format!("Failed to cache PCI profiles: {}", e));
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to read PCI profile response: {}", e));
                    }
                }
            } else {
                return Err(format!("Failed to download PCI profiles: HTTP {}", response.status()));
            }
        }
        Err(e) => {
            return Err(format!("Failed to download PCI profiles: {}", e));
        }
    }
    match reqwest::get(&profile_url.usb_json_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        if let Err(e) = cache_profile_file(usb_path, &text).await {
                            return Err(format!("Failed to cache USB profiles: {}", e));
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to read USB profile response: {}", e));
                    }
                }
            } else {
                return Err(format!("Failed to download USB profiles: HTTP {}", response.status()));
            }
        }
        Err(e) => {
            return Err(format!("Failed to download USB profiles: {}", e));
        }
    }

    Ok(())
}
async fn ensure_profiles_cached() -> Result<(), String> {

    let pci_path = Path::new("/var/cache/cfhdb/pci.json");
    let usb_path = Path::new("/var/cache/cfhdb/usb.json");

    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            url
        }
        Err(e) => {
            return Err(format!("Failed to read profile config: {}", e));
        }
    };
    if profiles_need_update(pci_path) {
        match reqwest::get(&profile_url.pci_json_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => {
                            if let Err(e) = cache_profile_file(pci_path, &text).await {
                                return Err(format!("Failed to cache PCI profiles: {}", e));
                            }
                        }
                        Err(e) => {
                            return Err(format!("Failed to read PCI profile response: {}", e));
                        }
                    }
                } else {
                    return Err(format!("Failed to download PCI profiles: HTTP {}", response.status()));
                }
            }
            Err(e) => {
                return Err(format!("Failed to download PCI profiles: {}", e));
            }
        }
    } else {
    }
    if profiles_need_update(usb_path) {
        match reqwest::get(&profile_url.usb_json_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => {
                            if let Err(e) = cache_profile_file(usb_path, &text).await {
                                return Err(format!("Failed to cache USB profiles: {}", e));
                            }
                        }
                        Err(e) => {
                            return Err(format!("Failed to read USB profile response: {}", e));
                        }
                    }
                } else {
                    return Err(format!("Failed to download USB profiles: HTTP {}", response.status()));
                }
            }
            Err(e) => {
                return Err(format!("Failed to download USB profiles: {}", e));
            }
        }
    } else {
    }

    Ok(())
}

async fn request_permissions() -> Result<(), String> {
    use tokio::process::Command as TokioCommand;

    let current_user = users::get_current_username()
        .ok_or("Failed to get current username")?
        .to_string_lossy()
        .to_string();

    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("sh");
    cmd.arg("-c");
    cmd.arg(&format!(
        "mkdir -p /var/cache/cfhdb && chown -R {}:{} /var/cache/cfhdb && chmod -R 777 /var/cache/cfhdb",
        current_user, current_user
    ));
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| {
            format!("Failed to request permissions: {}. Make sure polkit is installed.", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        return Err(format!("Failed to set up directory: {}", stderr));
    }

    Ok(())
}
async fn cache_profile_file(path: &Path, content: &str) -> Result<(), String> {
    use tokio::process::Command as TokioCommand;
    if let Some(parent) = path.parent() {
        match std::fs::create_dir_all(parent) {
            Ok(_) => {
                match std::fs::write(path, content) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(_e) => {
                    }
                }
            }
            Err(_e) => {
            }
        }
    }

    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_file = std::env::temp_dir().join(format!("cfhdb_{}.json", timestamp));

    if let Err(e) = std::fs::write(&temp_file, content) {
        return Err(format!("Failed to write temp file: {}", e));
    }
    let parent_dir = path.parent().ok_or("Invalid path")?;
    let parent_str = parent_dir.to_str().ok_or("Invalid path")?;
    let path_str = path.to_str().ok_or("Invalid path")?;
    let temp_str = temp_file.to_str().ok_or("Invalid temp path")?;
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("sh");
    cmd.arg("-c");
    cmd.arg(&format!(
        "mkdir -p {} && chmod 755 {} && cp {} {} && chmod 644 {}",
        parent_str, parent_str, temp_str, path_str, path_str
    ));

    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    let output = cmd
        .output()
        .await
        .map_err(|e| {
            let _ = std::fs::remove_file(&temp_file);
            format!("Failed to execute command: {}. Make sure polkit is installed.", e)
        })?;
    let _ = std::fs::remove_file(&temp_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        return Err(format!("Failed to cache profile: {}", stderr));
    }

    let mut fix_perms_cmd = TokioCommand::new("pkexec");
    fix_perms_cmd.arg("sh");
    fix_perms_cmd.arg("-c");
    fix_perms_cmd.arg("find /var/cache/cfhdb/ -type d -exec chmod 755 {} + && find /var/cache/cfhdb/ -type f -exec chmod 644 {} +");
    if let Ok(display) = std::env::var("DISPLAY") {
        fix_perms_cmd.env("DISPLAY", display);
    }
    let fix_perms_output = fix_perms_cmd
        .output()
        .await;

    match fix_perms_output {
        Ok(output) if output.status.success() => {
        }
        Ok(_output) => {
        }
        Err(_e) => {
        }
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct ProfileUrlConfig {
    pci_json_url: String,
    usb_json_url: String,
}

fn parse_pci_profiles(data: &str) -> Result<Vec<CfhdbPciProfile>, String> {
    let res: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut profiles = Vec::new();

    if let serde_json::Value::Array(profiles_array) = &res["profiles"] {
        for profile in profiles_array {

            let codename = profile["codename"].as_str().unwrap_or_default().to_string();
            let i18n_desc = profile["i18n_desc"].as_str().unwrap_or_default().to_string();
            let icon_name = profile["icon_name"].as_str().unwrap_or("package-x-generic").to_string();
            let license = profile["license"].as_str().unwrap_or("unknown").to_string();

            let class_ids: Vec<String> = profile["class_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let vendor_ids: Vec<String> = profile["vendor_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let device_ids: Vec<String> = profile["device_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let blacklisted_class_ids: Vec<String> = profile["blacklisted_class_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let blacklisted_vendor_ids: Vec<String> = profile["blacklisted_vendor_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let blacklisted_device_ids: Vec<String> = profile["blacklisted_device_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let packages: Option<Vec<String>> = if profile["packages"].is_string() {
                None
            } else {
                profile["packages"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            };

            let check_script = profile["check_script"].as_str().unwrap_or("false").to_string();

            let install_script = profile["install_script"].as_str()
                .and_then(|s| if s == "Option::is_none" { None } else { Some(s.to_string()) });

            let remove_script = profile["remove_script"].as_str()
                .and_then(|s| if s == "Option::is_none" { None } else { Some(s.to_string()) });

            let experimental = profile["experimental"].as_bool().unwrap_or(false);
            let removable = profile["removable"].as_bool().unwrap_or(false);
            let veiled = profile["veiled"].as_bool().unwrap_or(false);
            let priority = profile["priority"].as_i64().unwrap_or(0) as i32;

            let profile_struct = CfhdbPciProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                class_ids,
                vendor_ids,
                device_ids,
                blacklisted_class_ids,
                blacklisted_vendor_ids,
                blacklisted_device_ids,
                packages,
                check_script,
                install_script,
                remove_script,
                experimental,
                removable,
                veiled,
                priority,
            };

            profiles.push(profile_struct);
        }
    }

    profiles.sort_by_key(|p| p.priority);
    Ok(profiles)
}

fn parse_usb_profiles(data: &str) -> Result<Vec<CfhdbUsbProfile>, String> {
    let res: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut profiles = Vec::new();

    if let serde_json::Value::Array(profiles_array) = &res["profiles"] {
        for profile in profiles_array {

            let codename = profile["codename"].as_str().unwrap_or_default().to_string();
            let i18n_desc = profile["i18n_desc"].as_str().unwrap_or_default().to_string();
            let icon_name = profile["icon_name"].as_str().unwrap_or("package-x-generic").to_string();
            let license = profile["license"].as_str().unwrap_or("unknown").to_string();

            let class_codes: Vec<String> = profile["class_codes"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let vendor_ids: Vec<String> = profile["vendor_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let product_ids: Vec<String> = profile["product_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let blacklisted_class_codes: Vec<String> = profile["blacklisted_class_codes"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let blacklisted_vendor_ids: Vec<String> = profile["blacklisted_vendor_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let blacklisted_product_ids: Vec<String> = profile["blacklisted_product_ids"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let packages: Option<Vec<String>> = if profile["packages"].is_string() {
                None
            } else {
                profile["packages"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            };

            let check_script = profile["check_script"].as_str().unwrap_or("false").to_string();

            let install_script = profile["install_script"].as_str()
                .and_then(|s| if s == "Option::is_none" { None } else { Some(s.to_string()) });

            let remove_script = profile["remove_script"].as_str()
                .and_then(|s| if s == "Option::is_none" { None } else { Some(s.to_string()) });

            let experimental = profile["experimental"].as_bool().unwrap_or(false);
            let removable = profile["removable"].as_bool().unwrap_or(false);
            let veiled = profile["veiled"].as_bool().unwrap_or(false);
            let priority = profile["priority"].as_i64().unwrap_or(0) as i32;

            let profile_struct = CfhdbUsbProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                class_codes,
                vendor_ids,
                product_ids,
                blacklisted_class_codes,
                blacklisted_vendor_ids,
                blacklisted_product_ids,
                packages,
                check_script,
                install_script,
                remove_script,
                experimental,
                removable,
                veiled,
                priority,
            };

            profiles.push(profile_struct);
        }
    }

    profiles.sort_by_key(|p| p.priority);
    Ok(profiles)
}

fn get_pci_devices(profiles: &[Arc<PreCheckedPciProfile>]) -> Option<HashMap<String, Vec<PreCheckedPciDevice>>> {
    let devices = CfhdbPciDevice::get_devices()?;
    let hashmap = CfhdbPciDevice::create_class_hashmap(devices);

    Some(
        hashmap
            .into_iter()
            .map(|(class, devices)| {
                let pre_checked_devices: Vec<PreCheckedPciDevice> = devices
                    .into_iter()
                    .map(|device| get_pre_checked_pci_device(profiles, device))
                    .collect();
                (class, pre_checked_devices)
            })
            .collect()
    )
}

fn get_usb_devices(profiles: &[Arc<PreCheckedUsbProfile>]) -> Option<HashMap<String, Vec<PreCheckedUsbDevice>>> {
    let devices = CfhdbUsbDevice::get_devices()?;
    let hashmap = CfhdbUsbDevice::create_class_hashmap(devices);

    Some(
        hashmap
            .into_iter()
            .map(|(class, devices)| {
                let pre_checked_devices: Vec<PreCheckedUsbDevice> = devices
                    .into_iter()
                    .map(|device| get_pre_checked_usb_device(profiles, device))
                    .collect();
                (class, pre_checked_devices)
            })
            .collect()
    )
}

fn get_pre_checked_pci_device(
    profile_data: &[Arc<PreCheckedPciProfile>],
    device: CfhdbPciDevice,
) -> PreCheckedPciDevice {
    let mut available_profiles = Vec::new();

    if device.vendor_id == "10de" {
    }

    for profile_arc in profile_data {
        let profile = profile_arc.profile();

        if profile.vendor_ids.contains(&"10de".to_string()) {
        }

        let matching = {
            if (profile.blacklisted_class_ids.contains(&"*".to_owned())
                || profile.blacklisted_class_ids.contains(&device.class_id))
                || (profile.blacklisted_vendor_ids.contains(&"*".to_owned())
                    || profile.blacklisted_vendor_ids.contains(&device.vendor_id))
                || (profile.blacklisted_device_ids.contains(&"*".to_owned())
                    || profile.blacklisted_device_ids.contains(&device.device_id))
            {
                false
            } else {
                (profile.class_ids.contains(&"*".to_owned())
                    || profile.class_ids.contains(&device.class_id))
                    && (profile.vendor_ids.contains(&"*".to_owned())
                        || profile.vendor_ids.contains(&device.vendor_id))
                    && (profile.device_ids.contains(&"*".to_owned())
                        || profile.device_ids.contains(&device.device_id))
            }
        };

        if matching {
            if device.vendor_id == "10de" {
            }
            available_profiles.push(profile_arc.clone());
            // Log when NVIDIA or Mesa profiles are matched
            if profile.vendor_ids.contains(&"10de".to_string()) {
                logger::Logger::log_debug(&format!("[Device Tab] Matched NVIDIA profile '{}' to device {}", 
                    profile.codename, device.sysfs_busid));
            } else if profile.codename.to_lowercase().contains("mesa") || 
                      profile.i18n_desc.to_lowercase().contains("mesa") {
                logger::Logger::log_debug(&format!("[Device Tab] Matched Mesa profile '{}' to device {}", 
                    profile.codename, device.sysfs_busid));
            }
        }
    }

    if device.vendor_id == "10de" {
    }

    PreCheckedPciDevice {
        device,
        profiles: available_profiles,
    }
}

fn get_pre_checked_usb_device(
    profile_data: &[Arc<PreCheckedUsbProfile>],
    device: CfhdbUsbDevice,
) -> PreCheckedUsbDevice {
    let mut available_profiles = Vec::new();

    for profile_arc in profile_data {
        let profile = profile_arc.profile();

        let matching = {
            if (profile.blacklisted_class_codes.contains(&"*".to_owned())
                || profile.blacklisted_class_codes.contains(&device.class_code))
                || (profile.blacklisted_vendor_ids.contains(&"*".to_owned())
                    || profile.blacklisted_vendor_ids.contains(&device.vendor_id))
                || (profile.blacklisted_product_ids.contains(&"*".to_owned())
                    || profile.blacklisted_product_ids.contains(&device.product_id))
            {
                false
            } else {
                (profile.class_codes.contains(&"*".to_owned())
                    || profile.class_codes.contains(&device.class_code))
                    && (profile.vendor_ids.contains(&"*".to_owned())
                        || profile.vendor_ids.contains(&device.vendor_id))
                    && (profile.product_ids.contains(&"*".to_owned())
                        || profile.product_ids.contains(&device.product_id))
            }
        };

        if matching {
            available_profiles.push(profile_arc.clone());
        }
    }

    PreCheckedUsbDevice {
        device,
        profiles: available_profiles,
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

struct SidebarScrollableStyle {
    background_color: iced::Color,
    border_radius: f32,
    _theme: crate::gui::Theme,
}

impl ScrollableStyleSheet for SidebarScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ScrollableAppearance {
        let is_dark = self.background_color.r < 0.5;
        let divider_color = if is_dark {
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3)
        } else {
            iced::Color::from_rgba(0.3, 0.3, 0.3, 0.2)
        };
        let scroller_color = if is_dark {
            iced::Color::from_rgba(0.4, 0.4, 0.4, 0.6)
        } else {
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5)
        };

        ScrollableAppearance {
            container: Appearance {
                background: None,
                border: Border::default(),
                ..Default::default()
            },
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(iced::Background::Color(divider_color)),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: scroller_color,
                    border: Border {
                        radius: (self.border_radius * 0.3).into(),
                        width: 0.0,
                        color: iced::Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, style: &Self::Style, _is_mouse_over_scrollbar: bool) -> ScrollableAppearance {
        let mut appearance = self.active(style);
        let is_dark = self.background_color.r < 0.5;
        let scroller_color = if is_dark {
            iced::Color::from_rgba(0.5, 0.5, 0.5, 0.8)
        } else {
            iced::Color::from_rgba(0.4, 0.4, 0.4, 0.7)
        };
        appearance.scrollbar.scroller.color = scroller_color;
        appearance
    }
}

struct SidebarStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for SidebarStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(
                    palette.background.r * 0.95,
                    palette.background.g * 0.95,
                    palette.background.b * 0.95,
                    1.0,
                )
            )),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}

struct DeviceCardStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for DeviceCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(
                    palette.background.r * 0.98,
                    palette.background.g * 0.98,
                    palette.background.b * 0.98,
                    1.0,
                )
            )),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct StatusIndicatorStyle {
    color: iced::Color,
    radius: f32,
}

impl iced::widget::container::StyleSheet for StatusIndicatorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.color)),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct StatusBadgeContainerStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for StatusBadgeContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(
                    palette.background.r * 0.95,
                    palette.background.g * 0.95,
                    palette.background.b * 0.95,
                    1.0,
                )
            )),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct BadgeStyle {
    is_positive: bool,
    radius: f32,
}

impl iced::widget::container::StyleSheet for BadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                if self.is_positive {
                    iced::Color::from_rgba(
                        palette.primary.r * 0.1,
                        palette.primary.g * 0.1,
                        palette.primary.b * 0.1,
                        1.0,
                    )
                } else {
                    iced::Color::from_rgba(
                        palette.danger.r * 0.1,
                        palette.danger.g * 0.1,
                        palette.danger.b * 0.1,
                        1.0,
                    )
                }
            )),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct InfoBadgeStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for InfoBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(
                    palette.background.r * 0.98,
                    palette.background.g * 0.98,
                    palette.background.b * 0.98,
                    1.0,
                )
            )),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ExperimentalBadgeStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for ExperimentalBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(
                    palette.danger.r * 0.15,
                    palette.danger.g * 0.15,
                    palette.danger.b * 0.15,
                    1.0,
                )
            )),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ProfileCardStyle {
    radius: f32,
}

impl iced::widget::container::StyleSheet for ProfileCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let palette = style.palette();
        Appearance {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(
                    palette.background.r * 0.98,
                    palette.background.g * 0.98,
                    palette.background.b * 0.98,
                    1.0,
                )
            )),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}
