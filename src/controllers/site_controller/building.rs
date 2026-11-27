//! Controller: the one build, from nothing or from the last build and the paths that changed since, and what it leaves for the next one.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use super::SiteController;
use crate::entities::{
    BuildOptions, BuildSnapshot, BuildStart, BuiltSite, CollectedChanges, NothingToBuild,
    ServedSite, Settings,
};
use crate::errors::BuildError;
use crate::policies::Change;

/// What a build leaves for the next one, with what it has to say.
#[derive(Debug)]
pub(super) enum BuildOutcome {
    Built(Built),
    Stopped(Stopped),
    /// The paths concern nothing a build reads, so nothing is built or said, and the last build
    /// is left with the new input tree.
    NothingConcerned(BuildSnapshot),
    /// The settings read again differ from the last build's, so the caller builds from nothing
    /// with `settings`, or keeps `left`, the last build with the new input tree, when it cannot.
    SettingsChanged {
        left: BuildSnapshot,
        settings: Settings,
    },
}

/// How a build into the output folder that nothing stopped ended.
#[derive(Debug)]
pub(super) enum IntoFolder {
    Built(Built),
    NothingToBuild(NothingToBuild),
}

/// A build that wrote its site.
#[derive(Debug)]
pub(super) struct Built {
    /// What the next build starts from.
    pub(super) snapshot: BuildSnapshot,
    /// What the build says of the site.
    pub(super) built: BuiltSite,
    /// The site as it is served.
    pub(super) site: ServedSite,
}

/// A build that `error` stopped.
#[derive(Debug)]
pub(super) struct Stopped {
    /// What the next build starts from: the new input tree, or none when the output check
    /// stopped a build from nothing, or the list of `--only` could not be read, which leaves the
    /// last build as it was.
    pub(super) left: Option<BuildSnapshot>,
    pub(super) error: BuildError,
}

impl SiteController {
    /// Builds the site from `start`, reading the settings again from `options` when it starts
    /// from the last build and the paths concern what a build reads.
    pub(super) fn build_from(&self, start: BuildStart<'_>, options: &BuildOptions) -> BuildOutcome {
        match start {
            BuildStart::FromNothing(settings) => match self.build_from_nothing(settings) {
                Ok(built) => BuildOutcome::Built(built),
                Err(stopped) => BuildOutcome::Stopped(*stopped),
            },
            BuildStart::FromLast { last, paths } => self.build_from_last(last, paths, options),
        }
    }

    /// Checks the output folder, then collects the input from nothing, and builds the site, as
    /// `settings` say.
    pub(super) fn build_from_nothing(
        &self,
        settings: Arc<Settings>,
    ) -> Result<Built, Box<Stopped>> {
        let collected = self.check_and_collect(settings)?;
        self.parse_convert_and_write(collected)
    }

    /// Checks the output folder, then collects the input from nothing, and builds the site into
    /// the folder `settings` name, or builds nothing, saying why, when the input holds no error
    /// and no feature file is read.
    pub(super) fn build_into_folder(
        &self,
        settings: Arc<Settings>,
    ) -> Result<IntoFolder, Box<Stopped>> {
        let collected = self.check_and_collect(settings)?;
        if let Some(nothing) = NothingToBuild::found(&collected.settings, &collected.input_tree) {
            return Ok(IntoFolder::NothingToBuild(nothing));
        }
        self.parse_convert_and_write(collected)
            .map(IntoFolder::Built)
    }

    /// Checks the output folder, then collects the input from nothing, as `settings` say, into
    /// the snapshot a build from nothing starts from.
    fn check_and_collect(&self, settings: Arc<Settings>) -> Result<BuildSnapshot, Box<Stopped>> {
        self.site_writer
            .check(&settings, Change::Add)
            .map_err(|error| Stopped {
                left: None,
                error: BuildError::Output(error),
            })?;
        let input_tree = self
            .input_collector
            .collect(&settings.input, settings.only.as_ref())
            .map_err(|errors| Stopped {
                left: None,
                error: BuildError::Config(errors),
            })?;
        Ok(BuildSnapshot {
            settings,
            input_tree,
        })
    }

    /// Makes the new input tree from `last`'s and `paths`, then, when they call for a rebuild,
    /// reads the settings again and builds the site from `last`, unless they differ.
    fn build_from_last(
        &self,
        last: &BuildSnapshot,
        paths: &BTreeSet<PathBuf>,
        options: &BuildOptions,
    ) -> BuildOutcome {
        let CollectedChanges {
            input_tree,
            calls_for_rebuild,
        } = self
            .input_collector
            .collect_changes(&last.input_tree, paths);
        let left = BuildSnapshot {
            settings: Arc::clone(&last.settings),
            input_tree,
        };
        if !calls_for_rebuild {
            return BuildOutcome::NothingConcerned(left);
        }
        let settings = match self.settings.load(options, last.settings.destination) {
            Ok(settings) => settings,
            Err(errors) => {
                return BuildOutcome::Stopped(Stopped {
                    left: Some(left),
                    error: BuildError::Config(errors),
                });
            }
        };
        if settings != *last.settings {
            return BuildOutcome::SettingsChanged { left, settings };
        }
        match self.parse_convert_and_write(left) {
            Ok(built) => BuildOutcome::Built(built),
            Err(stopped) => BuildOutcome::Stopped(*stopped),
        }
    }

    /// Parses the files of `snapshot`'s input tree with nothing parsed, then converts the input
    /// tree into the site's store and writes the site, unless the input holds an error.
    fn parse_convert_and_write(&self, mut snapshot: BuildSnapshot) -> Result<Built, Box<Stopped>> {
        let settings = &snapshot.settings;
        let files = snapshot.input_tree.files_to_parse();
        let parsed = self
            .gherkin_parser
            .parse(&settings.input, files, settings.unnamed);
        snapshot.input_tree.put_parsed(parsed);
        if let Some(errors) = snapshot.input_tree.input_errors() {
            return Err(Box::new(Stopped {
                left: Some(snapshot),
                error: BuildError::Input(errors),
            }));
        }
        let store = self
            .converter
            .convert(&snapshot.input_tree, &snapshot.settings);
        match self.site_writer.write(&snapshot.settings, &store) {
            Ok((site, built)) => Ok(Built {
                built,
                snapshot,
                site,
            }),
            Err(error) => Err(Box::new(Stopped {
                left: Some(snapshot),
                error,
            })),
        }
    }
}
