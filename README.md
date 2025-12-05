# Rustora

A modern, user-friendly package manager for Fedora that makes system management actually enjoyable.

![Rustora Logo](https://raw.githubusercontent.com/ryzendew/Fedora-Forge/main/src/assets/rustora.svg)

## What is This?

Rustora is a comprehensive GUI package manager built with Rust that gives you a single, beautiful interface for managing pretty much everything on your Fedora-based system. Instead of juggling terminal commands and multiple tools, Rustora brings it all together in one place.

Think of it as your system's control center - you can search and install packages, manage Flatpaks, update your system, install kernels, set up gaming tools, configure repositories, and a whole lot more. All with a clean, modern interface that doesn't make you want to pull your hair out.

## Quick Start

**Just want to get started?** Here's the fastest way:

```bash
# Clone the repo
git clone https://github.com/ryzendew/Rustora.git
cd Rustora

# Run the install script
./build-and-install.sh

# Launch Rustora
rustora
```

That's it! The script handles dependencies, builds the app, and sets everything up for you.

**Want more control?** Check out the [Installation Guide](docs/INSTALLATION.md) for manual setup.

## Features

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

### System Tweaks & Gaming Tools

This is where Rustora really shines. The Tweaks tab gives you powerful tools for optimizing your system and setting up gaming:

**Gaming Meta** - One-click installation of your complete gaming stack (Steam, Lutris, MangoHud, Gamescope, and more)

**Proton & Wine Builds** - Manage all your compatibility layers in one place. Works with Steam, Lutris, Heroic, and Bottles.

**Hyprland** - Complete Hyprland setup with all the essential tools and utilities.

**DNF Configuration** - Speed up your package downloads with parallel downloads and fastest mirror selection.

And much more! See the [Usage Guide](docs/USAGE.md) for complete details.

## Documentation

### Getting Started

* **[Installation Guide](docs/INSTALLATION.md)** - Complete installation instructions for all methods
* **[Usage Guide](docs/USAGE.md)** - How to use Rustora's features
* **[Examples](docs/EXAMPLES.md)** - Real-world examples and common tasks

### Technical Details

* **[Technical Details](docs/TECHNICAL.md)** - Architecture, performance, and development information
* **[Contributing](docs/CONTRIBUTING.md)** - Guidelines for contributing to the project

## Getting Help

* **GitHub Issues:** Report bugs and request features on [GitHub](https://github.com/ryzendew/Fedora-Forge/issues)
* **Documentation:** Check the guides in the [docs](docs/) folder

<details>
<summary><strong>System Requirements</strong></summary>

- **OS**: Fedora 43
- **Architecture**: x86_64

For full dependency list, see the [Installation Guide](docs/INSTALLATION.md).

</details>

<details>
<summary><strong>Installation</strong></summary>

### Quick Install

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

For detailed installation instructions, dependencies, and manual build steps, see the [Installation Guide](docs/INSTALLATION.md).

</details>

<details>
<summary><strong>Usage</strong></summary>

### GUI Mode (Default)

Just run:

```bash
rustora
```

Or find "Rustora" in your application menu.

### Terminal Mode

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

For more details and examples, see the [Usage Guide](docs/USAGE.md) and [Examples](docs/EXAMPLES.md).

</details>

<details>
<summary><strong>Development</strong></summary>

Want to contribute or just build it yourself?

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

See [Contributing](docs/CONTRIBUTING.md) for guidelines and [Technical Details](docs/TECHNICAL.md) for architecture information.

</details>

## License

This project is released into the public domain under the Unlicense - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) GUI framework
- Proton/Wine build data from [ProtonPlus](https://github.com/Vysp3r/ProtonPlus)
- Device profiles from [CFHDB](https://github.com/Nobara-Project/cfhdb)

---

**Note:** According to some, using comments is bad. I learned a long time ago from the failed yuzu project that knowing what the code does line by line helps fix issues. In other words, this codebase has comments - deal with it! 😄
