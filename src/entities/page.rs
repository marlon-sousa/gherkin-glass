//! Entity: a feature's page, as its template shows it, every text in it as written or worded.

use crate::entities::{Breadcrumb, HeadingLevel, PageFrame, StepPart};

/// One page of the site. Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Page {
    /// Relative to the site's root, with `/` between folders.
    pub(crate) address: String,
    /// The feature file's, below the input folder, with `/` between folders.
    pub(crate) path: String,
    /// The `lang` of the page.
    pub(crate) language: String,
    pub(crate) title: String,
    /// The feature's keyword and name.
    pub(crate) heading: String,
    pub(crate) frame: PageFrame,
    pub(crate) breadcrumb: Breadcrumb,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) source: Option<SourceLink>,
    pub(crate) contents: Vec<ContentsEntry>,
    pub(crate) blocks: Vec<Block>,
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

/// A background whose steps a scenario starts with: shown inline as a list of
/// them, or linked, as a link to the background.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FromBackground {
    /// The `id` of the introduction, which names the list of steps: the
    /// scenario's anchor, `_background_` and the background's place, from 1.
    pub(crate) id: String,
    /// "From the background:" or "From the rule's background:".
    pub(crate) introduction: String,
    pub(crate) anchor: String,
    pub(crate) heading: String,
    pub(crate) steps: Vec<StepView>,
    /// Whether the page is written showing the steps rather than the link.
    pub(crate) inline: bool,
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
    /// Relative to the page that shows the tag, and percent-encoded.
    pub(crate) address: String,
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

/// A background, rule, scenario or examples block, or an outline's examples
/// as scenarios, with its heading and what is under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Block {
    pub(crate) kind: BlockKind,
    pub(crate) level: HeadingLevel,
    /// The `id` of its heading: a block's anchor, or for the examples as
    /// scenarios an `id` of the page's own, with a `_`.
    pub(crate) anchor: String,
    pub(crate) heading_parts: Vec<HeadingPart>,
    pub(crate) collapsed: bool,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    /// A scenario's: the feature's background, then its rule's.
    pub(crate) backgrounds: Vec<FromBackground>,
    pub(crate) steps: Vec<StepView>,
    /// An examples block's.
    pub(crate) table: Option<TableView>,
    /// A rule's background and scenarios, an outline's examples blocks and
    /// its examples as scenarios, or the scenarios those are.
    pub(crate) blocks: Vec<Block>,
}

/// A part of a heading; feature text inside gherkin-glass's own words is
/// isolated, with `<bdi>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadingPart {
    pub(crate) text: String,
    pub(crate) isolated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockKind {
    Background,
    Rule,
    Scenario,
    Examples,
    ExamplesAsScenarios,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StepView {
    /// As written, with the space after it when the language has one.
    pub(crate) keyword: String,
    /// The text after the keyword, in the parts its highlighting tells apart.
    pub(crate) parts: Vec<StepPart>,
    pub(crate) table: Option<TableView>,
    pub(crate) doc_string: Option<DocStringView>,
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

impl Page {
    pub(crate) fn rule_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| block.kind == BlockKind::Rule)
            .count()
    }

    /// Scenario outlines among them, and those inside rules.
    pub(crate) fn scenario_count(&self) -> usize {
        let scenarios = |blocks: &[Block]| {
            blocks
                .iter()
                .filter(|block| block.kind == BlockKind::Scenario)
                .count()
        };
        scenarios(&self.blocks)
            + self
                .blocks
                .iter()
                .filter(|block| block.kind == BlockKind::Rule)
                .map(|rule| scenarios(&rule.blocks))
                .sum::<usize>()
    }
}

impl Block {
    pub(crate) fn heading(&self) -> String {
        self.heading_parts
            .iter()
            .map(|part| part.text.as_str())
            .collect()
    }
}

impl BlockKind {
    /// The class the stylesheet and the script know the block by.
    pub(crate) fn class(self) -> &'static str {
        match self {
            BlockKind::Background => "ghg-background",
            BlockKind::Rule => "ghg-rule",
            BlockKind::Scenario => "ghg-scenario",
            BlockKind::Examples => "ghg-examples",
            BlockKind::ExamplesAsScenarios => "ghg-examples-as-scenarios",
        }
    }
}
