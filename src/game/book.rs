use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct BookPage {
    pub text: String,
    pub illustration: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BookDef {
    pub id: String,
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub pages: Vec<BookPage>,
}

#[derive(Deserialize)]
struct BookData {
    books: Vec<BookDef>,
}

static BOOKS: Lazy<HashMap<String, BookDef>> = Lazy::new(|| {
    let data = fs::read_to_string("data/books.json").expect("Unable to read data/books.json");
    let book_data: BookData = serde_json::from_str(&data).expect("Unable to parse data/books.json");
    book_data
        .books
        .into_iter()
        .map(|b| (b.id.clone(), b))
        .collect()
});

pub fn get_book_def(id: &str) -> Option<BookDef> {
    BOOKS.get(id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_books_have_tags() {
        // Check a few specific books to ensure tags are loaded correctly
        let soup_book = get_book_def("book_soup_twice").expect("book_soup_twice not found");
        assert!(soup_book.tags.contains(&"History".to_string()));
        assert!(soup_book.tags.contains(&"Engineers".to_string()));

        let wedding_book = get_book_def("book_wedding_refractions").expect("book_wedding_refractions not found");
        assert!(wedding_book.tags.contains(&"Culture".to_string()));
        assert!(wedding_book.tags.contains(&"Glassborn".to_string()));
        
        // Check that all books have at least one tag (optional, but good practice if that's the goal)
        // Note: We can't iterate over BOOKS directly as it's private, but we can check known IDs.
    }
}
