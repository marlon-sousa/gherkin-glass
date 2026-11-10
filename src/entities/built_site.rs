//! Entity: what a build that succeeded has to say about the site it wrote.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuiltSite {
    /// The size in bytes of the search documents, only when it is above the
    /// size at which search can be slow to start.
    pub large_search_documents: Option<usize>,
}
