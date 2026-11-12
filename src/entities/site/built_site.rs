//! Entity: what a build that succeeded has to say about the site it wrote.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct BuiltSite {
    /// The size in bytes of the search documents, only when it is above the
    /// size at which search can be slow to start.
    pub(crate) large_search_documents: Option<usize>,
    /// Whether no title was given, so that the default stands.
    pub(crate) untitled: bool,
}
