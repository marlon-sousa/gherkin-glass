//! Entity: a feature's page, as its template shows it, every text in it as written or worded.

/// One page of the site. Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Page {
    /// Relative to the site's root, with `/` between folders.
    pub(crate) address: String,
    /// The `lang` of the page.
    pub(crate) language: String,
    pub(crate) title: String,
    /// The feature's keyword and name.
    pub(crate) heading: String,
    pub(crate) tags: Vec<String>,
    pub(crate) description: Vec<Paragraph>,
    pub(crate) source: Option<SourceLink>,
    pub(crate) contents: Vec<ContentsEntry>,
    pub(crate) blocks: Vec<Block>,
    pub(crate) words: PageWords,
}

/// gherkin-glass's own words on a page, from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageWords {
    pub(crate) contents: String,
    pub(crate) collapse_all: String,
    pub(crate) expand_all: String,
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

/// A background, rule, scenario or examples block, with its heading and what is under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Block {
    pub(crate) kind: BlockKind,
    /// From 2 to 4.
    pub(crate) level: u8,
    pub(crate) anchor: String,
    pub(crate) heading: String,
    pub(crate) collapsed: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) description: Vec<Paragraph>,
    pub(crate) steps: Vec<StepView>,
    /// An examples block's.
    pub(crate) table: Option<TableView>,
    /// A rule's background and scenarios, or an outline's examples blocks.
    pub(crate) blocks: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockKind {
    Background,
    Rule,
    Scenario,
    Examples,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StepView {
    /// As written, with the space after it when the language has one.
    pub(crate) keyword: String,
    pub(crate) text: String,
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

impl BlockKind {
    /// The class the stylesheet and the script know the block by.
    pub(crate) fn class(self) -> &'static str {
        match self {
            BlockKind::Background => "ghg-background",
            BlockKind::Rule => "ghg-rule",
            BlockKind::Scenario => "ghg-scenario",
            BlockKind::Examples => "ghg-examples",
        }
    }
}
