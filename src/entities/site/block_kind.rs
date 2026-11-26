//! Entity: the kind of a block of a feature file, whatever keyword the file writes it with.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockKind {
    Feature,
    Background,
    Rule,
    Scenario,
    /// A scenario with examples, whatever its keyword.
    ScenarioOutline,
    Examples,
}

impl BlockKind {
    /// The kind in English, such as `scenario-outline`, as the store names it and the catalogue
    /// keys the kind's words.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            BlockKind::Feature => "feature",
            BlockKind::Background => "background",
            BlockKind::Rule => "rule",
            BlockKind::Scenario => "scenario",
            BlockKind::ScenarioOutline => "scenario-outline",
            BlockKind::Examples => "examples",
        }
    }
}
