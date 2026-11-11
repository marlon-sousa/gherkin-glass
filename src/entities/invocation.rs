//! Entity: what the command line asks for.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Invocation {
    Help,
    Version,
    Build(BuildOptions),
}

/// The options of `ghg build`, each `None` when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BuildOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) input: Option<PathBuf>,
}

/// A mistake on the command line. Each `String` is what was typed, shown
/// as it was typed, an option with its dashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArgumentProblem {
    MissingCommand,
    UnknownCommand {
        name: String,
    },
    /// `command` is `ghg` or `ghg build`.
    UnknownOption {
        command: String,
        option: String,
    },
    MissingValue {
        option: String,
    },
    UnexpectedValue {
        option: String,
    },
    RepeatedOption {
        option: String,
    },
    UnexpectedArgument {
        command: String,
        argument: String,
    },
}
