//! Entity: a doc string as its author wrote it, its media type apart from its content.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocString {
    pub(crate) media_type: Option<String>,
    pub(crate) content: String,
}
