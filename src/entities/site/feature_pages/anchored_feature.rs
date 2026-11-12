//! Entity: a feature with the anchor of each block of its page and of each row of its outlines' examples.

use gherkin::{Background, Examples, Rule, Scenario};

/// A feature, its blocks in file order, each with its anchor.
#[derive(Debug, Clone)]
pub(crate) struct AnchoredFeature<'a> {
    pub(crate) background: Option<Anchored<'a, Background>>,
    pub(crate) scenarios: Vec<AnchoredScenario<'a>>,
    pub(crate) rules: Vec<AnchoredRule<'a>>,
}

/// A block with no blocks of its own, and its anchor.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Anchored<'a, T> {
    pub(crate) block: &'a T,
    pub(crate) anchor: String,
}

/// A rule, its background and its scenarios, each with its anchor.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnchoredRule<'a> {
    pub(crate) rule: &'a Rule,
    pub(crate) anchor: String,
    pub(crate) background: Option<Anchored<'a, Background>>,
    pub(crate) scenarios: Vec<AnchoredScenario<'a>>,
}

/// A scenario or a scenario outline, its examples blocks, and the anchors of the scenarios
/// an outline produces, one for each row of its examples, in file order; a plain scenario has
/// no rows.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnchoredScenario<'a> {
    pub(crate) scenario: &'a Scenario,
    pub(crate) anchor: String,
    pub(crate) examples: Vec<Anchored<'a, Examples>>,
    pub(crate) rows: Vec<String>,
}
