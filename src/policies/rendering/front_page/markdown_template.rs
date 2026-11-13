//! Policy: a front page's Markdown template parsed, checked and rendered by pulldown-cmark into the HTML before and after its marker, with the page's title and the level of ghg's own headings.

use std::ops::Range;

use pulldown_cmark::html::push_html;
use pulldown_cmark::{
    Event, HeadingLevel as MarkdownLevel, OffsetIter, Options, Parser, Tag, TagEnd,
};

use crate::entities::{FrontPageTemplate, HeadingLevel};
use crate::errors::{FrontPageError, FrontPageErrorKind};

/// Where ghg puts its own part of the page, on a line of its own.
const MARKER: &str = "<!-- ghg:contents -->";

/// A template as the front page shows it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct MarkdownFrontPage {
    /// The text of its level-one heading.
    pub(super) title: String,
    /// The level of ghg's own headings: one below the last heading before the marker.
    pub(super) level: HeadingLevel,
    before: String,
    after: String,
}

/// What stops a template, at the offset in its text where it is.
type Refusal = (usize, FrontPageErrorKind);

/// The template's events read so far, and what they say.
#[derive(Default)]
struct Reading<'a> {
    before: Vec<Event<'a>>,
    /// The events after the marker, and the level of ghg's headings; `None` before it.
    after: Option<(Vec<Event<'a>>, HeadingLevel)>,
    last_heading: Option<MarkdownLevel>,
    title: Option<String>,
    in_title: bool,
    /// How many elements are open around the next event.
    depth: usize,
}

impl MarkdownFrontPage {
    /// `template` read, unless it breaks a rule of the front page.
    pub(super) fn of(template: &FrontPageTemplate) -> Result<MarkdownFrontPage, FrontPageError> {
        let text = &template.text;
        let error = |line, kind| FrontPageError {
            file: template.file.clone(),
            line,
            kind,
        };
        let mut events = Parser::new_ext(text, Options::ENABLE_TABLES).into_offset_iter();
        let mut reading = Reading::default();
        while let Some((event, range)) = events.next() {
            reading
                .read(event, range, &mut events)
                .map_err(|(offset, kind)| error(Some(line_of(text, offset)), kind))?;
        }
        reading.finished().map_err(|kind| error(None, kind))
    }

    /// The page's main content, with `contents`, ghg's part already rendered, where the
    /// marker is.
    pub(super) fn around(&self, contents: &str) -> String {
        let after = self.after.trim_end();
        let line_break = if after.is_empty() { "" } else { "\n" };
        let main = format!("{}{contents}{line_break}{after}", self.before.trim_end());
        main.trim_start().to_string()
    }
}

impl<'a> Reading<'a> {
    /// Takes `event`, at `range` of the text, and, for an HTML block, the rest of its events
    /// from `events`.
    fn read(
        &mut self,
        event: Event<'a>,
        range: Range<usize>,
        events: &mut OffsetIter<'a>,
    ) -> Result<(), Refusal> {
        match event {
            Event::Start(Tag::HtmlBlock) => {
                let block: String = events.by_ref().map_while(html_text).collect();
                self.html_block(block.trim(), range.start)
            }
            Event::Html(html) | Event::InlineHtml(html) => inline_html(html.trim(), range.start),
            Event::Start(tag) => {
                if let Tag::Heading { level, .. } = &tag {
                    self.heading(*level, range.start)?;
                }
                self.depth += 1;
                self.keep(Event::Start(tag));
                Ok(())
            }
            Event::End(tag) => {
                self.depth = self.depth.saturating_sub(1);
                if tag == TagEnd::Heading(MarkdownLevel::H1) {
                    self.in_title = false;
                }
                self.keep(Event::End(tag));
                Ok(())
            }
            other => {
                self.title_text(&other);
                self.keep(other);
                Ok(())
            }
        }
    }

