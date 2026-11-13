//! Entity: the front page's template, the Markdown text a site's front page is rendered from, with the file it was read from.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrontPageTemplate {
    /// As the settings give it, so that what is wrong with it names it so.
    pub(crate) file: PathBuf,
    pub(crate) text: String,
}
