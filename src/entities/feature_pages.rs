//! Entity: a feature's page and its content nodes, with its anchors, the levels of its headings and its highlighted steps.

mod anchored_feature;
mod feature_blocks;
mod heading_level;
mod highlighted_step;
mod page;

pub(crate) use anchored_feature::{Anchored, AnchoredFeature, AnchoredRule, AnchoredScenario};
pub(crate) use feature_blocks::{
    BackgroundBlock, BlockHeading, ExampleScenario, ExamplesAsScenarios, ExamplesBlock,
    FeatureBlock, HeadingPart, OutlineParts, RuleBlock, ScenarioBlock,
};
pub(crate) use heading_level::HeadingLevel;
pub(crate) use highlighted_step::{HighlightedStep, StepPart};
pub(crate) use page::{
    BackgroundsOption, ContentsEntry, DocStringView, FromBackground, OutsideLink, Page, PageWords,
    Paragraph, SourceLink, StepView, TableView, TagView,
};
