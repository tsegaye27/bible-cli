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

### Fonts for Amharic

Most terminal fonts have no Amharic letters, so the terminal borrows them from a proportional font, and they overlap. The installers add **[Hibur Mono](https://fonts.google.com/specimen/Hibur+Mono)** (SIL Open Font License 1.1), a monospaced font whose Ethiopic letters each fit one terminal cell. To skip it, run the installer with `BIBLE_CLI_NO_FONT=1`.

A program can't choose its terminal's font, so point your terminal at Hibur Mono. Where the terminal allows it, use Hibur Mono only for Amharic and keep your usual font for everything else. The installer prints the step for the terminal it runs in.

| Terminal | Setting |
|---|---|
| Ghostty | Add to `~/.config/ghostty/config`: `font-codepoint-map = U+1200-U+139F,U+2D80-U+2DDF,U+AB00-U+AB2F=Hibur Mono` |
| kitty | Add to `~/.config/kitty/kitty.conf`: `symbol_map U+1200-U+139F,U+2D80-U+2DDF,U+AB00-U+AB2F Hibur Mono` |
| iTerm2 | Settings > Profiles > Text > tick "Use a different font for non-ASCII text", choose Hibur Mono |
| WezTerm | `config.font = wezterm.font_with_fallback { 'JetBrains Mono', 'Hibur Mono' }` |
| Windows Terminal | Settings > your profile > Appearance > Font face: `Cascadia Mono, Hibur Mono` |
| VS Code terminal | `"terminal.integrated.fontFamily": "'JetBrains Mono', 'Hibur Mono'"` |
| macOS Terminal.app | One font per profile, so the installer adds a separate **Bible** profile that uses Hibur Mono. Read in a window with it (Shell > New Window > Bible). To add it by hand, download [`assets/Bible.terminal`](assets/Bible.terminal) and open it. |
| GNOME Terminal | One font per profile: make a profile with Hibur Mono under Preferences > Profiles > Text > Custom font. |

**Windows:** use Windows Terminal (built into Windows 11, free in the Microsoft Store for Windows 10). The old console window that `cmd` opens on Windows 10 has no font fallback and can't show Amharic properly.

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

See all the options:
```bash
bible-cli --help
```

### Setup

1. Clone this repository.
2. Run the application:
   ```bash
   cargo run --release
   ```
3. Run the checks (the same ones CI runs):
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

## Keybindings

- `j` / `Down`: Move down in list / scroll text.
- `k` / `Up`: Move up in list / scroll up.
- `h` / `Left`: Move focus to the left pane.
- `l` / `Right` / `Enter`: Move focus to the right pane / select. In the verses pane, `Enter` copies the verse (the status bar confirms it).
- `/`: Filter the current pane. Books match on their Amharic name, their English name (case-insensitive, e.g. `psal` or `Psa`), or their number.
- `f`: Search the whole Bible. Type a word, press `Enter`, then pick a result to jump to it.
- `m`: Bookmark the selected verse (press again to remove). Bookmarked verses show a `★`.
- `b`: List bookmarks. `Enter` jumps, `d` deletes, `Esc` closes.
- `Esc`: Exit search mode and clear filter.
- `g` / `Home`: Jump to top.
- `G` / `End`: Jump to bottom.
- `q` / `Esc`: Quit or exit search.

The reader reopens where you left off.

## Configuration

Bible CLI keeps two files in its config directory:

- macOS / Linux: `~/.config/bible-cli/` (or `$XDG_CONFIG_HOME/bible-cli/`)
- Windows: `%APPDATA%\bible-cli\`

`state.json` holds your last position and bookmarks; the app writes it. `config.json` is yours to create, to change the colors:

```json
{
  "theme": {
    "accent": "magenta",
    "muted": "darkgray",
    "text": "#abb2bf",
    "verse": "#f0f0f0",
    "highlight_bg": "#282c34",
    "status_bg": "#21252b",
    "search": "yellow"
  }
}
```

Every key is optional. Values are color names, `#rrggbb` hex codes, or 256-color indexes. If the file has a typo, the app opens with the default colors and shows the error in the status bar.

## Bible text

Amharic Bible text used with permission of the Bible Society of Ethiopia and the Ethiopian Orthodox Tewahedo Church Patriarchate. Free and non-commercial.

The text is the Ethiopian Orthodox Tewahedo Church's 81-book Bible (1980 E.C. edition), as transcribed in the EthioApps *Amharic 81 Orthodox Bible* app. Under the terms of that permission, Bible CLI must stay free and non-commercial, and the text must not be sold or monetized in any way.

## Deployment

This application is **completely self-contained**. The Bible data is embedded into the binary at compile time.

1. Build the release version:
   ```bash
   cargo build --release
   ```
2. The compiled binary will be located at `target/release/bible-cli`.
3. You can move this single binary to your `/usr/local/bin` or any directory in your `PATH`.
4. Run it from anywhere by simply typing `bible-cli`.
