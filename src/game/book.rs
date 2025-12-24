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
