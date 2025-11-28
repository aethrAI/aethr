#!/bin/bash
# Aethr Beta Installer
# Usage: curl -fsSL https://aethr-ai.dev/install-beta.sh | bash

set -e

echo ""
echo "Installing Aethr (Beta)..."
echo ""

# Detect OS and architecture  
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64)
        if [ "$OS" = "linux" ]; then
            echo "Error: Linux ARM64 not supported in beta. Use x86_64."
            exit 1
        fi
        ARCH="aarch64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Check OS
if [ "$OS" != "linux" ] && [ "$OS" != "darwin" ]; then
    echo "Error: Unsupported OS. Aethr supports Linux and macOS."
    exit 1
fi

# Download URL from aethr-ai.dev
URL="https://aethr-ai.dev/releases/aethr-${OS}-${ARCH}"

echo "Downloading from $URL..."
curl -fsSL "$URL" -o /tmp/aethr
chmod +x /tmp/aethr

# Install
if [ -w /usr/local/bin ]; then
    mv /tmp/aethr /usr/local/bin/aethr
else
    echo "Need sudo to install to /usr/local/bin"
    sudo mv /tmp/aethr /usr/local/bin/aethr
fi

echo ""
echo "Aethr installed successfully!"
aethr --version
echo ""
echo "Get started:"
echo "  aethr init      # Set up Aethr"  
echo "  aethr import    # Import your shell history"
echo "  aethr recall    # Search your commands"
echo ""
