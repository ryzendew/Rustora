# Usage Guide

How to use Rustora for managing your Fedora system.

## GUI Mode (Default)

Just run:

```bash
rustora
```

Or find "Rustora" in your application menu.

The GUI gives you access to everything through a simple tabbed interface:
- **Search**: Find and install packages
- **Installed**: Manage what you've got installed
- **Updates**: Keep your system up to date
- **Flatpak**: Manage Flatpak applications
- **Maintenance**: System maintenance tasks
- **Repositories**: Manage DNF repositories
- **Kernels**: Install and manage kernels
- **Devices**: Set up device drivers
- **FPM**: Convert packages from other formats
- **Tweaks**: System optimizations and gaming tools

## Terminal Mode

Rustora also works from the command line:

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

## Features Overview

### Package Management

**Search & Install** - Find packages across all your repositories with real-time search. See package details, dependencies, sizes, and which repo provides them. Install multiple packages at once with a few clicks.

**Installed Packages** - View everything you've got installed, search through them, and remove what you don't need. Bulk selection makes cleanup easy.

**System Updates** - Check for updates, see what's changing, and update selectively or all at once. Configure automatic updates if you want to set it and forget it.

**RPM Files** - Right-click any RPM file in your file manager and open it with Rustora. Preview package info before installing, and Rustora handles all the dependency resolution automatically.

### Flatpak Management

Search, install, update, and remove Flatpak applications from all your configured remotes. Everything you need for managing Flatpaks is right there in one tab.

### Package Conversion

Got a DEB file but need it as RPM? Rustora can convert it for you. The FPM tab handles DEB to RPM and TGZ to RPM conversions, then automatically opens an install dialog when it's done.

**Note:** Requires the `fpm` gem (installed automatically by the build script, or manually: `gem install fpm`)

### System Maintenance

Keep your system running smoothly with built-in maintenance tools:
- Rebuild kernel modules
- Regenerate initramfs
- Remove orphaned packages
- Clean package cache
- Or just run everything at once

Each task runs in its own progress window so you can see exactly what's happening.

### Repository Management

View all your repositories, enable or disable them with a click, and add new ones. No more editing config files manually.

### Kernel Management

Browse and install different kernel versions from multiple sources. Remove old kernels to free up space. Switch between kernel branches easily.

### Device Drivers

Automatically detect your PCI and USB devices, then install drivers using pre-configured profiles. Perfect for setting up new hardware without hunting down drivers manually.

### System Tweaks & Gaming Tools

This is where Rustora really shines. The Tweaks tab gives you powerful tools for optimizing your system and setting up gaming:

**Gaming Meta** - One-click installation of your complete gaming stack:
- Steam (system or Flatpak)
- Lutris
- MangoHud (performance overlay)
- Gamescope (compositor)
- MangoJuice (GPU monitoring)
- ProtonPlus
- Heroic Games Launcher

**DNF Configuration** - Speed up your package downloads with parallel downloads and fastest mirror selection. Configure it once and forget about it.

**CachyOS Kernel** - Install the optimized CachyOS kernel with all the performance tweaks. Configure kernel settings, install schedulers, and manage repositories.

**Hyprland** - Complete Hyprland setup with all the essential tools:
- Hyprland compositor
- Hyprpicker (color picker)
- Swww (wallpaper daemon)
- Quickshell (status bar)
- Fuzzel (application launcher)
- Wlogout (logout menu)
- Cliphist (clipboard manager)
- And all the other utilities you need

**Proton & Wine Builds** - Manage all your compatibility layers in one place:
- Install Proton-GE, Proton-CachyOS, Proton-EM, and more
- Install Wine builds (Vanilla, Staging, Tkg, etc.)
- Works with Steam, Lutris, Heroic, and Bottles
- View changelogs for each build
- Update "Latest" builds automatically
- See which games use which builds
- Real-time download and installation progress

**Steam Games** - Set compatibility tools per game. View all your Steam games and their current Proton/Wine versions. Find games without compatibility tools set.

### Customization

Make Rustora look and feel exactly how you want:
- Dark and light themes with custom colors
- Adjustable font sizes for every UI element
- UI scaling for different display sizes
- Show or hide tabs you don't use
- Save custom themes for reuse

All settings persist automatically, so you set it up once and it stays that way.

