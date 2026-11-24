//! Entity: how `ghg build` says how it ended, in sentences for a person or as JSON for a program as well.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MessageFormat {
    /// Sentences on standard error, and nothing on standard output.
    #[default]
    Human,
    /// The same sentences, and one JSON object on standard output.
    Json,
}
