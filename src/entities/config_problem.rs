//! Entity: what is wrong with a configuration file, each problem a thing that stops the run.

use thiserror::Error;

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
    #[error("{key} is not {expected:?}")]
    WrongType { key: String, expected: ValueType },
}

/// The kinds of value a key takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ValueType {
    Text,
}
