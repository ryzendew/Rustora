#!/bin/bash
# Build and Install script for FedoraForge

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 FedoraForge Build & Install${NC}"
echo "======================================"
echo ""

# ============================================================================
# STEP 1: BUILD
# ============================================================================
echo -e "${BLUE}Step 1/2: Building...${NC}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: cargo not found. Please install Rust first.${NC}"
    echo "   Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check Rust version (need 1.70+ for edition 2021)
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
REQUIRED_VERSION="1.70.0"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo -e "${YELLOW}⚠️  Warning: Rust version $RUST_VERSION may be too old. Recommended: $REQUIRED_VERSION or later${NC}"
fi

# Build in release mode
echo "📦 Compiling in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build successful!${NC}"
echo ""

# ============================================================================
# STEP 2: INSTALL
# ============================================================================
echo -e "${BLUE}Step 2/2: Installing...${NC}"

# Check if binary exists
if [ ! -f "target/release/fedoraforge" ]; then
    echo -e "${RED}❌ Error: Binary not found after build${NC}"
    exit 1
fi

# Check if running as root for system-wide install
if [ "$EUID" -eq 0 ]; then
    INSTALL_PREFIX="/usr/local"
    DESKTOP_DIR="/usr/share/applications"
    BIN_DIR="$INSTALL_PREFIX/bin"
    echo "🔧 Installing system-wide..."
else
    INSTALL_PREFIX="$HOME/.local"
    DESKTOP_DIR="$HOME/.local/share/applications"
    BIN_DIR="$INSTALL_PREFIX/bin"
    echo "🔧 Installing for current user..."
fi

# Create directories
mkdir -p "$BIN_DIR"
mkdir -p "$DESKTOP_DIR"

# Install binary
echo "📦 Installing binary to $BIN_DIR..."
cp target/release/fedoraforge "$BIN_DIR/fedoraforge"
chmod +x "$BIN_DIR/fedoraforge"

# Install icon if it exists
ICON_DIR="$INSTALL_PREFIX/share/icons/hicolor/scalable/apps"
if [ -f "src/assets/fedoraforge.svg" ]; then
    echo "🎨 Installing icon..."
    mkdir -p "$ICON_DIR"
    cp src/assets/fedoraforge.svg "$ICON_DIR/fedoraforge.svg"
    # Update icon cache if gtk-update-icon-cache exists
    if command -v gtk-update-icon-cache &> /dev/null; then
        gtk-update-icon-cache -f -t "$INSTALL_PREFIX/share/icons/hicolor" 2>/dev/null || true
    fi
    ICON_NAME="fedoraforge"
else
    ICON_NAME="application-x-executable"
fi

# Create .desktop file
echo "📝 Creating .desktop file..."
cat > "$DESKTOP_DIR/fedoraforge.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=FedoraForge
GenericName=Package Manager
Comment=A modern package manager for Fedora
Exec=$BIN_DIR/fedoraforge %f
Icon=$ICON_NAME
Terminal=false
Categories=System;PackageManager;
Keywords=package;manager;dnf;rpm;install;update;
StartupNotify=true
MimeType=application/x-rpm;
Actions=

[Desktop Action InstallRPM]
Name=Install RPM Package
Exec=$BIN_DIR/fedoraforge gui
EOF

# Make .desktop file executable
chmod +x "$DESKTOP_DIR/fedoraforge.desktop"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "🔄 Updating desktop database..."
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "📁 Binary installed to: $BIN_DIR/fedoraforge"
echo "📁 Desktop file: $DESKTOP_DIR/fedoraforge.desktop"
echo ""
echo "You can now run FedoraForge with:"
echo "  $BIN_DIR/fedoraforge"
echo ""
echo "Or find it in your application menu as 'FedoraForge'"
