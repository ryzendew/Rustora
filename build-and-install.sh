#!/bin/bash
# Build and Install script for Rustora

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Rustora Build & Install${NC}"
echo "======================================"
echo ""

# ============================================================================
# STEP 0: INSTALL BUILD DEPENDENCIES
# ============================================================================
echo -e "${BLUE}Step 0/3: Installing build dependencies...${NC}"

# Check if dnf is available
if ! command -v dnf &> /dev/null; then
    echo -e "${YELLOW}⚠️  Warning: dnf not found. Skipping dependency installation.${NC}"
    echo "   Please manually install: pciutils-devel libusb1-devel glibc-devel gcc clang"
else
    # Required build dependencies
    DEPS="pciutils-devel libusb1-devel glibc-devel gcc clang"
    
    # Check which packages are missing
    MISSING_DEPS=""
    for dep in $DEPS; do
        if ! rpm -q "$dep" &> /dev/null; then
            MISSING_DEPS="$MISSING_DEPS $dep"
        fi
    done
    
    if [ -z "$MISSING_DEPS" ]; then
        echo -e "${GREEN}✅ Build dependencies already installed${NC}"
    else
        echo "📦 Installing build dependencies:$MISSING_DEPS"
        if [ "$EUID" -eq 0 ]; then
            dnf install -y $MISSING_DEPS
        elif command -v sudo &> /dev/null; then
            echo "   (Using sudo to install dependencies...)"
            sudo dnf install -y $MISSING_DEPS
        else
            echo -e "${YELLOW}⚠️  Need sudo to install dependencies. Please run:${NC}"
            echo "   sudo dnf install -y$MISSING_DEPS"
            echo ""
            read -p "Press Enter to continue (build may fail if dependencies are missing)..." || true
        fi
    fi
fi

echo ""

# ============================================================================
# STEP 1: BUILD
# ============================================================================
echo -e "${BLUE}Step 1/3: Building...${NC}"

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

# Always do a clean build
echo "🧹 Cleaning previous build artifacts..."
cargo clean

# Check if debug build is requested
BUILD_MODE="release"
BINARY_PATH="target/release/rustora"
if [ "$1" = "debug" ]; then
    BUILD_MODE="debug"
    BINARY_PATH="target/debug/rustora"
    echo "📦 Compiling in debug mode..."
    cargo build
else
    echo "📦 Compiling in release mode..."
    cargo build --release
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build successful!${NC}"
echo ""

# ============================================================================
# STEP 2: INSTALL
# ============================================================================
echo -e "${BLUE}Step 2/3: Installing...${NC}"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
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
cp "$BINARY_PATH" "$BIN_DIR/rustora"
chmod +x "$BIN_DIR/rustora"

# Install icon if it exists
ICON_DIR="$INSTALL_PREFIX/share/icons/hicolor/scalable/apps"
if [ -f "src/assets/rustora.svg" ]; then
    echo "🎨 Installing icon..."
    mkdir -p "$ICON_DIR"
    cp src/assets/rustora.svg "$ICON_DIR/rustora.svg"
    # Update icon cache if gtk-update-icon-cache exists
    if command -v gtk-update-icon-cache &> /dev/null; then
        gtk-update-icon-cache -f -t "$INSTALL_PREFIX/share/icons/hicolor" 2>/dev/null || true
    fi
    ICON_NAME="rustora"
else
    ICON_NAME="application-x-executable"
fi

# Create .desktop file
echo "📝 Creating .desktop file..."
cat > "$DESKTOP_DIR/rustora.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Rustora
GenericName=Package Manager
Comment=A modern package manager for Fedora
Exec=$BIN_DIR/rustora %f
Icon=$ICON_NAME
Terminal=false
Categories=System;PackageManager;
Keywords=package;manager;dnf;rpm;install;update;
StartupNotify=true
MimeType=application/x-rpm;
Actions=

[Desktop Action InstallRPM]
Name=Install RPM Package
Exec=$BIN_DIR/rustora gui
EOF

# Make .desktop file executable
chmod +x "$DESKTOP_DIR/rustora.desktop"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "🔄 Updating desktop database..."
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "📁 Binary installed to: $BIN_DIR/rustora"
echo "📁 Desktop file: $DESKTOP_DIR/rustora.desktop"
echo ""
echo "You can now run Rustora with:"
echo "  $BIN_DIR/rustora"
echo ""
echo "Or find it in your application menu as 'Rustora'"
