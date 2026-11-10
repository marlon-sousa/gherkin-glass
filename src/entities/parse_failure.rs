//! Entity: why a feature file could not be parsed, as the feature parser port reports it.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFailure {
    pub line: Option<usize>,
    pub message: String,
}
