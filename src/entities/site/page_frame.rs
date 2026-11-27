//! Entity: what every shell and the versions page have around their content: the dialog opened when a page is opened from disk and what a shell then says in its `main`, the skip link, the folder tree's checkbox and the tree's landmark, the Theme control, the search box, the footer, what a shell says without JavaScript, and the themes' colours.

use crate::entities::{Accents, SearchWords};

/// The words of the dialog a page opens when it is opened from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiskNoticeWords {
    /// The dialog's name.
    pub(crate) title: String,
    pub(crate) text: String,
    /// Its one button, which closes it.
    pub(crate) agree: String,
    /// What a shell's `main` says when it cannot draw the page it is opened at.
    pub(crate) cannot_be_shown: String,
}

/// The footer's words, before and after the link to gherkin-glass's repository, the version of ghg among them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FooterWords {
    pub(crate) before: String,
    pub(crate) after: String,
}

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageFrame {
    pub(crate) opened_from_disk: DiskNoticeWords,
    pub(crate) skip_link: String,
    pub(crate) show_tree: String,
    /// The name of the tree's landmark and of the tree the script builds in it, and the words of the button that shows it on a narrow window.
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
    /// The name of the breadcrumb's landmark, which the script fills.
    pub(crate) breadcrumb: String,
    pub(crate) footer: FooterWords,
    /// What a shell says when JavaScript is turned off.
    pub(crate) needs_script: String,
    /// The note that the site shows only the feature files `--only` lists; `None` without it.
    pub(crate) only_note: Option<String>,
}
