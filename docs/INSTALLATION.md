# Installation Guide

Complete installation instructions for Rustora on Fedora and Nobara systems.

## Prerequisites

- **OS**: Fedora 43
- **Architecture**: x86_64

## Install Dependencies

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
- `zenity` (GNOME) or `kdialog` (KDE) is needed for file picker dialogs
- `fpm` gem is required for package conversion - installed automatically by the build script
- `polkit` is required for privilege escalation (sudo operations)

## Quick Install

The easiest way to get Rustora up and running:

```bash
./build-and-install.sh
```

This script does everything:
1. Builds the application in release mode
2. Installs the binary to `~/.local/bin/` (or `/usr/local/bin/` if run as root)
3. Installs the custom icon
4. Creates a desktop file for your application menu
5. Updates the desktop database

**System-wide installation:** Run with `sudo` if you want it installed system-wide.

## Manual Build

If you prefer to do it yourself or need more control:

```bash
# Build in release mode
cargo build --release

# Install binary
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

