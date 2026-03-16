#!/bin/bash

# Configuration
REPO="tsegaye27/bible-cli"
BINARY_NAME="bible-cli"
INSTALL_PATH="/usr/local/bin/$BINARY_NAME"

echo "----------------------------------------"
echo "  Installing Bible CLI (Amharic)        "
echo "----------------------------------------"

# 1. Download the latest binary
echo "Checking for latest release..."
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$BINARY_NAME"

echo "Downloading from GitHub..."
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
    echo "Note: For Amharic characters, ensure your terminal font"
    echo "is set to 'Noto Sans Ethiopic' or similar."
else
    echo "Error: Failed to move binary to $INSTALL_PATH."
    exit 1
fi
