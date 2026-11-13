//! Policy: the front page's main content, the template's heading and text around ghg's own part, rendered from the Markdown template the settings name, or from ghg's own, with the page's title.

mod markdown_template;
mod own_template;

use crate::entities::{FrontPageTemplate, HeadingLevel};
use crate::errors::FrontPageError;
use markdown_template::MarkdownFrontPage;
use own_template::own_template;

/// A page's main content and its title.
pub(super) struct PageMain {
    pub(super) title: String,
    pub(super) main: String,
}

/// The front page rendered from `template`, or, without one, from ghg's own, the heading
/// `title` and the marker; `contents` renders ghg's part, its headings at the level it is given.
pub(super) fn front_page(
    template: Option<&FrontPageTemplate>,
    title: &str,
    contents: impl FnOnce(HeadingLevel) -> String,
) -> Result<PageMain, FrontPageError> {
    let own;
    let template = match template {
        Some(template) => template,
        None => {
            own = own_template(title);
            &own
        }
    };
    let markdown = MarkdownFrontPage::of(template)?;
    let main = markdown.around(&contents(markdown.level));
    Ok(PageMain {
        title: markdown.title,
        main,
    })
}
