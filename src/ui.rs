use crate::app::{App, Pane, PopupKind};
use crate::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};
use textwrap::{Options, wrap};

pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                                       // Content
            Constraint::Length(if app.is_searching { 1 } else { 0 }), // Search input
            Constraint::Length(1),                                    // Status bar
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

    let active_border_style = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let inactive_border_style = Style::default().fg(theme.muted);
    let highlight_style = Style::default()
        .bg(theme.highlight_bg)
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let pane_block = |title: String, pane: Pane| {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .border_style(if app.active_pane == pane {
                active_border_style
            } else {
                inactive_border_style
            })
    };

    // --- Books Pane ---
    let books: Vec<ListItem> = app
        .filtered_books()
        .iter()
        .map(|b| {
            ListItem::new(format!(" {:02} {}", b.number, b.name_am))
                .style(Style::default().fg(theme.text))
        })
        .collect();
    let books_list = List::new(books)
        .block(pane_block(" መጻሕፍት ".to_string(), Pane::Books))
        .highlight_style(highlight_style)
        .highlight_symbol(" ");

    // --- Chapters Pane ---
    let chapters: Vec<ListItem> = app
        .filtered_chapters()
        .iter()
        .map(|c| ListItem::new(format!(" ምዕራፍ {}", c)).style(Style::default().fg(theme.text)))
        .collect();
    let chapters_list = List::new(chapters)
        .block(pane_block(" ምዕራፎች ".to_string(), Pane::Chapters))
        .highlight_style(highlight_style)
        .highlight_symbol(" ");

    // --- Verses Pane ---
    let filtered_verses = app.filtered_verses();
    // Borders, padding, and a column for the scrollbar.
    let verses_pane_width = content_chunks[2].width.saturating_sub(5).max(1) as usize;

    let filtering = app.is_searching && !app.search_everywhere && app.active_pane == Pane::Verses;
    let notes = app.chapter_notes();
    let verses: Vec<ListItem> = filtered_verses
        .iter()
        .enumerate()
        .map(|(i, (n, t))| {
            let mut lines: Vec<Line> = Vec::new();
            if let Some(heading) = app.heading_before(*n) {
                for l in wrap(heading, Options::new(verses_pane_width)) {
                    lines.push(Line::styled(
                        l.into_owned(),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::ITALIC),
                    ));
                }
            }
            let mark = if app.is_bookmarked(*n) { "★ " } else { "" };
            let full_text = format!("{}{}. {}", mark, app.verse_label(*n), t);
            for l in wrap(&full_text, Options::new(verses_pane_width)) {
                lines.push(Line::styled(
                    l.into_owned(),
                    Style::default().fg(theme.verse),
                ));
            }
            // Footnotes follow the last verse, unless a filter has reordered the list.
            if !filtering && i + 1 == filtered_verses.len() {
                for note in notes {
                    lines.push(Line::default());
                    for l in wrap(note, Options::new(verses_pane_width)) {
                        lines.push(Line::styled(
                            l.into_owned(),
                            Style::default().fg(theme.muted),
                        ));
                    }
                }
            }
            lines.push(Line::default());
            ListItem::new(lines)
        })
        .collect();

    let verse_count = filtered_verses.len();
    let verse_pos = app.verses_state.selected().filter(|&i| i < verse_count);
    let verses_title = match verse_pos {
        Some(i) => format!(" ቁጥሮች {}/{} ", i + 1, verse_count),
        None => " ቁጥሮች ".to_string(),
    };
    let verses_list = List::new(verses)
        .block(pane_block(verses_title, Pane::Verses).padding(Padding::horizontal(1)))
        .highlight_style(highlight_style)
        .highlight_symbol(" ");

    frame.render_stateful_widget(books_list, content_chunks[0], &mut app.books_state);
    frame.render_stateful_widget(chapters_list, content_chunks[1], &mut app.chapters_state);
    frame.render_stateful_widget(verses_list, content_chunks[2], &mut app.verses_state);

    if verse_count > 1 {
        // Verses wrap to different heights, so the visible count isn't known;
        // a one-item viewport keeps the thumb small and marks the position.
        let mut scroll = ScrollbarState::new(verse_count)
            .position(verse_pos.unwrap_or(0))
            .viewport_content_length(1);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .style(Style::default().fg(theme.muted)),
            content_chunks[2].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scroll,
        );
    }

    // --- Search Input ---
    if app.is_searching {
        let title = if app.search_everywhere {
            "ፈልግ (ሁሉም መጻሕፍት)"
        } else {
            match app.active_pane {
                Pane::Books => "ፈልግ (መጻሕፍት)",
                Pane::Chapters => "ፈልግ (ምዕራፎች)",
                Pane::Verses => "ፈልግ (ቁጥሮች)",
            }
        };
        let search_text = format!(" {} > /{}", title, app.search_query);
        let search_bar = Paragraph::new(search_text).style(
            Style::default()
                .fg(theme.search)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(search_bar, main_layout[1]);
    }

    // --- Status Bar ---
    let status_area = main_layout[if app.is_searching { 2 } else { 1 }];
    let status_bar = if let Some(message) = &app.status {
        Paragraph::new(format!(" {}", message)).style(
            Style::default()
                .bg(theme.status_bg)
                .fg(theme.search)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        let location = app
            .current_location()
            .map(|l| {
                let name = app
                    .current_book
                    .as_ref()
                    .map(|b| b.book_name_am.as_str())
                    .unwrap_or("");
                format!("{} {}:{}", name, l.chapter, l.verse)
            })
            .unwrap_or_default();
        Paragraph::new(format!(
            " [q] Quit  [/] Filter  [f] Search all  [m] Bookmark  [b] Bookmarks  [Enter] Copy | {}",
            location
        ))
        .style(Style::default().bg(theme.status_bg).fg(theme.muted))
    };
    frame.render_widget(status_bar, status_area);

    // --- Popup ---
    if let Some(popup) = &mut app.popup {
        let area = centered(frame.area(), 80, 70);
        let hint = match popup.kind {
            PopupKind::Bookmarks => " Enter: open  d: delete  Esc: close ",
            PopupKind::Results => " Enter: open  Esc: close ",
        };
        let items: Vec<ListItem> = if popup.hits.is_empty() {
            let empty = match popup.kind {
                PopupKind::Bookmarks => " No bookmarks yet. Press m on a verse to add one.",
                PopupKind::Results => " No verses found.",
            };
            vec![ListItem::new(empty).style(Style::default().fg(theme.muted))]
        } else {
            popup
                .hits
                .iter()
                .map(|h| {
                    ListItem::new(format!(" {}", h.label)).style(Style::default().fg(theme.verse))
                })
                .collect()
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(active_border_style)
                    .title(popup.title.clone())
                    .title_bottom(hint),
            )
            .highlight_style(highlight_style);
        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, &mut popup.state);
    }
}

/// A rectangle `width_pct` by `height_pct` of `area`, centred in it.
fn centered(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(height_pct)])
        .flex(ratatui::layout::Flex::Center)
        .split(area)[0];
    Layout::horizontal([Constraint::Percentage(width_pct)])
        .flex(ratatui::layout::Flex::Center)
        .split(vertical)[0]
}
