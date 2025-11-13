// Device Manager Tab - Ported from nobara-device-manager
// This is a massive port from GTK4/libadwaita to Iced

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Padding, Border};
use iced::widget::container::Appearance;
use iced::widget::button::Appearance as ButtonAppearance;
use iced::widget::button::StyleSheet as ButtonStyleSheet;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

// Re-export types from cfhdb for convenience
// Note: The cfhdb crate exports modules as libcfhdb
use libcfhdb::pci::{CfhdbPciDevice, CfhdbPciProfile};
use libcfhdb::usb::{CfhdbUsbDevice, CfhdbUsbProfile};

// Wrapper types similar to nobara-device-manager
#[derive(Debug, Clone)]
pub struct PreCheckedPciDevice {
    pub device: CfhdbPciDevice,
    pub profiles: Vec<Arc<PreCheckedPciProfile>>,
}

#[derive(Debug, Clone)]
pub struct PreCheckedPciProfile {
    profile: CfhdbPciProfile,
    installed: Arc<std::sync::Mutex<bool>>,
    driver_version: Arc<std::sync::Mutex<Option<String>>>,
    repository: Arc<std::sync::Mutex<Option<String>>>,
    package_size: Arc<std::sync::Mutex<Option<String>>>,
    dependencies: Arc<std::sync::Mutex<Option<Vec<String>>>>,
}

