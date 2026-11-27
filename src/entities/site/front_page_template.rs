//! Entity: the front page's template, the Markdown text a site's front page is rendered from, with the file it was read from, or the output folder that keeps it.

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct FrontPageTemplate {
    /// As the settings give it, so that what is wrong with it names it so; for a template the
    /// store kept from when the versions page was last written, the output folder that keeps it.
    pub(crate) file: PathBuf,
    pub(crate) text: String,
}
