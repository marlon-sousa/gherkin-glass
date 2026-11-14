//! Policy: the blocks of the backgrounds a scenario starts with, inside it, each one level below it, and the heading of the scenario's own steps after them.

use super::{BlockKind, BlockView, StepsHeading, block};
use crate::entities::{
    BackgroundBlock, BackgroundsOption, BlockHeading, HeadingLevel, SiteTreeNode, plain_parts,
};

/// The blocks of the backgrounds the scenario `node`, headed by `scenario`,
/// starts with, each one level below it and expanded while the page's
/// checkbox is `checked`; none for any other block.
pub(super) fn backgrounds_in_scenario(
    node: SiteTreeNode<'_>,
    scenario: &BlockView<'_>,
    checked: bool,
) -> Vec<String> {
    node.backgrounds()
        .into_iter()
        .zip(1..)
        .map(|((background, background_node), place)| {
            background_in_scenario(background, background_node, scenario, place, checked)
        })
        .collect()
}

/// The block of `background`, the `place`th the scenario headed by `scenario`
/// starts with, counted from 1, holding the steps below `background_node`;
/// it is expanded while the page's checkbox is `checked`.
fn background_in_scenario(
    background: &BackgroundBlock,
    background_node: SiteTreeNode<'_>,
    scenario: &BlockView<'_>,
    place: usize,
    checked: bool,
) -> String {
    let heading = BlockHeading {
        level: scenario.level.below(),
        anchor: format!("{}_background_{place}", scenario.anchor),
        parts: plain_parts(&background.heading_in_scenario),
        collapsed: !checked,
    };
    let view = BlockView {
        collapsed_when_unchecked: Some(true),
        ..BlockView::of(BlockKind::BackgroundInScenario, &heading)
    };
    block(view, background_node)
}

/// The heading, at `level`, of a scenario's `steps` after its `backgrounds`,
/// in the words of the page's `option`; `None` when the page has no
/// background, or either is empty.
pub(super) fn steps_heading<'a>(
    option: Option<&'a BackgroundsOption>,
    level: HeadingLevel,
    backgrounds: &[String],
    steps: &str,
) -> Option<StepsHeading<'a>> {
    let option = option?;
    (!backgrounds.is_empty() && !steps.is_empty()).then(|| StepsHeading {
        level,
        text: &option.steps_heading,
    })
}
