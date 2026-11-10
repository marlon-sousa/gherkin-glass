//! Entity: the distinct step sentences of the suite, grouped by the kind of step, with every step that uses each.

use std::path::PathBuf;

/// Ordered as the groups are shown: Given, then When, then Then.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StepKind {
    Given,
    When,
    Then,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StepSentences {
    /// Only the kinds that have a sentence, in [`StepKind`]'s order.
    pub groups: Vec<SentenceGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceGroup {
    pub kind: StepKind,
    /// Sorted as docs/BEHAVIOUR.md, rule 138, says.
    pub sentences: Vec<Sentence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// With every slot shown as `"…"`.
    pub text: String,
    /// One for each step that has the sentence, so how often it is used is their number.
    pub uses: Vec<SentenceUse>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceUse {
    /// Of the step's feature, relative to the input folder.
    pub path: PathBuf,
    /// Of the scenario, scenario outline or background the step is in.
    pub anchor: String,
    /// That block's keyword as written, then `: ` and its name, or its keyword alone.
    pub heading: String,
    pub line: usize,
    /// Each slot's text as written, quotes or angle brackets included.
    pub values: Vec<String>,
}
