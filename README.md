# Bible CLI

A modern, high-performance Bible reading terminal application with a Yazi-inspired three-pane display and Vim-like navigation. Built in Rust with `ratatui`.

## Features

- **Yazi-like Display**: Three columns for Books, Chapters, and Verses.
- **Search**: Real-time filtering for books and chapters using `/`.
- **Vim-like Navigation**: Use `h`, `j`, `k`, `l` or arrow keys to navigate.
- **Modern TUI**: Smooth, rounded borders and a refined dark theme.
- **Amharic Support**: Full support for Ethiopic characters.
- **Cross-Platform**: Support for Linux (x86_64, aarch64), macOS (Intel & Apple Silicon), and Windows (x86_64, ARM64).

## Installation

### Quick Install (Linux & macOS)

Run the following command in your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/tsegaye27/bible-cli/main/install.sh | bash
```

### Quick Install (Windows)

Run this in **PowerShell** (not Command Prompt):

```powershell
irm https://raw.githubusercontent.com/tsegaye27/bible-cli/main/install.ps1 | iex
```

If you get an execution policy error, run this instead:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm 'https://raw.githubusercontent.com/tsegaye27/bible-cli/main/install.ps1' | iex"
```

**Git Bash on Windows:** the Linux/macOS install command above also works (it installs the Windows `.exe`).

**WSL on Windows:** use the Linux/macOS install command above.

### Prerequisites

To see Amharic characters correctly, you **must** use a terminal font that supports Ethiopic characters.

**Recommended Fonts:**
1. **[Abyssinica SIL](https://software.sil.org/abyssinica/)** (Highly Recommended)
2. **Noto Sans Ethiopic** (Standard)
3. **Kawsar** (Clear and modern)
4. **FiraGO** (Excellent multi-script support)

#### How to change terminal font:
- **macOS (Terminal.app)**: Settings > Profiles > Text > Font.
- **macOS (iTerm2)**: Settings > Profiles > Text > Font.
- **Linux (Gnome Terminal)**: Preferences > Profiles > Text > Custom font.
- **VS Code**: Settings > `editor.fontFamily`.
- **Windows Terminal**: Settings > Profiles > Appearance > Font face.

### Updating

To update to the latest version, simply re-run the installation command:

```bash
curl -fsSL https://raw.githubusercontent.com/tsegaye27/bible-cli/main/install.sh | bash
```

Check your current version with:
```bash
bible-cli --version
```

Print a random verse:
```bash
bible-cli --quote
# or
bible-cli -q
```

### Setup

1. Clone this repository.
2. Run the application:
   ```bash
   cargo run --release
   ```

## Keybindings

- `j` / `Down`: Move down in list / scroll text.
- `k` / `Up`: Move up in list / scroll up.
- `h` / `Left`: Move focus to the left pane.
- `l` / `Right` / `Enter`: Move focus to the right pane / select.
- `/`: Enter search mode (filters current pane).
- `Esc`: Exit search mode and clear filter.
- `Enter`: (In search mode) Confirm selection and exit search.
- `g` / `Home`: Jump to top.
- `G` / `End`: Jump to bottom.
- `q` / `Esc`: Quit or exit search.

## Deployment

This application is **completely self-contained**. The Bible data is embedded into the binary at compile time.

1. Build the release version:
   ```bash
   cargo build --release
   ```
2. The compiled binary will be located at `target/release/bible-cli`.
3. You can move this single binary to your `/usr/local/bin` or any directory in your `PATH`.
4. Run it from anywhere by simply typing `bible-cli`.
