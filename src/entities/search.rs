//! Entity: search, the documents it reads and the words of its box.

mod search_document;
mod search_words;

pub(crate) use search_document::{SearchDocument, SearchKind};
pub(crate) use search_words::SearchWords;
