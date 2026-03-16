use crate::data::{Book, BookMetadata, Chapter, load_book};
use ratatui::widgets::ListState;
use anyhow::Result;

#[derive(PartialEq)]
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
        }
    }

    pub fn filtered_books(&self) -> Vec<&BookMetadata> {
        if self.search_query.is_empty() {
            self.books_metadata.iter().collect()
        } else {
            self.books_metadata.iter()
                .filter(|b| b.name_am.contains(&self.search_query))
                .collect()
        }
    }

    pub fn filtered_chapters(&self) -> Vec<u32> {
        if let Some(book) = &self.current_book {
            let chapters: Vec<u32> = book.chapters.iter().map(|c| c.chapter).collect();
            if self.search_query.is_empty() {
                chapters
            } else {
                chapters.into_iter()
                    .filter(|c| c.to_string().contains(&self.search_query))
                    .collect()
            }
        } else {
            Vec::new()
        }
    }

    pub fn next_item(&mut self) {
        match self.active_pane {
            Pane::Books => {
                let filtered = self.filtered_books();
                if filtered.is_empty() { return; }
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
                if filtered.is_empty() { return; }
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
                let verses = self.get_flattened_verses();
                if verses.is_empty() { return; }
                let i = match self.verses_state.selected() {
                    Some(i) => {
                        if i >= verses.len() - 1 {
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
                if filtered.is_empty() { return; }
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
                if filtered.is_empty() { return; }
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
                let verses = self.get_flattened_verses();
                if verses.is_empty() { return; }
                let i = match self.verses_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            verses.len() - 1
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

    pub fn next_pane(&mut self) {
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
        let filtered = self.filtered_books();
        if let Some(i) = self.books_state.selected() {
            if let Some(metadata) = filtered.get(i) {
                let book = load_book(&metadata.file_path)?;
                self.current_book = Some(book);
                self.chapters_state.select(Some(0));
                self.verses_state.select(Some(0));
            }
        }
        Ok(())
    }

    pub fn get_current_chapter(&self) -> Option<&Chapter> {
        let book = self.current_book.as_ref()?;
        let filtered_chapters = self.filtered_chapters();
        let i = self.chapters_state.selected()?;
        let chapter_num = filtered_chapters.get(i)?;
        book.chapters.iter().find(|c| c.chapter == *chapter_num)
    }

    pub fn get_flattened_verses(&self) -> Vec<(u32, String)> {
        if let Some(chapter) = self.get_current_chapter() {
            chapter.sections.iter()
                .flat_map(|s| s.verses.iter().map(|v| (v.verse, v.text.clone())))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_selected_verse(&self) -> Option<String> {
        let verses = self.get_flattened_verses();
        let i = self.verses_state.selected()?;
        let (n, t) = verses.get(i)?;
        let book_name = self.current_book.as_ref().map(|b| b.book_name_am.clone()).unwrap_or_default();
        let chapter = self.get_current_chapter().map(|c| c.chapter).unwrap_or(0);
        Some(format!("{} {}:{} - {}", book_name, chapter, n, t))
    }
}
