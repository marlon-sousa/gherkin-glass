//! Policy: the front page's main content, the template's heading and text around ghg's own part, rendered from the Markdown template the settings name, or from ghg's own, built in, with the page's title.

mod markdown_template;

use askama::Template;

use super::node_rendering::rendered;
use crate::entities::{FrontPageTemplate, HeadingLevel};
use crate::errors::FrontPageError;
use markdown_template::MarkdownFrontPage;

/// A page's main content and its title.
pub(super) struct PageMain {
    pub(super) title: String,
    pub(super) main: String,
}

/// The `h1` of a page, then its content, already rendered, which starts with its own line
/// break when it is not empty.
#[derive(Template)]
#[template(path = "page_heading.html")]
pub(super) struct PageHeading<'a> {
    pub(super) heading: &'a str,
    pub(super) content: String,
}

/// The front page rendered from `template`, or, without one, from ghg's own, whose `h1` and
/// title are `title`; `contents` renders ghg's part, its headings at the level it is given.
pub(super) fn front_page(
    template: Option<&FrontPageTemplate>,
    title: &str,
    contents: impl FnOnce(HeadingLevel) -> String,
) -> Result<PageMain, FrontPageError> {
    let Some(template) = template else {
        return Ok(PageMain {
            title: title.to_string(),
            main: rendered(&PageHeading {
                heading: title,
                content: contents(HeadingLevel::Two),
            }),
        });
    };
    let markdown = MarkdownFrontPage::of(template)?;
    let main = markdown.around(&contents(markdown.level));
    Ok(PageMain {
        title: markdown.title,
        main,
    })
}
