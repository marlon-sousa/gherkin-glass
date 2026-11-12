//! Entity: the content nodes of a feature's page, one type for each kind of block, each holding only what its kind can have, the blocks and steps inside it being the nodes below it.

use crate::entities::{HeadingLevel, Paragraph, TableView, TagView};

/// What every block's heading holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockHeading {
    pub(crate) level: HeadingLevel,
    /// The `id` of the heading: the block's anchor, or for the examples as
    /// scenarios an `id` of the page's own, with a `_`.
    pub(crate) anchor: String,
    pub(crate) parts: Vec<HeadingPart>,
    pub(crate) collapsed: bool,
}

/// A part of a heading; feature text inside gherkin-glass's own words is
/// isolated, with `<bdi>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadingPart {
    pub(crate) text: String,
    pub(crate) isolated: bool,
}

/// A background, the feature's or a rule's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundBlock {
    pub(crate) heading: BlockHeading,
    pub(crate) description: Option<Paragraph>,
    /// The words that introduce its steps in a scenario that starts with them:
    /// "From the background:" or "From the rule's background:".
    pub(crate) introduction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleBlock {
    pub(crate) heading: BlockHeading,
    /// The rule's name alone, empty when it has none.
    pub(crate) name: String,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
}

/// A scenario or an outline, which has examples below it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScenarioBlock {
    pub(crate) heading: BlockHeading,
    /// The scenario's name alone, empty when it has none.
    pub(crate) name: String,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesBlock {
    pub(crate) heading: BlockHeading,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) table: Option<TableView>,
}

impl BlockHeading {
    /// The heading's text, without its markup.
    pub(crate) fn text(&self) -> String {
        self.parts.iter().map(|part| part.text.as_str()).collect()
    }
}
