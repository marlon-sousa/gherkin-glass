//! Policy: the note of the keyword a block's file writes, rendered from its template, with the button that opens the page's dialog about it.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::KeywordNote;

#[derive(Template)]
#[template(path = "keyword_note.html")]
struct KeywordNoteTemplate<'a> {
    note: &'a KeywordNote,
}

impl Render for KeywordNote {
    fn render(&self) -> String {
        rendered(&KeywordNoteTemplate { note: self })
    }
}
