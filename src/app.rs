use crate::data::{Book, BookMetadata, Chapter, load_book};
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
}

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

    fn load_books_metadata_or_skip() -> Option<Vec<BookMetadata>> {
        let books = crate::data::load_books_metadata().ok()?;
        (books.len() >= 2).then_some(books)
    }
}