impl PreCheckedPciProfile {
    pub fn new(profile: CfhdbPciProfile) -> Self {
        Self {
            profile,
            installed: Arc::new(std::sync::Mutex::new(false)),
            driver_version: Arc::new(std::sync::Mutex::new(None)),
            repository: Arc::new(std::sync::Mutex::new(None)),
            package_size: Arc::new(std::sync::Mutex::new(None)),
            dependencies: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    pub fn profile(&self) -> CfhdbPciProfile {
        self.profile.clone()
    }
    
    #[allow(dead_code)]
    pub fn installed(&self) -> bool {
        *self.installed.lock().unwrap()
    }
    
    pub fn update_installed(&self) {
        *self.installed.lock().unwrap() = self.profile.get_status();
    }
    
    pub fn driver_version(&self) -> Option<String> {
        self.driver_version.lock().unwrap().clone()
    }
    
    pub fn set_driver_version(&self, version: Option<String>) {
        *self.driver_version.lock().unwrap() = version;
    }
    
    pub fn repository(&self) -> Option<String> {
        self.repository.lock().unwrap().clone()
    }
    
    pub fn set_repository(&self, repo: Option<String>) {
        *self.repository.lock().unwrap() = repo;
    }
    
    pub fn package_size(&self) -> Option<String> {
        self.package_size.lock().unwrap().clone()
    }
    
    pub fn set_package_size(&self, size: Option<String>) {
        *self.package_size.lock().unwrap() = size;
    }
    
    pub fn dependencies(&self) -> Option<Vec<String>> {
        self.dependencies.lock().unwrap().clone()
    }
    
    pub fn set_dependencies(&self, deps: Option<Vec<String>>) {
        *self.dependencies.lock().unwrap() = deps;
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
    installed: Arc<std::sync::Mutex<bool>>,
    driver_version: Arc<std::sync::Mutex<Option<String>>>,
    #[allow(dead_code)]
    repository: Arc<std::sync::Mutex<Option<String>>>,
    #[allow(dead_code)]
    package_size: Arc<std::sync::Mutex<Option<String>>>,
    #[allow(dead_code)]
    dependencies: Arc<std::sync::Mutex<Option<Vec<String>>>>,
}

impl PreCheckedUsbProfile {
    pub fn new(profile: CfhdbUsbProfile) -> Self {
        Self {
            profile,
            installed: Arc::new(std::sync::Mutex::new(false)),
            driver_version: Arc::new(std::sync::Mutex::new(None)),
            repository: Arc::new(std::sync::Mutex::new(None)),
            package_size: Arc::new(std::sync::Mutex::new(None)),
            dependencies: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    pub fn profile(&self) -> CfhdbUsbProfile {
        self.profile.clone()
    }
    
    #[allow(dead_code)]
    pub fn installed(&self) -> bool {
        *self.installed.lock().unwrap()
    }
    
    pub fn update_installed(&self) {
        *self.installed.lock().unwrap() = self.profile.get_status();
    }
    
    pub fn driver_version(&self) -> Option<String> {
        self.driver_version.lock().unwrap().clone()
    }
    
    pub fn set_driver_version(&self, version: Option<String>) {
        *self.driver_version.lock().unwrap() = version;
    }
    
    #[allow(dead_code)]
    pub fn repository(&self) -> Option<String> {
        self.repository.lock().unwrap().clone()
    }
    
    #[allow(dead_code)]
    pub fn set_repository(&self, repo: Option<String>) {
        *self.repository.lock().unwrap() = repo;
    }
    
    #[allow(dead_code)]
    pub fn package_size(&self) -> Option<String> {
        self.package_size.lock().unwrap().clone()
    }
    
    #[allow(dead_code)]
    pub fn set_package_size(&self, size: Option<String>) {
        *self.package_size.lock().unwrap() = size;
    }
    
    #[allow(dead_code)]
    pub fn dependencies(&self) -> Option<Vec<String>> {
        self.dependencies.lock().unwrap().clone()
    }
    
    #[allow(dead_code)]
    pub fn set_dependencies(&self, deps: Option<Vec<String>>) {
        *self.dependencies.lock().unwrap() = deps;
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
    SelectDevice(DeviceType, String, usize), // type, class, index
    LoadDevicesAfterCache,
    DownloadProfiles,
    #[allow(dead_code)]
    DownloadProfilesForce,
    ProfilesDownloaded(Result<(), String>),
    BackToDeviceList,
    StartDevice(DeviceType, String, usize),
    StopDevice(DeviceType, String, usize),
    EnableDevice(DeviceType, String, usize),
    DisableDevice(DeviceType, String, usize),
    DeviceControlComplete,
    InstallProfile(DeviceType, String, usize, String), // type, class, device_index, profile_codename
    RemoveProfile(DeviceType, String, usize, usize),
    ProfileOperationComplete,
    Error(String),
    ClearError,
    UpdateStatus,
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
    // Loading state
    is_loading: bool,
    loading_message: String,
    
    // Device data
    pci_devices: Vec<(String, Vec<PreCheckedPciDevice>)>,
    usb_devices: Vec<(String, Vec<PreCheckedUsbDevice>)>,
    pci_profiles: Vec<Arc<PreCheckedPciProfile>>,
    usb_profiles: Vec<Arc<PreCheckedUsbProfile>>,
    
    // UI state
    selected_category: Option<(CategoryType, String)>,
    selected_device: Option<(DeviceType, String, usize)>,
    
    // Error state
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
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::RequestPermissions => {
                self.is_loading = true;
                self.loading_message = "Requesting elevated permissions...".to_string();
                self.error = None;
                // Use pkexec to verify permissions (shows GUI password dialog)
                iced::Command::perform(request_permissions(), |result| {
                    match result {
                        Ok(_) => Message::PermissionsGranted,
                        Err(e) => Message::PermissionsDenied(e),
                    }
                })
            }
            Message::PermissionsGranted => {
                self.loading_message = "Permissions granted. Loading devices...".to_string();
                // Now proceed to load devices
                iced::Command::perform(async {}, |_| Message::LoadDevices)
            }
            Message::PermissionsDenied(error) => {
                self.is_loading = false;
                self.loading_message = String::new();
                self.error = Some(format!("Permission denied: {}\n\nDevice management requires elevated permissions. Please try again.", error));
                iced::Command::none()
            }
            Message::LoadDevices => {
                if self.pci_devices.is_empty() && self.usb_devices.is_empty() {
                    self.is_loading = true;
                    self.loading_message = "Checking device profiles...".to_string();
                    // First ensure profiles are downloaded and cached
                    iced::Command::perform(ensure_profiles_cached(), |result| {
                        match result {
                            Ok(_) => Message::LoadDevicesAfterCache,
                            Err(e) => Message::Error(format!("Failed to cache profiles: {}", e)),
                        }
                    })
                } else {
                    iced::Command::none()
                }
            }
            Message::LoadDevicesAfterCache => {
                self.loading_message = "Loading device profiles...".to_string();
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
                self.loading_message = "Downloading and caching profiles...".to_string();
                iced::Command::perform(ensure_profiles_cached_force(), |result| {
                    Message::ProfilesDownloaded(result)
                })
            }
            Message::DownloadProfilesForce => {
                self.is_loading = true;
                self.loading_message = "Downloading and caching profiles...".to_string();
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
                        // Reload devices after downloading profiles
                        iced::Command::perform(async {}, |_| Message::LoadDevices)
                    }
                    Err(e) => {
                        self.loading_message = String::new();
                        // Provide more user-friendly error messages
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
                // Load driver versions asynchronously in the background
                iced::Command::perform(load_profile_versions(self.pci_profiles.clone(), self.usb_profiles.clone()), |_| Message::UpdateStatus)
            }
            Message::SelectCategory(cat_type, class) => {
                self.selected_category = Some((cat_type, class));
                self.selected_device = None;
                iced::Command::none()
            }
            Message::SelectDevice(dev_type, class, index) => {
                self.selected_device = Some((dev_type, class, index));
                // Update device status when selected
                iced::Command::perform(async {}, |_| Message::UpdateStatus)
            }
            Message::BackToDeviceList => {
                self.selected_device = None;
                iced::Command::none()
            }
            Message::ClearError => {
                self.error = None;
                // If we have a selected device, go back to it; otherwise stay on device list
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
                    Err(e) => iced::Command::perform(async {}, move |_| Message::Error(format!("Failed to start device: {}", e))),
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
                    Err(e) => iced::Command::perform(async {}, move |_| Message::Error(format!("Failed to stop device: {}", e))),
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
                    Err(e) => iced::Command::perform(async {}, move |_| Message::Error(format!("Failed to enable device: {}", e))),
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
                    Err(e) => iced::Command::perform(async {}, move |_| Message::Error(format!("Failed to disable device: {}", e))),
                }
            }
            Message::DeviceControlComplete => {
                // Refresh device status after control action
                iced::Command::perform(async {}, |_| Message::UpdateStatus)
            }
            Message::InstallProfile(dev_type, class, device_idx, profile_codename) => {
                eprintln!("[DEBUG] InstallProfile called: codename={}", profile_codename);
                // Get the profile and device information to install
                // Find profile by codename instead of index (since profiles are sorted for display)
                let profile_data = match dev_type {
                    DeviceType::Pci => {
                        if let Some((_, devices)) = self.pci_devices.iter().find(|(c, _)| c == &class) {
                            if let Some(device) = devices.get(device_idx) {
                                eprintln!("[DEBUG] Looking for profile with codename: {}", profile_codename);
                                eprintln!("[DEBUG] Available profiles: {:?}", device.profiles.iter().map(|p| p.profile().codename.clone()).collect::<Vec<_>>());
                                // Find profile by codename
                                if let Some(profile) = device.profiles.iter().find(|p| p.profile().codename == profile_codename) {
                                    let p = profile.profile();
                                    let d = &device.device;
                                    eprintln!("[DEBUG] Found profile: codename={}, desc={}, install_script={:?}", 
                                             p.codename, p.i18n_desc, p.install_script.is_some());
                                    // Get driver version from the profile (the one being installed)
                                    let driver_version = profile.driver_version().unwrap_or_default();
                                    eprintln!("[DEBUG] Driver version from profile: {}", driver_version);
                                    // Use driver version as the driver name (e.g., "580.95.08")
                                    let driver_name = if !driver_version.is_empty() {
                                        driver_version.clone()
                                    } else {
                                        // Fallback to profile description if version not available
                                        p.i18n_desc.split(" (").next()
                                            .unwrap_or_else(|| p.i18n_desc.split(" for ").next().unwrap_or(&p.i18n_desc))
                                            .trim()
                                            .to_string()
                                    };
                                    // Extract repository information from install script
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
                                // Find profile by codename
                                if let Some(profile) = device.profiles.iter().find(|p| p.profile().codename == profile_codename) {
                                    let p = profile.profile();
                                    let d = &device.device;
                                    // Get driver version from the profile (the one being installed)
                                    let driver_version = profile.driver_version().unwrap_or_default();
                                    // Use driver version as the driver name (e.g., "580.95.08")
                                    let driver_name = if !driver_version.is_empty() {
                                        driver_version.clone()
                                    } else {
                                        // Fallback to profile description if version not available
                                        p.i18n_desc.split(" (").next()
                                            .unwrap_or_else(|| p.i18n_desc.split(" for ").next().unwrap_or(&p.i18n_desc))
                                            .trim()
                                            .to_string()
                                    };
                                    // Extract repository information from install script
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
                        // Spawn separate window for driver installation
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("fedoraforge"));
                        let exe_str = exe_path.to_string_lossy().to_string();
                        let profile_name_clone = profile_name.clone();
                        let script_clone = script.clone();
                        
                        iced::Command::perform(
                            async move {
                                use tokio::process::Command as TokioCommand;
                                // Base64 encode all strings to pass as arguments (to avoid shell escaping issues)
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
            Message::RemoveProfile(_dev_type, _class, _device_idx, _profile_idx) => {
                // TODO: Implement profile removal
                iced::Command::none()
            }
            Message::ProfileOperationComplete => {
                // Refresh device status
                iced::Command::perform(async {}, |_| Message::UpdateStatus)
            }
            Message::UpdateStatus => {
                // Update device status and profile installation statuses
                if let Some((dev_type, class, device_idx)) = &self.selected_device {
                    // Refresh the specific device
                    match dev_type {
                        DeviceType::Pci => {
                            if let Some((_, devices)) = self.pci_devices.iter_mut().find(|(c, _)| c == class) {
                                if let Some(device) = devices.get_mut(*device_idx) {
                                    // Update device by fetching fresh data
                                    if let Ok(updated) = libcfhdb::pci::CfhdbPciDevice::get_device_from_busid(&device.device.sysfs_busid) {
                                        device.device = updated;
                                    }
                                    // Update all profiles
                                    for profile in &device.profiles {
                                        profile.update_installed();
                                    }
                                }
                            }
                        }
                        DeviceType::Usb => {
                            if let Some((_, devices)) = self.usb_devices.iter_mut().find(|(c, _)| c == class) {
                                if let Some(device) = devices.get_mut(*device_idx) {
                                    // Update device by fetching fresh data
                                    if let Ok(updated) = libcfhdb::usb::CfhdbUsbDevice::get_device_from_busid(&device.device.sysfs_busid) {
                                        device.device = updated;
                                    }
                                    // Update all profiles
                                    for profile in &device.profiles {
                                        profile.update_installed();
                                    }
                                }
                            }
                        }
                    }
                }
                // Also update all other devices' profiles
                for (_, devices) in &mut self.pci_devices {
                    for device in devices {
                        for profile in &device.profiles {
                            profile.update_installed();
                        }
                    }
                }
                for (_, devices) in &mut self.usb_devices {
                    for device in devices {
                        for profile in &device.profiles {
                            profile.update_installed();
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
        }
    }

    pub fn view(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        if self.is_loading {
            container(
                column![
                    text("Loading device manager...").size(16),
                    Space::with_height(Length::Fixed(10.0)),
                    text(&self.loading_message).size(14).style(iced::theme::Text::Color(theme.secondary_text())),
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
                text("← Back").size(14)
            )
            .on_press(Message::ClearError)
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: false,
            })))
            .padding(Padding::new(10.0));
            
            container(
                column![
                    back_button,
                    Space::with_height(Length::Fixed(20.0)),
                    text("Error").size(18).style(iced::theme::Text::Color(theme.danger())),
                    Space::with_height(Length::Fixed(10.0)),
                    text(err).size(14),
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
                    text("No devices found").size(16),
                    Space::with_height(Length::Fixed(10.0)),
                    button("Load Devices")
                        .on_press(Message::LoadDevices)
                        .padding(Padding::new(12.0))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
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
            // Main view with sidebar and content
            self.view_main(theme)
        }
    }

    fn view_main(&self, theme: &crate::gui::Theme) -> Element<'_, Message> {
        let material_font = crate::gui::fonts::get_material_symbols_font();

        // Sidebar with categories
        let sidebar = self.view_sidebar(theme, &material_font);
        
        // Main content area
        let content = self.view_content(theme, &material_font);

        container(
            row![
                container(sidebar).width(Length::Fixed(250.0)),
                Space::with_width(Length::Fixed(1.0)),
                container(content).width(Length::Fill),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }

    fn view_sidebar(&self, theme: &crate::gui::Theme, material_font: &iced::Font) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        
        let mut sidebar_items = column![].spacing(4);

        // Download Profiles Button
        let download_button_text = if self.is_loading && self.loading_message.contains("Downloading") {
            row![
                text(glyphs::REFRESH_SYMBOL).font(*material_font).size(16),
                text(" Downloading...").size(13),
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        } else {
            row![
                text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(16),
                text(" Download Profiles").size(13),
            ]
            .spacing(8)
            .align_items(Alignment::Center)
        };
        
        let download_button = if self.is_loading && self.loading_message.contains("Downloading") {
            button(download_button_text)
                .width(Length::Fill)
                .padding(Padding::new(10.0))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
        } else {
            button(download_button_text)
                .on_press(Message::DownloadProfiles)
                .width(Length::Fill)
                .padding(Padding::new(10.0))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                })))
        };

        sidebar_items = sidebar_items.push(download_button);
        sidebar_items = sidebar_items.push(Space::with_height(Length::Fixed(20.0)));

        // PCI Devices Section
        sidebar_items = sidebar_items.push(
            text("PCI Devices")
                .size(14)
                .style(iced::theme::Text::Color(theme.secondary_text()))
                .width(Length::Fill)
        );

        for (class, _devices) in &self.pci_devices {
            let class_name = get_pci_class_name(class);
            let is_selected = self.selected_category.as_ref()
                .map(|(t, c)| *t == CategoryType::Pci && c == class)
                .unwrap_or(false);
            
            let class_button = button(
                row![
                    text(glyphs::SETTINGS_SYMBOL).font(*material_font).size(16),
                    text(&class_name).size(13),
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .on_press(Message::SelectCategory(CategoryType::Pci, class.clone()))
            .width(Length::Fill)
            .padding(Padding::new(10.0))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: is_selected,
            })));

            sidebar_items = sidebar_items.push(class_button);
        }

        sidebar_items = sidebar_items.push(Space::with_height(Length::Fixed(20.0)));

        // USB Devices Section
        sidebar_items = sidebar_items.push(
            text("USB Devices")
                .size(14)
                .style(iced::theme::Text::Color(theme.secondary_text()))
                .width(Length::Fill)
        );

        for (class, _devices) in &self.usb_devices {
            let class_name = get_usb_class_name(class);
            let is_selected = self.selected_category.as_ref()
                .map(|(t, c)| *t == CategoryType::Usb && c == class)
                .unwrap_or(false);
            
            let class_button = button(
                row![
                    text(glyphs::SETTINGS_SYMBOL).font(*material_font).size(16),
                    text(&class_name).size(13),
                ]
                .spacing(8)
                .align_items(Alignment::Center)
            )
            .on_press(Message::SelectCategory(CategoryType::Usb, class.clone()))
            .width(Length::Fill)
            .padding(Padding::new(10.0))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: is_selected,
            })));

            sidebar_items = sidebar_items.push(class_button);
        }

        container(
            scrollable(sidebar_items)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(SidebarStyle)))
        .padding(10)
        .into()
    }

    fn view_content(&self, theme: &crate::gui::Theme, material_font: &iced::Font) -> Element<'_, Message> {
        if let Some((cat_type, class)) = &self.selected_category {
            if let Some((dev_type, _, device_idx)) = &self.selected_device {
                // Show device details
                self.view_device_details(theme, material_font, *dev_type, class, *device_idx)
            } else {
                // Show device list for category
                self.view_device_list(theme, material_font, *cat_type, class)
            }
        } else {
            // Welcome/empty state
            container(
                column![
                    text("Device Manager").size(24).style(iced::theme::Text::Color(theme.primary())),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Select a device category from the sidebar to get started.")
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
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

    fn view_device_list(&self, theme: &crate::gui::Theme, _material_font: &iced::Font, cat_type: CategoryType, class: &str) -> Element<'_, Message> {
        match cat_type {
            CategoryType::Pci => {
                let devices: Vec<_> = self.pci_devices.iter()
                    .find(|(c, _)| c == class)
                    .map(|(_, devices)| devices.iter().enumerate().collect::<Vec<_>>())
                    .unwrap_or_default();

                if devices.is_empty() {
                    return container(
                        text("No devices found in this category")
                            .size(14)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
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
                            DeviceStatus::ActiveEnabled => theme.primary(),
                            DeviceStatus::ActiveDisabled => iced::Color::from_rgb(0.2, 0.5, 0.9),
                            DeviceStatus::InactiveEnabled => iced::Color::from_rgb(0.2, 0.9, 0.2),
                            DeviceStatus::InactiveDisabled => theme.danger(),
                        };

                        button(
                            container(
                                column![
                                    row![
                                        text(&name).size(16).width(Length::Fill),
                                        container(
                                            Space::with_width(Length::Fixed(12.0))
                                                .height(Length::Fixed(12.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                            color: status_color,
                                        }))),
                                    ]
                                    .spacing(10)
                                    .width(Length::Fill)
                                    .align_items(Alignment::Center),
                                    Space::with_height(Length::Fixed(8.0)),
                                    text(format!("Bus ID: {}", bus_id))
                                        .size(12)
                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                ]
                                .spacing(4)
                                .padding(16)
                                .width(Length::Fill)
                            )
                            .style(iced::theme::Container::Custom(Box::new(DeviceCardStyle)))
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
                            .size(14)
                            .style(iced::theme::Text::Color(theme.secondary_text()))
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
                            DeviceStatus::ActiveEnabled => theme.primary(),
                            DeviceStatus::ActiveDisabled => iced::Color::from_rgb(0.2, 0.5, 0.9),
                            DeviceStatus::InactiveEnabled => iced::Color::from_rgb(0.2, 0.9, 0.2),
                            DeviceStatus::InactiveDisabled => theme.danger(),
                        };

                        button(
                            container(
                                column![
                                    row![
                                        text(&name).size(16).width(Length::Fill),
                                        container(
                                            Space::with_width(Length::Fixed(12.0))
                                                .height(Length::Fixed(12.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                            color: status_color,
                                        }))),
                                    ]
                                    .spacing(10)
                                    .width(Length::Fill)
                                    .align_items(Alignment::Center),
                                    Space::with_height(Length::Fixed(8.0)),
                                    text(format!("Bus ID: {}", bus_id))
                                        .size(12)
                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                ]
                                .spacing(4)
                                .padding(16)
                                .width(Length::Fill)
                            )
                            .style(iced::theme::Container::Custom(Box::new(DeviceCardStyle)))
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
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    fn view_device_details(&self, theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        
        // Get the device
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

        // Back button
        let back_button = button(
            row![
                text(glyphs::CLOSE_SYMBOL).font(*material_font).size(16),
                text(" Back").size(13),
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .on_press(Message::BackToDeviceList)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(10.0));

        // Device title
        let device_title = text(&device_name)
            .size(20)
            .style(iced::theme::Text::Color(theme.primary()))
            .width(Length::Fill);

        // Status indicator
        let status_color = match status {
            DeviceStatus::ActiveEnabled => theme.primary(),
            DeviceStatus::ActiveDisabled => iced::Color::from_rgb(0.2, 0.5, 0.9),
            DeviceStatus::InactiveEnabled => iced::Color::from_rgb(0.2, 0.9, 0.2),
            DeviceStatus::InactiveDisabled => theme.danger(),
        };

        let status_indicator = container(
            Space::with_width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0))
        )
        .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
            color: status_color,
        })));

        // Status badges
        let status_badges = self.view_status_badges(theme, &device_info);

        // Control buttons
        let control_buttons = self.view_control_buttons(theme, material_font, dev_type, class, device_idx, &device_info);

        // Profiles section
        let profiles_section = match (profiles_pci, profiles_usb) {
            (Some(p), None) => self.view_profiles_section_pci(theme, material_font, dev_type, class, device_idx, p),
            (None, Some(p)) => self.view_profiles_section_usb(theme, material_font, dev_type, class, device_idx, p),
            _ => self.view_error("No profiles available"),
        };

        container(
            scrollable(
                column![
                    row![
                        back_button,
                        Space::with_width(Length::Fill),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(20.0)),
                    row![
                        device_title,
                        status_indicator,
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(20.0)),
                    status_badges,
                    Space::with_height(Length::Fixed(20.0)),
                    control_buttons,
                    Space::with_height(Length::Fixed(20.0)),
                    profiles_section,
                ]
                .spacing(10)
                .padding(20)
                .width(Length::Fill)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_error(&self, msg: &str) -> Element<'_, Message> {
        container(
            text(msg).size(14)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn view_status_badges(&self, theme: &crate::gui::Theme, device_info: &DeviceInfo) -> Element<'_, Message> {
        let mut badges = column![].spacing(10);

        match device_info {
            DeviceInfo::Pci { started, enabled, driver, driver_version, bus_id, vendor_id, device_id, .. } => {
                badges = badges.push(self.create_status_badge(theme, "Started", if *started { "Yes" } else { "No" }, *started));
                badges = badges.push(self.create_status_badge(theme, "Enabled", if *enabled { "Yes" } else { "No" }, *enabled));
                badges = badges.push(self.create_info_badge(theme, "Driver", driver));
                if !driver_version.is_empty() {
                    badges = badges.push(self.create_info_badge(theme, "Driver Version", driver_version));
                }
                badges = badges.push(self.create_info_badge(theme, "Bus ID", bus_id));
                badges = badges.push(self.create_info_badge(theme, "Vendor ID", vendor_id));
                badges = badges.push(self.create_info_badge(theme, "Device ID", device_id));
            }
            DeviceInfo::Usb { started, enabled, driver, driver_version, bus_id, vendor_id, product_id, .. } => {
                badges = badges.push(self.create_status_badge(theme, "Started", if *started { "Yes" } else { "No" }, *started));
                badges = badges.push(self.create_status_badge(theme, "Enabled", if *enabled { "Yes" } else { "No" }, *enabled));
                badges = badges.push(self.create_info_badge(theme, "Driver", driver));
                if !driver_version.is_empty() {
                    badges = badges.push(self.create_info_badge(theme, "Driver Version", driver_version));
                }
                badges = badges.push(self.create_info_badge(theme, "Bus ID", bus_id));
                badges = badges.push(self.create_info_badge(theme, "Vendor ID", vendor_id));
                badges = badges.push(self.create_info_badge(theme, "Product ID", product_id));
            }
        }

        container(
            badges
        )
        .width(Length::Fill)
        .padding(16)
        .style(iced::theme::Container::Custom(Box::new(StatusBadgeContainerStyle)))
        .into()
    }

    fn create_status_badge(&self, theme: &crate::gui::Theme, label: &str, value: &str, is_positive: bool) -> Element<'_, Message> {
        container(
            row![
                text(label)
                    .size(12)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
                    .width(Length::Fixed(100.0)),
                text(value)
                    .size(12)
                    .style(iced::theme::Text::Color(if is_positive { theme.primary() } else { theme.danger() })),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fill)
        )
        .padding(12)
        .style(iced::theme::Container::Custom(Box::new(BadgeStyle {
            is_positive,
        })))
        .width(Length::Fill)
        .into()
    }

    fn create_info_badge(&self, theme: &crate::gui::Theme, label: &str, value: &str) -> Element<'_, Message> {
        container(
            row![
                text(label)
                    .size(12)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
                    .width(Length::Fixed(100.0)),
                text(value)
                    .size(12),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fill)
        )
        .padding(12)
        .style(iced::theme::Container::Custom(Box::new(InfoBadgeStyle)))
        .width(Length::Fill)
        .into()
    }

    fn view_control_buttons(&self, _theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, device_info: &DeviceInfo) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        
        let (_started, _enabled) = match device_info {
            DeviceInfo::Pci { started, enabled, .. } => (*started, *enabled),
            DeviceInfo::Usb { started, enabled, .. } => (*started, *enabled),
        };

        let start_button = button(
            text(glyphs::REFRESH_SYMBOL).font(*material_font).size(20)
        )
        .on_press(Message::StartDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(12.0))
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0));

        let stop_button = button(
            text(glyphs::CLOSE_SYMBOL).font(*material_font).size(20)
        )
        .on_press(Message::StopDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(12.0))
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0));

        let enable_button = button(
            text(glyphs::CHECK_SYMBOL).font(*material_font).size(20)
        )
        .on_press(Message::EnableDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: true,
        })))
        .padding(Padding::new(12.0))
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0));

        let disable_button = button(
            text(glyphs::CANCEL_SYMBOL).font(*material_font).size(20)
        )
        .on_press(Message::DisableDevice(dev_type, class.to_string(), device_idx))
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
            is_primary: false,
        })))
        .padding(Padding::new(12.0))
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0));

        container(
            row![
                start_button,
                stop_button,
                enable_button,
                disable_button,
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .center_x()
        .into()
    }

    fn view_profiles_section_pci(&self, theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, profiles: Vec<Arc<PreCheckedPciProfile>>) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        
        if profiles.is_empty() {
            return container(
                text("No profiles available for this device")
                    .size(14)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
            )
            .width(Length::Fill)
            .padding(20)
            .into();
        }

        let mut profile_cards = column![].spacing(10);

        // Sort profiles - NVIDIA profiles first (always), then by priority
        let mut sorted_profiles = profiles;
        sorted_profiles.sort_by(|a, b| {
            let a_profile = a.profile();
            let b_profile = b.profile();
            let a_is_nvidia = a_profile.vendor_ids.contains(&"10de".to_string());
            let b_is_nvidia = b_profile.vendor_ids.contains(&"10de".to_string());
            
            // NVIDIA profiles always first, regardless of device type
            match (a_is_nvidia, b_is_nvidia) {
                (true, false) => std::cmp::Ordering::Less,  // NVIDIA before non-NVIDIA
                (false, true) => std::cmp::Ordering::Greater, // non-NVIDIA after NVIDIA
                (true, true) => {
                    // Both NVIDIA - sort by priority (higher priority first)
                    b_profile.priority.cmp(&a_profile.priority)
                }
                (false, false) => {
                    // Neither NVIDIA - sort by priority (higher priority first)
                    b_profile.priority.cmp(&a_profile.priority)
                }
            }
        });

        for (profile_idx, profile) in sorted_profiles.iter().enumerate() {
            let profile_data = profile.profile();
            let is_installed = profile.installed();
            
            let install_button = if is_installed {
                button(
                    row![
                        text(glyphs::CHECK_SYMBOL).font(*material_font).size(16),
                        text(" Installed").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(10.0))
            } else {
                button(
                    row![
                        text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(16),
                        text(" Install").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::InstallProfile(dev_type, class.to_string(), device_idx, profile_data.codename.clone()))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                })))
                .padding(Padding::new(10.0))
            };

            let remove_button = if profile_data.removable && is_installed {
                button(
                    row![
                        text(glyphs::DELETE_SYMBOL).font(*material_font).size(16),
                        text(" Remove").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::RemoveProfile(dev_type, class.to_string(), device_idx, profile_idx))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(10.0))
            } else {
                button(
                    row![
                        text(glyphs::DELETE_SYMBOL).font(*material_font).size(16),
                        text(" Remove").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(10.0))
            };

            let experimental_badge: Element<Message> = if profile_data.experimental {
                container(
                    text("Experimental")
                        .size(11)
                        .style(iced::theme::Text::Color(theme.danger()))
                )
                .padding(Padding::from([4.0, 8.0, 4.0, 8.0]))
                .style(iced::theme::Container::Custom(Box::new(ExperimentalBadgeStyle)))
                .into()
            } else {
                Space::with_width(Length::Shrink).into()
            };

            let profile_card = container(
                column![
                    row![
                        text(&profile_data.i18n_desc)
                            .size(16)
                            .style(iced::theme::Text::Color(theme.primary()))
                            .width(Length::Fill),
                        {
                            let indicator: Element<Message> = if is_installed {
                                container(
                                    Space::with_width(Length::Fixed(8.0))
                                        .height(Length::Fixed(8.0))
                                )
                                .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                    color: theme.primary(),
                                })))
                                .into()
                            } else {
                                Space::with_width(Length::Shrink).into()
                            };
                            indicator
                        },
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(8.0)),
                    text(&profile_data.codename)
                        .size(12)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    {
                        // Use cached driver version (loaded asynchronously) - extract clean version
                        let driver_version_display = if let Some(driver_version) = profile.driver_version() {
                            if !driver_version.is_empty() {
                                // For repository profiles, prefer version from codename if available
                                let cleaned = if profile_data.codename.starts_with("nvidia-") {
                                    // Extract from codename first (most reliable for repo profiles)
                                    if let Some(version_from_codename) = profile_data.codename.strip_prefix("nvidia-") {
                                        let clean: String = version_from_codename
                                            .chars()
                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                            .collect();
                                        if !clean.is_empty() && clean.matches('.').count() >= 1 {
                                            clean
                                        } else {
                                            // Fallback: clean up concatenated versions
                                            if driver_version.contains("580") && driver_version.contains("390") {
                                                // Multiple versions concatenated - extract the one matching the profile
                                                if profile_data.codename.contains("580") {
                                                    // Try to extract 580.x.x.x
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
                                                    // Try to extract 390.x.x
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
                                                // Single version - just clean it
                                                driver_version
                                                    .chars()
                                                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                    .collect()
                                            }
                                        }
                                    } else {
                                        // Fallback: clean up concatenated versions
                                        if driver_version.contains("580") && driver_version.contains("390") {
                                            // Multiple versions concatenated - extract the one matching the profile
                                            if profile_data.codename.contains("580") {
                                                // Try to extract 580.x.x.x
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
                                                // Try to extract 390.x.x
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
                                            // Single version - just clean it
                                            driver_version
                                                .chars()
                                                .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                .collect()
                                        }
                                    }
                                } else if profile_data.codename.starts_with("mesa-") {
                                    // Extract from Mesa codename
                                    if let Some(version_from_codename) = profile_data.codename.strip_prefix("mesa-") {
                                        let clean: String = version_from_codename
                                            .chars()
                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                            .collect();
                                        if !clean.is_empty() && clean.matches('.').count() >= 1 {
                                            clean
                                        } else {
                                            // Fallback: clean the version string
                                            driver_version
                                                .chars()
                                                .take_while(|c| c.is_ascii_digit() || *c == '.')
                                                .collect()
                                        }
                                    } else {
                                        // Fallback: clean the version string
                                        driver_version
                                            .chars()
                                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                                            .collect()
                                    }
                                } else {
                                    // For other profiles, just clean the version
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
                                    .size(11)
                                    .style(iced::theme::Text::Color(theme.primary()))
                            );
                        }
                        
                        // Show repository if available
                        if let Some(repo) = profile.repository() {
                            info_rows = info_rows.push(
                                text(format!("Repository: {}", repo))
                                    .size(11)
                                    .style(iced::theme::Text::Color(theme.secondary_text()))
                            );
                        }
                        
                        // Show package size if available
                        if let Some(size) = profile.package_size() {
                            info_rows = info_rows.push(
                                text(format!("Total Size: {}", size))
                                    .size(11)
                                    .style(iced::theme::Text::Color(theme.secondary_text()))
                            );
                        }
                        
                        // Show dependencies count if available
                        if let Some(deps) = profile.dependencies() {
                            if !deps.is_empty() {
                                info_rows = info_rows.push(
                                    text(format!("Dependencies: {} packages", deps.len()))
                                        .size(11)
                                        .style(iced::theme::Text::Color(theme.secondary_text()))
                                );
                            }
                        }
                        
                        info_rows.width(Length::Fill)
                    },
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        text(format!("License: {}", profile_data.license))
                            .size(11)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_width(Length::Fill),
                        experimental_badge,
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(12.0)),
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
            .style(iced::theme::Container::Custom(Box::new(ProfileCardStyle)))
            .width(Length::Fill);

            profile_cards = profile_cards.push(profile_card);
        }

        container(
            column![
                text("Available Profiles")
                    .size(18)
                    .style(iced::theme::Text::Color(theme.primary()))
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

    fn view_profiles_section_usb(&self, theme: &crate::gui::Theme, material_font: &iced::Font, dev_type: DeviceType, class: &str, device_idx: usize, profiles: Vec<Arc<PreCheckedUsbProfile>>) -> Element<'_, Message> {
        use crate::gui::fonts::glyphs;
        
        if profiles.is_empty() {
            return container(
                text("No profiles available for this device")
                    .size(14)
                    .style(iced::theme::Text::Color(theme.secondary_text()))
            )
            .width(Length::Fill)
            .padding(20)
            .into();
        }

        let mut profile_cards = column![].spacing(10);

        // Sort profiles by priority
        let mut sorted_profiles = profiles;
        sorted_profiles.sort_by_key(|p| p.profile().priority);

        for (profile_idx, profile) in sorted_profiles.iter().enumerate() {
            let profile_data = profile.profile();
            let is_installed = profile.installed();
            
            let install_button = if is_installed {
                button(
                    row![
                        text(glyphs::CHECK_SYMBOL).font(*material_font).size(16),
                        text(" Installed").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(10.0))
            } else {
                button(
                    row![
                        text(glyphs::DOWNLOAD_SYMBOL).font(*material_font).size(16),
                        text(" Install").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::InstallProfile(dev_type, class.to_string(), device_idx, profile_data.codename.clone()))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                })))
                .padding(Padding::new(10.0))
            };

            let remove_button = if profile_data.removable && is_installed {
                button(
                    row![
                        text(glyphs::DELETE_SYMBOL).font(*material_font).size(16),
                        text(" Remove").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .on_press(Message::RemoveProfile(dev_type, class.to_string(), device_idx, profile_idx))
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(10.0))
            } else {
                button(
                    row![
                        text(glyphs::DELETE_SYMBOL).font(*material_font).size(16),
                        text(" Remove").size(13),
                    ]
                    .spacing(4)
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                })))
                .padding(Padding::new(10.0))
            };

            let experimental_badge: Element<Message> = if profile_data.experimental {
                container(
                    text("Experimental")
                        .size(11)
                        .style(iced::theme::Text::Color(theme.danger()))
                )
                .padding(Padding::from([4.0, 8.0, 4.0, 8.0]))
                .style(iced::theme::Container::Custom(Box::new(ExperimentalBadgeStyle)))
                .into()
            } else {
                Space::with_width(Length::Shrink).into()
            };

            let profile_card = container(
                column![
                    row![
                        text(&profile_data.i18n_desc)
                            .size(16)
                            .style(iced::theme::Text::Color(theme.primary()))
                            .width(Length::Fill),
                        {
                            let indicator: Element<Message> = if is_installed {
                                container(
                                    Space::with_width(Length::Fixed(8.0))
                                        .height(Length::Fixed(8.0))
                                )
                                .style(iced::theme::Container::Custom(Box::new(StatusIndicatorStyle {
                                    color: theme.primary(),
                                })))
                                .into()
                            } else {
                                Space::with_width(Length::Shrink).into()
                            };
                            indicator
                        },
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(8.0)),
                    text(&profile_data.codename)
                        .size(12)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    {
                        // Use cached driver version (loaded asynchronously)
                        if let Some(driver_version) = profile.driver_version() {
                            if !driver_version.is_empty() {
                                row![
                                    text(format!("Driver Version: {}", driver_version))
                                        .size(11)
                                        .style(iced::theme::Text::Color(theme.primary())),
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
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        text(format!("License: {}", profile_data.license))
                            .size(11)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_width(Length::Fill),
                        experimental_badge,
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center)
                    .width(Length::Fill),
                    Space::with_height(Length::Fixed(12.0)),
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
            .style(iced::theme::Container::Custom(Box::new(ProfileCardStyle)))
            .width(Length::Fill);

            profile_cards = profile_cards.push(profile_card);
        }

        container(
            column![
                text("Available Profiles")
                    .size(18)
                    .style(iced::theme::Text::Color(theme.primary()))
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

// Helper functions
// Load driver versions and installed status for all profiles asynchronously
async fn load_profile_versions(pci_profiles: Vec<Arc<PreCheckedPciProfile>>, usb_profiles: Vec<Arc<PreCheckedUsbProfile>>) -> () {
    use tokio::task;
    
    // Load PCI profile versions and installed status in parallel
    let pci_handles: Vec<_> = pci_profiles.into_iter().map(|profile| {
        let profile_clone = profile.clone();
        task::spawn_blocking(move || {
            // Update installed status (this can be slow, so do it in background)
            profile_clone.update_installed();
            // Extract driver version
            let version = extract_driver_version_sync(&profile_clone.profile());
            profile_clone.set_driver_version(Some(version));
        })
    }).collect();
    
    // Load USB profile versions and installed status in parallel
    let usb_handles: Vec<_> = usb_profiles.into_iter().map(|profile| {
        let profile_clone = profile.clone();
        task::spawn_blocking(move || {
            // Update installed status (this can be slow, so do it in background)
            profile_clone.update_installed();
            // Extract driver version
            let version = extract_driver_version_sync_usb(&profile_clone.profile());
            profile_clone.set_driver_version(Some(version));
        })
    }).collect();
    
    // Wait for all to complete (but don't block UI)
    for handle in pci_handles {
        let _ = handle.await;
    }
    for handle in usb_handles {
        let _ = handle.await;
    }
}

// Extract driver version synchronously (called from background thread)
fn extract_driver_version_sync(profile: &CfhdbPciProfile) -> String {
    // For repository-based NVIDIA profiles, extract from codename/description first
    if profile.vendor_ids.contains(&"10de".to_string()) && profile.priority == 100 {
        // Try to extract version from codename (format: "nvidia-580.95.05" or "nvidia-390.157")
        if let Some(version_part) = profile.codename.strip_prefix("nvidia-") {
            // Check if it's a valid version (has at least one dot and digits)
            if version_part.matches('.').count() >= 1 && version_part.chars().any(|c| c.is_ascii_digit()) {
                // Extract just the version part (stop at first non-digit/non-dot character)
                let clean_version: String = version_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if !clean_version.is_empty() {
                    return clean_version;
                }
            }
        }
        // If not found, try to extract from description
        if profile.i18n_desc.contains("Driver ") {
            // Look for pattern like "NVIDIA Graphics Driver 580.95.05" or "NVIDIA Graphics Driver 390.157"
            let parts: Vec<&str> = profile.i18n_desc.split("Driver ").collect();
            if parts.len() > 1 {
                let version_part = parts[1].trim();
                // Extract just the version part (stop at first non-digit/non-dot character)
                let clean_version: String = version_part
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                // Check if it looks like a version (contains dots and numbers)
                if clean_version.matches('.').count() >= 1 && clean_version.chars().any(|c| c.is_ascii_digit()) {
                    return clean_version;
                }
            }
        }
    }
    
    // For repository-based Mesa profiles, extract from codename/description
    if profile.codename.starts_with("mesa-") && profile.priority == 90 {
        // Try to extract version from codename (format: "mesa-25.2.4")
        if let Some(version_part) = profile.codename.strip_prefix("mesa-") {
            // Extract just the version part (stop at first non-digit/non-dot character)
            let clean_version: String = version_part
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                return clean_version;
            }
        }
        // If not found, try to extract from description
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
    
    // Try to extract from package names first (fast, no blocking)
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

// Fast version extraction (no blocking I/O, just parsing)
// NOTE: This function should NOT call get_package_version() as it's supposed to be non-blocking
fn extract_driver_version_from_packages_fast(packages: &Option<Vec<String>>) -> String {
    if let Some(pkgs) = packages {
        for pkg in pkgs {
            // Look for patterns like "nvidia-driver-580", "nvidia-driver-580.95", etc.
            if pkg.contains("nvidia-driver") {
                if let Some(version_part) = pkg.strip_prefix("nvidia-driver-") {
                    if version_part.chars().any(|c| c.is_ascii_digit()) {
                        if version_part.contains('.') && version_part.matches('.').count() >= 2 {
                            return version_part.to_string();
                        }
                        if let Some(major_version) = version_part.split('.').next() {
                            if major_version.chars().all(|c| c.is_ascii_digit()) {
                                // Return major version only (full version will be queried separately if needed)
                                return major_version.to_string();
                            }
                        }
                    }
                }
            }
            if pkg.contains("akmod-nvidia") {
                // Try to extract from package name (format: "akmod-nvidia-580.95.05")
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

// Fast version extraction from install script (no blocking I/O, just parsing)
// NOTE: This function should NOT call get_package_version() as it's supposed to be non-blocking
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
                                        // Return major version only (full version will be queried separately if needed)
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


// Get package version from package manager (dnf/rpm)
// This uses std::thread to avoid blocking the async runtime
// Returns the version that would be installed (not just what's available)
#[allow(dead_code)]
fn get_package_version(package_name: &str) -> Result<String, ()> {
    let pkg = package_name.to_string();
    // Use a thread to run the blocking command
    let handle = std::thread::spawn(move || {
        // First, try dnf repoquery to get the exact version that would be installed
        // This is the most accurate method as it respects repository priorities
        if let Ok(output) = std::process::Command::new("dnf")
            .args(["repoquery", "--qf", "%{VERSION}", "--whatprovides", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(versions) = String::from_utf8(output.stdout) {
                    // Get the first (highest priority) version
                    if let Some(first_line) = versions.lines().next() {
                        let version = first_line.trim();
                        if !version.is_empty() && version.contains('.') {
                            // Extract just the version part (before the dash if present)
                            let version = version.split('-').next().unwrap_or(version);
                            if version.chars().any(|c| c.is_ascii_digit()) {
                                return Ok(version.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: Try dnf list to see what's available (shows what would be installed)
        if let Ok(output) = std::process::Command::new("dnf")
            .args(["list", "--available", "--quiet", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(info) = String::from_utf8(output.stdout) {
                    for line in info.lines() {
                        // Skip header lines
                        if line.contains("Available Packages") || line.contains("Installed Packages") || line.contains("Last metadata") {
                            continue;
                        }
                        // Format: "package-name.arch  version-release    repo"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            // The version-release is in the second column
                            let version_release = parts[1];
                            // Extract version part (before the dash)
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
        
        // Try dnf info as another fallback
        if let Ok(output) = std::process::Command::new("dnf")
            .args(["info", "--available", "--quiet", &pkg])
            .output()
        {
            if output.status.success() {
                if let Ok(info) = String::from_utf8(output.stdout) {
                    for line in info.lines() {
                        if line.starts_with("Version") {
                            if let Some(version) = line.split_whitespace().nth(1) {
                                // Version format is usually "580.99.99" or "580.99.99-1.fc43"
                                // Extract just the version part (before the dash if present)
                                let version = version.split('-').next().unwrap_or(version);
                                // Check if it looks like a proper version (contains dots and numbers)
                                if version.contains('.') && version.chars().any(|c| c.is_ascii_digit()) {
                                    return Ok(version.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Try rpm -q for installed packages (if already installed)
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

// Query package size and dependencies (batched for efficiency)
fn query_package_info(profile: &PreCheckedPciProfile, package_names: &[String]) {
    use std::process::Command;
    use std::collections::HashSet;
    
    if package_names.is_empty() {
        return;
    }
    
    // Query total size of all packages in one batch query
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
    
    // Query dependencies in one batch query
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
    
    // Format total size
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

// Extract repository names from install script
fn extract_repositories_from_script(install_script: &Option<String>) -> Vec<String> {
    let mut repos = Vec::new();
    if let Some(script) = install_script {
        for line in script.lines() {
            // Look for dnf config-manager commands that enable/disable repos
            if line.contains("dnf config-manager") && line.contains("setopt") {
                // Extract repo names from patterns like:
                // "dnf config-manager setopt nobara-nvidia-beta.enabled=0"
                // "dnf config-manager setopt nobara-nvidia-new-feature.enabled=1"
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in &parts {
                    if part.contains(".enabled=") {
                        // Extract repo name (before .enabled=)
                        if let Some(repo_name) = part.split(".enabled=").next() {
                            let enabled = part.contains("=1");
                            if enabled {
                                repos.push(repo_name.to_string());
                            }
                        }
                    }
                }
            }
            // Also look for dnf install commands that might reference repos
            if line.contains("dnf install") && line.contains("--enablerepo") {
                // Extract repo names from --enablerepo flags
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if *part == "--enablerepo" && i + 1 < parts.len() {
                        repos.push(parts[i + 1].to_string());
                    }
                }
            }
        }
    }
    // Remove duplicates and sort
    repos.sort();
    repos.dedup();
    repos
}

// Get driver version for a given driver name
fn get_driver_version(driver: &str) -> String {
    if driver.is_empty() || driver == "none" {
        return String::new();
    }
    
    // For NVIDIA drivers, check /proc/driver/nvidia/version first
    // Format: "NVRM version: NVIDIA UNIX Open Kernel Module for x86_64  580.95.05  Release Build ..."
    if driver == "nvidia" {
        if let Ok(version) = std::fs::read_to_string("/proc/driver/nvidia/version") {
            if let Some(line) = version.lines().next() {
                // Find the version number (format: X.YY.ZZ) after "x86_64" or "Kernel Module"
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in &parts {
                    // Look for a pattern like "580.95.05" (numbers with dots)
                    if part.contains('.') && part.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        // Count dots to ensure it's a version number (at least 2 dots)
                        if part.matches('.').count() >= 1 {
                            return part.to_string();
                        }
                    }
                }
            }
        }
    }
    
    // Try to get version using modinfo
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
    
    // If modinfo version fails, try vermagic
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
                    // Extract version from vermagic (format: "5.x.x-MODULE_VERSION x86_64")
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
    // Map PCI class codes to names
    match class {
        "0300" => "VGA Compatible Controller".to_string(),
        "0200" => "Ethernet Controller".to_string(),
        "0403" => "Audio Device".to_string(),
        "0c03" => "USB Controller".to_string(),
        _ => format!("PCI Class {}", class),
    }
}

fn get_usb_class_name(class: &str) -> String {
    // Map USB class codes to names
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

// Query repositories for NVIDIA driver packages
async fn query_nvidia_driver_packages() -> Result<Vec<NvidiaDriverPackage>, String> {
    use std::process::Command;
    use tokio::task;
    
    // Run dnf repoquery in a blocking task to avoid blocking the async runtime
    let result = task::spawn_blocking(move || {
        // Query rpmfusion (free and non-free) and negativo17 repositories for NVIDIA driver packages
        // Query each package pattern in parallel using rayon for better performance
        use rayon::prelude::*;
        
        let patterns = vec!["nvidia-driver*", "akmod-nvidia*", "xorg-x11-drv-nvidia*"];
        let all_packages: Vec<String> = patterns
            .par_iter()
            .flat_map(|pattern| {
                // Query from all enabled repos first (general query)
                let mut results = Vec::new();
                
                // Query from RPM Fusion free repositories
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
                
                // Query from RPM Fusion non-free repositories
                if let Ok(output) = Command::new("dnf")
                    .args(&["repoquery", "--available", "--quiet", "--qf", "%{name}|%{version}|%{repoid}", "--enablerepo=rpmfusion-nonfree*"])
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
                
                // Query from negativo17/fedora-nvidia repositories
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
        
        // Process the combined results
        let packages: Vec<NvidiaDriverPackage> = all_packages
            .into_iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    let repo = parts[2].to_string();
                    
                    // Include packages from:
                    // - RPM Fusion free repositories (rpmfusion-free, rpmfusion-free-updates)
                    // - RPM Fusion non-free repositories (rpmfusion-nonfree, rpmfusion-nonfree-updates)
                    // - negativo17/fedora-nvidia repositories
                    let is_rpmfusion_free = repo.contains("rpmfusion-free");
                    let is_rpmfusion_nonfree = repo.contains("rpmfusion-nonfree");
                    let is_negativo17 = repo.contains("negativo17") || repo.contains("negativo") || repo.contains("fedora-nvidia");
                    
                    if is_rpmfusion_free || is_rpmfusion_nonfree || is_negativo17 {
                        // Extract driver version from package name or version
                        let driver_version = extract_nvidia_version(&name, &version);
                        
                        Some(NvidiaDriverPackage {
                            name,
                            version,
                            repo,
                            driver_version,
                        })
                    } else {
                        None
                    }
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
    // Try to extract from akmod-nvidia package version first (most accurate)
    if package_name.contains("akmod-nvidia") {
        // akmod-nvidia version format is usually like "580.95.05-1.fc40"
        // Extract the version part before the first dash
        if let Some(version) = package_version.split('-').next() {
            // Check if it's a proper version format (e.g., "580.95.05")
            if version.matches('.').count() >= 2 && version.chars().any(|c| c.is_ascii_digit()) {
                return version.to_string();
            }
        }
    }
    
    // Try to extract version from package name (e.g., "nvidia-driver-580" -> "580")
    if let Some(version_part) = package_name.strip_prefix("nvidia-driver-") {
        // If it's a major version like "580", we need to query for the full version
        if let Ok(major_version) = version_part.parse::<u32>() {
            // Try to get full version from akmod-nvidia package for this series
            // Query dnf for akmod-nvidia packages matching this major version
            if let Ok(output) = std::process::Command::new("dnf")
                .args(&["repoquery", "--available", "--quiet", "--qf", "%{VERSION}"])
                .arg("akmod-nvidia")
                .output()
            {
                if output.status.success() {
                    if let Ok(versions) = String::from_utf8(output.stdout) {
                        // Find the version that matches this major version series
                        for line in versions.lines() {
                            let version = line.trim();
                            if let Some(version_part) = version.split('-').next() {
                                // Check if version starts with the major version
                                if version_part.starts_with(&format!("{}.", major_version)) {
                                    // This is a full version like "580.95.05"
                                    if version_part.matches('.').count() >= 2 {
                                        return version_part.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Fallback: return major version if we can't find full version
            return version_part.to_string();
        }
    }
    
    // Try to extract from xorg-x11-drv-nvidia package version
    if package_name.contains("xorg-x11-drv-nvidia") {
        if let Some(version) = package_version.split('-').next() {
            if version.matches('.').count() >= 2 {
                return version.to_string();
            }
        }
    }
    
    // Fallback: use package version (extract before dash)
    package_version.split('-').next().unwrap_or(package_version).to_string()
}

// Extract Mesa version from package name and version
fn extract_mesa_version(package_name: &str, package_version: &str) -> String {
    // Mesa packages typically have versions like "25.2.4" or "25.2.6"
    // The main package is mesa-dri-drivers
    if package_name.contains("mesa-dri-drivers") || package_name.contains("mesa-libGL") || package_name.contains("mesa-vulkan-drivers") {
        // Extract version from package version (format: "25.2.4-1.fc40")
        if let Some(version) = package_version.split('-').next() {
            // Check if it's a proper version format (e.g., "25.2.4")
            if version.matches('.').count() >= 1 && version.chars().any(|c| c.is_ascii_digit()) {
                return version.to_string();
            }
        }
    }
    
    // Fallback: use package version (extract before dash)
    package_version.split('-').next().unwrap_or(package_version).to_string()
}

// Query repositories for Mesa driver packages
async fn query_mesa_driver_packages() -> Result<Vec<MesaDriverPackage>, String> {
    use std::process::Command;
    use tokio::task;
    
    // Run dnf repoquery in a blocking task to avoid blocking the async runtime
    let result = task::spawn_blocking(move || {
        // Query Mesa driver packages from all enabled repositories
        // Query each package pattern in parallel using rayon for better performance
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
        
        // Process the combined results
        let packages: Vec<MesaDriverPackage> = all_packages
            .into_iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    let repo = parts[2].to_string();
                    
                    // Include packages from all repositories (Mesa is in Fedora repos)
                    // Extract driver version from package name or version
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

// Create Mesa profiles from repository packages
// Returns (profile, repository_name, package_names)
fn create_mesa_profiles_from_repos(packages: Vec<MesaDriverPackage>) -> Vec<(CfhdbPciProfile, String, Vec<String>)> {
    let mut profiles = Vec::new();
    
    // Group packages by driver version
    let mut version_groups: HashMap<String, Vec<MesaDriverPackage>> = HashMap::new();
    for pkg in packages {
        version_groups.entry(pkg.driver_version.clone())
            .or_insert_with(Vec::new)
            .push(pkg);
    }
    
    // Create a profile for each driver version
    for (driver_version, pkgs) in version_groups {
        // Determine repository (prefer updates, then fedora)
        let repo = pkgs.iter()
            .find(|p| p.repo.contains("updates"))
            .map(|p| p.repo.clone())
            .or_else(|| pkgs.first().map(|p| p.repo.clone()))
            .unwrap_or_default();
        
        // Build install script - Mesa is in standard Fedora repos
        let package_names: Vec<String> = pkgs.iter()
            .map(|p| p.name.clone())
            .collect();
        
        // Store repository name for display (clean it up)
        let repo_display = if repo.contains("updates") {
            "Fedora Updates".to_string()
        } else if repo.contains("fedora") {
            "Fedora".to_string()
        } else {
            repo.clone()
        };
        
        let install_script = format!("dnf install -y {}", package_names.join(" "));
        
        // Create profile description with full driver version
        // Format: "Mesa Graphics Driver 25.2.4"
        let desc = if driver_version.matches('.').count() >= 2 {
            // Full version like "25.2.4"
            format!("Mesa Graphics Driver {}", driver_version)
        } else if driver_version.matches('.').count() >= 1 {
            // Partial version like "25.2"
            format!("Mesa Graphics Driver {}", driver_version)
        } else {
            format!("Mesa Graphics Driver {}", driver_version)
        };
        
        eprintln!("[DEBUG] Creating Mesa profile for driver version: {}", driver_version);
        eprintln!("[DEBUG] Packages: {:?}", package_names);
        eprintln!("[DEBUG] Repository: {}", repo);
        
        let profile = CfhdbPciProfile {
            codename: format!("mesa-{}", driver_version),
            i18n_desc: desc.clone(),
            icon_name: "mesa".to_string(),
            license: "open-source".to_string(),
            class_ids: vec!["0300".to_string(), "*".to_string()], // VGA controller or any class
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
            priority: 90, // High priority for repository-based profiles, but below NVIDIA
        };
        
        eprintln!("[DEBUG] Created Mesa profile: codename={}, desc={}, vendor_ids={:?}, class_ids={:?}", 
                  profile.codename, profile.i18n_desc, profile.vendor_ids, profile.class_ids);
        
        profiles.push((profile, repo_display, package_names));
    }
    
    profiles.sort_by_key(|(p, _, _)| p.priority);
    profiles
}

// Create NVIDIA profiles from repository packages
// Returns (profile, repository_name, package_names)
fn create_nvidia_profiles_from_repos(packages: Vec<NvidiaDriverPackage>) -> Vec<(CfhdbPciProfile, String, Vec<String>)> {
    let mut profiles = Vec::new();
    
    // Group packages by driver version directly (use the version already extracted from packages)
    // Don't query dnf here to avoid blocking - full versions will be extracted later if needed
    let mut version_groups: HashMap<String, Vec<NvidiaDriverPackage>> = HashMap::new();
    for pkg in packages {
        // Use the driver version as-is (it should already be extracted from package name/version)
        // If it's only a major version, that's fine - we'll display it as "XXX Series" or extract full version later
        version_groups.entry(pkg.driver_version.clone())
            .or_insert_with(Vec::new)
            .push(pkg);
    }
    
    // Create a profile for each driver version
    for (driver_version, pkgs) in version_groups {
        // Determine repository (prefer negativo17/fedora-nvidia, then rpmfusion)
        let repo = pkgs.iter()
            .find(|p| p.repo.contains("negativo17") || p.repo.contains("negativo") || p.repo.contains("fedora-nvidia"))
            .map(|p| p.repo.clone())
            .or_else(|| pkgs.first().map(|p| p.repo.clone()))
            .unwrap_or_default();
        
        // Build install script
        let package_names: Vec<String> = pkgs.iter()
            .map(|p| p.name.clone())
            .collect();
        
        // Store repository name for display (clean it up)
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
        
        // Build install script with removal of old NVIDIA drivers first
        // Use || true to continue even if packages don't exist (graceful removal)
        let removal_commands = "(dnf remove -y nvidia* || true) && (dnf remove -y kmod-nvidia* || true) && (dnf remove -y akmod-nvidia || true) && (dnf remove -y dkms-nvidia || true) && (dnf remove -y xorg-x11-drv-nvidia* || true) && (rm -rf /var/lib/dkms/nvidia* || true)";
        
        // Determine package list based on repository
        // negativo17 uses: nvidia-driver, nvidia-settings
        // RPM Fusion uses: akmod-nvidia (pulls in xorg-x11-drv-nvidia automatically)
        let (nvidia_packages, install_script) = if repo.contains("negativo17") || repo.contains("negativo") || repo.contains("fedora-nvidia") {
            // negativo17 repository - only add if not already present
            // According to negativo17.org documentation, installation includes:
            // dnf install nvidia-driver nvidia-driver-cuda nvidia-settings
            // This will automatically pull in akmod-nvidia and all required dependencies
            // nvidia-driver-cuda provides CUDA/NVDEC/NVENC support
            // We avoid nvidia-persistenced (not needed for normal use) and nvidia-xconfig (not required with modular X.org)
            // nvidia-gpu-firmware is NOT needed - firmware is in nvidia-kmod-common
            let packages = vec![
                "nvidia-driver",
                "nvidia-driver-cuda",
                "nvidia-settings",
            ];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && (dnf repoinfo fedora-nvidia > /dev/null 2>&1 || dnf config-manager addrepo --from-repofile=https://negativo17.org/repos/fedora-nvidia.repo) && dnf install -y {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        } else if repo.contains("rpmfusion-nonfree") {
            // RPM Fusion non-free repository
            // According to rpmfusion.org/Howto/NVIDIA, minimal installation is:
            // dnf install akmod-nvidia
            // This will automatically pull in xorg-x11-drv-nvidia and all required dependencies
            // CUDA support is optional via xorg-x11-drv-nvidia-cuda (not installed by default)
            let packages = vec![
                "akmod-nvidia",
            ];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && dnf install -y --enablerepo=rpmfusion-nonfree-updates {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        } else if repo.contains("rpmfusion-free") {
            // RPM Fusion free repository (unlikely for NVIDIA, but handle it)
            let packages = vec![
                "akmod-nvidia",
            ];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && dnf install -y --enablerepo=rpmfusion-free-updates {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        } else {
            // Fallback: try both RPM Fusion repos (assume RPM Fusion package names)
            let packages = vec![
                "akmod-nvidia",
            ];
            let install_packages = packages.join(" ");
            let script = format!(
                "{} && dnf install -y --enablerepo=rpmfusion-free-updates --enablerepo=rpmfusion-nonfree-updates {}",
                removal_commands,
                install_packages
            );
            (packages, script)
        };
        
        // Create profile description with full driver version
        // Format: "NVIDIA Graphics Driver 580.95.05" or "NVIDIA Graphics Driver 390.xx.xx"
        let desc = if driver_version.matches('.').count() >= 2 {
            // Full version like "580.95.05"
            format!("NVIDIA Graphics Driver {}", driver_version)
        } else if driver_version.chars().all(|c| c.is_ascii_digit()) {
            // Major version only like "580"
            format!("NVIDIA Graphics Driver {} Series", driver_version)
        } else {
            format!("NVIDIA Graphics Driver {}", driver_version)
        };
        
        let profile = CfhdbPciProfile {
            codename: format!("nvidia-{}", driver_version),
            i18n_desc: desc.clone(),
            icon_name: "nvidia".to_string(),
            license: "proprietary".to_string(),
            class_ids: vec!["0300".to_string(), "*".to_string()], // VGA controller or any class
            vendor_ids: vec!["10de".to_string(), "*".to_string()], // NVIDIA vendor ID or any vendor
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

// Async function to load all devices
async fn load_all_devices() -> Result<(
    Vec<(String, Vec<PreCheckedPciDevice>)>,
    Vec<(String, Vec<PreCheckedUsbDevice>)>,
    Vec<Arc<PreCheckedPciProfile>>,
    Vec<Arc<PreCheckedUsbProfile>>,
), String> {
    // Load PCI profiles
    let mut pci_profiles = load_pci_profiles().await?;
    
    // Filter out Nobara profiles for NVIDIA devices (vendor_id "10de")
    pci_profiles.retain(|p| {
        // Keep non-NVIDIA profiles
        !p.vendor_ids.contains(&"10de".to_string()) ||
        // Keep NVIDIA profiles that don't come from Nobara (check install script)
        !p.install_script.as_ref().map(|s| s.contains("nobara") || s.contains("Nobara")).unwrap_or(false)
    });
    
    // Filter out Mesa profiles from downloaded profiles (we'll use repository versions instead)
    pci_profiles.retain(|p| {
        // Remove profiles that are clearly Mesa-related from downloaded profiles
        !p.codename.to_lowercase().contains("mesa") &&
        !p.i18n_desc.to_lowercase().contains("mesa") &&
        !p.install_script.as_ref().map(|s| s.to_lowercase().contains("mesa")).unwrap_or(false)
    });
    
    // Query repositories for NVIDIA and Mesa drivers in parallel
    let (nvidia_result, mesa_result) = tokio::join!(
        query_nvidia_driver_packages(),
        query_mesa_driver_packages()
    );
    
    let mut repo_profiles_with_info: Vec<(CfhdbPciProfile, String, Vec<String>)> = Vec::new();
    
    // Process NVIDIA packages
    match nvidia_result {
        Ok(repo_packages) if !repo_packages.is_empty() => {
            let nvidia_profiles = create_nvidia_profiles_from_repos(repo_packages);
            repo_profiles_with_info.extend(nvidia_profiles);
        }
        Ok(_) => {
            // No packages found, continue
        }
        Err(e) => {
            // Log error but continue without repository profiles
            eprintln!("[WARN] Failed to query NVIDIA driver packages: {}", e);
        }
    }
    
    // Process Mesa packages
    match mesa_result {
        Ok(repo_packages) if !repo_packages.is_empty() => {
            let mesa_profiles = create_mesa_profiles_from_repos(repo_packages);
            repo_profiles_with_info.extend(mesa_profiles);
        }
        Ok(_) => {
            // No packages found, continue
        }
        Err(e) => {
            // Log error but continue without repository profiles
            eprintln!("[WARN] Failed to query Mesa driver packages: {}", e);
        }
    }
    
    // Total PCI profiles after adding repository profiles
    
    // Convert repository profiles and add them to the list
    let mut all_pci_profiles = pci_profiles;
    let mut repo_info_map: HashMap<String, (String, Vec<String>)> = HashMap::new();
    for (profile, repo_name, package_names) in repo_profiles_with_info {
        let codename = profile.codename.clone();
        repo_info_map.insert(codename, (repo_name, package_names));
        all_pci_profiles.push(profile);
    }
    
    let pci_profiles_arc: Vec<Arc<PreCheckedPciProfile>> = all_pci_profiles
        .into_iter()
        .map(|p| {
            let profile = PreCheckedPciProfile::new(p.clone());
            // Defer update_installed() to avoid blocking - will be called later in background
            // profile.update_installed();
            
            // For NVIDIA profiles from repositories, set the driver version, repository, and query package info
            if p.vendor_ids.contains(&"10de".to_string()) {
                // Try to extract version from codename (format: "nvidia-580.95.05" or "nvidia-390.157")
                if let Some(version_part) = p.codename.strip_prefix("nvidia-") {
                    // Extract clean version (stop at first non-digit/non-dot character)
                    let clean_version: String = version_part
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                        profile.set_driver_version(Some(clean_version));
                    }
                }
                // If not found, try to extract from description
                else if p.i18n_desc.contains("Driver ") {
                    // Look for pattern like "NVIDIA Graphics Driver 580.95.05" or "NVIDIA Graphics Driver 390.157"
                    let parts: Vec<&str> = p.i18n_desc.split("Driver ").collect();
                    if parts.len() > 1 {
                        let version_part = parts[1].trim();
                        // Extract clean version (stop at first non-digit/non-dot character)
                        let clean_version: String = version_part
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        // Check if it looks like a version (contains dots and numbers)
                        if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                            profile.set_driver_version(Some(clean_version));
                        }
                    }
                }
                
                // Check if this is a repository profile (high priority and has packages)
                if p.priority == 100 && p.packages.is_some() {
                    // This is a repository profile - get repo info from the map
                    if let Some((repo_name, package_names)) = repo_info_map.get(&p.codename) {
                        profile.set_repository(Some(repo_name.clone()));
                        
                        // Query package size and dependencies asynchronously
                        let profile_clone = profile.clone();
                        let pkgs_clone = package_names.clone();
                        std::thread::spawn(move || {
                            query_package_info(&profile_clone, &pkgs_clone);
                        });
                    } else {
                        // Fallback: determine repository from install script
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
                                
                                // Query package size and dependencies asynchronously
                                let profile_clone = profile.clone();
                                let pkgs_clone = pkgs.clone();
                                std::thread::spawn(move || {
                                    query_package_info(&profile_clone, &pkgs_clone);
                                });
                            }
                        }
                    }
                }
            }
            // For Mesa profiles from repositories, set the driver version, repository, and query package info
            else if p.codename.starts_with("mesa-") || p.i18n_desc.contains("Mesa") {
                // Try to extract version from codename (format: "mesa-25.2.4")
                if let Some(version_part) = p.codename.strip_prefix("mesa-") {
                    // Extract clean version (stop at first non-digit/non-dot character)
                    let clean_version: String = version_part
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                        profile.set_driver_version(Some(clean_version));
                    }
                }
                // If not found, try to extract from description
                else if p.i18n_desc.contains("Driver ") {
                    // Look for pattern like "Mesa Graphics Driver 25.2.4"
                    let parts: Vec<&str> = p.i18n_desc.split("Driver ").collect();
                    if parts.len() > 1 {
                        let version_part = parts[1].trim();
                        // Extract clean version (stop at first non-digit/non-dot character)
                        let clean_version: String = version_part
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        // Check if it looks like a version (contains dots and numbers)
                        if !clean_version.is_empty() && clean_version.matches('.').count() >= 1 {
                            profile.set_driver_version(Some(clean_version));
                        }
                    }
                }
                
                // Check if this is a repository profile (priority 90 and has packages)
                if p.priority == 90 && p.packages.is_some() {
                    // This is a repository profile - get repo info from the map
                    if let Some((repo_name, package_names)) = repo_info_map.get(&p.codename) {
                        profile.set_repository(Some(repo_name.clone()));
                        
                        // Query package size and dependencies asynchronously
                        let profile_clone = profile.clone();
                        let pkgs_clone = package_names.clone();
                        std::thread::spawn(move || {
                            query_package_info(&profile_clone, &pkgs_clone);
                        });
                    } else {
                        // Fallback: determine repository from install script
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
                                
                                // Query package size and dependencies asynchronously
                                let profile_clone = profile.clone();
                                let pkgs_clone = pkgs.clone();
                                std::thread::spawn(move || {
                                    query_package_info(&profile_clone, &pkgs_clone);
                                });
                            }
                        }
                    }
                }
            }
            
            Arc::new(profile)
        })
        .collect();

    // Load USB profiles
    let usb_profiles = load_usb_profiles().await?;
    let usb_profiles_arc: Vec<Arc<PreCheckedUsbProfile>> = usb_profiles
        .into_iter()
        .map(|p| {
            let profile = PreCheckedUsbProfile::new(p);
            // Defer update_installed() to avoid blocking - will be called later in background
            // profile.update_installed();
            Arc::new(profile)
        })
        .collect();

    // Get PCI devices
    let pci_devices = get_pci_devices(&pci_profiles_arc)
        .ok_or_else(|| "Failed to get PCI devices".to_string())?;

    // Get USB devices
    let usb_devices = get_usb_devices(&usb_profiles_arc)
        .ok_or_else(|| "Failed to get USB devices".to_string())?;

    // Convert to vectors and sort
    let mut pci_vec: Vec<(String, Vec<PreCheckedPciDevice>)> = pci_devices.into_iter().collect();
    let mut usb_vec: Vec<(String, Vec<PreCheckedUsbDevice>)> = usb_devices.into_iter().collect();

    pci_vec.sort_by(|a, b| a.0.cmp(&b.0));
    usb_vec.sort_by(|a, b| a.0.cmp(&b.0));

    Ok((pci_vec, usb_vec, pci_profiles_arc, usb_profiles_arc))
}

async fn load_pci_profiles() -> Result<Vec<CfhdbPciProfile>, String> {
    eprintln!("[DEBUG] Starting PCI profile loading...");
    let cached_db_path = Path::new("/var/cache/cfhdb/pci.json");
    
    // Try to read from cache first
    eprintln!("[DEBUG] Checking cache at: {:?}", cached_db_path);
    if cached_db_path.exists() {
        eprintln!("[DEBUG] Cache file exists, attempting to read...");
        match std::fs::read_to_string(cached_db_path) {
            Ok(data) => {
                eprintln!("[DEBUG] Successfully read cache file ({} bytes), parsing...", data.len());
                match parse_pci_profiles(&data) {
                    Ok(profiles) => {
                        eprintln!("[DEBUG] Successfully parsed {} PCI profiles from cache", profiles.len());
                        return Ok(profiles);
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to parse cached PCI profiles: {}", e);
                        eprintln!("[DEBUG] Will try to download fresh profiles...");
                    }
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to read cache file: {}", e);
            }
        }
    } else {
        eprintln!("[DEBUG] Cache file does not exist");
    }
    
    // If cache doesn't exist or is unreadable, try to download
    eprintln!("[DEBUG] Getting profile URL config...");
    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            eprintln!("[DEBUG] Profile URL config loaded: PCI={}, USB={}", url.pci_json_url, url.usb_json_url);
            url
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to read profile config: {}", e);
            return Err(format!("Failed to read profile config: {}", e));
        }
    };
    
    eprintln!("[DEBUG] Attempting to download PCI profiles from: {}", profile_url.pci_json_url);
    match reqwest::get(&profile_url.pci_json_url).await {
        Ok(response) => {
            eprintln!("[DEBUG] HTTP response status: {}", response.status());
            if response.status().is_success() {
                eprintln!("[DEBUG] Download successful, reading response text...");
                match response.text().await {
                    Ok(text) => {
                        eprintln!("[DEBUG] Downloaded {} bytes of profile data", text.len());
                        eprintln!("[DEBUG] Attempting to cache profile file...");
                        match cache_profile_file(cached_db_path, &text).await {
                            Ok(_) => eprintln!("[DEBUG] Successfully cached PCI profiles"),
                            Err(e) => eprintln!("[DEBUG] Warning: Failed to cache profiles (will continue anyway): {}", e),
                        }
                        eprintln!("[DEBUG] Parsing downloaded PCI profiles...");
                        match parse_pci_profiles(&text) {
                            Ok(profiles) => {
                                eprintln!("[DEBUG] Successfully parsed {} PCI profiles", profiles.len());
                                Ok(profiles)
                            }
                            Err(e) => {
                                eprintln!("[DEBUG] Failed to parse PCI profiles: {}", e);
                                Err(format!("Failed to parse PCI profiles: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to read response text: {}", e);
                        Err(format!("Failed to read response: {}", e))
                    }
                }
            } else {
                eprintln!("[DEBUG] HTTP error: {}", response.status());
                Err(format!("HTTP error: {}", response.status()))
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to download PCI profiles: {}", e);
            Err(format!("Failed to download PCI profiles: {}", e))
        }
    }
}

async fn load_usb_profiles() -> Result<Vec<CfhdbUsbProfile>, String> {
    eprintln!("[DEBUG] Starting USB profile loading...");
    let cached_db_path = Path::new("/var/cache/cfhdb/usb.json");
    
    // Try to read from cache first
    eprintln!("[DEBUG] Checking cache at: {:?}", cached_db_path);
    if cached_db_path.exists() {
        eprintln!("[DEBUG] Cache file exists, attempting to read...");
        match std::fs::read_to_string(cached_db_path) {
            Ok(data) => {
                eprintln!("[DEBUG] Successfully read cache file ({} bytes), parsing...", data.len());
                match parse_usb_profiles(&data) {
                    Ok(profiles) => {
                        eprintln!("[DEBUG] Successfully parsed {} USB profiles from cache", profiles.len());
                        return Ok(profiles);
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to parse cached USB profiles: {}", e);
                        eprintln!("[DEBUG] Will try to download fresh profiles...");
                    }
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to read cache file: {}", e);
            }
        }
    } else {
        eprintln!("[DEBUG] Cache file does not exist");
    }
    
    // If cache doesn't exist or is unreadable, try to download
    eprintln!("[DEBUG] Getting profile URL config...");
    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            eprintln!("[DEBUG] Profile URL config loaded: PCI={}, USB={}", url.pci_json_url, url.usb_json_url);
            url
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to read profile config: {}", e);
            return Err(format!("Failed to read profile config: {}", e));
        }
    };
    
    eprintln!("[DEBUG] Attempting to download USB profiles from: {}", profile_url.usb_json_url);
    match reqwest::get(&profile_url.usb_json_url).await {
        Ok(response) => {
            eprintln!("[DEBUG] HTTP response status: {}", response.status());
            if response.status().is_success() {
                eprintln!("[DEBUG] Download successful, reading response text...");
                match response.text().await {
                    Ok(text) => {
                        eprintln!("[DEBUG] Downloaded {} bytes of profile data", text.len());
                        eprintln!("[DEBUG] Attempting to cache profile file...");
                        match cache_profile_file(cached_db_path, &text).await {
                            Ok(_) => eprintln!("[DEBUG] Successfully cached USB profiles"),
                            Err(e) => eprintln!("[DEBUG] Warning: Failed to cache profiles (will continue anyway): {}", e),
                        }
                        eprintln!("[DEBUG] Parsing downloaded USB profiles...");
                        match parse_usb_profiles(&text) {
                            Ok(profiles) => {
                                eprintln!("[DEBUG] Successfully parsed {} USB profiles", profiles.len());
                                Ok(profiles)
                            }
                            Err(e) => {
                                eprintln!("[DEBUG] Failed to parse USB profiles: {}", e);
                                Err(format!("Failed to parse USB profiles: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to read response text: {}", e);
                        Err(format!("Failed to read response: {}", e))
                    }
                }
            } else {
                eprintln!("[DEBUG] HTTP error: {}", response.status());
                Err(format!("HTTP error: {}", response.status()))
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to download USB profiles: {}", e);
            Err(format!("Failed to download USB profiles: {}", e))
        }
    }
}

fn get_profile_url_config() -> Result<ProfileUrlConfig, std::io::Error> {
    let file_path = "/etc/cfhdb/profile-config.json";
    
    // Try to read from config file, fallback to defaults if not found
    if let Ok(json_content) = std::fs::read_to_string(file_path) {
        if let Ok(config) = serde_json::from_str::<ProfileUrlConfig>(&json_content) {
            return Ok(config);
        }
    }
    
    // Default URLs (from Nobara project)
    // Using GitHub raw content URLs
    Ok(ProfileUrlConfig {
        pci_json_url: "https://raw.githubusercontent.com/Nobara-Project/cfhdb/refs/heads/master/data/profiles/pci.json".to_string(),
        usb_json_url: "https://raw.githubusercontent.com/Nobara-Project/cfhdb/refs/heads/master/data/profiles/usb.json".to_string(),
    })
}

// Check if profiles need to be updated (check once per day)
fn profiles_need_update(cached_path: &Path) -> bool {
    eprintln!("[DEBUG] Checking if profile needs update: {:?}", cached_path);
    if !cached_path.exists() {
        eprintln!("[DEBUG] Profile file does not exist, needs update");
        return true;
    }
    
    // Check file modification time - update if older than 24 hours
    match std::fs::metadata(cached_path) {
        Ok(metadata) => {
            match metadata.modified() {
                Ok(modified) => {
                    match modified.elapsed() {
                        Ok(elapsed) => {
                            let hours = elapsed.as_secs() / 3600;
                            let needs_update = elapsed.as_secs() > 24 * 60 * 60;
                            eprintln!("[DEBUG] Profile file age: {} hours, needs update: {}", hours, needs_update);
                            needs_update
                        }
                        Err(e) => {
                            eprintln!("[DEBUG] Failed to calculate elapsed time: {}", e);
                            true // If we can't determine age, update to be safe
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DEBUG] Failed to get modification time: {}", e);
                    true // If we can't get mod time, update to be safe
                }
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to get file metadata: {}", e);
            true // If we can't get metadata, update to be safe
        }
    }
}

// Ensure profiles are cached, downloading if needed (force download)
async fn ensure_profiles_cached_force() -> Result<(), String> {
    eprintln!("[DEBUG] ensure_profiles_cached_force() called - forcing download");
    
    let pci_path = Path::new("/var/cache/cfhdb/pci.json");
    let usb_path = Path::new("/var/cache/cfhdb/usb.json");
    
    eprintln!("[DEBUG] Getting profile URL config...");
    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            eprintln!("[DEBUG] Profile URLs: PCI={}, USB={}", url.pci_json_url, url.usb_json_url);
            url
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to get profile URL config: {}", e);
            return Err(format!("Failed to read profile config: {}", e));
        }
    };
    
    // Force download PCI profiles
    eprintln!("[DEBUG] Force downloading PCI profiles...");
    match reqwest::get(&profile_url.pci_json_url).await {
        Ok(response) => {
            eprintln!("[DEBUG] PCI download response status: {}", response.status());
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        eprintln!("[DEBUG] PCI profile downloaded ({} bytes), caching...", text.len());
                        match cache_profile_file(pci_path, &text).await {
                            Ok(_) => eprintln!("[DEBUG] PCI profiles cached successfully"),
                            Err(e) => {
                                eprintln!("[DEBUG] Failed to cache PCI profiles: {}", e);
                                return Err(format!("Failed to cache PCI profiles: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to read PCI profile response text: {}", e);
                        return Err(format!("Failed to read PCI profile response: {}", e));
                    }
                }
            } else {
                eprintln!("[DEBUG] PCI profile download failed with status: {}", response.status());
                return Err(format!("Failed to download PCI profiles: HTTP {}", response.status()));
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to download PCI profiles: {}", e);
            return Err(format!("Failed to download PCI profiles: {}", e));
        }
    }
    
    // Force download USB profiles
    eprintln!("[DEBUG] Force downloading USB profiles...");
    match reqwest::get(&profile_url.usb_json_url).await {
        Ok(response) => {
            eprintln!("[DEBUG] USB download response status: {}", response.status());
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        eprintln!("[DEBUG] USB profile downloaded ({} bytes), caching...", text.len());
                        match cache_profile_file(usb_path, &text).await {
                            Ok(_) => eprintln!("[DEBUG] USB profiles cached successfully"),
                            Err(e) => {
                                eprintln!("[DEBUG] Failed to cache USB profiles: {}", e);
                                return Err(format!("Failed to cache USB profiles: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Failed to read USB profile response text: {}", e);
                        return Err(format!("Failed to read USB profile response: {}", e));
                    }
                }
            } else {
                eprintln!("[DEBUG] USB profile download failed with status: {}", response.status());
                return Err(format!("Failed to download USB profiles: HTTP {}", response.status()));
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to download USB profiles: {}", e);
            return Err(format!("Failed to download USB profiles: {}", e));
        }
    }
    
    eprintln!("[DEBUG] ensure_profiles_cached_force() completed successfully");
    Ok(())
}

// Ensure profiles are cached, downloading if needed
async fn ensure_profiles_cached() -> Result<(), String> {
    eprintln!("[DEBUG] ensure_profiles_cached() called");
    
    let pci_path = Path::new("/var/cache/cfhdb/pci.json");
    let usb_path = Path::new("/var/cache/cfhdb/usb.json");
    
    eprintln!("[DEBUG] Getting profile URL config...");
    let profile_url = match get_profile_url_config() {
        Ok(url) => {
            eprintln!("[DEBUG] Profile URLs: PCI={}, USB={}", url.pci_json_url, url.usb_json_url);
            url
        }
        Err(e) => {
            eprintln!("[DEBUG] Failed to get profile URL config: {}", e);
            return Err(format!("Failed to read profile config: {}", e));
        }
    };
    
    // Check if we need to download/update PCI profiles
    eprintln!("[DEBUG] Checking PCI profiles...");
    if profiles_need_update(pci_path) {
        eprintln!("[DEBUG] PCI profiles need update, downloading...");
        match reqwest::get(&profile_url.pci_json_url).await {
            Ok(response) => {
                eprintln!("[DEBUG] PCI download response status: {}", response.status());
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => {
                            eprintln!("[DEBUG] PCI profile downloaded ({} bytes), caching...", text.len());
                            match cache_profile_file(pci_path, &text).await {
                                Ok(_) => eprintln!("[DEBUG] PCI profiles cached successfully"),
                                Err(e) => {
                                    eprintln!("[DEBUG] Failed to cache PCI profiles: {}", e);
                                    return Err(format!("Failed to cache PCI profiles: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[DEBUG] Failed to read PCI profile response text: {}", e);
                            return Err(format!("Failed to read PCI profile response: {}", e));
                        }
                    }
                } else {
                    eprintln!("[DEBUG] PCI profile download failed with status: {}", response.status());
                    return Err(format!("Failed to download PCI profiles: HTTP {}", response.status()));
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to download PCI profiles: {}", e);
                return Err(format!("Failed to download PCI profiles: {}", e));
            }
        }
    } else {
        eprintln!("[DEBUG] PCI profiles are up to date, skipping download");
    }
    
    // Check if we need to download/update USB profiles
    eprintln!("[DEBUG] Checking USB profiles...");
    if profiles_need_update(usb_path) {
        eprintln!("[DEBUG] USB profiles need update, downloading...");
        match reqwest::get(&profile_url.usb_json_url).await {
            Ok(response) => {
                eprintln!("[DEBUG] USB download response status: {}", response.status());
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => {
                            eprintln!("[DEBUG] USB profile downloaded ({} bytes), caching...", text.len());
                            match cache_profile_file(usb_path, &text).await {
                                Ok(_) => eprintln!("[DEBUG] USB profiles cached successfully"),
                                Err(e) => {
                                    eprintln!("[DEBUG] Failed to cache USB profiles: {}", e);
                                    return Err(format!("Failed to cache USB profiles: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[DEBUG] Failed to read USB profile response text: {}", e);
                            return Err(format!("Failed to read USB profile response: {}", e));
                        }
                    }
                } else {
                    eprintln!("[DEBUG] USB profile download failed with status: {}", response.status());
                    return Err(format!("Failed to download USB profiles: HTTP {}", response.status()));
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to download USB profiles: {}", e);
                return Err(format!("Failed to download USB profiles: {}", e));
            }
        }
    } else {
        eprintln!("[DEBUG] USB profiles are up to date, skipping download");
    }
    
    eprintln!("[DEBUG] ensure_profiles_cached() completed successfully");
    Ok(())
}

// Request elevated permissions using pkexec (shows GUI password dialog)
// Also ensures /var/cache/cfhdb/ directory exists and is writable
async fn request_permissions() -> Result<(), String> {
    use tokio::process::Command as TokioCommand;
    
    eprintln!("[DEBUG] Requesting elevated permissions via pkexec...");
    
    // First, ensure the directory exists and has proper permissions
    // This will show the GUI password dialog
    let mut mkdir_cmd = TokioCommand::new("pkexec");
    mkdir_cmd.args(["mkdir", "-p", "/var/cache/cfhdb"]);
    
    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        mkdir_cmd.env("DISPLAY", display);
    }
    
    let mkdir_output = mkdir_cmd
        .output()
        .await
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to execute pkexec mkdir: {}", e);
            format!("Failed to request permissions: {}. Make sure polkit is installed.", e)
        })?;
    
    if !mkdir_output.status.success() {
        let stderr = String::from_utf8_lossy(&mkdir_output.stderr);
        eprintln!("[DEBUG] Failed to create directory: {}", stderr);
        // Check if user cancelled the password dialog
        if mkdir_output.status.code() == Some(126) || mkdir_output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        return Err(format!("Failed to create directory: {}", stderr));
    }
    
    // Get current user to set ownership
    let current_user = users::get_current_username()
        .ok_or("Failed to get current username")?
        .to_string_lossy()
        .to_string();
    
    // Change ownership of the directory to the current user so the cfhdb library can write to it
    // (The library needs to write check_cmd.sh)
    eprintln!("[DEBUG] Changing directory ownership to user: {}", current_user);
    let mut chown_cmd = TokioCommand::new("pkexec");
    chown_cmd.args(["chown", "-R", &format!("{}:{}", current_user, current_user), "/var/cache/cfhdb"]);
    if let Ok(display) = std::env::var("DISPLAY") {
        chown_cmd.env("DISPLAY", display);
    }
    
    let chown_output = chown_cmd
        .output()
        .await;
    
    match chown_output {
        Ok(output) if output.status.success() => {
            eprintln!("[DEBUG] Directory ownership changed successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[DEBUG] Warning: Failed to change directory ownership: {}", stderr);
        }
        Err(e) => {
            eprintln!("[DEBUG] Warning: Failed to execute chown: {}", e);
        }
    }
    
    // Set directory permissions to 777 so the cfhdb library can write to it
    eprintln!("[DEBUG] Setting directory permissions to allow writes...");
    let mut chmod_cmd = TokioCommand::new("pkexec");
    chmod_cmd.args(["chmod", "-R", "777", "/var/cache/cfhdb"]);
    if let Ok(display) = std::env::var("DISPLAY") {
        chmod_cmd.env("DISPLAY", display);
    }
    
    let chmod_output = chmod_cmd
        .output()
        .await;
    
    match chmod_output {
        Ok(output) if output.status.success() => {
            eprintln!("[DEBUG] Directory permissions set successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[DEBUG] Warning: Failed to set directory permissions: {}", stderr);
        }
        Err(e) => {
            eprintln!("[DEBUG] Warning: Failed to execute chmod: {}", e);
        }
    }
    
    eprintln!("[DEBUG] Permissions granted and directory prepared successfully");
    Ok(())
}

// Cache a profile file to /var/cache/cfhdb/ using sudo if needed
async fn cache_profile_file(path: &Path, content: &str) -> Result<(), String> {
    eprintln!("[DEBUG] cache_profile_file() called for: {:?}", path);
    use tokio::process::Command as TokioCommand;
    
    // Try to write directly first (in case we're already root or have permissions)
    eprintln!("[DEBUG] Attempting direct write...");
    if let Some(parent) = path.parent() {
        eprintln!("[DEBUG] Parent directory: {:?}", parent);
        match std::fs::create_dir_all(parent) {
            Ok(_) => {
                eprintln!("[DEBUG] Directory created/exists, attempting write...");
                match std::fs::write(path, content) {
                    Ok(_) => {
                        eprintln!("[DEBUG] Successfully wrote file directly (no sudo needed)");
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] Direct write failed: {}, will try sudo", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] Failed to create directory: {}, will try sudo", e);
            }
        }
    }
    
    // If direct write failed, use sudo to write to /var/cache/
    // Write to a temp file first, then use sudo to move it
    eprintln!("[DEBUG] Using sudo to cache file...");
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_file = std::env::temp_dir().join(format!("cfhdb_{}.json", timestamp));
    eprintln!("[DEBUG] Temp file: {:?}", temp_file);
    
    match std::fs::write(&temp_file, content) {
        Ok(_) => eprintln!("[DEBUG] Temp file written successfully"),
        Err(e) => {
            eprintln!("[DEBUG] Failed to write temp file: {}", e);
            return Err(format!("Failed to write temp file: {}", e));
        }
    }
    
    // Use sudo to create directory and copy file
    let parent_dir = path.parent().ok_or("Invalid path")?;
    let parent_str = parent_dir.to_str().ok_or("Invalid path")?;
    let path_str = path.to_str().ok_or("Invalid path")?;
    let temp_str = temp_file.to_str().ok_or("Invalid temp path")?;
    
    // Create directory with pkexec (shows GUI password dialog via polkit)
    eprintln!("[DEBUG] Creating directory with pkexec: {}", parent_str);
    let mut create_cmd = TokioCommand::new("pkexec");
    create_cmd.args(["mkdir", "-p", parent_str]);
    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        create_cmd.env("DISPLAY", display);
    }
    let create_dir_output = create_cmd
        .output()
        .await
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to execute pkexec mkdir: {}", e);
            let _ = std::fs::remove_file(&temp_file);
            format!("Failed to execute mkdir: {}. Make sure polkit is installed.", e)
        })?;
    
    if !create_dir_output.status.success() {
        let stderr = String::from_utf8_lossy(&create_dir_output.stderr);
        eprintln!("[DEBUG] pkexec mkdir failed: {}", stderr);
        let _ = std::fs::remove_file(&temp_file);
        // Check if user cancelled the password dialog (exit code 126/127 = auth failure)
        if create_dir_output.status.code() == Some(126) || create_dir_output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        return Err(format!("Failed to create directory: {}", stderr));
    }
    
    // Set directory permissions so all users can read it (cfhdb library needs to read files in it)
    eprintln!("[DEBUG] Setting directory permissions for: {}", parent_str);
    let mut chmod_dir_cmd = TokioCommand::new("pkexec");
    chmod_dir_cmd.args(["chmod", "755", parent_str]);
    if let Ok(display) = std::env::var("DISPLAY") {
        chmod_dir_cmd.env("DISPLAY", display);
    }
    let chmod_dir_output = chmod_dir_cmd
        .output()
        .await;
    
    match chmod_dir_output {
        Ok(output) if output.status.success() => {
            eprintln!("[DEBUG] Directory permissions set successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[DEBUG] Warning: Failed to set directory permissions: {}", stderr);
        }
        Err(e) => {
            eprintln!("[DEBUG] Warning: Failed to execute chmod on directory: {}", e);
        }
    }
    
    eprintln!("[DEBUG] Directory created successfully");
    
    // Copy file with pkexec (shows GUI password dialog via polkit)
    eprintln!("[DEBUG] Copying file with pkexec: {} -> {}", temp_str, path_str);
    let mut copy_cmd = TokioCommand::new("pkexec");
    copy_cmd.args(["cp", temp_str, path_str]);
    // Ensure DISPLAY is set for GUI dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        copy_cmd.env("DISPLAY", display);
    }
    let copy_output = copy_cmd
        .output()
        .await
        .map_err(|e| {
            eprintln!("[DEBUG] Failed to execute pkexec cp: {}", e);
            let _ = std::fs::remove_file(&temp_file);
            format!("Failed to execute cp: {}. Make sure polkit is installed.", e)
        })?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);
    
    if !copy_output.status.success() {
        let stderr = String::from_utf8_lossy(&copy_output.stderr);
        eprintln!("[DEBUG] pkexec cp failed: {}", stderr);
        // Check if user cancelled the password dialog
        if copy_output.status.code() == Some(126) || copy_output.status.code() == Some(127) {
            return Err("Authentication cancelled or failed. Please try again.".to_string());
        }
        return Err(format!("Failed to cache profile: {}", stderr));
    }
    
    // Set file permissions so all users can read it (cfhdb library needs to read these files)
    eprintln!("[DEBUG] Setting file permissions for: {}", path_str);
    let mut chmod_cmd = TokioCommand::new("pkexec");
    chmod_cmd.args(["chmod", "644", path_str]);
    if let Ok(display) = std::env::var("DISPLAY") {
        chmod_cmd.env("DISPLAY", display);
    }
    let chmod_output = chmod_cmd
        .output()
        .await;
    
    match chmod_output {
        Ok(output) if output.status.success() => {
            eprintln!("[DEBUG] File permissions set successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[DEBUG] Warning: Failed to set file permissions: {}", stderr);
            // Don't fail if chmod fails, the file was copied successfully
        }
        Err(e) => {
            eprintln!("[DEBUG] Warning: Failed to execute chmod: {}", e);
            // Don't fail if chmod fails, the file was copied successfully
        }
    }
    
    eprintln!("[DEBUG] File cached successfully with pkexec");
    
    // Fix permissions on all files in the directory (including check_cmd.sh that cfhdb library needs)
    // Use find to set files to 644 and directories to 755 separately
    eprintln!("[DEBUG] Fixing permissions on all files in /var/cache/cfhdb/");
    
    // Fix directory permissions first
    let mut fix_dir_perms_cmd = TokioCommand::new("pkexec");
    fix_dir_perms_cmd.args(["find", "/var/cache/cfhdb/", "-type", "d", "-exec", "chmod", "755", "{}", "+"]);
    if let Ok(display) = std::env::var("DISPLAY") {
        fix_dir_perms_cmd.env("DISPLAY", display);
    }
    let _ = fix_dir_perms_cmd.output().await;
    
    // Fix file permissions
    let mut fix_perms_cmd = TokioCommand::new("pkexec");
    fix_perms_cmd.args(["find", "/var/cache/cfhdb/", "-type", "f", "-exec", "chmod", "644", "{}", "+"]);
    if let Ok(display) = std::env::var("DISPLAY") {
        fix_perms_cmd.env("DISPLAY", display);
    }
    let fix_perms_output = fix_perms_cmd
        .output()
        .await;
    
    match fix_perms_output {
        Ok(output) if output.status.success() => {
            eprintln!("[DEBUG] File permissions fixed successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[DEBUG] Warning: Failed to fix file permissions: {}", stderr);
        }
        Err(e) => {
            eprintln!("[DEBUG] Warning: Failed to execute chmod on files: {}", e);
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
            // Parse profile similar to nobara-device-manager
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
            // Similar parsing for USB profiles
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
    
    // Debug: Log device info for NVIDIA devices
    if device.vendor_id == "10de" {
        eprintln!("[DEBUG] Matching profiles for NVIDIA device: vendor_id={}, device_id={}, class_id={}, name={}", 
                  device.vendor_id, device.device_id, device.class_id, device.device_name);
    }
    
    for profile_arc in profile_data {
        let profile = profile_arc.profile();
        
        // Debug: Log NVIDIA profiles
        if profile.vendor_ids.contains(&"10de".to_string()) {
            eprintln!("[DEBUG] Checking NVIDIA profile: {} - vendor_ids={:?}, class_ids={:?}, device_ids={:?}", 
                      profile.i18n_desc, profile.vendor_ids, profile.class_ids, profile.device_ids);
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
                eprintln!("[DEBUG] Profile matched: {}", profile.i18n_desc);
            }
            available_profiles.push(profile_arc.clone());
        }
    }
    
    if device.vendor_id == "10de" {
        eprintln!("[DEBUG] Total profiles matched for NVIDIA device: {}", available_profiles.len());
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

// Style sheets
struct RoundedButtonStyle {
    is_primary: bool,
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
                radius: 12.0.into(),
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

struct SidebarStyle;

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
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            },
            ..Default::default()
        }
    }
}

struct DeviceCardStyle;

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
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct StatusIndicatorStyle {
    color: iced::Color,
}

impl iced::widget::container::StyleSheet for StatusIndicatorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(iced::Background::Color(self.color)),
            border: Border {
                radius: 6.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct StatusBadgeContainerStyle;

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
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

struct BadgeStyle {
    is_positive: bool,
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
                radius: 8.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct InfoBadgeStyle;

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
                radius: 8.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ExperimentalBadgeStyle;

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
                radius: 6.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

struct ProfileCardStyle;

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
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.15),
            },
            ..Default::default()
        }
    }
}

