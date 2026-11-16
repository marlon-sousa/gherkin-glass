//! Entity: whether the pages of a site let search engines index them, or ask them not to, as a preview's do.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum SearchIndexing {
    #[default]
    Allowed,
    /// Given `--preview`.
    Refused,
}
