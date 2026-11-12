use crate::data::{Book, BookMetadata, Chapter, load_all_books, load_book};
use crate::store::Location;
use anyhow::Result;
use ratatui::widgets::ListState;

#[derive(PartialEq, Clone, Copy)]
pub enum Pane {
    Books,
    Chapters,
    Verses,
}

pub struct App {
    pub books_metadata: Vec<BookMetadata>,
    pub books_state: ListState,
    pub current_book: Option<Book>,
    pub chapters_state: ListState,
    pub verses_state: ListState,
    pub active_pane: Pane,
    pub search_query: String,
    pub is_searching: bool,
    /// Path of the book currently held in `current_book`, so that repeated
    /// selections of the same book skip both the JSON parse and the reset of
    /// the chapter and verse cursors.
    loaded_book_path: Option<String>,
    /// One-shot message for the status bar, cleared on the next key press.
    pub status: Option<String>,
    pub popup: Option<Popup>,
    pub bookmarks: Vec<Location>,
    /// The search prompt is open for the whole Bible rather than the active pane.
    pub search_everywhere: bool,
    /// Every book, parsed on first use by whole-Bible search and bookmarks.
    all_books: Option<Vec<Book>>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PopupKind {
    Bookmarks,
    Results,
}

/// A list of verses floating over the panes: bookmarks or search results.
pub struct Popup {
    pub kind: PopupKind,
    pub title: String,
    pub hits: Vec<Hit>,
    pub state: ListState,
}

pub struct Hit {
    pub location: Location,
    pub label: String,
}

/// Whole-Bible search stops collecting after this many verses.
pub const MAX_HITS: usize = 1000;

impl App {
    pub fn new(books_metadata: Vec<BookMetadata>) -> App {
        let mut books_state = ListState::default();
        if !books_metadata.is_empty() {
            books_state.select(Some(0));
        }

        App {
            books_metadata,
            books_state,
            current_book: None,
            chapters_state: ListState::default(),
            verses_state: ListState::default(),
            active_pane: Pane::Books,
            search_query: String::new(),
            is_searching: false,
            loaded_book_path: None,
            status: None,
            popup: None,
            bookmarks: Vec::new(),
            search_everywhere: false,
            all_books: None,
        }
    }

    pub fn filtered_books(&self) -> Vec<&BookMetadata> {
        if self.search_query.is_empty() || self.active_pane != Pane::Books {
            self.books_metadata.iter().collect()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.books_metadata
                .iter()
                .filter(|b| {
                    b.name_am.contains(&self.search_query)
                        || b.number.to_string().contains(&self.search_query)
                        || b.name_en.to_lowercase().contains(&query_lower)
                        || b.short_name_en.to_lowercase().contains(&query_lower)
                })
                .collect()
        }
    }

    pub fn filtered_chapters(&self) -> Vec<u32> {
        if let Some(book) = &self.current_book {
            let chapters: Vec<u32> = book.chapters.iter().map(|c| c.chapter).collect();
            if self.search_query.is_empty() || self.active_pane != Pane::Chapters {
                chapters
            } else {
                chapters
                    .into_iter()
                    .filter(|c| c.to_string().starts_with(&self.search_query))
                    .collect()
            }
        } else {
            Vec::new()
        }
    }

    pub fn filtered_verses(&self) -> Vec<(u32, String)> {
        let all_verses = self.get_flattened_verses_raw();
        if self.search_query.is_empty() || self.active_pane != Pane::Verses {
            all_verses
        } else {
            all_verses
                .into_iter()
                .filter(|(n, t)| {
                    n.to_string().starts_with(&self.search_query) || t.contains(&self.search_query)
                })
                .collect()
        }
    }

