//! Entity: what every page has around its content, a feature's or a folder's: the dialog opened when it is opened from disk, the skip link, the folder tree's checkbox and the tree's landmark, the Theme control, the search box, the words every breadcrumb has, and the themes' colours.

use crate::entities::{Accents, BreadcrumbWords, SearchWords};

/// The words of the dialog a page opens when it is opened from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiskNoticeWords {
    /// The dialog's name.
    pub(crate) title: String,
    /// What it says.
    pub(crate) text: String,
    /// Its one button, which closes it.
    pub(crate) agree: String,
}

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageFrame {
    pub(crate) opened_from_disk: DiskNoticeWords,
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
