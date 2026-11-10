//! Entity: the trail of links from the home page to a page, through the folders above it.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Breadcrumb {
    /// The landmark's name.
    pub(crate) label: String,
    /// From the home page in; the last is the page itself.
    pub(crate) crumbs: Vec<Crumb>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Crumb {
    /// Relative to the page the breadcrumb is on.
    pub(crate) address: String,
    pub(crate) text: String,
}