    /// An HTML block holding `html`, at `offset`: the marker, a comment left out, or raw HTML.
    fn html_block(&mut self, html: &str, offset: usize) -> Result<(), Refusal> {
        if html != MARKER {
            return inline_html(html, offset);
        }
        if self.depth > 0 {
            return Err((offset, FrontPageErrorKind::MarkerNotAlone));
        }
        if self.after.is_some() {
            return Err((offset, FrontPageErrorKind::SecondMarker));
        }
        let level =
            level_below(self.last_heading).ok_or((offset, FrontPageErrorKind::MarkerTooDeep))?;
        self.after = Some((Vec::new(), level));
        Ok(())
    }

    /// A heading at `level` starts at `offset`; a second level-one heading stops the template.
    fn heading(&mut self, level: MarkdownLevel, offset: usize) -> Result<(), Refusal> {
        if self.after.is_none() {
            self.last_heading = Some(level);
        }
        if level != MarkdownLevel::H1 {
            return Ok(());
        }
        if self.title.is_some() {
            return Err((offset, FrontPageErrorKind::SecondTitle));
        }
        self.title = Some(String::new());
        self.in_title = true;
        Ok(())
    }

    /// Adds the text `event` writes to the title, while in it.
    fn title_text(&mut self, event: &Event<'a>) {
        let (true, Some(title)) = (self.in_title, self.title.as_mut()) else {
            return;
        };
        match event {
            Event::Text(text) | Event::Code(text) => title.push_str(text),
            Event::SoftBreak | Event::HardBreak => title.push(' '),
            _ => {}
        }
    }

    fn keep(&mut self, event: Event<'a>) {
        match &mut self.after {
            Some((after, _)) => after.push(event),
            None => self.before.push(event),
        }
    }

    /// The template, once it has a title and a marker.
    fn finished(self) -> Result<MarkdownFrontPage, FrontPageErrorKind> {
        let title = self.title.ok_or(FrontPageErrorKind::NoTitle)?;
        let (after, level) = self.after.ok_or(FrontPageErrorKind::NoMarker)?;
        Ok(MarkdownFrontPage {
            title,
            level,
            before: html(self.before),
            after: html(after),
        })
    }
}

/// HTML that is not an HTML block of its own, `html` at `offset`: a comment, left out, or a
/// marker that is not alone, or raw HTML, which stop the template.
fn inline_html(html: &str, offset: usize) -> Result<(), Refusal> {
    if html == MARKER {
        return Err((offset, FrontPageErrorKind::MarkerNotAlone));
    }
    if is_comment(html) {
        return Ok(());
    }
    Err((offset, FrontPageErrorKind::RawHtml))
}

/// Whether `html` is one comment and nothing else.
fn is_comment(html: &str) -> bool {
    html.strip_prefix("<!--")
        .and_then(|rest| rest.strip_suffix("-->"))
        .is_some_and(|inside| !inside.contains("-->"))
}

/// The text of a line of an HTML block, until the block ends.
fn html_text<'a>((event, _): (Event<'a>, Range<usize>)) -> Option<String> {
    match event {
        Event::Html(html) => Some(html.into_string()),
        _ => None,
    }
}

/// The level of ghg's headings below one at `level`, `h2` below none; none below `h5` and `h6`.
fn level_below(level: Option<MarkdownLevel>) -> Option<HeadingLevel> {
    match level {
        None | Some(MarkdownLevel::H1) => Some(HeadingLevel::Two),
        Some(MarkdownLevel::H2) => Some(HeadingLevel::Three),
        Some(MarkdownLevel::H3) => Some(HeadingLevel::Four),
        Some(MarkdownLevel::H4) => Some(HeadingLevel::Five),
        Some(MarkdownLevel::H5 | MarkdownLevel::H6) => None,
    }
}

fn html(events: Vec<Event<'_>>) -> String {
    let mut html = String::new();
    push_html(&mut html, events.into_iter());
    html
}

/// The line of `text` at `offset`, counted from 1.
fn line_of(text: &str, offset: usize) -> usize {
    text.get(..offset)
        .map_or(0, |before| before.matches('\n').count())
        + 1
}

#[cfg(test)]
mod tests;
