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
    /// The note, at the top of its content, of the keyword its file writes
    /// when that is another word than its kind's.
    pub(crate) keyword_note: Option<KeywordNote>,
    pub(crate) collapsed: bool,
}

/// The note at the top of a block whose file writes its kind with another
/// keyword than its kind's word: the words before the button that explains
/// it, the button's words, with the keyword isolated, and the words after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeywordNote {
    pub(crate) before: String,
    pub(crate) opener: Vec<HeadingPart>,
    pub(crate) after: String,
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
    /// An outline's: the `id` of the line before its list of examples, which
    /// says it runs once for each row, when one of them has a row.
    pub(crate) runs_anchor: Option<String>,
}

/// An examples block, an item of its outline's list of examples, which is
/// not a heading and starts collapsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesBlock {
    /// The `id` of its item.
    pub(crate) anchor: String,
    /// The text of its button, worded as a heading, and saying so when it
    /// has no row; the item's only text when it holds nothing else.
    pub(crate) label: Vec<HeadingPart>,
    /// The note of the keyword its file writes, when that is another word
    /// than `Examples`.
    pub(crate) keyword_note: Option<KeywordNote>,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) table: Option<TableView>,
}

impl ExamplesBlock {
    /// Whether it has no note of its keyword, no table, no tag and no
    /// description, so that its item has nothing to show but its label.
    pub(crate) fn holds_nothing(&self) -> bool {
        self.keyword_note.is_none()
            && self.table.is_none()
            && self.tags.is_empty()
            && self.description.is_none()
    }
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
