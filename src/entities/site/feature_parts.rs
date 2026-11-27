//! Entity: what ghg works out of a feature for its object and its entry on the front page: its anchors, its highlighted steps, the words of its blocks' headings and its contents.

mod anchored_feature;
mod contents_entry;
mod heading_part;
mod highlighted_step;

pub(crate) use anchored_feature::{Anchored, AnchoredFeature, AnchoredRule, AnchoredScenario};
pub(crate) use contents_entry::{ContentsEntry, OutsideLink};
pub(crate) use heading_part::{HeadingPart, plain_parts, text_of};
pub(crate) use highlighted_step::{HighlightedStep, StepPart};
