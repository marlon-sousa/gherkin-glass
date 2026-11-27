//! Entity: what the command line asks for.

use std::path::PathBuf;

use crate::entities::{
    BuildTarget, Colour, Destination, KeptInOutput, MessageFormat, PathList, TagLink,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Invocation {
    Help,
    Version,
    Build {
        options: BuildOptions,
        message_format: MessageFormat,
    },
    Serve(ServeOptions),
    /// `ghg version remove` or `ghg preview remove`, whose target is a version or a preview.
    Remove {
        options: RemoveOptions,
        target: BuildTarget,
        message_format: MessageFormat,
    },
}

/// The options of `ghg build`, `ghg version add` and `ghg preview add`, each `None` when not
/// given, and what the command writes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BuildOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) input: Option<PathBuf>,
    pub(crate) target: BuildTarget,
    pub(crate) output: Option<PathBuf>,
    pub(crate) title: Option<String>,
    pub(crate) front_page: Option<PathBuf>,
    pub(crate) only: Option<PathList>,
    pub(crate) keep_in_output: Option<KeptInOutput>,
    pub(crate) source_link: Option<String>,
    pub(crate) allow_unnamed: Option<bool>,
    pub(crate) collapse_rules: Option<bool>,
    pub(crate) collapse_scenarios: Option<bool>,
    pub(crate) show_tree: Option<bool>,
    pub(crate) inline_backgrounds: Option<bool>,
    pub(crate) replace_existing_version: Option<bool>,
    pub(crate) accent_light: Option<Colour>,
    pub(crate) accent_dark: Option<Colour>,
    pub(crate) tag_links: Option<Vec<TagLink>>,
}

/// The options of `ghg version remove` and `ghg preview remove`, each `None` when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemoveOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) keep_in_output: Option<KeptInOutput>,
}

impl RemoveOptions {
    /// The options of a build with these, which loads the same settings, writing `target`.
    pub(crate) fn as_build(&self, target: &BuildTarget) -> BuildOptions {
        BuildOptions {
            config: self.config.clone(),
            output: self.output.clone(),
            keep_in_output: self.keep_in_output.clone(),
            target: target.clone(),
            ..BuildOptions::default()
        }
    }
}

/// The options of `ghg serve`: those of `ghg build`, and how it serves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ServeOptions {
    pub(crate) build: BuildOptions,
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) mode: ServeMode,
}

/// What `ghg serve` serves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ServeMode {
    /// The output folder as it is, under `--no-build`.
    Folder,
    /// The site built into the output folder first.
    #[default]
    Build,
    /// The site built in memory and built again on every change, under `--watch`.
    Watch,
}

impl ServeMode {
    /// Whether the site is built again on every change.
    pub(crate) fn watches(self) -> bool {
        self == ServeMode::Watch
    }

    pub(crate) fn destination(self) -> Destination {
        match self {
            ServeMode::Watch => Destination::Memory,
            ServeMode::Folder | ServeMode::Build => Destination::Folder,
        }
    }
}
