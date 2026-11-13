//! Policy: ghg's own front page template, used when the settings name none: the site's title as a level-one heading, escaped by pulldown-cmark-to-cmark so that it reads as plain text, then the marker.

use std::path::PathBuf;

use pulldown_cmark::{CowStr, Event, HeadingLevel as MarkdownLevel, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

use super::markdown_template::{MARKER, OPTIONS};
use crate::entities::FrontPageTemplate;

/// ghg's own template: a level-one heading holding `title`, escaped by pulldown-cmark-to-cmark
/// so that it reads as plain text, then the marker.
#[expect(
    clippy::expect_used,
    reason = "a heading of plain text and an HTML block always have a Markdown spelling"
)]
pub(super) fn own_template(title: &str) -> FrontPageTemplate {
    let heading = MarkdownLevel::H1;
    let events = [
        Event::Start(Tag::Heading {
            level: heading,
            id: None,
            classes: Vec::new(),
            attrs: Vec::new(),
        }),
        Event::Text(CowStr::from(title)),
        Event::End(TagEnd::Heading(heading)),
        Event::Start(Tag::HtmlBlock),
        Event::Html(CowStr::from(MARKER)),
        Event::End(TagEnd::HtmlBlock),
    ];
    let mut text = String::new();
    cmark(events.iter(), &mut text, OPTIONS)
        .expect("a heading of plain text and an HTML block always have a Markdown spelling");
    FrontPageTemplate {
        file: PathBuf::new(),
        text,
    }
}

#[cfg(test)]
mod tests;
