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
/// blocks and, when one of them has a row, its examples as scenarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScenarioContent {
    pub(crate) block: ScenarioBlock,
    pub(crate) steps: Vec<StepContent>,
    pub(crate) examples: Vec<ExamplesBlock>,
    pub(crate) as_scenarios: Option<ExamplesAsScenarios>,
}

/// An outline's examples, as the scenarios their rows make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesAsScenarios {
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
