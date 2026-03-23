#!/bin/bash

# Configuration
REPO="tsegaye27/bible-cli"
BINARY_NAME="bible-cli"
INSTALL_PATH="/usr/local/bin/$BINARY_NAME"

echo "----------------------------------------"
echo "  Installing Bible CLI (Amharic)        "
echo "----------------------------------------"

# Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        PLATFORM="linux"
        ;;
    darwin)
        PLATFORM="macos"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_SUFFIX="x86_64"
        ;;
    aarch64|arm64)
        ARCH_SUFFIX="aarch64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# 1. Download the latest binary
echo "Checking for latest release for $PLATFORM-$ARCH_SUFFIX..."
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/${BINARY_NAME}-${PLATFORM}-${ARCH_SUFFIX}"

echo "Downloading from GitHub: $DOWNLOAD_URL"
if curl -L "$DOWNLOAD_URL" -o "$BINARY_NAME"; then
    echo "Download successful."
else
    echo "Error: Download failed. Check your internet or GitHub URL."
    exit 1
fi

# 2. Make it executable
chmod +x "$BINARY_NAME"

# 3. Move to /usr/local/bin (requires sudo)
echo "Moving binary to $INSTALL_PATH (Sudo password may be required)..."
if sudo mv "$BINARY_NAME" "$INSTALL_PATH"; then
    echo "----------------------------------------"
    echo "Successfully installed Bible CLI!"
    echo "You can now run it by typing: $BINARY_NAME"
    echo "----------------------------------------"
    echo "Font Recommendations for Amharic:"
    echo "----------------------------------------"
    echo "1. Abyssinica SIL (Highly Recommended)"
    echo "2. Noto Sans Ethiopic"
    echo "3. Kawsar"
    echo "4. FiraGO"
    echo ""
    echo "Make sure to set your terminal font to one of these"
    echo "to see Amharic characters correctly."
    echo "----------------------------------------"
else
    echo "Error: Failed to move binary to $INSTALL_PATH."
    exit 1
fi
