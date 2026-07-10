#!/bin/bash

set -euo pipefail

# Configuration
REPO="tsegaye27/bible-cli"
BINARY_NAME="bible-cli"

echo "----------------------------------------"
echo "  Installing Bible CLI (Amharic)        "
echo "----------------------------------------"

# Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        PLATFORM="linux"
        EXT=""
        ;;
    darwin)
        PLATFORM="macos"
        EXT=""
        ;;
    mingw*|msys*|cygwin*)
        PLATFORM="windows"
        EXT=".exe"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        echo ""
        echo "On Windows, use PowerShell instead:"
        echo "  irm https://raw.githubusercontent.com/tsegaye27/bible-cli/main/install.ps1 | iex"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
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

DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/${BINARY_NAME}-${PLATFORM}-${ARCH_SUFFIX}${EXT}"
TMP_FILE="$(mktemp "${TMPDIR:-/tmp}/${BINARY_NAME}.XXXXXX")"
trap 'rm -f "$TMP_FILE"' EXIT

echo "Checking for latest release for $PLATFORM-$ARCH_SUFFIX..."
echo "Downloading from GitHub: $DOWNLOAD_URL"

if ! curl -fL "$DOWNLOAD_URL" -o "$TMP_FILE"; then
    echo "Error: Download failed. Check your internet or GitHub URL."
    exit 1
fi
echo "Download successful."

print_success() {
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
}

if [ "$PLATFORM" = "windows" ]; then
    # Prefer LOCALAPPDATA; fall back to HOME for unusual Git Bash setups.
    if [ -n "${LOCALAPPDATA:-}" ]; then
        INSTALL_DIR="$LOCALAPPDATA/bible-cli"
    else
        INSTALL_DIR="$HOME/AppData/Local/bible-cli"
    fi
    INSTALL_PATH="$INSTALL_DIR/${BINARY_NAME}.exe"

    mkdir -p "$INSTALL_DIR"
    mv "$TMP_FILE" "$INSTALL_PATH"
    chmod +x "$INSTALL_PATH"

    # Add install dir to the user PATH if missing (persists for new shells).
    WIN_INSTALL_DIR="$(cygpath -w "$INSTALL_DIR" 2>/dev/null || echo "$INSTALL_DIR")"
    powershell.exe -NoProfile -Command "
        \$dir = '$WIN_INSTALL_DIR'
        \$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if (\$userPath -notlike ('*' + \$dir + '*')) {
            \$newPath = if (\$userPath) { \$userPath + ';' + \$dir } else { \$dir }
            [Environment]::SetEnvironmentVariable('Path', \$newPath, 'User')
            Write-Host \"Added \$dir to your user PATH.\"
        }
    " || echo "Warning: could not update PATH automatically. Add $INSTALL_DIR to PATH manually."

    # Make it available in the current Git Bash session too.
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) export PATH="$INSTALL_DIR:$PATH" ;;
    esac

    print_success
    echo ""
    echo "Note: Open a new terminal if $BINARY_NAME is not found."
else
    INSTALL_PATH="/usr/local/bin/$BINARY_NAME"
    chmod +x "$TMP_FILE"
    echo "Moving binary to $INSTALL_PATH (Sudo password may be required)..."
    if sudo mv "$TMP_FILE" "$INSTALL_PATH"; then
        print_success
    else
        echo "Error: Failed to move binary to $INSTALL_PATH."
        exit 1
    fi
fi
