//! Entity: what the command line asks for.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Invocation {
    Help,
    Version,
    Build(BuildOptions),
    Serve(ServeOptions),
}

/// The options of `ghg build`, each `None` when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BuildOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) input: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

/// The options of `ghg serve`: those of `ghg build`, and how it serves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ServeOptions {
    pub(crate) build: BuildOptions,
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) no_build: bool,
    pub(crate) watch: bool,
}

/// A mistake on the command line. Each `String` is what was typed, shown
/// as it was typed, an option with its dashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArgumentProblem {
    MissingCommand,
    UnknownCommand {
        name: String,
    },
    /// `command` is `ghg`, `ghg build` or `ghg serve`.
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
    NotAPort {
        value: String,
    },
    WatchWithoutBuild,
}
