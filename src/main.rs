mod data;
mod app;
mod ui;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use anyhow::Result;
use arboard::Clipboard;
use crate::app::{App, Pane};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("bible-cli version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.contains(&"--quote".to_string()) || args.contains(&"-q".to_string()) {
        let quote = data::random_quote()?;
        println!("{}", quote.text);
        println!();
        println!(
            "— {} ({}) {}:{}",
            quote.book_name_am, quote.book_name_en, quote.chapter, quote.verse
        );
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let books_metadata = data::load_books_metadata()?;
    let mut app = App::new(books_metadata);
    app.load_current_book_filtered().ok();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> 
where 
    B::Error: std::error::Error + Send + Sync + 'static 
{
    let mut clipboard = Clipboard::new().ok();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
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
                            match app.active_pane {
                                Pane::Books => {
                                    app.books_state.select(Some(0));
                                    app.load_current_book_filtered().ok();
                                }
                                Pane::Chapters => {
                                    app.chapters_state.select(Some(0));
                                }
                                Pane::Verses => {
                                    app.verses_state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            match app.active_pane {
                                Pane::Books => {
                                    app.books_state.select(Some(0));
                                    app.load_current_book_filtered().ok();
                                }
                                Pane::Chapters => {
                                    app.chapters_state.select(Some(0));
                                }
                                Pane::Verses => {
                                    app.verses_state.select(Some(0));
                                }
                            }
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
                                if let (Some(cb), Some(text)) = (&mut clipboard, app.get_selected_verse()) {
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
                        KeyCode::Char('g') | KeyCode::Home => {
                            match app.active_pane {
                                Pane::Books => {
                                    app.books_state.select(Some(0));
                                    app.load_current_book_filtered().ok();
                                },
                                Pane::Chapters => app.chapters_state.select(Some(0)),
                                Pane::Verses => app.verses_state.select(Some(0)),
                            }
                        }
                        KeyCode::Char('G') | KeyCode::End => {
                            match app.active_pane {
                                Pane::Books => {
                                    let len = app.filtered_books().len();
                                    app.books_state.select(Some(len.saturating_sub(1)));
                                    app.load_current_book_filtered().ok();
                                },
                                Pane::Chapters => {
                                    let len = app.filtered_chapters().len();
                                    app.chapters_state.select(Some(len.saturating_sub(1)));
                                }
                                Pane::Verses => {
                                    let len = app.filtered_verses().len();
                                    app.verses_state.select(Some(len.saturating_sub(1)));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
