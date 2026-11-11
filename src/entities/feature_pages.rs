//! Entity: a feature's page, with its anchors, the levels of its headings and its highlighted steps.

mod anchored_feature;
mod heading_level;
mod highlighted_step;
mod page;

pub(crate) use anchored_feature::{Anchored, AnchoredFeature, AnchoredRule, AnchoredScenario};
pub(crate) use heading_level::HeadingLevel;
pub(crate) use highlighted_step::{HighlightedStep, StepPart};
pub(crate) use page::{
    BackgroundsOption, Block, BlockKind, ContentsEntry, DocStringView, FromBackground, HeadingPart,
    OutsideLink, Page, PageWords, Paragraph, SourceLink, StepView, TableView, TagView,
};
