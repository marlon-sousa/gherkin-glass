//! Entity: the distinct step sentences of the suite, grouped by the kind of step, with every step that uses each.

use crate::entities::{FeaturePath, SiteAddress};

/// Ordered as the groups are shown: Given, then When, then Then.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum StepKind {
    Given,
    When,
    Then,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StepSentences {
    /// Only the kinds that have a sentence, in [`StepKind`]'s order.
    pub(crate) groups: Vec<SentenceGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SentenceGroup {
    pub(crate) kind: StepKind,
    /// Sorted by their text lower-cased, then by their character codes.
    pub(crate) sentences: Vec<Sentence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Sentence {
    /// With every slot shown as `"…"`.
    pub(crate) text: String,
    /// One for each step that has the sentence, so how often it is used is their number.
    pub(crate) uses: Vec<SentenceUse>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SentenceUse {
    /// Of the step's feature file.
    pub(crate) path: FeaturePath,
    /// The address of the feature's page.
    pub(crate) page: SiteAddress,
    /// The `lang` of the feature's page.
    pub(crate) language: String,
    /// Of the scenario, scenario outline or background the step is in.
    pub(crate) anchor: String,
    /// That block's keyword as written, then `: ` and its name, or its keyword alone.
    pub(crate) heading: String,
    pub(crate) line: usize,
    /// Each slot's text as written, quotes or angle brackets included.
    pub(crate) values: Vec<String>,
}