    fn get_flattened_verses_raw(&self) -> Vec<(u32, String)> {
        if let Some(chapter) = self.get_current_chapter_raw() {
            chapter
                .sections
                .iter()
                .flat_map(|s| s.verses.iter().map(|v| (v.verse, v.text.clone())))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn next_item(&mut self) {
        match self.active_pane {
            Pane::Books => {
                let filtered = self.filtered_books();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.books_state.selected() {
                    Some(i) => {
                        if i >= filtered.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.books_state.select(Some(i));
                self.load_current_book_filtered().ok();
            }
            Pane::Chapters => {
                let filtered = self.filtered_chapters();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.chapters_state.selected() {
                    Some(i) => {
                        if i >= filtered.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.chapters_state.select(Some(i));
                self.verses_state.select(Some(0));
            }
            Pane::Verses => {
                let filtered = self.filtered_verses();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.verses_state.selected() {
                    Some(i) => {
                        if i >= filtered.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.verses_state.select(Some(i));
            }
        }
    }

    pub fn previous_item(&mut self) {
        match self.active_pane {
            Pane::Books => {
                let filtered = self.filtered_books();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.books_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            filtered.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.books_state.select(Some(i));
                self.load_current_book_filtered().ok();
            }
            Pane::Chapters => {
                let filtered = self.filtered_chapters();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.chapters_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            filtered.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.chapters_state.select(Some(i));
                self.verses_state.select(Some(0));
            }
            Pane::Verses => {
                let filtered = self.filtered_verses();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.verses_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            filtered.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.verses_state.select(Some(i));
            }
        }
    }

    pub fn commit_search(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        match self.active_pane {
            Pane::Books => {
                let filtered = self.filtered_books();
                if let Some(i) = self.books_state.selected()
                    && let Some(selected_book) = filtered.get(i)
                {
                    let target_number = selected_book.number;
                    self.search_query.clear();
                    if let Some(new_i) = self
                        .books_metadata
                        .iter()
                        .position(|b| b.number == target_number)
                    {
                        self.books_state.select(Some(new_i));
                    }
                }
            }
            Pane::Chapters => {
                let filtered = self.filtered_chapters();
                if let Some(i) = self.chapters_state.selected()
                    && let Some(&chapter_num) = filtered.get(i)
                {
                    self.search_query.clear();
                    if let Some(book) = &self.current_book
                        && let Some(new_i) =
                            book.chapters.iter().position(|c| c.chapter == chapter_num)
                    {
                        self.chapters_state.select(Some(new_i));
                    }
                }
            }
            Pane::Verses => {
                let filtered = self.filtered_verses();
                if let Some(i) = self.verses_state.selected()
                    && let Some(&(verse_num, _)) = filtered.get(i)
                {
                    self.search_query.clear();
                    let all_verses = self.get_flattened_verses_raw();
                    if let Some(new_i) = all_verses.iter().position(|(v, _)| *v == verse_num) {
                        self.verses_state.select(Some(new_i));
                    }
                }
            }
        }
        self.search_query.clear();
        self.is_searching = false;
    }

    pub fn next_pane(&mut self) {
        self.commit_search();
        match self.active_pane {
            Pane::Books => {
                if self.current_book.is_some() {
                    self.active_pane = Pane::Chapters;
                    if self.chapters_state.selected().is_none() {
                        self.chapters_state.select(Some(0));
                    }
                }
            }
            Pane::Chapters => {
                self.active_pane = Pane::Verses;
                if self.verses_state.selected().is_none() {
                    self.verses_state.select(Some(0));
                }
            }
            Pane::Verses => {}
        }
    }

    pub fn previous_pane(&mut self) {
        self.commit_search();
        match self.active_pane {
            Pane::Books => {}
            Pane::Chapters => {
                self.active_pane = Pane::Books;
            }
            Pane::Verses => {
                self.active_pane = Pane::Chapters;
            }
        }
    }

    pub fn load_current_book_filtered(&mut self) -> Result<()> {
        let Some(i) = self.books_state.selected() else {
            return Ok(());
        };
        let Some(file_path) = self.filtered_books().get(i).map(|m| m.file_path.clone()) else {
            return Ok(());
        };

        if self.loaded_book_path.as_deref() == Some(file_path.as_str()) {
            return Ok(());
        }

        let book = load_book(&file_path)?;
        self.current_book = Some(book);
        self.loaded_book_path = Some(file_path);
        self.chapters_state.select(Some(0));
        self.verses_state.select(Some(0));
        Ok(())
    }

    fn get_current_chapter_raw(&self) -> Option<&Chapter> {
        let book = self.current_book.as_ref()?;
        let filtered_chapters = self.filtered_chapters();
        let i = self.chapters_state.selected()?;
        let chapter_num = filtered_chapters.get(i)?;
        book.chapters.iter().find(|c| c.chapter == *chapter_num)
    }

    pub fn get_selected_verse(&self) -> Option<String> {
        let verses = self.filtered_verses();
        let i = self.verses_state.selected()?;
        let (n, t) = verses.get(i)?;
        let book_name = self
            .current_book
            .as_ref()
            .map(|b| b.book_name_am.clone())
            .unwrap_or_default();
        let chapter = self
            .get_current_chapter_raw()
            .map(|c| c.chapter)
            .unwrap_or(0);
        Some(format!("{} {}:{} - {}", book_name, chapter, n, t))
    }

    /// The heading printed above verse `n`: a psalm superscription, a book's
    /// introduction, or a section title.
    pub fn heading_before(&self, n: u32) -> Option<&str> {
        self.get_current_chapter_raw()?
            .sections
            .iter()
            .find(|s| s.verses.first().is_some_and(|v| v.verse == n))?
            .title
            .as_deref()
    }

    /// "15-16" for a verse the source joins to the next, otherwise "15".
    pub fn verse_label(&self, n: u32) -> String {
        let end = self.get_current_chapter_raw().and_then(|c| {
            c.sections
                .iter()
                .flat_map(|s| &s.verses)
                .find(|v| v.verse == n)?
                .verse_end
        });
        match end {
            Some(end) => format!("{}-{}", n, end),
            None => n.to_string(),
        }
    }

    pub fn chapter_notes(&self) -> &[String] {
        self.get_current_chapter_raw()
            .map(|c| c.notes.as_slice())
            .unwrap_or_default()
    }

    /// The verse under the cursor, if a book, chapter and verse are selected.
    pub fn current_location(&self) -> Option<Location> {
        let book = self
            .filtered_books()
            .get(self.books_state.selected()?)?
            .number;
        let chapter = self.get_current_chapter_raw()?.chapter;
        let verse = self.filtered_verses().get(self.verses_state.selected()?)?.0;
        Some(Location {
            book,
            chapter,
            verse,
        })
    }

    /// Move every cursor to `loc` and focus the verses pane. Returns `false`,
    /// leaving the cursors wherever they ended up, if `loc` doesn't exist —
    /// e.g. a saved position from an older copy of the data.
    pub fn jump_to(&mut self, loc: Location) -> Result<bool> {
        self.search_query.clear();
        self.is_searching = false;
        self.popup = None;

        let Some(book_i) = self
            .books_metadata
            .iter()
            .position(|b| b.number == loc.book)
        else {
            return Ok(false);
        };
        self.books_state.select(Some(book_i));
        self.load_current_book_filtered()?;

        let Some(chapter_i) = self
            .current_book
            .as_ref()
            .and_then(|b| b.chapters.iter().position(|c| c.chapter == loc.chapter))
        else {
            return Ok(false);
        };
        self.chapters_state.select(Some(chapter_i));

        let verse_i = self
            .get_flattened_verses_raw()
            .iter()
            .position(|(v, _)| *v == loc.verse);
        self.verses_state.select(Some(verse_i.unwrap_or(0)));
        self.active_pane = Pane::Verses;
        Ok(verse_i.is_some())
    }

    /// Bookmark the verse under the cursor, or remove the bookmark if it has one.
    /// Returns whether the bookmark list changed.
    pub fn toggle_bookmark(&mut self) -> bool {
        let Some(loc) = self.current_location() else {
            self.status = Some("Select a verse to bookmark it".to_string());
            return false;
        };
        if let Some(i) = self.bookmarks.iter().position(|b| *b == loc) {
            self.bookmarks.remove(i);
            self.status = Some(format!("Bookmark removed: {}", self.reference(loc)));
        } else {
            self.bookmarks.push(loc);
            self.status = Some(format!("Bookmarked: {}", self.reference(loc)));
        }
        true
    }

    pub fn is_bookmarked(&self, verse: u32) -> bool {
        let (Some(book), Some(chapter)) = (
            self.current_book.as_ref().map(|b| b.book_number),
            self.get_current_chapter_raw().map(|c| c.chapter),
        ) else {
            return false;
        };
        self.bookmarks.contains(&Location {
            book,
            chapter,
            verse,
        })
    }

    pub fn open_bookmarks(&mut self) -> Result<()> {
        self.ensure_all_books()?;
        let hits = self
            .bookmarks
            .iter()
            .map(|&location| Hit {
                location,
                label: self.describe(location),
            })
            .collect();
        self.open_popup(
            PopupKind::Bookmarks,
            " ዕልባቶች (Bookmarks) ".to_string(),
            hits,
        );
        Ok(())
    }

    /// Find `query` in every verse of every book and list the matches.
    pub fn search_everywhere(&mut self, query: &str) -> Result<()> {
        self.ensure_all_books()?;
        let books = self.all_books.as_deref().unwrap_or_default();
        let mut hits = Vec::new();
        'outer: for book in books {
            for chapter in &book.chapters {
                for verse in chapter.sections.iter().flat_map(|s| &s.verses) {
                    if verse.text.contains(query) {
                        hits.push(Location {
                            book: book.book_number,
                            chapter: chapter.chapter,
                            verse: verse.verse,
                        });
                        if hits.len() == MAX_HITS {
                            break 'outer;
                        }
                    }
                }
            }
        }
        let count = if hits.len() == MAX_HITS {
            format!("first {MAX_HITS}")
        } else {
            hits.len().to_string()
        };
        let title = format!(" \"{query}\": {count} ");
        let hits = hits
            .into_iter()
            .map(|location| Hit {
                location,
                label: self.describe(location),
            })
            .collect();
        self.open_popup(PopupKind::Results, title, hits);
        Ok(())
    }

    fn open_popup(&mut self, kind: PopupKind, title: String, hits: Vec<Hit>) {
        let mut state = ListState::default();
        if !hits.is_empty() {
            state.select(Some(0));
        }
        self.popup = Some(Popup {
            kind,
            title,
            hits,
            state,
        });
    }

    pub fn popup_move(&mut self, down: bool) {
        let Some(popup) = &mut self.popup else { return };
        let len = popup.hits.len();
        if len == 0 {
            return;
        }
        let i = popup.state.selected().unwrap_or(0);
        popup.state.select(Some(if down {
            (i + 1) % len
        } else {
            (i + len - 1) % len
        }));
    }

    pub fn popup_accept(&mut self) -> Result<()> {
        let Some(loc) = self
            .popup
            .as_ref()
            .and_then(|p| p.hits.get(p.state.selected()?))
            .map(|h| h.location)
        else {
            return Ok(());
        };
        if !self.jump_to(loc)? {
            self.status = Some(format!("{} is not in this Bible", self.reference(loc)));
        }
        Ok(())
    }

    /// Delete the selected bookmark from the bookmarks popup. Returns whether
    /// the bookmark list changed.
    pub fn popup_delete(&mut self) -> bool {
        let Some(popup) = &mut self.popup else {
            return false;
        };
        if popup.kind != PopupKind::Bookmarks {
            return false;
        }
        let Some(i) = popup.state.selected().filter(|&i| i < popup.hits.len()) else {
            return false;
        };
        let loc = popup.hits.remove(i).location;
        self.bookmarks.retain(|b| *b != loc);
        if popup.hits.is_empty() {
            popup.state.select(None);
        } else if i >= popup.hits.len() {
            popup.state.select(Some(popup.hits.len() - 1));
        }
        true
    }

    fn ensure_all_books(&mut self) -> Result<()> {
        if self.all_books.is_none() {
            self.all_books = Some(load_all_books(&self.books_metadata)?);
        }
        Ok(())
    }

    /// "ዘፍ 1:3" for a location, falling back to the book number.
    fn reference(&self, loc: Location) -> String {
        let name = self
            .books_metadata
            .iter()
            .find(|b| b.number == loc.book)
            .map(|b| b.name_am.clone())
            .unwrap_or_else(|| loc.book.to_string());
        format!("{} {}:{}", name, loc.chapter, loc.verse)
    }

    /// The reference plus the start of the verse text, for popup rows.
    fn describe(&self, loc: Location) -> String {
        let text = self
            .all_books
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find(|b| b.book_number == loc.book)
            .and_then(|b| b.chapters.iter().find(|c| c.chapter == loc.chapter))
            .and_then(|c| {
                c.sections
                    .iter()
                    .flat_map(|s| &s.verses)
                    .find(|v| v.verse == loc.verse)
            })
            .map(|v| v.text.as_str())
            .unwrap_or("");
        format!("{}  {}", self.reference(loc), text)
    }

    /// Hand the parsed books to the app directly, so tests never touch disk.
    #[cfg(test)]
    fn with_all_books(mut self, books: Vec<Book>) -> Self {
        self.all_books = Some(books);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Section, Verse};

    fn metadata(number: u32, name_am: &str, name_en: &str, short_en: &str) -> BookMetadata {
        BookMetadata {
            number,
            name_am: name_am.to_string(),
            name_en: name_en.to_string(),
            short_name_en: short_en.to_string(),
            file_path: format!("am/{:02}-{}.json", number, name_en.to_lowercase()),
        }
    }

    /// Three books, none of which exist on disk — enough to drive every code
    /// path that does not itself call `load_book`.
    fn sample_books() -> Vec<BookMetadata> {
        vec![
            metadata(1, "ኦሪት ዘፍጥረት", "Genesis", "Gen"),
            metadata(2, "ኦሪት ዘጸአት", "Exodus", "Exo"),
            metadata(19, "መዝሙረ ዳዊት", "Psalms", "Psa"),
        ]
    }

    fn verse(n: u32, text: &str) -> Verse {
        Verse {
            verse: n,
            verse_end: None,
            text: text.to_string(),
        }
    }

    /// A two-chapter book: chapter 1 has verses 1-3 in one section, chapter 2
    /// has verses 1-2 split across two sections.
    fn sample_book() -> Book {
        Book {
            book_number: 1,
            book_name_am: "ኦሪት ዘፍጥረት".to_string(),
            book_short_name_am: "ዘፍ".to_string(),
            book_name_en: "Genesis".to_string(),
            book_short_name_en: "Gen".to_string(),
            testament: "old".to_string(),
            chapters: vec![
                Chapter {
                    chapter: 1,
                    notes: Vec::new(),
                    sections: vec![Section {
                        title: None,
                        verses: vec![
                            verse(1, "in the beginning"),
                            verse(2, "and the earth"),
                            verse(3, "let there be light"),
                        ],
                    }],
                },
                Chapter {
                    chapter: 2,
                    notes: Vec::new(),
                    sections: vec![
                        Section {
                            title: Some("rest".to_string()),
                            verses: vec![verse(1, "thus the heavens")],
                        },
                        Section {
                            title: None,
                            verses: vec![verse(2, "and he rested")],
                        },
                    ],
                },
            ],
        }
    }

    fn app_with_book() -> App {
        let mut app = App::new(sample_books());
        app.current_book = Some(sample_book());
        app.chapters_state.select(Some(0));
        app.verses_state.select(Some(0));
        app
    }

    #[test]
    fn selects_the_first_book_when_there_is_one() {
        assert_eq!(App::new(sample_books()).books_state.selected(), Some(0));
    }

    #[test]
    fn selects_nothing_when_there_are_no_books() {
        assert_eq!(App::new(Vec::new()).books_state.selected(), None);
    }

    #[test]
    fn book_search_matches_the_amharic_name() {
        let mut app = App::new(sample_books());
        app.search_query = "ዘጸአት".to_string();

        let filtered = app.filtered_books();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].number, 2);
    }

    #[test]
    fn book_search_matches_the_english_name_case_insensitively() {
        let mut app = App::new(sample_books());
        app.search_query = "psal".to_string();

        let filtered = app.filtered_books();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].number, 19);
    }

    #[test]
    fn book_search_matches_the_english_short_name() {
        let mut app = App::new(sample_books());
        app.search_query = "Exo".to_string();

        assert_eq!(app.filtered_books()[0].number, 2);
    }

    #[test]
    fn book_search_matches_the_book_number() {
        let mut app = App::new(sample_books());
        app.search_query = "19".to_string();

        let filtered = app.filtered_books();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].number, 19);
    }

    #[test]
    fn book_search_only_filters_while_the_books_pane_is_active() {
        let mut app = App::new(sample_books());
        app.search_query = "Genesis".to_string();
        app.active_pane = Pane::Chapters;

        assert_eq!(app.filtered_books().len(), 3);
    }

    #[test]
    fn chapters_come_from_the_loaded_book() {
        assert_eq!(app_with_book().filtered_chapters(), vec![1, 2]);
    }

    #[test]
    fn there_are_no_chapters_without_a_book() {
        assert!(App::new(sample_books()).filtered_chapters().is_empty());
    }

    #[test]
    fn chapter_search_matches_on_prefix_not_substring() {
        let mut app = app_with_book();
        app.active_pane = Pane::Chapters;
        app.search_query = "2".to_string();

        assert_eq!(app.filtered_chapters(), vec![2]);
    }

    #[test]
    fn verses_are_flattened_across_sections_in_order() {
        let mut app = app_with_book();
        app.chapters_state.select(Some(1));

        let verses = app.filtered_verses();

        assert_eq!(verses.len(), 2);
        assert_eq!(verses[0], (1, "thus the heavens".to_string()));
        assert_eq!(verses[1], (2, "and he rested".to_string()));
    }

    #[test]
    fn verse_search_matches_text() {
        let mut app = app_with_book();
        app.active_pane = Pane::Verses;
        app.search_query = "light".to_string();

        let verses = app.filtered_verses();

        assert_eq!(verses.len(), 1);
        assert_eq!(verses[0].0, 3);
    }

    #[test]
    fn verse_search_matches_the_verse_number_by_prefix() {
        let mut app = app_with_book();
        app.active_pane = Pane::Verses;
        app.search_query = "2".to_string();

        assert_eq!(app.filtered_verses().len(), 1);
    }

    #[test]
    fn moving_down_past_the_last_verse_wraps_to_the_first() {
        let mut app = app_with_book();
        app.active_pane = Pane::Verses;
        app.verses_state.select(Some(2));

        app.next_item();

        assert_eq!(app.verses_state.selected(), Some(0));
    }

    #[test]
    fn moving_up_from_the_first_verse_wraps_to_the_last() {
        let mut app = app_with_book();
        app.active_pane = Pane::Verses;
        app.verses_state.select(Some(0));

        app.previous_item();

        assert_eq!(app.verses_state.selected(), Some(2));
    }

    #[test]
    fn moving_between_chapters_resets_the_verse_cursor() {
        let mut app = app_with_book();
        app.active_pane = Pane::Chapters;
        app.verses_state.select(Some(2));

        app.next_item();

        assert_eq!(app.chapters_state.selected(), Some(1));
        assert_eq!(app.verses_state.selected(), Some(0));
    }

    #[test]
    fn moving_within_an_empty_pane_is_a_no_op() {
        let mut app = App::new(Vec::new());

        app.next_item();
        app.previous_item();

        assert_eq!(app.books_state.selected(), None);
    }

    #[test]
    fn committing_a_book_search_reselects_the_book_in_the_unfiltered_list() {
        let mut app = App::new(sample_books());
        app.is_searching = true;
        app.search_query = "Psalms".to_string();
        app.books_state.select(Some(0)); // index 0 *of the filtered list*

        app.commit_search();

        assert_eq!(app.books_state.selected(), Some(2));
        assert!(app.search_query.is_empty());
        assert!(!app.is_searching);
    }

    #[test]
    fn committing_a_verse_search_reselects_the_verse_in_the_unfiltered_list() {
        let mut app = app_with_book();
        app.active_pane = Pane::Verses;
        app.is_searching = true;
        app.search_query = "light".to_string();
        app.verses_state.select(Some(0));

        app.commit_search();

        assert_eq!(app.verses_state.selected(), Some(2));
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn committing_an_empty_search_leaves_the_selection_alone() {
        let mut app = App::new(sample_books());
        app.books_state.select(Some(1));

        app.commit_search();

        assert_eq!(app.books_state.selected(), Some(1));
    }

    #[test]
    fn advancing_from_books_needs_a_loaded_book() {
        let mut app = App::new(sample_books());

        app.next_pane();

        assert!(app.active_pane == Pane::Books);
    }

    #[test]
    fn panes_advance_and_retreat_without_running_off_either_end() {
        let mut app = app_with_book();

        app.previous_pane();
        assert!(app.active_pane == Pane::Books);

        app.next_pane();
        assert!(app.active_pane == Pane::Chapters);
        app.next_pane();
        assert!(app.active_pane == Pane::Verses);
        app.next_pane();
        assert!(app.active_pane == Pane::Verses);

        app.previous_pane();
        assert!(app.active_pane == Pane::Chapters);
    }

    #[test]
    fn the_selected_verse_is_formatted_with_its_reference() {
        let mut app = app_with_book();
        app.verses_state.select(Some(2));

        assert_eq!(
            app.get_selected_verse().unwrap(),
            "ኦሪት ዘፍጥረት 1:3 - let there be light"
        );
    }

    #[test]
    fn there_is_no_selected_verse_without_a_book() {
        assert!(App::new(sample_books()).get_selected_verse().is_none());
    }

    #[test]
    fn reselecting_the_same_book_keeps_the_chapter_and_verse_cursors() {
        let books = load_books_metadata_or_skip();
        let Some(books) = books else { return };

        let mut app = App::new(books);
        app.load_current_book_filtered().unwrap();
        app.chapters_state.select(Some(1));
        app.verses_state.select(Some(4));

        app.load_current_book_filtered().unwrap();

        assert_eq!(app.chapters_state.selected(), Some(1));
        assert_eq!(app.verses_state.selected(), Some(4));
    }

    #[test]
    fn selecting_a_different_book_resets_the_chapter_and_verse_cursors() {
        let books = load_books_metadata_or_skip();
        let Some(books) = books else { return };

        let mut app = App::new(books);
        app.load_current_book_filtered().unwrap();
        app.chapters_state.select(Some(1));
        app.verses_state.select(Some(4));

        app.books_state.select(Some(1));
        app.load_current_book_filtered().unwrap();

        assert_eq!(app.chapters_state.selected(), Some(0));
        assert_eq!(app.verses_state.selected(), Some(0));
    }

    fn loc(book: u32, chapter: u32, verse: u32) -> Location {
        Location {
            book,
            chapter,
            verse,
        }
    }

    #[test]
    fn the_current_location_is_the_verse_under_the_cursor() {
        let mut app = app_with_book();
        app.chapters_state.select(Some(1));
        app.verses_state.select(Some(1));

        assert_eq!(app.current_location(), Some(loc(1, 2, 2)));
    }

    #[test]
    fn headings_ranges_and_notes_come_from_the_current_chapter() {
        let mut book = sample_book();
        book.chapters[1].sections[1].verses[0].verse_end = Some(3);
        book.chapters[1].notes = vec!["*1 a note".to_string()];
        let mut app = App::new(sample_books());
        app.current_book = Some(book);
        app.chapters_state.select(Some(1));

        assert_eq!(app.heading_before(1), Some("rest"));
        assert_eq!(app.heading_before(2), None); // second section has no title
        assert_eq!(app.verse_label(1), "1");
        assert_eq!(app.verse_label(2), "2-3");
        assert_eq!(app.chapter_notes(), ["*1 a note".to_string()]);
    }

    #[test]
    fn toggling_a_bookmark_adds_then_removes_it() {
        let mut app = app_with_book();
        app.verses_state.select(Some(2));

        assert!(app.toggle_bookmark());
        assert_eq!(app.bookmarks, vec![loc(1, 1, 3)]);
        assert!(app.is_bookmarked(3));
        assert!(!app.is_bookmarked(2));

        assert!(app.toggle_bookmark());
        assert!(app.bookmarks.is_empty());
    }

    #[test]
    fn whole_bible_search_lists_matches_across_chapters() {
        let mut app = app_with_book().with_all_books(vec![sample_book()]);

        app.search_everywhere("and").unwrap();

        let popup = app.popup.as_ref().unwrap();
        let found: Vec<Location> = popup.hits.iter().map(|h| h.location).collect();
        assert_eq!(found, vec![loc(1, 1, 2), loc(1, 2, 2)]);
        assert!(
            popup.hits[0].label.contains("1:2"),
            "{}",
            popup.hits[0].label
        );
        assert!(popup.hits[0].label.contains("and the earth"));
        assert!(popup.title.contains(": 2 "), "{}", popup.title);
    }

    #[test]
    fn whole_bible_search_with_no_match_opens_an_empty_list() {
        let mut app = app_with_book().with_all_books(vec![sample_book()]);

        app.search_everywhere("zzz").unwrap();

        let popup = app.popup.as_ref().unwrap();
        assert!(popup.hits.is_empty());
        assert_eq!(popup.state.selected(), None);
        app.popup_move(true); // must not panic on an empty list
    }

    #[test]
    fn deleting_from_the_bookmarks_popup_removes_the_bookmark() {
        let mut app = app_with_book().with_all_books(vec![sample_book()]);
        app.bookmarks = vec![loc(1, 1, 1), loc(1, 2, 1)];
        app.open_bookmarks().unwrap();
        app.popup_move(true);

        assert!(app.popup_delete());

        assert_eq!(app.bookmarks, vec![loc(1, 1, 1)]);
        assert_eq!(app.popup.as_ref().unwrap().state.selected(), Some(0));
    }

    #[test]
    fn jumping_moves_every_cursor_and_focuses_the_verses() {
        let Some(books) = load_books_metadata_or_skip() else {
            return;
        };
        let mut app = App::new(books);
        app.search_query = "x".to_string();

        assert!(app.jump_to(loc(2, 3, 5)).unwrap());

        assert!(app.active_pane == Pane::Verses);
        assert!(app.search_query.is_empty());
        assert_eq!(app.current_location(), Some(loc(2, 3, 5)));
    }

    #[test]
    fn jumping_to_a_location_that_does_not_exist_reports_it() {
        let Some(books) = load_books_metadata_or_skip() else {
            return;
        };
        let mut app = App::new(books);

        assert!(!app.jump_to(loc(999, 1, 1)).unwrap());
        assert!(!app.jump_to(loc(1, 9999, 1)).unwrap());
    }

    fn load_books_metadata_or_skip() -> Option<Vec<BookMetadata>> {
        let books = crate::data::load_books_metadata().ok()?;
        (books.len() >= 2).then_some(books)
    }
}
