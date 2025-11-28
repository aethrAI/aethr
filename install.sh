#!/bin/bash
# Aethr Install Script
# Usage: curl -fsSL https://aethr-ai.dev/install.sh | bash

set -e

REPO="aethrAI/aethr"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="aethr"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}Aethr Installer${NC}"
echo ""

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

case "$OS" in
    linux)
        TARGET="${ARCH}-unknown-linux-gnu"
        ;;
    darwin)
        TARGET="${ARCH}-apple-darwin"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

echo "Detected: $OS/$ARCH"
echo "Target: $TARGET"
echo ""

# Get latest release (or specific version)
VERSION="${AETHR_VERSION:-}"
if [ -z "$VERSION" ]; then
    echo "Fetching latest release..."
    LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
else
    echo "Installing version: $VERSION"
    LATEST="$VERSION"
fi

if [ -z "$LATEST" ]; then
    echo -e "${RED}No releases found. Building from source...${NC}"
    echo ""
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Clone and build
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    git clone "https://github.com/$REPO.git" aethr
    cd aethr
    cargo build --release
    
    # Install
    if [ -w "$INSTALL_DIR" ]; then
        cp target/release/aethr "$INSTALL_DIR/"
    else
        sudo cp target/release/aethr "$INSTALL_DIR/"
    fi
    
    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"
else
    echo "Latest version: $LATEST"
    
    # Download binary
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST/aethr-$TARGET"
    
    echo "Downloading from $DOWNLOAD_URL..."
    
    TEMP_FILE=$(mktemp)
    if curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
        chmod +x "$TEMP_FILE"
        
        if [ -w "$INSTALL_DIR" ]; then
            mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
        else
            sudo mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
        fi
    else
        echo -e "${RED}Failed to download binary. Building from source...${NC}"
        rm -f "$TEMP_FILE"
        
        # Fallback to source build
        if ! command -v cargo &> /dev/null; then
            echo "Installing Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        fi
        
        TEMP_DIR=$(mktemp -d)
        cd "$TEMP_DIR"
        git clone "https://github.com/$REPO.git" aethr
        cd aethr
        cargo build --release
        
        if [ -w "$INSTALL_DIR" ]; then
            cp target/release/aethr "$INSTALL_DIR/"
        else
            sudo cp target/release/aethr "$INSTALL_DIR/"
        fi
        
        cd /
        rm -rf "$TEMP_DIR"
    fi
fi

# Verify installation
if command -v aethr &> /dev/null; then
    echo ""
    echo -e "${GREEN}Aethr installed successfully!${NC}"
    echo ""
    aethr --version
    echo ""
    echo "Next steps:"
    echo "  1. Run: aethr init"
    echo "  2. Run: aethr import"
    echo "  3. Run: aethr hook --install"
    echo ""
else
    echo -e "${RED}Installation failed${NC}"
    exit 1
fi
