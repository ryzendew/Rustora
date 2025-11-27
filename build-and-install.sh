#!/bin/bash
# Build and Install script for Rustora

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if sudo works
check_sudo() {
    if [ "$EUID" -eq 0 ]; then
        return 0
    fi
    if ! command -v sudo &> /dev/null; then
        return 1
    fi
    sudo -n true 2>/dev/null || sudo -v 2>/dev/null
    return $?
}

echo -e "${BLUE}Rustora Build & Install${NC}"
echo "======================================"
echo ""

# ============================================================================
# STEP 0: INSTALL BUILD DEPENDENCIES
# ============================================================================
echo -e "${BLUE}Step 0/4: Installing build dependencies...${NC}"

# Check if dnf is available
if ! command -v dnf &> /dev/null; then
    echo -e "${YELLOW}[WARN] dnf not found. Skipping dependency installation.${NC}"
    echo "   Please manually install all build dependencies"
else
    # Required build dependencies for rustora
    DEPS="rust rustc cargo \
          gcc gcc-c++ pkg-config clang \
          openssl-devel \
          libX11-devel libXcursor-devel libXrandr-devel libXi-devel \
          mesa-libGL-devel \
          fontconfig-devel freetype-devel expat-devel \
          pciutils-devel libusb1-devel glibc-devel \
          dnf rpm polkit zenity curl unzip fontconfig \
          cairo-gobject cairo-gobject-devel \
          rust-gdk4-sys+default-devel \
          gtk4-layer-shell-devel \
          qt5-qtgraphicaleffects \
          qt6-qt5compat \
          python3-pyqt6 \
          python3.11 python3.11-libs \
          libxcrypt-compat libcurl libcurl-devel apr fuse-libs \
          golang git make"
    
    # Check which packages are missing
    MISSING_DEPS=""
    for dep in $DEPS; do
        if ! rpm -q "$dep" &> /dev/null 2>&1; then
            MISSING_DEPS="$MISSING_DEPS $dep"
        fi
    done
    
    if [ -z "$MISSING_DEPS" ]; then
        echo -e "${GREEN}[OK] Build dependencies already installed${NC}"
    else
        echo "[INSTALL] Installing build dependencies..."
        if [ "$EUID" -eq 0 ]; then
            dnf install -y $MISSING_DEPS
        elif check_sudo; then
            echo "   (Using sudo to install dependencies...)"
            sudo dnf install -y $MISSING_DEPS
        else
            echo -e "${YELLOW}[WARN] Cannot use sudo (no new privileges flag or not available)${NC}"
            echo -e "${YELLOW}[WARN] Please install dependencies manually:${NC}"
            echo "   dnf install -y$MISSING_DEPS"
            echo ""
            echo -e "${YELLOW}[WARN] Or run this script as root, or fix sudo configuration${NC}"
            echo ""
            read -p "Press Enter to continue (build may fail if dependencies are missing)..." || true
        fi
    fi
fi

echo ""

# ============================================================================
# STEP 0.5: INSTALL FPM (Effing Package Manager)
# ============================================================================
echo -e "${BLUE}Step 0.5/4: Checking fpm installation...${NC}"

# Check if fpm is already installed and working
FPM_INSTALLED=false
if command -v fpm &> /dev/null; then
    # Verify fpm actually works (not just a broken symlink)
    if fpm --version &> /dev/null; then
        FPM_VERSION=$(fpm --version 2>/dev/null | head -n1 || echo "unknown")
        echo -e "${GREEN}[OK] fpm already installed (version: $FPM_VERSION)${NC}"
        echo "   Skipping fpm installation."
        FPM_INSTALLED=true
    else
        echo -e "${YELLOW}[WARN] fpm found but not working properly. Will reinstall...${NC}"
    fi
fi

