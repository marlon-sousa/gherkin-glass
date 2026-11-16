//! Entity: search, the documents it reads, the words of its box and the page of its results.

mod search_document;
mod search_page;
mod search_words;

pub(crate) use search_document::{OutlinePlace, SearchDocument, SearchKind};
pub(crate) use search_page::{KindHeadings, SearchPage, SearchPageWords};
pub(crate) use search_words::SearchWords;
