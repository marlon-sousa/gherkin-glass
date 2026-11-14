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
    /// The heading of the block that holds its steps in a scenario that starts
    /// with them: "From the background" or "From the rule's background".
    pub(crate) heading_in_scenario: String,
    /// For a background whose steps a scenario starts with, whether it starts
    /// collapsed in its own place while the page's checkbox is unchecked; the
    /// checked box starts it collapsed. `None` for any other background, whose
    /// state the checkbox does not change.
    pub(crate) collapsed_when_unchecked: Option<bool>,
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

/// An examples block, an item of its outline's list of examples, which is
/// not a heading and starts collapsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesBlock {
    /// The `id` of its item.
    pub(crate) anchor: String,
    /// The text of its button, worded as a heading, and saying so when it
    /// has no row.
    pub(crate) label: Vec<HeadingPart>,
    /// Whether its table has a row below its header, which its outline runs
    /// once for.
    pub(crate) has_rows: bool,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) table: Option<TableView>,
}

/// `text` as the only part of a heading, gherkin-glass's own or a name not
/// inside its words, not isolated.
pub(crate) fn plain_parts(text: &str) -> Vec<HeadingPart> {
    vec![HeadingPart {
        text: text.to_string(),
        isolated: false,
    }]
}

/// The text of a heading's `parts`, without their markup.
pub(crate) fn text_of(parts: &[HeadingPart]) -> String {
    parts.iter().map(|part| part.text.as_str()).collect()
}
