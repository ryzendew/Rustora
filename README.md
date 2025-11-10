# FedoraForge

A modern, GUI-based package manager for Fedora and Nobara.

![FedoraForge Icon](src/assets/fedoraforge.svg)

## Features

- Modern, clean GUI interface
- Search for packages
- Install/remove packages
- Check for and install updates
- Install RPM files directly
- Optimized for 720p+ screens and tiling window managers
- Material Symbols icons throughout

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
    dnf rpm polkit zenity curl unzip fontconfig
```

**Note**: `zenity` (GNOME) or `kdialog` (KDE) is required for file picker dialogs. At least one should be installed.

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
cp target/release/fedoraforge ~/.local/bin/
chmod +x ~/.local/bin/fedoraforge
```

</details>

## Usage

### GUI Mode (Default)

Simply run:

```bash
fedoraforge
```

Or find "FedoraForge" in your application menu.

### Terminal Mode

```bash
# Search for packages
fedoraforge search <package-name>

# Install packages
fedoraforge install <package1> <package2>

# List installed packages
fedoraforge list

# Show package info
fedoraforge info <package-name>

# Update packages
fedoraforge update --all
```

## Installation Locations

### User Installation (default)
- Binary: `~/.local/bin/fedoraforge`
- Desktop file: `~/.local/share/applications/fedoraforge.desktop`
- Icon: `~/.local/share/icons/hicolor/scalable/apps/fedoraforge.svg`

### System-wide Installation (run as root)
- Binary: `/usr/local/bin/fedoraforge`
- Desktop file: `/usr/share/applications/fedoraforge.desktop`
- Icon: `/usr/local/share/icons/hicolor/scalable/apps/fedoraforge.svg`

## Uninstall

<details>
<summary>Remove FedoraForge</summary>

### User Installation

```bash
rm ~/.local/bin/fedoraforge
rm ~/.local/share/applications/fedoraforge.desktop
rm ~/.local/share/icons/hicolor/scalable/apps/fedoraforge.svg
update-desktop-database ~/.local/share/applications
```

### System-wide Installation

```bash
sudo rm /usr/local/bin/fedoraforge
sudo rm /usr/share/applications/fedoraforge.desktop
sudo rm /usr/local/share/icons/hicolor/scalable/apps/fedoraforge.svg
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
