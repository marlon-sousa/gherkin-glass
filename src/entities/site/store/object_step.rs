//! Entity: a step as an object holds it, with what ghg works out of it: its kind, its sentence, its slots and its highlighted parts.

use crate::entities::{DocString, StepKind, StepPart};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectStep {
    /// Exactly as the language's keywords give it, its space after it included when it has one.
    pub(crate) keyword: String,
    pub(crate) text: String,
    pub(crate) kind: StepKind,
    pub(crate) sentence: String,
    /// The text of each slot, as written.
    pub(crate) slots: Vec<String>,
    pub(crate) parts: Vec<StepPart>,
    /// Its data table's rows; `None` for none.
    pub(crate) table: Option<Vec<Vec<String>>>,
    pub(crate) doc_string: Option<DocString>,
}
