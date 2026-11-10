//! Entity: what every page has around its content, a feature's or a folder's: the skip link, the folder tree's checkbox and the tree's landmark, the Theme control, the search box, and the themes' colours.

use crate::entities::SearchWords;

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageFrame {
    pub(crate) skip_link: String,
    pub(crate) show_tree: String,
    /// The name of the tree's landmark, and of the tree the script builds in it.
    pub(crate) tree: String,
    /// As the configuration says, before the script reads the reader's choice.
    pub(crate) tree_shown: bool,
    pub(crate) theme: String,
    pub(crate) theme_system: String,
    pub(crate) theme_light: String,
    pub(crate) theme_dark: String,
    /// The stylesheet of the themes' colours, the site's accents among them,
    /// written into the page's head as it is.
    pub(crate) theme_style: String,
    pub(crate) search: SearchWords,
}
