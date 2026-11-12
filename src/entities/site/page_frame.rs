//! Entity: what every page has around its content, a feature's or a folder's: the skip link, the folder tree's checkbox and the tree's landmark, the Theme control, the search box, the words every breadcrumb has, and the themes' colours.

use crate::entities::{Accents, BreadcrumbWords, SearchWords};

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
    /// The site's accents, among the themes' colours.
    pub(crate) accents: Accents,
    pub(crate) search: SearchWords,
    pub(crate) breadcrumb: BreadcrumbWords,
}
