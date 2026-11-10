//! Entity: what is wrong with a configuration file, each problem a thing that stops the run.

use thiserror::Error;

use crate::entities::AccentShortfall;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ConfigProblem {
    pub(crate) line: Option<usize>,
    pub(crate) kind: ConfigProblemKind,
}

/// What is wrong. The words a person reads come from the interface text
/// catalogue; the `#[error]` text is for developers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub(crate) enum ConfigProblemKind {
    /// `reason` is the operating system's.
    #[error("cannot be read: {reason}")]
    CannotBeRead { reason: String },
    /// Not TOML; `message` is the `toml` crate's.
    #[error("{message}")]
    Syntax { message: String },
    #[error("unknown key {key}")]
    UnknownKey { key: String },
    #[error("{key} is empty")]
    Empty { key: String },
    /// `name` is the text `version_name` was given.
    #[error("{name} cannot be a version name")]
    InvalidVersionName { name: String },
    #[error("{key} is not {expected:?}")]
    WrongType { key: String, expected: ValueType },
    #[error("{key} reaches {} hundredths", shortfall.hundredths)]
    TooLittleContrast {
        key: String,
        shortfall: AccentShortfall,
    },
    #[error("tag_links is not a list of tables")]
    NotTableList,
    /// A key a table of `tag_links` needs.
    #[error("{key} is missing")]
    Missing { key: String },
    /// `reason` is the `regex` crate's.
    #[error("{key} is not a regular expression: {reason}")]
    NotRegex { key: String, reason: String },
    #[error("{key} is too large")]
    RegexTooLarge { key: String },
    /// `group` is a number or a name, as the replacement writes it.
    #[error("{key} names no group {group}")]
    UnknownGroup { key: String, group: String },
}

/// The kinds of value a key takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ValueType {
    Text,
    Boolean,
    /// Text naming a colour, such as `"#0b4f9c"`.
    Colour,
}
