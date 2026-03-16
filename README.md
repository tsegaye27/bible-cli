# Bible CLI

A modern, high-performance Bible reading terminal application with a Yazi-inspired three-pane display and Vim-like navigation. Built in Rust with `ratatui`.

## Features

- **Yazi-like Display**: Three columns for Books, Chapters, and Verses.
- **Search**: Real-time filtering for books and chapters using `/`.
- **Vim-like Navigation**: Use `h`, `j`, `k`, `l` or arrow keys to navigate.
- **Modern TUI**: Smooth, rounded borders and a refined dark theme.
- **Amharic Support**: Full support for Ethiopic characters (requires a compatible terminal font).

## Credits

This project uses the Bible data provided by the **80-weahadu** project.
Special thanks to the [EOTCOpenSource/80-weahadu](https://github.com/EOTCOpenSource/80-weahadu) repository for the comprehensive JSON-formatted Amharic Bible data.

## Installation

### Prerequisites

- [Rust and Cargo](https://rustup.rs/) installed.
- A terminal font that supports Ethiopic characters (e.g., **Noto Sans Ethiopic**).

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
