#!/bin/bash
set -e

VERSION="${AETHR_VERSION:-v1.0.0-beta}"
INSTALL_DIR="${AETHR_INSTALL_DIR:-$HOME/.local/bin}"
REPO="aethrAI/aethr"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

info() { echo -e "${BLUE}==>${NC} $1"; }
success() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}==>${NC} $1"; }
error() { echo -e "${RED}ERROR:${NC} $1"; exit 1; }

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"
    
    case "$os" in
        Linux)
            case "$arch" in
                x86_64) echo "x86_64-unknown-linux-gnu" ;;
                *) error "Unsupported Linux architecture: $arch (only x86_64 supported)" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64) echo "aarch64-apple-darwin" ;;
                *) error "Unsupported macOS architecture: $arch" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "x86_64-pc-windows-msvc"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
}

is_windows() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) return 0 ;;
        *) return 1 ;;
    esac
}

main() {
    info "Installing Aethr $VERSION..."
    
    local target
    target="$(detect_platform)"
    info "Platform: $target"
    
    # Set install directory for Windows
    if is_windows; then
        INSTALL_DIR="${AETHR_INSTALL_DIR:-$HOME/bin}"
    fi
    
    mkdir -p "$INSTALL_DIR"
    
    local binary_name="aethr"
    local url="https://github.com/$REPO/releases/download/$VERSION/aethr-$target"
    
    if is_windows; then
        binary_name="aethr.exe"
        url="${url}.exe"
    fi
    
    info "Downloading..."
    
    if command -v curl &> /dev/null; then
        curl -fsSL "$url" -o "$INSTALL_DIR/$binary_name" || error "Download failed. Check that release $VERSION exists."
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$INSTALL_DIR/$binary_name" || error "Download failed. Check that release $VERSION exists."
    else
        error "Neither curl nor wget found."
    fi
    
    chmod +x "$INSTALL_DIR/$binary_name"
    
    success "Installed to $INSTALL_DIR/$binary_name"
    
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        if is_windows; then
            echo "  Add $INSTALL_DIR to your Windows PATH environment variable"
            echo "  Or run: export PATH=\"$INSTALL_DIR:\$PATH\""
        else
            echo "  Add to ~/.bashrc or ~/.zshrc:"
            echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    fi
    
    echo ""
    success "Done. Run 'aethr init' to get started."
}

main

