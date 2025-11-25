# Rustora

A modern, GUI-based package manager for Fedora and Nobara with comprehensive system management capabilities.

![Rustora Logo](https://raw.githubusercontent.com/ryzendew/Fedora-Forge/main/src/assets/rustora.svg)

## Overview

Rustora is a feature-rich package management application built with Rust and the Iced GUI framework. It provides a unified interface for managing packages, Flatpaks, kernels, device drivers, gaming tools, and system configurations on Fedora-based distributions.

## Notice 

According to some using comments is bad .. I learned along time ago from the failed yuzu project that knowning what the code does line by line helps fix issues. 

## Features

### 📦 Core Package Management

#### Search Tab
- **Real-time package search** with debounced input for optimal performance
- **Package details** including name, version, description, size, and dependencies
- **Multi-select installation** - select and install multiple packages at once
- **Package information panel** with full dependency tree
- **Repository filtering** to see which repository provides each package

**Example:**
```bash
# Search for packages
rustora search firefox

# Or use the GUI:
# 1. Open Rustora
# 2. Navigate to Search tab
# 3. Type package name in search box
# 4. Select packages and click Install
```

#### Installed Tab
- **View all installed packages** with detailed information
- **Remove packages** individually or in bulk selection
- **Search and filter** installed packages
- **Package details** including version, size, and installation date
- **Refresh package list** to get latest information

**Example:**
```bash
# List installed packages
rustora list

# Or use the GUI:
# 1. Navigate to Installed tab
# 2. Use search box to find specific packages
# 3. Select packages and click Remove
```

#### Updates Tab
- **Check for system updates** with detailed update information
- **View update details** including changelogs and package sizes
- **Selective updates** - choose which packages to update
- **Update all** option for system-wide updates
- **Update settings** configuration (automatic updates, update frequency)
- **Update history** and changelog viewing

**Example:**
```bash
# Check for updates
rustora update

# Update all packages
rustora update --all

# Or use the GUI:
# 1. Navigate to Updates tab
# 2. Click "Check for Updates"
# 3. Select packages to update or click "Update All"
```

#### RPM Installation
- **Direct RPM file installation** from file manager integration
- **Package information preview** before installation
- **Automatic dependency resolution** and installation
- **Installation progress** with detailed output
- **Error handling** with clear messages

**Example:**
```bash
# Install RPM file
rustora install package.rpm

# Or right-click RPM file in file manager and select "Open with Rustora"
```

### 📱 Flatpak Management

#### Flatpak Tab
- **Search Flatpak applications** from all configured remotes
- **Install Flatpak applications** with dependency handling
- **Update Flatpaks** - check and install available updates
- **Remove Flatpaks** with cleanup options
- **View installed Flatpaks** with detailed information
- **Package details panel** showing description, size, and metadata
- **Remote management** - view and manage Flatpak remotes

**Example:**
```bash
# Search for Flatpak apps
# 1. Navigate to Flatpak tab
# 2. Type application name in search box
# 3. Click Install on desired application

# Update Flatpaks
# 1. Navigate to Flatpak tab
# 2. Click "Check for Updates"
# 3. Select updates to install
```

### 🔄 Package Conversion

#### FPM Tab
- **Convert DEB to RPM** - convert Debian/Ubuntu packages to RPM format
- **Convert TGZ to RPM** - convert Slackware packages to RPM format
- **Real-time conversion progress** with detailed output
- **Automatic install dialog** after successful conversion
- **Error reporting** with conversion logs
- **Output file location** - converted RPM saved in source directory

**Example:**
```bash
# Convert a DEB package
# 1. Navigate to FPM tab
# 2. Click "Convert DEB to RPM"
# 3. Select .deb file using file picker
# 4. Wait for conversion to complete
# 5. Install dialog opens automatically
```

**Requirements:** `fpm` gem (installed automatically by `build-and-install.sh`, or manually: `gem install fpm`)

### 🛠️ System Maintenance

#### Maintenance Tab
- **Rebuild kernel modules** - rebuild modules for current kernel
- **Regenerate initramfs** - recreate initial RAM filesystem
- **Remove orphaned packages** - clean up unused dependencies
- **Clean package cache** - free up disk space
- **Run all tasks** - execute all maintenance tasks sequentially
- **Progress tracking** with detailed terminal output
- **Separate progress windows** for each task

**Example:**
```bash
# Run maintenance tasks
# 1. Navigate to Maintenance tab
# 2. Click individual task buttons or "Run All"
# 3. Monitor progress in separate window
```

### 📚 Repository Management

#### Repositories Tab
- **View all repositories** - enabled and disabled
- **Enable/disable repositories** with one click
- **Add new repositories** - configure custom repositories
- **Repository information** - view repository details and status
- **Repository priority** management

**Example:**
```bash
# Manage repositories
# 1. Navigate to Repositories tab
# 2. Toggle repositories on/off
# 3. Add new repository with "Add Repository" button
```

### 🐧 Kernel Management

#### Kernels Tab
- **View available kernel branches** from multiple sources
- **Install different kernel versions** - choose from available kernels
- **Remove old kernels** - clean up unused kernel versions
- **Kernel information** - view kernel details and features
- **Kernel branch selection** - switch between different kernel sources

**Example:**
```bash
# Manage kernels
# 1. Navigate to Kernels tab
# 2. Select kernel branch
# 3. Choose kernel version to install
# 4. Click Install
```

### 🖥️ Device Driver Management

#### Devices Tab
- **PCI device detection** - automatically detect PCI hardware
- **USB device detection** - automatically detect USB hardware
- **Driver profile management** - install drivers from profiles
- **Device information** - view detailed hardware information
- **Driver installation** - install and configure device drivers
- **Driver removal** - uninstall device drivers
- **Profile-based installation** - use pre-configured driver profiles

**Example:**
```bash
# Manage device drivers
# 1. Navigate to Devices tab
# 2. Select PCI or USB devices
# 3. View available driver profiles
# 4. Click Install to install driver
```

### ⚙️ System Tweaks & Optimizations

#### Tweaks Tab

The Tweaks tab provides comprehensive system optimization and gaming tools management:

##### Gaming Meta
- **One-click gaming stack installation** - install complete gaming toolchain
- **Component management:**
  - Steam (system or Flatpak)
  - Lutris
  - MangoHud (performance overlay)
  - Gamescope (compositor)
  - MangoJuice (GPU monitoring)
  - ProtonPlus
  - Heroic Games Launcher
- **Status checking** - view installation status of each component
- **Batch installation** - install all components at once

**Example:**
```bash
# Install gaming meta
# 1. Navigate to Tweaks tab → Gaming Meta
# 2. Click "Check Status" to see what's installed
# 3. Click "Install Gaming Meta" to install all components
```

##### DNF Configuration
- **Parallel downloads** - configure number of simultaneous downloads
- **Fastest mirror** - enable/disable fastest mirror selection
- **Configuration persistence** - save DNF settings
- **Real-time configuration** - see changes immediately

**Example:**
```bash
# Configure DNF
# 1. Navigate to Tweaks tab → DNF Config
# 2. Set parallel downloads (e.g., 10)
# 3. Toggle "Fastest Mirror" if desired
# 4. Click "Save Configuration"
```

##### CachyOS Kernel
- **CachyOS kernel installation** - install optimized CachyOS kernel
- **CachyOS settings** - configure kernel settings
- **Ananicy-CPP** - install process scheduler
- **SCX scheduler** - install and configure SCX scheduler
- **Repository management** - manage CachyOS repositories

**Example:**
```bash
# Install CachyOS kernel
# 1. Navigate to Tweaks tab → Cachyos Kernel
# 2. Check installation status
# 3. Click "Install Cachyos Kernel" to install
```

##### Hyprland
- **Hyprland compositor** - install Hyprland window manager
- **Hyprland tools:**
  - Hyprpicker (color picker)
  - Swww (wallpaper daemon)
  - Quickshell (status bar)
  - Fuzzel (application launcher)
  - Wlogout (logout menu)
  - Cliphist (clipboard manager)
  - Brightnessctl (brightness control)
  - Grim/Slurp (screenshot tools)
  - Swappy (screenshot editor)
- **Dotfiles installation** - install Hyprland configuration
- **Repository setup** - configure required repositories

**Example:**
```bash
# Install Hyprland
# 1. Navigate to Tweaks tab → Hyprland
# 2. Check installation status
# 3. Click "Install Hyprland" to install compositor
# 4. Click "Install Dotfiles" to install configuration
```

##### Proton & Wine Builds
- **Proton build management** - install and manage Proton compatibility layers
- **Wine build management** - install and manage Wine builds
- **Supported runners:**
  - Proton-GE
  - Proton-CachyOS
  - Proton-EM
  - Proton-Sarek
  - Proton-Tkg
  - Wine-Vanilla (Kron4ek)
  - Wine-Staging (Kron4ek)
  - Wine-Staging-Tkg (Kron4ek)
  - Wine-Proton (Kron4ek)
  - And more...
- **Launcher support:**
  - Steam
  - Lutris
  - Heroic Games Launcher
  - Bottles
- **Build features:**
  - View available builds with descriptions
  - Install builds to selected launcher
  - Update "Latest" builds when new versions available
  - Remove installed builds
  - View changelogs for each build
  - Filter by installed/used/unused builds
  - Check which games use which builds
  - Real-time download and extraction progress
  - Separate installation windows with progress bars
- **Caching** - local cache for faster loading
- **Auto-detection** - automatically detect installed launchers

**Example:**
```bash
# Manage Proton/Wine builds
# 1. Navigate to Tweaks tab → Proton
# 2. Select launcher (Steam, Lutris, etc.)
# 3. Select runner (Proton-GE, Wine-Staging, etc.)
# 4. Click Install on desired build
# 5. Monitor progress in separate window
# 6. View changelog with Info button
```

##### Steam Games
- **Game compatibility tool management** - set Proton/Wine version per game
- **Game list** - view all Steam games with current compatibility tools
- **Tool selection** - change compatibility tool for individual games
- **Undefined tools** - identify games without compatibility tools set

**Example:**
```bash
# Manage Steam game compatibility
# 1. Navigate to Tweaks tab → Steam Games
# 2. View list of all Steam games
# 3. Select game and choose compatibility tool
# 4. Changes apply immediately
```

### 🎨 Customization & Settings

#### Settings Dialog
- **Theme customization:**
  - Dark and light themes
  - Custom color schemes
  - Background, text, and primary colors
  - Border radius adjustment
- **Font customization:**
  - Universal font size
  - Individual font sizes (buttons, titles, body, inputs, tabs, icons)
  - Font family selection
- **UI scaling:**
  - Universal UI scale
  - Individual component scaling
  - Optimized for 720p+ displays
- **Tab visibility** - show/hide specific tabs
- **Theme management:**
  - Save custom themes
  - Load saved themes
  - Delete themes
- **Settings persistence** - all settings saved automatically

**Example:**
```bash
# Access settings
# 1. Click Settings button in top bar
# 2. Navigate through categories (General, Appearance, Fonts, Tabs)
# 3. Adjust settings and click Save
# 4. Save as theme for reuse
```

### 🖼️ User Interface

- **Modern design** - clean, professional interface built with Iced
- **Dark/Light themes** - toggle between themes in top bar
- **Material Symbols icons** - consistent iconography throughout
- **Responsive layout** - optimized for various screen sizes
- **Smooth animations** - polished user experience
- **Progress indicators** - real-time progress for all operations
- **Separate dialog windows** - dedicated windows for installations and operations
- **Terminal output** - view detailed command output for operations

## Installation

<details>
<summary><strong>Prerequisites</strong></summary>

### System Requirements

- **OS**: Fedora 38+ or Nobara (Fedora-based)
- **Architecture**: x86_64
- **Display**: 720p minimum recommended

### Install Dependencies

```bash
sudo dnf install -y \
    rust cargo \
    gcc gcc-c++ pkg-config \
    openssl-devel \
    libX11-devel libXcursor-devel libXrandr-devel libXi-devel \
    mesa-libGL-devel \
    fontconfig-devel freetype-devel expat-devel \
    dnf rpm polkit zenity curl unzip fontconfig \
    ruby ruby-devel rubygems
```

**Notes:**
- `zenity` (GNOME) or `kdialog` (KDE) is required for file picker dialogs
- `fpm` gem is required for the FPM tab (package conversion feature) - installed automatically by `build-and-install.sh`, or manually: `gem install fpm`
- `polkit` is required for privilege escalation

</details>

<details>
<summary><strong>Build and Install</strong></summary>

### Quick Install

```bash
./build-and-install.sh
```

This script will:
1. Build the application in release mode
2. Install the binary to `~/.local/bin/` (or `/usr/local/bin/` if run as root)
3. Install the custom icon
4. Create a `.desktop` file for your application menu
5. Update the desktop database

**System-wide installation:** Run with `sudo` for system-wide installation.

### Manual Build

```bash
# Build in release mode
cargo build --release

# Install binary manually
cp target/release/rustora ~/.local/bin/
chmod +x ~/.local/bin/rustora

# Install icon
mkdir -p ~/.local/share/icons/hicolor/scalable/apps/
cp src/assets/rustora.svg ~/.local/share/icons/hicolor/scalable/apps/

# Create desktop file
mkdir -p ~/.local/share/applications/
cat > ~/.local/share/applications/rustora.desktop << EOF
[Desktop Entry]
Name=Rustora
Comment=Package Manager for Fedora
Exec=rustora
Icon=rustora
Terminal=false
Type=Application
Categories=System;PackageManager;
EOF

# Update desktop database
update-desktop-database ~/.local/share/applications
```

</details>

## Usage

### GUI Mode (Default)

Simply run:

```bash
rustora
```

Or find "Rustora" in your application menu.

The GUI provides access to all features through a tabbed interface:
- **Search**: Search and install packages
- **Installed**: View and manage installed packages
- **Updates**: Check for and install system updates
- **Flatpak**: Manage Flatpak applications
- **Maintenance**: System maintenance tasks
- **Repositories**: Manage DNF repositories
- **Kernels**: Manage kernel installations
- **Devices**: Manage device drivers
- **FPM**: Convert packages from other formats
- **Tweaks**: System optimizations and gaming tools

### Terminal Mode

```bash
# Search for packages
rustora search <package-name>

# Install packages
rustora install <package1> <package2>

# List installed packages
rustora list [--details]

# Show package info
rustora info <package-name>

# Update packages
rustora update [--all]
```

## Examples

### Example 1: Installing a Package

**GUI Method:**
1. Open Rustora
2. Navigate to **Search** tab
3. Type package name (e.g., "firefox")
4. Select the package from results
5. Click **Install**
6. Review dependencies and confirm installation

**CLI Method:**
```bash
rustora install firefox
```

### Example 2: Managing Proton Builds

1. Open Rustora
2. Navigate to **Tweaks** tab → **Proton**
3. Wait for builds to load (uses local cache for speed)
4. Select launcher (Steam, Lutris, etc.)
5. Select runner (Proton-GE, Wine-Staging, etc.)
6. Click **Install** on desired build
7. Monitor progress in separate window:
   - Download progress (0-100%)
   - Extraction progress (0-100%)
   - Installation progress
8. View changelog with **Info** button
9. Update "Latest" builds when new versions are available

### Example 3: Converting a DEB Package

1. Open Rustora
2. Navigate to **FPM** tab
3. Click **Convert DEB to RPM**
4. Select `.deb` file using file picker
5. Wait for conversion (progress shown in real-time)
6. Install dialog opens automatically
7. Review package information
8. Click **Install** to install converted package

### Example 4: Configuring DNF for Faster Downloads

1. Open Rustora
2. Navigate to **Tweaks** tab → **DNF Config**
3. Set **Parallel Downloads** to 10 (or higher)
4. Enable **Fastest Mirror** option
5. Click **Save Configuration**
6. Future package operations will use these settings

### Example 5: Setting Up Gaming Environment

1. Open Rustora
2. Navigate to **Tweaks** tab → **Gaming Meta**
3. Click **Check Status** to see what's installed
4. Click **Install Gaming Meta** to install:
   - Steam
   - Lutris
   - MangoHud
   - Gamescope
   - Heroic Games Launcher
   - And more...
5. Navigate to **Proton** sub-tab
6. Install Proton-GE or other compatibility layers
7. Navigate to **Steam Games** sub-tab
8. Configure compatibility tools for individual games

### Example 6: Customizing Appearance

1. Open Rustora
2. Click **Settings** button in top bar
3. Navigate to **Appearance** category
4. Adjust colors (background, text, primary)
5. Set border radius
6. Navigate to **Fonts** category
7. Adjust font sizes for different UI elements
8. Navigate to **Tabs** category
9. Show/hide tabs as needed
10. Click **Save Settings**
11. Optionally save as theme for reuse

## Installation Locations

### User Installation (default)
- Binary: `~/.local/bin/rustora`
- Desktop file: `~/.local/share/applications/rustora.desktop`
- Icon: `~/.local/share/icons/hicolor/scalable/apps/rustora.svg`
- Settings: `~/.config/rustora/settings.json`
- Cache: `~/.cache/rustora/`

### System-wide Installation (run as root)
- Binary: `/usr/local/bin/rustora`
- Desktop file: `/usr/share/applications/rustora.desktop`
- Icon: `/usr/local/share/icons/hicolor/scalable/apps/rustora.svg`

## Uninstall

<details>
<summary><strong>Remove Rustora</strong></summary>

### User Installation

```bash
rm ~/.local/bin/rustora
rm ~/.local/share/applications/rustora.desktop
rm ~/.local/share/icons/hicolor/scalable/apps/rustora.svg
rm -rf ~/.config/rustora
rm -rf ~/.cache/rustora
update-desktop-database ~/.local/share/applications
```

### System-wide Installation

```bash
sudo rm /usr/local/bin/rustora
sudo rm /usr/share/applications/rustora.desktop
sudo rm /usr/local/share/icons/hicolor/scalable/apps/rustora.svg
sudo update-desktop-database /usr/share/applications
```

</details>

## Development

<details>
<summary><strong>Build and Run</strong></summary>

```bash
# Build in debug mode
cargo build

# Run with GUI
cargo run

# Run with specific command
cargo run -- search firefox

# Run tests
cargo test

# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

</details>

## Technical Details

### Architecture
- **Language**: Rust
- **GUI Framework**: Iced 0.12
- **Package Management**: DNF (via command-line interface)
- **Async Runtime**: Tokio
- **Serialization**: Serde

### Data Storage
- **Settings**: `~/.config/rustora/settings.json`
- **Cache**: `~/.cache/rustora/proton_builds.json`
- **Themes**: `~/.config/rustora/themes/*.json`

### Performance
- **Caching**: Proton/Wine builds cached locally for fast loading
- **Background updates**: Cache updated in background without blocking UI
- **Debounced search**: Optimized search performance
- **Lazy loading**: Tabs load data only when accessed

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is released into the public domain under the Unlicense - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) GUI framework
- Proton/Wine build data from [ProtonPlus](https://github.com/Vysp3r/ProtonPlus)
- Device profiles from [CFHDB](https://github.com/Nobara-Project/cfhdb)
