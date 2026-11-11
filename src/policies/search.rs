//! Policy: search, the documents it reads, and the words of its box.

mod search_documents;
mod search_wording;

pub(crate) use search_documents::LARGE_SEARCH_DOCUMENTS;
pub(crate) use search_documents::{search_documents, search_json};
pub(crate) use search_wording::search_words;
