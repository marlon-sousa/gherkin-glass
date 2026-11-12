//! Entity: the content nodes of a feature's page, one type for each kind of block, each holding only what its kind can have.

use crate::entities::{FromBackground, HeadingLevel, Paragraph, StepView, TableView, TagView};

/// What every block's heading holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockHeading {
    pub(crate) level: HeadingLevel,
    /// The `id` of the heading: the block's anchor, or for the examples as
    /// scenarios an `id` of the page's own, with a `_`.
    pub(crate) anchor: String,
    pub(crate) parts: Vec<HeadingPart>,
    /// Whether the block is written collapsed.
    pub(crate) collapsed: bool,
}

/// A part of a heading; feature text inside gherkin-glass's own words is
/// isolated, with `<bdi>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadingPart {
    pub(crate) text: String,
    pub(crate) isolated: bool,
}

/// A block directly under the feature, in the order of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FeatureBlock {
    Background(BackgroundBlock),
    Rule(RuleBlock),
    Scenario(ScenarioBlock),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundBlock {
    pub(crate) heading: BlockHeading,
    pub(crate) description: Option<Paragraph>,
    pub(crate) steps: Vec<StepView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleBlock {
    pub(crate) heading: BlockHeading,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) background: Option<BackgroundBlock>,
    pub(crate) scenarios: Vec<ScenarioBlock>,
}

/// A scenario, an outline, or one of the scenarios an outline's examples make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScenarioBlock {
    pub(crate) heading: BlockHeading,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    /// The feature's background, then its rule's.
    pub(crate) backgrounds: Vec<FromBackground>,
    pub(crate) steps: Vec<StepView>,
    /// An outline's.
    pub(crate) examples: Vec<ExamplesBlock>,
    /// An outline's, when its examples have a row.
    pub(crate) as_scenarios: Option<ExamplesAsScenarios>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesBlock {
    pub(crate) heading: BlockHeading,
    pub(crate) tags: Vec<TagView>,
    pub(crate) description: Option<Paragraph>,
    pub(crate) table: Option<TableView>,
}

/// An outline's examples, as the scenarios their rows make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesAsScenarios {
    pub(crate) heading: BlockHeading,
    pub(crate) scenarios: Vec<ScenarioBlock>,
}

impl BlockHeading {
    /// The heading's text, without its markup.
    pub(crate) fn text(&self) -> String {
        self.parts.iter().map(|part| part.text.as_str()).collect()
    }
}

impl ScenarioBlock {
    /// A scenario at `heading`, with only its steps.
    pub(crate) fn of_steps(heading: BlockHeading, steps: Vec<StepView>) -> Self {
        ScenarioBlock {
            heading,
            tags: Vec::new(),
            description: None,
            backgrounds: Vec::new(),
            steps,
            examples: Vec::new(),
            as_scenarios: None,
        }
    }
}
