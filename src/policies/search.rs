//! Policy: search, the documents it reads, and the words of its box.

mod search_documents;
mod search_wording;

pub(crate) use search_documents::search_documents;
pub(crate) use search_wording::search_words;
