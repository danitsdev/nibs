#!/usr/bin/env bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}==>${NC} Installing Nibble..."

# Determine OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ]; then
    echo -e "${RED}Error:${NC} Nibble is a Linux-first tool. Unsupported OS: $OS"
    exit 1
fi

if [ "$ARCH" != "x86_64" ]; then
    echo -e "${RED}Error:${NC} Unsupported architecture: $ARCH. Nibble currently only provides official release binaries for x86_64."
    exit 1
fi

# Fetch the latest release tag from GitHub API
echo -e "${BLUE}==>${NC} Fetching latest release info..."
TAG=$(curl -sfS https://api.github.com/repos/danitsdev/nibble/releases/latest | grep -o '"tag_name": "[^"]*' | grep -o '[^"]*$' || echo "v0.1.0")

if [ -z "$TAG" ]; then
    TAG="v0.1.0"
fi

echo -e "${BLUE}==>${NC} Downloading version $TAG..."
URL="https://github.com/danitsdev/nibble/releases/download/${TAG}/nibble-${TAG}-linux-x86_64.tar.gz"

TEMP_DIR=$(mktemp -d)
TARBALL="${TEMP_DIR}/nibble.tar.gz"

if ! curl -fL -o "$TARBALL" "$URL"; then
    echo -e "${RED}Error:${NC} Failed to download release from $URL"
    rm -rf "$TEMP_DIR"
    exit 1
fi

echo -e "${BLUE}==>${NC} Extracting binary..."
tar -xzf "$TARBALL" -C "$TEMP_DIR"

# The tarball contains a folder named nibble-${TAG}
BINARY="${TEMP_DIR}/nibble-${TAG}/nibs"

if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error:${NC} Binary 'nibs' not found in the release archive."
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Determine install directory
INSTALL_DIR="/usr/local/bin"
USE_SUDO=false

if [ ! -w "$INSTALL_DIR" ]; then
    if [ "$EUID" -ne 0 ]; then
        # Check if sudo is available
        if command -v sudo >/dev/null 2>&1; then
            USE_SUDO=true
        else
            # Fallback to user binary directory
            INSTALL_DIR="${HOME}/.local/bin"
            mkdir -p "$INSTALL_DIR"
            if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
                echo -e "${RED}Warning:${NC} $INSTALL_DIR is not in your PATH."
            fi
        fi
    fi
fi

echo -e "${BLUE}==>${NC} Installing nibs binary to ${INSTALL_DIR}..."

if [ "$USE_SUDO" = true ]; then
    sudo mv "$BINARY" "${INSTALL_DIR}/nibs"
    sudo chmod +x "${INSTALL_DIR}/nibs"
else
    mv "$BINARY" "${INSTALL_DIR}/nibs"
    chmod +x "${INSTALL_DIR}/nibs"
fi

# Cleanup
rm -rf "$TEMP_DIR"

echo -e "${GREEN}✓${NC} Nibble has been successfully installed!"
echo -e "Try running: ${GREEN}nibs${NC}"
