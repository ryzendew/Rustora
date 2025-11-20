pub mod search;
pub mod installed;
pub mod update;
pub mod flatpak;
pub mod maintenance;
pub mod repo;
pub mod kernel;
pub mod device;
pub mod fpm;
pub mod tweaks;

pub use search::SearchTab;
pub use installed::InstalledTab;
pub use update::UpdateTab;
pub use flatpak::FlatpakTab;
pub use maintenance::MaintenanceTab;
pub use repo::RepoTab;
pub use kernel::KernelTab;
pub use device::DeviceTab;
pub use fpm::FpmTab;
pub use tweaks::TweaksTab;




