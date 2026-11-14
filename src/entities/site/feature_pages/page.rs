//! Entity: a feature's page, as its template shows it, every text in it as written or worded.

use crate::entities::{
    FeaturePath, HeadingPart, LinkAddress, Revision, SiteAddress, StepKind, StepPart, Trail,
    text_of,
};

/// A feature's page, its blocks the nodes below it in the site tree. Every
/// `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Page {
    pub(crate) address: SiteAddress,
    pub(crate) path: FeaturePath,
    /// The revision of the file's node of the input tree the page was converted from, which a
    /// later build compares with the node's to keep the page; a revision is given to one file
    /// once, so it names the file too.
    pub(crate) revision: Revision,
    /// The `lang` of the page.
    pub(crate) language: String,
    pub(crate) title: String,
    /// The feature's kind, its name and the keyword it is written with, the
    /// page's `h1`.
    pub(crate) heading: Vec<HeadingPart>,
    /// The feature's name alone, empty when it has none.
    pub(crate) name: String,
    pub(crate) trail: Trail,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) source: Option<SourceLink>,
    pub(crate) words: PageWords,
    /// On a page with a background, the checkbox that chooses whether the
    /// backgrounds inside the scenarios or those in their own place are
    /// expanded, and the heading of a scenario's steps after those inside it.
    pub(crate) backgrounds_option: Option<BackgroundsOption>,
}

impl Page {
    /// The feature's heading, without its markup.
    pub(crate) fn heading_text(&self) -> String {
        text_of(&self.heading)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundsOption {
    pub(crate) label: String,
    /// Whether the page is written with the box checked, the backgrounds inside
    /// each scenario expanded.
    pub(crate) inline: bool,
    pub(crate) steps_heading: String,
}

/// gherkin-glass's own words on a page, from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageWords {
    pub(crate) contents: String,
    pub(crate) collapse_all: String,
    pub(crate) expand_all: String,
    /// Before the list of an outline's examples.
    pub(crate) runs_for_each_row: String,
}

/// A tag as written, with its `@`, linked to its tag's page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagView {
    pub(crate) text: String,
    /// Relative to the page that shows the tag.
    pub(crate) address: LinkAddress,
    pub(crate) outside: Option<OutsideLink>,
}

/// A tag's link to another site, from the configuration's `tag_links`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutsideLink {
    pub(crate) address: String,
    pub(crate) text: String,
}

/// The lines of one paragraph of a description.
pub(crate) type Paragraph = Vec<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceLink {
    pub(crate) address: String,
    /// The words before and after the file's path, which is isolated in the markup.
    pub(crate) before: String,
    pub(crate) path: FeaturePath,
    pub(crate) after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContentsEntry {
    pub(crate) anchor: String,
    pub(crate) heading: Vec<HeadingPart>,
    /// A rule's scenarios.
    pub(crate) entries: Vec<ContentsEntry>,
}

/// A step's keyword and text; its data table or its doc string is a node below it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StepView {
    /// As written, with the space after it when the language has one.
    pub(crate) keyword: String,
    /// The text after the keyword, in the parts its highlighting tells apart.
    pub(crate) parts: Vec<StepPart>,
    /// Given, When or Then, an And or a But step taking the kind of the step before it.
    pub(crate) kind: StepKind,
    /// The line of the feature file it is written on.
    pub(crate) line: usize,
}

impl StepView {
    /// The text after the keyword, as written or filled with an example's values.
    pub(crate) fn text(&self) -> String {
        self.parts.iter().map(StepPart::text).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableView {
    pub(crate) header: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocStringView {
    pub(crate) media_type: Option<String>,
    pub(crate) content: String,
}
