//! Entity: the kind of a step, as its sentence is grouped.

/// Ordered as the groups are shown: Given, then When, then Then.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum StepKind {
    Given,
    When,
    Then,
}
