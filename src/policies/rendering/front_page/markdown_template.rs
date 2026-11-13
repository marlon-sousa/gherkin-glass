//! Policy: a front page's Markdown template parsed, checked and rendered by pulldown-cmark into the HTML before and after its marker, with the page's title and the level of ghg's own headings.

use pulldown_cmark::html::push_html;
use pulldown_cmark::{Event, HeadingLevel as MarkdownLevel, Options, Parser, Tag, TagEnd};

use crate::entities::{FrontPageTemplate, HeadingLevel};
use crate::errors::{FrontPageError, FrontPageErrorKind};

/// Where ghg puts its own part of the page, on a line of its own.
pub(super) const MARKER: &str = "<!-- ghg:contents -->";

/// A template as the front page shows it.
#[derive(Debug)]
pub(super) struct MarkdownFrontPage {
    /// The text of its level-one heading.
    pub(super) title: String,
    /// The level of ghg's own headings: one below the last heading before the marker.
    pub(super) level: HeadingLevel,
    before: String,
    after: String,
}

/// The parser's options, which tables are read with.
pub(super) const OPTIONS: Options = Options::ENABLE_TABLES;

/// Whether a template's level-one heading must hold text.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Title {
    Needed,
    MayBeBlank,
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
    /// The text of the HTML block being read, with the offset it starts at; `None` outside one.
    open_html_block: Option<(String, usize)>,
}

impl MarkdownFrontPage {
    /// `template` read, unless it breaks a rule of the front page.
    pub(super) fn of(template: &FrontPageTemplate) -> Result<MarkdownFrontPage, FrontPageError> {
        Self::read(template, Title::Needed)
    }

    /// ghg's own `template` read: its heading holds the site's title, which may be only
    /// characters Markdown reads as blank, so it needs no text.
    pub(super) fn of_own(
        template: &FrontPageTemplate,
    ) -> Result<MarkdownFrontPage, FrontPageError> {
        Self::read(template, Title::MayBeBlank)
    }

    fn read(
        template: &FrontPageTemplate,
        title: Title,
    ) -> Result<MarkdownFrontPage, FrontPageError> {
        let text = &template.text;
        let error = |line, kind| FrontPageError {
            file: template.file.clone(),
            line,
            kind,
        };
        let mut reading = Reading::default();
        Parser::new_ext(text, OPTIONS)
            .into_offset_iter()
            .try_for_each(|(event, range)| reading.read(event, range.start))
            .map_err(|(offset, kind)| error(Some(line_of(text, offset)), kind))?;
        reading.finished(title).map_err(|kind| error(None, kind))
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
    /// Takes `event`, which starts at `offset` in the text. An HTML block's lines, and the
    /// text pulldown-cmark puts for their indentation, are gathered until it ends.
    fn read(&mut self, event: Event<'a>, offset: usize) -> Result<(), Refusal> {
        if let Some((block, start)) = self.open_html_block.take() {
            return self.in_html_block(block, start, event);
        }
        match event {
            Event::Start(Tag::HtmlBlock) => {
                self.open_html_block = Some((String::new(), offset));
                Ok(())
            }
            Event::Html(html) | Event::InlineHtml(html) => inline_html(html.trim(), offset),
            Event::Start(tag) => {
                if let Tag::Heading { level, .. } = &tag {
                    self.heading(*level, offset)?;
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

    /// Takes `event` inside the HTML block that starts at `start` and holds `block` so far.
    fn in_html_block(
        &mut self,
        mut block: String,
        start: usize,
        event: Event<'a>,
    ) -> Result<(), Refusal> {
        match event {
            Event::End(TagEnd::HtmlBlock) => return self.html_block(block.trim(), start),
            Event::Html(text) | Event::Text(text) => block.push_str(&text),
            _ => {}
        }
        self.open_html_block = Some((block, start));
        Ok(())
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

    /// The template, once it has a title, with text unless `needs` says it may be blank,
    /// and a marker.
    fn finished(self, needs: Title) -> Result<MarkdownFrontPage, FrontPageErrorKind> {
        let title = self
            .title
            .filter(|title| needs == Title::MayBeBlank || !title.is_empty())
            .ok_or(FrontPageErrorKind::NoTitle)?;
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
    if html.contains(MARKER) {
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
