//! Entity: a feature's page as the site stage builds it, each block holding the blocks and steps inside it, which the site tree's builder places as the nodes below the page.

use crate::entities::{
    BackgroundBlock, BlockHeading, DocStringView, ExamplesBlock, Page, RuleBlock, ScenarioBlock,
    StepView, TableView,
};

/// A feature's page and its blocks, in the order of the file: the feature's
/// background, its scenarios, then its rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageContent {
    pub(crate) page: Page,
    pub(crate) background: Option<BackgroundContent>,
    /// The heading of the block the scenarios are in, in a file with a rule
    /// and a scenario outside every rule; `None` in any other file.
    pub(crate) outside_rules: Option<BlockHeading>,
    pub(crate) scenarios: Vec<ScenarioContent>,
    pub(crate) rules: Vec<RuleContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackgroundContent {
    pub(crate) block: BackgroundBlock,
    pub(crate) steps: Vec<StepContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleContent {
    pub(crate) block: RuleBlock,
    pub(crate) background: Option<BackgroundContent>,
    pub(crate) scenarios: Vec<ScenarioContent>,
}

/// A scenario or an outline, with what only an outline has: its examples
/// blocks, each with its runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScenarioContent {
    pub(crate) block: ScenarioBlock,
    pub(crate) steps: Vec<StepContent>,
    pub(crate) examples: Vec<ExamplesContent>,
}

/// An examples block, with the runs its rows make when it has a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesContent {
    pub(crate) block: ExamplesBlock,
    pub(crate) runs: Option<ExamplesRuns>,
}

/// An examples block's rows, as the scenarios they make: the runs of its outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesRuns {
    pub(crate) heading: BlockHeading,
    pub(crate) scenarios: Vec<ExampleScenario>,
}

/// The scenario one row of an outline's examples makes, its steps filled with the row's values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExampleScenario {
    pub(crate) heading: BlockHeading,
    pub(crate) steps: Vec<StepContent>,
}

/// A step, with its data table or its doc string, if it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StepContent {
    pub(crate) step: StepView,
    pub(crate) table: Option<TableView>,
    pub(crate) doc_string: Option<DocStringView>,
}

impl PageContent {
    /// Whether the feature or any of its blocks has a note of the keyword
    /// its file writes, which the page's dialog explains.
    pub(crate) fn has_keyword_note(&self) -> bool {
        self.page.keyword_note.is_some()
            || self
                .background
                .as_ref()
                .is_some_and(BackgroundContent::has_keyword_note)
            || self.scenarios.iter().any(ScenarioContent::has_keyword_note)
            || self.rules.iter().any(RuleContent::has_keyword_note)
    }
}

impl BackgroundContent {
    fn has_keyword_note(&self) -> bool {
        self.block.heading.keyword_note.is_some()
    }
}

impl RuleContent {
    fn has_keyword_note(&self) -> bool {
        self.block.heading.keyword_note.is_some()
            || self
                .background
                .as_ref()
                .is_some_and(BackgroundContent::has_keyword_note)
            || self.scenarios.iter().any(ScenarioContent::has_keyword_note)
    }
}

impl ScenarioContent {
    fn has_keyword_note(&self) -> bool {
        self.block.heading.keyword_note.is_some()
            || self.examples.iter().any(ExamplesContent::has_keyword_note)
    }
}

impl ExamplesContent {
    fn has_keyword_note(&self) -> bool {
        self.block.keyword_note.is_some()
    }
}