# Only install if not already installed
if [ "$FPM_INSTALLED" = false ]; then
    echo "[INSTALL] Installing fpm and Ruby dependencies..."
    
    # Check if dnf is available
    if ! command -v dnf &> /dev/null; then
        echo -e "${YELLOW}[WARN] dnf not found. Skipping fpm installation.${NC}"
        echo "   Please manually install: ruby ruby-devel rubygems gcc make"
        echo "   Then run: gem install fpm"
    else
        # FPM dependencies
        FPM_DEPS="ruby ruby-devel rubygems gcc make"
        
        # Check which packages are missing
        MISSING_FPM_DEPS=""
        for dep in $FPM_DEPS; do
            if ! rpm -q "$dep" &> /dev/null; then
                MISSING_FPM_DEPS="$MISSING_FPM_DEPS $dep"
            fi
        done
        
        if [ -n "$MISSING_FPM_DEPS" ]; then
            echo "[INSTALL] Installing fpm dependencies:$MISSING_FPM_DEPS"
            if [ "$EUID" -eq 0 ]; then
                dnf install -y $MISSING_FPM_DEPS
            elif check_sudo; then
                echo "   (Using sudo to install dependencies...)"
                sudo dnf install -y $MISSING_FPM_DEPS
            else
                echo -e "${YELLOW}[WARN] Cannot use sudo. Please install fpm dependencies manually:${NC}"
                echo "   dnf install -y$MISSING_FPM_DEPS"
                echo ""
                read -p "Press Enter to continue (fpm installation may fail)..." || true
            fi
        fi
        
        # Install bundler if not present
        if ! gem list -i bundler &> /dev/null; then
            echo "[INSTALL] Installing bundler..."
            if [ "$EUID" -eq 0 ]; then
                gem install bundler
            elif check_sudo; then
                sudo gem install bundler
            else
                echo "[INSTALL] Installing bundler for current user..."
                gem install bundler --user-install
                export PATH="$HOME/.local/share/gem/ruby/$(ruby -e 'puts RUBY_VERSION[/\d+\.\d+/]')/bin:$PATH"
            fi
        fi
        
        # Install fpm
        echo "[INSTALL] Installing fpm gem..."
        if [ "$EUID" -eq 0 ]; then
            gem install fpm
        elif check_sudo; then
            sudo gem install fpm
        else
            echo "[INSTALL] Installing fpm for current user..."
            gem install fpm --user-install
            export PATH="$HOME/.local/share/gem/ruby/$(ruby -e 'puts RUBY_VERSION[/\d+\.\d+/]')/bin:$PATH"
        fi
        
        # Verify installation
        if command -v fpm &> /dev/null && fpm --version &> /dev/null; then
            FPM_VERSION=$(fpm --version 2>/dev/null | head -n1 || echo "unknown")
            echo -e "${GREEN}[OK] fpm installed successfully (version: $FPM_VERSION)${NC}"
        else
            echo -e "${YELLOW}[WARN] fpm installed but not in PATH. You may need to add gem bin directory to PATH.${NC}"
            echo "   Try: export PATH=\"\$HOME/.local/share/gem/ruby/\$(ruby -e 'puts RUBY_VERSION[/\d+\.\d+/]')/bin:\$PATH\""
        fi
    fi
fi

echo ""

# ============================================================================
# STEP 1: BUILD
# ============================================================================
echo -e "${BLUE}Step 1/4: Building...${NC}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null || ! command -v rustc &> /dev/null; then
    echo -e "${RED}[ERROR] Rust toolchain not found.${NC}"
    echo "   The build dependencies installation should have installed rust, rustc, and cargo."
    echo "   If they are still missing, please run:"
    echo "   sudo dnf install -y rust rustc cargo"
    exit 1
fi

# Check Rust version (need 1.70+ for edition 2021)
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    REQUIRED_VERSION="1.70.0"
    
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        echo -e "${YELLOW}[WARN] Rust version $RUST_VERSION may be too old. Recommended: $REQUIRED_VERSION or later${NC}"
    else
        echo -e "${GREEN}[OK] Rust version $RUST_VERSION is compatible${NC}"
    fi
fi

# Always do a clean build
echo "[CLEAN] Cleaning previous build artifacts..."
cargo clean

# Check if debug build is requested
BUILD_MODE="release"
BINARY_PATH="target/release/rustora"
if [ "$1" = "debug" ]; then
    BUILD_MODE="debug"
    BINARY_PATH="target/debug/rustora"
    echo "[BUILD] Compiling in debug mode..."
    cargo build
else
    echo "[BUILD] Compiling in release mode..."
    cargo build --release
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}[ERROR] Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}[OK] Build successful!${NC}"
echo ""

# ============================================================================
# STEP 2: INSTALL
# ============================================================================
echo -e "${BLUE}Step 2/4: Installing...${NC}"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}[ERROR] Binary not found after build${NC}"
    exit 1
fi

# Check if running as root for system-wide install
if [ "$EUID" -eq 0 ]; then
    INSTALL_PREFIX="/usr/local"
    DESKTOP_DIR="/usr/share/applications"
    BIN_DIR="$INSTALL_PREFIX/bin"
    echo "[INSTALL] Installing system-wide..."
else
    INSTALL_PREFIX="$HOME/.local"
    DESKTOP_DIR="$HOME/.local/share/applications"
    BIN_DIR="$INSTALL_PREFIX/bin"
    echo "[INSTALL] Installing for current user..."
fi

# Create directories
mkdir -p "$BIN_DIR"
mkdir -p "$DESKTOP_DIR"

# Install binary
echo "[INSTALL] Installing binary to $BIN_DIR..."
cp "$BINARY_PATH" "$BIN_DIR/rustora"
chmod +x "$BIN_DIR/rustora"

# Install icon if it exists
ICON_DIR="$INSTALL_PREFIX/share/icons/hicolor/scalable/apps"
if [ -f "src/assets/rustora.svg" ]; then
    echo "[ICON] Installing icon..."
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
echo "[DESKTOP] Creating .desktop file..."
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
    echo "[UPDATE] Updating desktop database..."
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}[OK] Installation complete!${NC}"
echo ""
echo "[PATH] Binary installed to: $BIN_DIR/rustora"
echo "[PATH] Desktop file: $DESKTOP_DIR/rustora.desktop"
echo ""
if command -v fpm &> /dev/null; then
    echo "[INFO] fpm (Effing Package Manager) is installed and ready to use"
    echo "   Run 'fpm --version' to verify"
fi
echo ""
echo "You can now run Rustora with:"
echo "  $BIN_DIR/rustora"
echo ""
echo "Or find it in your application menu as 'Rustora'"
