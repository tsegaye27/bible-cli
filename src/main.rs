mod app;
mod cli;
mod data;
mod ui;

use anyhow::Result;
use arboard::Clipboard;
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Write};
use std::process::ExitCode;

use crate::app::{App, Pane};
use crate::cli::Command;

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{}", message);
            return ExitCode::from(2);
        }
    };

    let result = match command {
        Command::Help => {
            print!("{}", cli::HELP);
            Ok(())
        }
        Command::Version => {
            println!("bible-cli version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Quote => print_random_quote(),
        Command::Tui => run_tui(),
    };

    if let Err(err) = result {
        eprintln!("bible-cli: {:?}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn print_random_quote() -> Result<()> {
    let quote = data::random_quote()?;

    println!("{}", quote.text);
    println!();
    println!(
        "— {} ({}) {}:{}",
        quote.book_name_am, quote.book_name_en, quote.chapter, quote.verse
    );

    Ok(())
}

/// Puts the terminal back the way it was found.
///
/// Deliberately best-effort and idempotent: it runs from the panic hook, from
/// `TerminalGuard::drop`, and potentially from both for the same panic.
fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show);
    let _ = disable_raw_mode();
    let _ = stdout.flush();
}

/// Restores the terminal when the TUI scope ends, however it ends.
///
/// Covers the early returns that `?` introduces; the panic hook covers the
/// `panic = "abort"` case, where no destructor runs at all.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// Runs `restore_terminal` before the usual panic output, so the message is
/// printed to a terminal that can actually display it.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
}

fn run_tui() -> Result<()> {
    install_panic_hook();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let books_metadata = data::load_books_metadata()?;
    let mut app = App::new(books_metadata);
    app.load_current_book_filtered()?;

    run_app(&mut terminal, &mut app)
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B::Error: std::error::Error + Send + Sync + 'static,
{
    let mut clipboard = Clipboard::new().ok();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if app.is_searching {
                match key.code {
                    KeyCode::Enter => {
                        app.commit_search();
                    }
                    KeyCode::Esc => {
                        app.is_searching = false;
                        app.search_query.clear();
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        reset_selection_for_search(app);
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        reset_selection_for_search(app);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.next_item(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_item(),
                    KeyCode::Char('h') | KeyCode::Left => app.previous_pane(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        if app.active_pane == Pane::Verses {
                            if let (Some(cb), Some(text)) =
                                (&mut clipboard, app.get_selected_verse())
                            {
                                cb.set_text(text).ok();
                            }
                        } else {
                            app.next_pane();
                        }
                    }
                    KeyCode::Char('/') => {
                        app.is_searching = true;
                        app.search_query.clear();
                    }
                    KeyCode::Char('g') | KeyCode::Home => match app.active_pane {
                        Pane::Books => {
                            app.books_state.select(Some(0));
                            app.load_current_book_filtered().ok();
                        }
                        Pane::Chapters => app.chapters_state.select(Some(0)),
                        Pane::Verses => app.verses_state.select(Some(0)),
                    },
                    KeyCode::Char('G') | KeyCode::End => match app.active_pane {
                        Pane::Books => {
                            let len = app.filtered_books().len();
                            app.books_state.select(Some(len.saturating_sub(1)));
                            app.load_current_book_filtered().ok();
                        }
                        Pane::Chapters => {
                            let len = app.filtered_chapters().len();
                            app.chapters_state.select(Some(len.saturating_sub(1)));
                        }
                        Pane::Verses => {
                            let len = app.filtered_verses().len();
                            app.verses_state.select(Some(len.saturating_sub(1)));
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

/// After the query changes, the old cursor position means nothing — the list
/// underneath it has been rebuilt.
fn reset_selection_for_search(app: &mut App) {
    match app.active_pane {
        Pane::Books => {
            app.books_state.select(Some(0));
            app.load_current_book_filtered().ok();
        }
        Pane::Chapters => app.chapters_state.select(Some(0)),
        Pane::Verses => app.verses_state.select(Some(0)),
    }
}
