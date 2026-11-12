//! Entity: a feature's page, its content nodes and its content as the site stage builds it, with its anchors, the levels of its headings and its highlighted steps.

mod anchored_feature;
mod feature_blocks;
mod heading_level;
mod highlighted_step;
mod page;
mod page_content;

pub(crate) use anchored_feature::{Anchored, AnchoredFeature, AnchoredRule, AnchoredScenario};
pub(crate) use feature_blocks::{
    BackgroundBlock, BlockHeading, ExamplesBlock, HeadingPart, RuleBlock, ScenarioBlock,
};
pub(crate) use heading_level::HeadingLevel;
pub(crate) use highlighted_step::{HighlightedStep, StepPart};
pub(crate) use page::{
    BackgroundsOption, ContentsEntry, DocStringView, OutsideLink, Page, PageWords, Paragraph,
    SourceLink, StepView, TableView, TagView,
};
pub(crate) use page_content::{
    BackgroundContent, ExampleScenario, ExamplesAsScenarios, PageContent, RuleContent,
    ScenarioContent, StepContent,
};
