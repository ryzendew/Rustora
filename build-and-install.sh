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
echo -e "${BLUE}Step 0/5: Installing build dependencies...${NC}"

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
echo -e "${BLUE}Step 0.5/5: Checking fpm installation...${NC}"

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
# STEP 0.75: INSTALL CFHDB (Required for device management)
# ============================================================================
echo -e "${BLUE}Step 0.75/5: Installing cfhdb package...${NC}"

# Check if cfhdb is already installed
if rpm -q cfhdb &> /dev/null 2>&1; then
    CFHDB_VERSION=$(rpm -q cfhdb 2>/dev/null | head -n1 || echo "unknown")
    echo -e "${GREEN}[OK] cfhdb already installed (version: $CFHDB_VERSION)${NC}"
    echo "   Skipping cfhdb installation."
else
    echo "[DOWNLOAD] Fetching latest cfhdb build from Copr..."
    
    # Copr repository details
    COPR_USER="gloriouseggroll"
    COPR_PROJECT="nobara-43"
    COPR_PACKAGE="cfhdb"
    ARCH="x86_64"
    FEDORA_VERSION="43"
    
    # Get the latest build ID from Copr package page
    LATEST_BUILD_ID=""
    
    if command -v curl &> /dev/null; then
        echo "[INFO] Fetching latest build information from Copr..."
        BUILD_PAGE_URL="https://copr.fedorainfracloud.org/coprs/${COPR_USER}/${COPR_PROJECT}/package/${COPR_PACKAGE}/"
        
        # Get the latest build ID from the HTML page (look for the first build link in the table)
        LATEST_BUILD_ID=$(curl -s "${BUILD_PAGE_URL}" | grep -oP 'href="/coprs/[^"]+/build/\K[0-9]+' | head -n1 || echo "")
        
        if [ -z "$LATEST_BUILD_ID" ]; then
            # Try alternative pattern
            LATEST_BUILD_ID=$(curl -s "${BUILD_PAGE_URL}" | grep -oP 'build/\K[0-9]+' | head -n1 || echo "")
        fi
    elif command -v wget &> /dev/null; then
        echo "[INFO] Fetching latest build information from Copr..."
        BUILD_PAGE_URL="https://copr.fedorainfracloud.org/coprs/${COPR_USER}/${COPR_PROJECT}/package/${COPR_PACKAGE}/"
        LATEST_BUILD_ID=$(wget -qO- "${BUILD_PAGE_URL}" | grep -oP 'href="/coprs/[^"]+/build/\K[0-9]+' | head -n1 || echo "")
    else
        echo -e "${YELLOW}[WARN] Neither curl nor wget found. Cannot fetch latest build ID.${NC}"
        echo "   Please install curl or wget, or manually install cfhdb package."
    fi
    
    if [ -z "$LATEST_BUILD_ID" ]; then
        echo -e "${YELLOW}[WARN] Could not determine latest build ID from web page.${NC}"
        echo "   Attempting to find latest build in download directory..."
        LATEST_BUILD_ID=""
    else
        echo "[INFO] Found latest build ID: $LATEST_BUILD_ID"
    fi
    
    # Build results base URL
    # Pattern: https://download.copr.fedorainfracloud.org/results/{USER}/{PROJECT}/fedora-{VERSION}-{ARCH}/{BUILD_ID}-{PACKAGE}/{PACKAGE}-{VERSION}-{RELEASE}.fc{VERSION}.{ARCH}.rpm
    # Example: https://download.copr.fedorainfracloud.org/results/gloriouseggroll/nobara-43/fedora-43-x86_64/09821493-cfhdb/cfhdb-0.1.2-1.fc43.x86_64.rpm
    BUILD_RESULTS_BASE="https://download.copr.fedorainfracloud.org/results/${COPR_USER}/${COPR_PROJECT}/fedora-${FEDORA_VERSION}-${ARCH}/"
    
    echo "[DOWNLOAD] Attempting to download cfhdb RPM..."
    
    # Create temporary directory for download
    TEMP_DIR=$(mktemp -d)
    RPM_FILE="${TEMP_DIR}/cfhdb.rpm"
    
    DOWNLOAD_SUCCESS=false
    
    if command -v curl &> /dev/null; then
        # If we have a build ID, try the correct URL pattern: {BUILD_ID}-{PACKAGE}/
        if [ -n "$LATEST_BUILD_ID" ]; then
            # Format build ID with leading zeros (8 digits)
            FORMATTED_BUILD_ID=$(printf "%08d" "$LATEST_BUILD_ID")
            BUILD_DIR_NAME="${FORMATTED_BUILD_ID}-${COPR_PACKAGE}"
            BUILD_DIR_URL="${BUILD_RESULTS_BASE}${BUILD_DIR_NAME}/"
            echo "[INFO] Checking build directory: $BUILD_DIR_URL"
            
            # Get directory listing and find the RPM file
            DIR_LISTING=$(curl -s "${BUILD_DIR_URL}" 2>/dev/null || echo "")
            if [ -n "$DIR_LISTING" ]; then
                # Extract RPM filename from directory listing
                RPM_FILENAME=$(echo "$DIR_LISTING" | grep -oP 'href="\K[^"]*\.rpm' | grep "fc${FEDORA_VERSION}" | grep "${ARCH}" | head -n1 || echo "")
                if [ -n "$RPM_FILENAME" ]; then
                    RPM_URL="${BUILD_DIR_URL}${RPM_FILENAME}"
                    echo "[DOWNLOAD] Downloading: $RPM_FILENAME"
                    if curl -f -L -o "$RPM_FILE" "$RPM_URL" 2>/dev/null && [ -f "$RPM_FILE" ] && [ -s "$RPM_FILE" ]; then
                        DOWNLOAD_SUCCESS=true
                        echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                    fi
                fi
            fi
            
            # If directory listing failed, try direct download with common filename pattern
            if [ "$DOWNLOAD_SUCCESS" = false ]; then
                # Try common RPM filename patterns (based on known version 0.1.2-1)
                for VERSION_PATTERN in "0.1.2-1" "0.1.2" "0.1"; do
                    RPM_FILENAME="${COPR_PACKAGE}-${VERSION_PATTERN}.fc${FEDORA_VERSION}.${ARCH}.rpm"
                    RPM_URL="${BUILD_DIR_URL}${RPM_FILENAME}"
                    echo "[TRY] Attempting direct download: $RPM_FILENAME"
                    if curl -f -L -o "$RPM_FILE" "$RPM_URL" 2>/dev/null && [ -f "$RPM_FILE" ] && [ -s "$RPM_FILE" ]; then
                        # Verify it's actually an RPM file (check file magic or use rpm command)
                        if command -v file &> /dev/null && file "$RPM_FILE" | grep -qE "(RPM|rpm)"; then
                            DOWNLOAD_SUCCESS=true
                            echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                            break
                        elif command -v rpm &> /dev/null && rpm -qp "$RPM_FILE" &> /dev/null; then
                            DOWNLOAD_SUCCESS=true
                            echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                            break
                        elif [ -s "$RPM_FILE" ]; then
                            # If file exists and has content, assume it's valid (fallback)
                            DOWNLOAD_SUCCESS=true
                            echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                            break
                        fi
                    fi
                done
            fi
        fi
        
        # If that failed, try to find the latest build directory
        if [ "$DOWNLOAD_SUCCESS" = false ]; then
            echo "[INFO] Searching for latest build directory..."
            DIR_LISTING=$(curl -s "${BUILD_RESULTS_BASE}" 2>/dev/null || echo "")
            if [ -n "$DIR_LISTING" ]; then
                # Find directories matching pattern: {BUILD_ID}-{PACKAGE}
                # Sort by build ID (extract number before the dash)
                LATEST_BUILD_DIR=$(echo "$DIR_LISTING" | grep -oP "href=\"\K[0-9]+-${COPR_PACKAGE}/" | sed 's|/$||' | sed 's/-.*//' | sort -rn | head -n1 || echo "")
                if [ -n "$LATEST_BUILD_DIR" ]; then
                    FORMATTED_BUILD_ID=$(printf "%08d" "$LATEST_BUILD_DIR")
                    BUILD_DIR_NAME="${FORMATTED_BUILD_ID}-${COPR_PACKAGE}"
                    BUILD_DIR_URL="${BUILD_RESULTS_BASE}${BUILD_DIR_NAME}/"
                    echo "[INFO] Found build directory: $BUILD_DIR_NAME"
                    DIR_LISTING=$(curl -s "${BUILD_DIR_URL}" 2>/dev/null || echo "")
                    if [ -n "$DIR_LISTING" ]; then
                        RPM_FILENAME=$(echo "$DIR_LISTING" | grep -oP 'href="\K[^"]*\.rpm' | grep "fc${FEDORA_VERSION}" | grep "${ARCH}" | head -n1 || echo "")
                        if [ -n "$RPM_FILENAME" ]; then
                            RPM_URL="${BUILD_DIR_URL}${RPM_FILENAME}"
                            echo "[DOWNLOAD] Downloading: $RPM_FILENAME"
                            if curl -f -L -o "$RPM_FILE" "$RPM_URL" 2>/dev/null && [ -f "$RPM_FILE" ] && [ -s "$RPM_FILE" ]; then
                                DOWNLOAD_SUCCESS=true
                                echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                            fi
                        fi
                    fi
                fi
            fi
        fi
    elif command -v wget &> /dev/null; then
        # Similar logic for wget
        if [ -n "$LATEST_BUILD_ID" ]; then
            FORMATTED_BUILD_ID=$(printf "%08d" "$LATEST_BUILD_ID")
            BUILD_DIR_NAME="${FORMATTED_BUILD_ID}-${COPR_PACKAGE}"
            BUILD_DIR_URL="${BUILD_RESULTS_BASE}${BUILD_DIR_NAME}/"
            DIR_LISTING=$(wget -qO- "${BUILD_DIR_URL}" 2>/dev/null || echo "")
            if [ -n "$DIR_LISTING" ]; then
                RPM_FILENAME=$(echo "$DIR_LISTING" | grep -oP 'href="\K[^"]*\.rpm' | grep "fc${FEDORA_VERSION}" | grep "${ARCH}" | head -n1 || echo "")
                if [ -n "$RPM_FILENAME" ]; then
                    RPM_URL="${BUILD_DIR_URL}${RPM_FILENAME}"
                    if wget -q -O "$RPM_FILE" "$RPM_URL" 2>/dev/null && [ -f "$RPM_FILE" ] && [ -s "$RPM_FILE" ]; then
                        DOWNLOAD_SUCCESS=true
                        echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                    fi
                fi
            fi
            
            # Try direct download if listing failed
            if [ "$DOWNLOAD_SUCCESS" = false ]; then
                for VERSION_PATTERN in "0.1.2-1" "0.1.2" "0.1"; do
                    RPM_FILENAME="${COPR_PACKAGE}-${VERSION_PATTERN}.fc${FEDORA_VERSION}.${ARCH}.rpm"
                    RPM_URL="${BUILD_DIR_URL}${RPM_FILENAME}"
                    if wget -q -O "$RPM_FILE" "$RPM_URL" 2>/dev/null && [ -f "$RPM_FILE" ] && [ -s "$RPM_FILE" ]; then
                        if file "$RPM_FILE" | grep -q "RPM"; then
                            DOWNLOAD_SUCCESS=true
                            echo -e "${GREEN}[OK] Successfully downloaded cfhdb RPM${NC}"
                            break
                        fi
                    fi
                done
            fi
        fi
    fi
    
    if [ "$DOWNLOAD_SUCCESS" = true ] && [ -f "$RPM_FILE" ] && [ -s "$RPM_FILE" ]; then
        echo "[INSTALL] Installing cfhdb RPM..."
        if [ "$EUID" -eq 0 ]; then
            dnf install -y "$RPM_FILE"
        elif check_sudo; then
            echo "   (Using sudo to install cfhdb...)"
            sudo dnf install -y "$RPM_FILE"
        else
            echo -e "${YELLOW}[WARN] Cannot use sudo. Please install cfhdb manually:${NC}"
            echo "   sudo dnf install -y $RPM_FILE"
            echo ""
            read -p "Press Enter to continue (device management features may not work)..." || true
        fi
        
        # Clean up
        rm -f "$RPM_FILE"
        rmdir "$TEMP_DIR" 2>/dev/null || true
        
        # Verify installation
        if rpm -q cfhdb &> /dev/null; then
            CFHDB_VERSION=$(rpm -q cfhdb)
            echo -e "${GREEN}[OK] cfhdb installed successfully (version: $CFHDB_VERSION)${NC}"
        else
            echo -e "${YELLOW}[WARN] cfhdb installation may have failed. Device management features may not work.${NC}"
        fi
    else
        echo -e "${YELLOW}[WARN] Failed to download cfhdb RPM automatically.${NC}"
        echo "   Device management features (start/stop/enable/disable) will not work."
        echo "   To install manually, visit:"
        echo "   https://copr.fedorainfracloud.org/coprs/${COPR_USER}/${COPR_PROJECT}/package/${COPR_PACKAGE}/"
        echo "   Or enable the Copr repository and install:"
        echo "   sudo dnf copr enable ${COPR_USER}/${COPR_PROJECT}"
        echo "   sudo dnf install -y ${COPR_PACKAGE}"
        echo ""
        read -p "Press Enter to continue (device management features may not work)..." || true
        rm -f "$RPM_FILE" 2>/dev/null || true
        rmdir "$TEMP_DIR" 2>/dev/null || true
    fi
fi

echo ""

# ============================================================================
# STEP 1: BUILD
# ============================================================================
echo -e "${BLUE}Step 1/5: Building...${NC}"

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
echo -e "${BLUE}Step 2/5: Installing...${NC}"

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
