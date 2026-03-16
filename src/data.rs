use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Verse {
    pub verse: u32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    pub title: Option<String>,
    pub verses: Vec<Verse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub chapter: u32,
    pub sections: Vec<Section>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub book_number: u32,
    pub book_name_am: String,
    pub book_short_name_am: String,
    pub book_name_en: String,
    pub book_short_name_en: String,
    pub testament: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone)]
pub struct BookMetadata {
    pub number: u32,
    pub name_am: String,
    pub file_path: String,
}

pub fn load_books_metadata(data_dir: &str) -> Result<Vec<BookMetadata>> {
    let mut books = Vec::new();
    let paths = fs::read_dir(format!("{}/am", data_dir))?;

    for path in paths {
        let path = path?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let book: Book = serde_json::from_str(&content)?;
            books.push(BookMetadata {
                number: book.book_number,
                name_am: book.book_name_am,
                file_path: path.to_str().unwrap().to_string(),
            });
        }
    }

    books.sort_by_key(|b| b.number);
    Ok(books)
}

pub fn load_book(file_path: &str) -> Result<Book> {
    let content = fs::read_to_string(file_path)?;
    let book: Book = serde_json::from_str(&content)?;
    Ok(book)
}
