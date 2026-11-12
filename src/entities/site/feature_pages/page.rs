//! Entity: a feature's page, as its template shows it, every text in it as written or worded.

use crate::entities::{LinkAddress, SiteAddress, StepKind, StepPart, Trail};

/// A feature's page, its blocks the nodes below it in the site tree. Every
/// `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Page {
    pub(crate) address: SiteAddress,
    /// The feature file's, below the input folder, with `/` between folders.
    pub(crate) path: String,
    /// The `lang` of the page.
    pub(crate) language: String,
    pub(crate) title: String,
    /// The feature's keyword and name.
    pub(crate) heading: String,
    /// The feature's name alone, empty when it has none.
    pub(crate) name: String,
    pub(crate) trail: Trail,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) source: Option<SourceLink>,
    pub(crate) words: PageWords,
    /// The checkbox that switches how scenarios show their backgrounds, on a
    /// page with a background.
    pub(crate) backgrounds_option: Option<BackgroundsOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundsOption {
    pub(crate) label: String,
    /// Whether the page is written with each scenario showing its backgrounds' steps.
    pub(crate) inline: bool,
    /// Whether a scenario starts with a background's steps, so that the checkbox can act.
    pub(crate) enabled: bool,
}

/// gherkin-glass's own words on a page, from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageWords {
    pub(crate) contents: String,
    pub(crate) collapse_all: String,
    pub(crate) expand_all: String,
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
    pub(crate) path: String,
    pub(crate) after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContentsEntry {
    pub(crate) anchor: String,
    pub(crate) heading: String,
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
