# Rustora

A modern, GUI-based package manager for Fedora and Nobara.

![Rustora Logo](https://raw.githubusercontent.com/ryzendew/Fedora-Forge/main/src/assets/Rustora_logo.png)

## Features

Rustora provides a comprehensive package management solution with the following features:

### Core Package Management
- **Search Tab**: Search for packages in Fedora repositories with real-time search results
  - Debounced search for better performance
  - View package details (name, version, description, size)
  - Select and install multiple packages at once
  - Package information display with dependencies

- **Installed Tab**: Manage installed packages
  - View all installed packages with details
  - Remove packages individually or in bulk
  - Refresh package list
  - Filter and search installed packages

- **Updates Tab**: System updates management
  - Check for available updates
  - View update details and changelogs
  - Install selected updates or all updates
  - Update settings configuration

- **RPM Installation**: Direct RPM file installation
  - Install RPM files from file manager
  - View package information before installation
  - Dependency resolution and installation

### Flatpak Management
- **Flatpak Tab**: Complete Flatpak application management
  - Search Flatpak applications from configured remotes
  - Install, update, and remove Flatpak applications
  - View installed Flatpaks with details
  - Check for Flatpak updates
  - Package details panel with full information

### Package Conversion
- **Alien Tab**: Convert packages from other Linux distributions
  - Convert DEB packages to RPM format
  - Convert TGZ (Slackware) packages to RPM format
  - Real-time conversion progress display
  - Automatic integration with install dialog after conversion
  - View conversion output and errors
  - **Requires**: `alien` package (install with `sudo dnf install alien`)

### System Maintenance
- **Maintenance Tab**: System maintenance and optimization
  - Rebuild kernel modules
  - Regenerate initramfs
  - Remove orphaned packages
  - Clean package cache
  - Run all maintenance tasks at once

### Repository Management
- **Repositories Tab**: Manage DNF repositories
  - View enabled/disabled repositories
  - Enable or disable repositories
  - Add new repositories
  - Repository status and information

### Kernel Management
- **Kernels Tab**: Manage kernel installations
  - View available kernel branches
  - Install different kernel versions
  - Remove old kernels
  - Kernel information and details

### Device Driver Management
- **Devices Tab**: Hardware device driver management
  - Detect and manage PCI devices
  - Detect and manage USB devices
  - Install device drivers from profiles
  - Remove device drivers
  - Device information and driver details

### User Interface
- Modern, clean GUI interface built with Iced
- Dark and light theme support (toggle in top bar)
- Material Symbols icons throughout
- Optimized for 720p+ screens and tiling window managers
- Responsive design with smooth animations
- Professional styling consistent across all tabs

## Installation

<details>
<summary>Prerequisites</summary>

### System Requirements

- **OS**: Fedora 38+ or Nobara (Fedora-based)
- **Architecture**: x86_64

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
    alien
```

**Notes**:
- `zenity` (GNOME) or `kdialog` (KDE) is required for file picker dialogs. At least one should be installed.
- `alien` is required for the Alien tab (package conversion feature). Install with `sudo dnf install alien` if not already installed.

</details>

<details>
<summary>Build and Install</summary>

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

**System-wide installation**: Run with `sudo` for system-wide installation.

### Manual Build

```bash
# Build in release mode
cargo build --release

# Install binary manually
cp target/release/rustora ~/.local/bin/
chmod +x ~/.local/bin/rustora
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
- **Alien**: Convert packages from other formats

### Terminal Mode

```bash
# Search for packages
rustora search <package-name>

# Install packages
rustora install <package1> <package2>

# List installed packages
rustora list

# Show package info
rustora info <package-name>

# Update packages
rustora update --all
```

### Using the Alien Tab (Package Conversion)

1. Open Rustora and navigate to the **Alien** tab
2. Click either:
   - **Convert DEB to RPM**: Select a `.deb` file to convert
   - **Convert TGZ to RPM**: Select a `.tgz` or `.tar.gz` file to convert
3. Select the package file using the file picker dialog
4. Wait for the conversion to complete (progress is shown in real-time)
5. After successful conversion, the install dialog will automatically open
6. Review package information and dependencies, then install if desired

**Note**: The converted RPM file is saved in the same directory as the source file.

## Installation Locations

### User Installation (default)
- Binary: `~/.local/bin/rustora`
- Desktop file: `~/.local/share/applications/rustora.desktop`
- Icon: `~/.local/share/icons/hicolor/scalable/apps/rustora.svg`

### System-wide Installation (run as root)
- Binary: `/usr/local/bin/rustora`
- Desktop file: `/usr/share/applications/rustora.desktop`
- Icon: `/usr/local/share/icons/hicolor/scalable/apps/rustora.svg`

## Uninstall

<details>
<summary>Remove Rustora</summary>

### User Installation

```bash
rm ~/.local/bin/rustora
rm ~/.local/share/applications/rustora.desktop
rm ~/.local/share/icons/hicolor/scalable/apps/rustora.svg
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
<summary>Build and Run</summary>

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

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
