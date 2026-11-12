use anyhow::{Result, anyhow};
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};

static DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Directory inside `data/` holding the Amharic book JSONs.
const AM_DIR: &str = "am";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Verse {
    pub verse: u32,
    /// Last verse of a range such as 15-16, where the source joins verses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verse_end: Option<u32>,
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
    /// Footnotes printed at the end of the chapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
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

/// The header fields of a book, without its chapters.
///
/// Deserializing into this instead of [`Book`] lets `serde_json` walk past the
/// chapter array without allocating any of it. `serde_json` still has to scan
/// every byte, so this saves about 12% of the time it takes to build the book
/// list — but it keeps 81 fully-materialised books out of memory at startup.
#[derive(Debug, Deserialize)]
struct BookHeader {
    book_number: u32,
    book_name_am: String,
    book_name_en: String,
    book_short_name_en: String,
}

#[derive(Debug, Clone)]
pub struct BookMetadata {
    pub number: u32,
    pub name_am: String,
    pub name_en: String,
    pub short_name_en: String,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct Quote {
    pub book_name_am: String,
    pub book_name_en: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

pub fn load_books_metadata() -> Result<Vec<BookMetadata>> {
    let mut books = Vec::new();

    // Look into the embedded am directory
    let am_dir = DATA_DIR
        .get_dir(AM_DIR)
        .ok_or_else(|| anyhow!("Could not find am directory in embedded data"))?;

    for file in am_dir.files() {
        if file.path().extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let path = file
            .path()
            .to_str()
            .ok_or_else(|| anyhow!("Embedded file path is not valid UTF-8: {:?}", file.path()))?;

        let content = file
            .contents_utf8()
            .ok_or_else(|| anyhow!("Could not read embedded file as UTF-8: {}", path))?;

        let header: BookHeader = serde_json::from_str(content)?;
        books.push(BookMetadata {
            number: header.book_number,
            name_am: header.book_name_am,
            name_en: header.book_name_en,
            short_name_en: header.book_short_name_en,
            file_path: path.to_string(),
        });
    }

    books.sort_by_key(|b| b.number);
    Ok(books)
}

pub fn load_book(file_path: &str) -> Result<Book> {
    let file = DATA_DIR
        .get_file(file_path)
        .ok_or_else(|| anyhow!("Could not find embedded file: {}", file_path))?;

    let content = file
        .contents_utf8()
        .ok_or_else(|| anyhow!("Could not read embedded file as UTF-8: {}", file_path))?;

    let book: Book = serde_json::from_str(content)?;
    Ok(book)
}

/// Parse every book in `metadata`, for searches that span the whole Bible.
pub fn load_all_books(metadata: &[BookMetadata]) -> Result<Vec<Book>> {
    metadata.iter().map(|m| load_book(&m.file_path)).collect()
}

/// Pick a verse at random from the whole Bible.
///
/// Only one book is parsed. The book is drawn with probability proportional to
/// its JSON byte length, which is a close enough stand-in for its verse count to
/// keep the result near-uniform across verses without parsing all 81 files to
/// count them.
pub fn random_quote() -> Result<Quote> {
    let am_dir = DATA_DIR
        .get_dir(AM_DIR)
        .ok_or_else(|| anyhow!("Could not find am directory in embedded data"))?;

    let weighted: Vec<(&str, u64)> = am_dir
        .files()
        .filter(|f| f.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .filter_map(|f| f.path().to_str().map(|p| (p, f.contents().len() as u64)))
        .collect();

    let total: u64 = weighted.iter().map(|(_, len)| len).sum();
    if total == 0 {
        return Err(anyhow!("No book data found"));
    }

    let mut rng = Rng::new();

    let mut target = rng.below(total);
    let file_path = weighted
        .iter()
        .find_map(|(path, len)| {
            if target < *len {
                Some(*path)
            } else {
                target -= len;
                None
            }
        })
        .ok_or_else(|| anyhow!("Failed to select a book"))?;

    let book = load_book(file_path)?;

    let verse_count: usize = book
        .chapters
        .iter()
        .flat_map(|c| c.sections.iter())
        .map(|s| s.verses.len())
        .sum();
    if verse_count == 0 {
        return Err(anyhow!("No verses found in {}", file_path));
    }

    let mut index = rng.below(verse_count as u64) as usize;
    for chapter in &book.chapters {
        for section in &chapter.sections {
            if index < section.verses.len() {
                let verse = &section.verses[index];
                return Ok(Quote {
                    book_name_am: book.book_name_am.clone(),
                    book_name_en: book.book_name_en.clone(),
                    chapter: chapter.chapter,
                    verse: verse.verse,
                    text: verse.text.clone(),
                });
            }
            index -= section.verses.len();
        }
    }

    Err(anyhow!("Failed to select a verse from {}", file_path))
}

/// A SplitMix64 generator, seeded from the clock, the pid and a stack address.
///
/// Not cryptographic, and not meant to be — it exists so `--quote` can draw a
/// couple of independent numbers without pulling in a dependency.
struct Rng(u64);

impl Rng {
    fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let local = 0u8;
        let addr = &local as *const u8 as u64;

        Rng(nanos ^ (u64::from(std::process::id()) << 32) ^ addr.rotate_left(17))
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform value in `0..n`, rejecting the tail of the range that would
    /// otherwise make low values more likely than high ones.
    fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0);
        let limit = (u64::MAX / n) * n;
        loop {
            let x = self.next_u64();
            if x < limit {
                return x % n;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_every_embedded_book_in_number_order() {
        let books = load_books_metadata().unwrap();

        assert!(!books.is_empty());
        assert!(books.windows(2).all(|w| w[0].number <= w[1].number));
        assert!(books.iter().all(|b| !b.name_am.is_empty()));
        assert!(books.iter().all(|b| !b.name_en.is_empty()));
        assert!(books.iter().all(|b| b.file_path.ends_with(".json")));
    }

    #[test]
    fn loads_a_book_by_its_metadata_path() {
        let books = load_books_metadata().unwrap();
        let first = &books[0];

        let book = load_book(&first.file_path).unwrap();

        assert_eq!(book.book_number, first.number);
        assert_eq!(book.book_name_am, first.name_am);
        assert!(!book.chapters.is_empty());
    }

    #[test]
    fn loading_an_unknown_path_is_an_error_not_a_panic() {
        assert!(load_book("am/does-not-exist.json").is_err());
    }

    #[test]
    fn random_quote_returns_a_real_verse() {
        for _ in 0..5 {
            let quote = random_quote().unwrap();

            assert!(!quote.text.is_empty());
            assert!(!quote.book_name_am.is_empty());
            assert!(quote.chapter >= 1);
            assert!(quote.verse >= 1);
        }
    }

    #[test]
    fn rng_below_stays_in_range() {
        let mut rng = Rng::new();
        for n in [1u64, 2, 7, 81, 31_102] {
            for _ in 0..100 {
                assert!(rng.below(n) < n);
            }
        }
    }

    #[test]
    fn rng_covers_the_range_it_is_given() {
        let mut rng = Rng::new();
        let mut seen = [false; 8];
        for _ in 0..500 {
            seen[rng.below(8) as usize] = true;
        }
        assert!(seen.iter().all(|s| *s), "some values were never drawn");
    }
}
