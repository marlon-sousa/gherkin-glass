//! Entity: what a shell says of itself in its head, and what the script finds in it: its title, its description, its descriptor and where the assets it links are.

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShellPage {
    /// The site's title, and the version's name for a version.
    pub(crate) title: String,
    pub(crate) description: String,
    /// Its descriptor's path below `data/`, such as `versions/v1.0.0.json`.
    pub(crate) descriptor: String,
    /// The folder of the assets it links, relative to the shell, such as `../assets/2.0.0`.
    pub(crate) assets: String,
    /// Whether its `main` holds the front page, as a site's and a preview's do, rather than
    /// starting empty and busy, as a version's does.
    pub(crate) holds_front_page: bool,
    /// Whether it asks search engines not to index it, as a preview does.
    pub(crate) unindexed: bool,
}
