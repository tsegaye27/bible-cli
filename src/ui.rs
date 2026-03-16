use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, BorderType},
    Frame,
};
use crate::app::{App, Pane};

pub fn render(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Content
            Constraint::Length(if app.is_searching { 1 } else { 0 }), // Search input
            Constraint::Length(1),    // Status bar
        ])
        .split(frame.area());

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Books
            Constraint::Percentage(15), // Chapters
            Constraint::Percentage(65), // Verses
        ])
        .split(main_layout[0]);

    // Themes
    let active_border_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let inactive_border_style = Style::default().fg(Color::DarkGray);
    let highlight_style = Style::default()
        .bg(Color::Rgb(40, 44, 52))
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    // --- Books Pane ---
    let filtered_books = app.filtered_books();
    let books: Vec<ListItem> = filtered_books.iter()
        .map(|b| {
            ListItem::new(format!(" {:02} {}", b.number, b.name_am))
                .style(Style::default().fg(Color::Rgb(171, 178, 191)))
        })
        .collect();

    let books_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" መጻሕፍት ")
        .border_style(if app.active_pane == Pane::Books {
            active_border_style
        } else {
            inactive_border_style
        });

    let books_list = List::new(books)
        .block(books_block)
        .highlight_style(highlight_style)
        .highlight_symbol(" ");
    frame.render_stateful_widget(books_list, content_chunks[0], &mut app.books_state);

    // --- Chapters Pane ---
    let filtered_chapters = app.filtered_chapters();
    let chapters: Vec<ListItem> = filtered_chapters.iter()
        .map(|c| {
            ListItem::new(format!(" ምዕራፍ {}", c))
                .style(Style::default().fg(Color::Rgb(171, 178, 191)))
        })
        .collect();

    let chapters_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" ምዕራፎች ")
        .border_style(if app.active_pane == Pane::Chapters {
            active_border_style
        } else {
            inactive_border_style
        });

    let chapters_list = List::new(chapters)
        .block(chapters_block)
        .highlight_style(highlight_style)
        .highlight_symbol(" ");
    frame.render_stateful_widget(chapters_list, content_chunks[1], &mut app.chapters_state);

    // --- Verses Pane ---
    let verses_data = app.get_flattened_verses();
    let verses_text: String = verses_data.iter()
        .map(|(n, t)| format!("{}. {}", n, t))
        .collect::<Vec<String>>()
        .join("\n\n");

    let verses_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" ቁጥሮች ")
        .border_style(if app.active_pane == Pane::Verses {
            active_border_style
        } else {
            inactive_border_style
        });

    let verses_paragraph = Paragraph::new(verses_text)
        .block(verses_block.padding(ratatui::widgets::Padding::horizontal(1)))
        .wrap(Wrap { trim: true })
        .scroll((app.verse_scroll, 0))
        .style(Style::default().fg(Color::Rgb(240, 240, 240))); // Clean white for text

    frame.render_widget(verses_paragraph, content_chunks[2]);

    // --- Search Input ---
    if app.is_searching {
        let search_text = format!(" /{}", app.search_query);
        let search_bar = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        frame.render_widget(search_bar, main_layout[1]);
    }

    // --- Status Bar ---
    let current_book_name = app.current_book.as_ref()
        .map(|b| b.book_name_am.clone())
        .unwrap_or_else(|| "...".to_string());
    
    let filtered_chapters = app.filtered_chapters();
    let current_chapter = app.chapters_state.selected()
        .and_then(|i| filtered_chapters.get(i))
        .cloned()
        .unwrap_or(0);

    let status_text = format!(
        " [q: Quit] [/: Search] [h/j/k/l: Nav] | {} - ምዕራፍ {}",
        current_book_name, current_chapter
    );
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Rgb(33, 37, 43)).fg(Color::DarkGray));
    frame.render_widget(status_bar, main_layout[if app.is_searching { 2 } else { 1 }]);
}
