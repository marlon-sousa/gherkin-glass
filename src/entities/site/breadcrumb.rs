//! Entity: what a page has of its own in its breadcrumb, the trail of links from the home page to it, after the folders of the input it is in, which its node finds above it, and the words every breadcrumb shares.

/// What a page adds to its breadcrumb after the folders of the input it is in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Trail {
    /// The folders of the site it is in that are not folders of the input,
    /// such as the tag pages' folder, by the names their crumbs show, from the
    /// outermost.
    pub(crate) folders: Vec<String>,
    /// The page's own crumb, or `None` for a folder's own page, whose crumb is
    /// its innermost folder's, or the home page's when it is in none.
    pub(crate) page: Option<OwnCrumb>,
}

/// A page's own crumb: its text, as written or worded, and its file's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnCrumb {
    pub(crate) text: String,
    pub(crate) file: String,
}

/// The words every breadcrumb has, from the interface text catalogue and the settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BreadcrumbWords {
    /// The landmark's name.
    pub(crate) label: String,
    /// The home page's crumb: the site's title.
    pub(crate) home: String,
}
