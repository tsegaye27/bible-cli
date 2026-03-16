use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use include_dir::{include_dir, Dir};

static DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

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

pub fn load_books_metadata() -> Result<Vec<BookMetadata>> {
    let mut books = Vec::new();
    
    // Look into the embedded am directory
    let am_dir = DATA_DIR.get_dir("am")
        .ok_or_else(|| anyhow!("Could not find am directory in embedded data"))?;

    for file in am_dir.files() {
        if file.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let content = file.contents_utf8()
                .ok_or_else(|| anyhow!("Could not read embedded file as UTF-8"))?;
            
            let book: Book = serde_json::from_str(content)?;
            books.push(BookMetadata {
                number: book.book_number,
                name_am: book.book_name_am,
                file_path: file.path().to_str().unwrap().to_string(),
            });
        }
    }

    books.sort_by_key(|b| b.number);
    Ok(books)
}

pub fn load_book(file_path: &str) -> Result<Book> {
    let file = DATA_DIR.get_file(file_path)
        .ok_or_else(|| anyhow!("Could not find embedded file: {}", file_path))?;
    
    let content = file.contents_utf8()
        .ok_or_else(|| anyhow!("Could not read embedded file as UTF-8"))?;
    
    let book: Book = serde_json::from_str(content)?;
    Ok(book)
}
