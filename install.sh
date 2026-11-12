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

# Hibur Mono: a monospaced Ethiopic font (SIL Open Font License 1.1), so
# Amharic letters fit the terminal grid instead of overlapping.
# Set BIBLE_CLI_NO_FONT=1 to skip.
FONT_URL="https://raw.githubusercontent.com/google/fonts/main/ofl/hiburmono/HiburMono-Regular.ttf"
FONT_FILE="HiburMono-Regular.ttf"
FONT_INSTALLED=0

install_font() {
    if [ "${BIBLE_CLI_NO_FONT:-0}" = "1" ]; then
        return
    fi

    local font_tmp
    font_tmp="$(mktemp "${TMPDIR:-/tmp}/${FONT_FILE}.XXXXXX")"
    echo "Downloading the Hibur Mono font for Amharic..."
    if ! curl -fsSL "$FONT_URL" -o "$font_tmp"; then
        rm -f "$font_tmp"
        echo "Warning: could not download the font. Bible CLI still works; see the font notes below."
        return
    fi

    case "$PLATFORM" in
        macos)
            mkdir -p "$HOME/Library/Fonts"
            mv "$font_tmp" "$HOME/Library/Fonts/$FONT_FILE"
            ;;
        linux)
            local font_dir="${XDG_DATA_HOME:-$HOME/.local/share}/fonts"
            mkdir -p "$font_dir"
            mv "$font_tmp" "$font_dir/$FONT_FILE"
            if command -v fc-cache >/dev/null 2>&1; then
                fc-cache -f "$font_dir" >/dev/null 2>&1 || true
            fi
            ;;
        windows)
            # Per-user font install: no admin rights needed (Windows 10 1809+).
            local win_tmp
            win_tmp="$(cygpath -w "$font_tmp" 2>/dev/null || echo "$font_tmp")"
            if ! powershell.exe -NoProfile -Command "
                \$dir = Join-Path \$env:LOCALAPPDATA 'Microsoft\\Windows\\Fonts'
                New-Item -ItemType Directory -Force -Path \$dir | Out-Null
                \$dest = Join-Path \$dir '$FONT_FILE'
                Copy-Item -Force '$win_tmp' \$dest
                New-ItemProperty -Force -Path 'HKCU:\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts' -Name 'Hibur Mono Regular (TrueType)' -Value \$dest | Out-Null
            "; then
                rm -f "$font_tmp"
                echo "Warning: could not install the font."
                return
            fi
            rm -f "$font_tmp"
            ;;
    esac
    FONT_INSTALLED=1
}

# Unicode ranges for Ethiopic, Ethiopic Supplement and Ethiopic Extended(-A).
ETHIOPIC_RANGES="U+1200-U+139F,U+2D80-U+2DDF,U+AB00-U+AB2F"
PROFILE_URL="https://raw.githubusercontent.com/$REPO/main/assets/Bible.terminal"

# Which terminal is this? Ghostty and kitty set their own variables, which
# survive inside tmux; TERM_PROGRAM covers the rest.
detect_terminal() {
    if [ -n "${KITTY_WINDOW_ID:-}" ]; then echo kitty
    elif [ -n "${GHOSTTY_RESOURCES_DIR:-}" ]; then echo ghostty
    elif [ -n "${WT_SESSION:-}" ]; then echo windows-terminal
    else
        case "${TERM_PROGRAM:-}" in
            Apple_Terminal) echo terminal-app ;;
            iTerm.app) echo iterm ;;
            WezTerm) echo wezterm ;;
            *) echo unknown ;;
        esac
    fi
}

# Terminal.app has one font per profile, so add a separate "Bible" profile
# instead of changing the user's own. Opening the file imports it.
install_terminal_profile() {
    local profile="${TMPDIR:-/tmp}/Bible.terminal"
    curl -fsSL "$PROFILE_URL" -o "$profile" && open "$profile"
}

print_font_help() {
    if [ "$FONT_INSTALLED" != "1" ]; then
        echo "For Amharic, install the monospaced Hibur Mono font:"
        echo "  https://fonts.google.com/specimen/Hibur+Mono"
        echo "then follow the font steps in the README:"
        echo "  https://github.com/$REPO#fonts-for-amharic"
        return
    fi
    echo "Installed the Hibur Mono font for Amharic."
    case "$(detect_terminal)" in
        ghostty)
            echo "Ghostty: add this line to ~/.config/ghostty/config, then restart Ghostty:"
            echo "  font-codepoint-map = $ETHIOPIC_RANGES=Hibur Mono"
            ;;
        kitty)
            echo "kitty: add this line to ~/.config/kitty/kitty.conf, then restart kitty:"
            echo "  symbol_map $ETHIOPIC_RANGES Hibur Mono"
            ;;
        iterm)
            echo "iTerm2: Settings > Profiles > Text > tick 'Use a different font"
            echo "for non-ASCII text' and choose Hibur Mono."
            ;;
        wezterm)
            echo "WezTerm: in ~/.wezterm.lua, set"
            echo "  config.font = wezterm.font_with_fallback { 'JetBrains Mono', 'Hibur Mono' }"
            ;;
        windows-terminal)
            echo "Windows Terminal: Settings > your profile > Appearance > Font face,"
            echo "enter: Cascadia Mono, Hibur Mono"
            ;;
        terminal-app)
            if install_terminal_profile; then
                echo "Terminal.app: added a 'Bible' profile that uses Hibur Mono and"
                echo "opened a window with it. Read in that window (Shell > New Window > Bible);"
                echo "your usual profile is unchanged."
            else
                echo "Terminal.app: Settings > Profiles > Text > Font, choose Hibur Mono."
            fi
            ;;
        *)
            echo "Point your terminal at Hibur Mono for Amharic; steps for each terminal:"
            echo "  https://github.com/$REPO#fonts-for-amharic"
            ;;
    esac
}

print_success() {
    echo "----------------------------------------"
    echo "Successfully installed Bible CLI!"
    echo "You can now run it by typing: $BINARY_NAME"
    echo "----------------------------------------"
    print_font_help
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

    install_font
    print_success
    echo ""
    echo "Note: Open a new terminal if $BINARY_NAME is not found."
else
    INSTALL_PATH="/usr/local/bin/$BINARY_NAME"
    chmod +x "$TMP_FILE"
    echo "Moving binary to $INSTALL_PATH (Sudo password may be required)..."
    if sudo mv "$TMP_FILE" "$INSTALL_PATH"; then
        install_font
        print_success
    else
        echo "Error: Failed to move binary to $INSTALL_PATH."
        exit 1
    fi
fi
