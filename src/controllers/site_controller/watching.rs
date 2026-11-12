//! Controller: ghg serve's watch, rebuilding from nothing for each change that calls for it, and watching again on the input the settings name.

use std::path::PathBuf;

use super::{SiteController, SiteListener};
use crate::entities::{BuildOptions, BuildSnapshot, Destination, InputChanges, Preview, Settings};
use crate::errors::{BuildError, RebuildError, ServeError, WatchError};
use crate::services::WatchedChanges;

/// What ghg serve watches with: the controller, the options it was started with, the
/// configuration file they name, the site it serves and the listener it tells.
pub(super) struct Watching<'a> {
    pub(super) controller: &'a SiteController,
    pub(super) options: &'a BuildOptions,
    pub(super) config: PathBuf,
    pub(super) preview: &'a Preview,
    pub(super) listener: &'a dyn SiteListener,
}

impl Watching<'_> {
    /// Starts watching the input of `first`, the first build, and the configuration file.
    pub(super) fn start(&self, first: &BuildSnapshot) -> Result<WatchedChanges, ServeError> {
        self.controller
            .server
            .watch(&first.settings.input, &self.config)
            .map_err(ServeError::Watch)
    }

    /// Rebuilds for each of `changes` that calls for it, `last` being the first build, until
    /// the watching ends.
    pub(super) fn rebuild_on(&self, mut changes: WatchedChanges, mut last: BuildSnapshot) {
        while let Some(input_changes) = changes.next() {
            if !self.calls_for_rebuild(&input_changes, &last) {
                continue;
            }
            let Some(mut settings) = self.read_settings() else {
                continue;
            };
            let watches_again =
                input_changes == InputChanges::Everything || settings != last.settings;
            if watches_again {
                match self.watched_again(settings.input) {
                    Ok((watched, read)) => {
                        changes = watched;
                        let Some(read) = read else {
                            continue;
                        };
                        settings = read;
                    }
                    Err(error) => {
                        self.listener.not_watched(&error);
                        changes = self.config_watched_alone(changes);
                        continue;
                    }
                }
            }
            last = self.rebuild_from_nothing(settings).unwrap_or(last);
        }
    }

    /// Whether `input_changes` call for a rebuild, judged against `last`.
    fn calls_for_rebuild(&self, input_changes: &InputChanges, last: &BuildSnapshot) -> bool {
        match input_changes {
            InputChanges::Everything => true,
            InputChanges::Paths(paths) => self
                .controller
                .input_collector
                .calls_for_rebuild(&last.input_tree, paths),
        }
    }

    /// The settings read again, or nothing once the listener is told what stops the rebuild.
    fn read_settings(&self) -> Option<Settings> {
        let settings = self
            .controller
            .settings
            .load(self.options, Destination::Memory)
            .map_err(BuildError::Config);
        self.reported(settings)
    }

    /// A new watch of `input` and the configuration file, with the settings read once it has
    /// started, watching again while they name another input, so that a change to the
    /// configuration file saved before the watch started is never lost; the settings are nothing
    /// once the listener is told what stops the rebuild.
    fn watched_again(
        &self,
        mut input: PathBuf,
    ) -> Result<(WatchedChanges, Option<Settings>), WatchError> {
        loop {
            let watched = self.controller.server.watch(&input, &self.config)?;
            match self.read_settings() {
                Some(settings) if settings.input != input => input = settings.input,
                settings => return Ok((watched, settings)),
            }
        }
    }

    /// The configuration file watched alone, or, when even it cannot be watched, `changes`, the
    /// only ones left to tell of a change.
    fn config_watched_alone(&self, changes: WatchedChanges) -> WatchedChanges {
        match self.controller.server.watch_config(&self.config) {
            Ok(watched) => watched,
            Err(error) => {
                self.listener.not_watched(&error);
                changes
            }
        }
    }

    /// Builds the site from nothing with `settings` and serves it, or says what stopped it;
    /// what the next change is judged against, unless the output check stopped it.
    fn rebuild_from_nothing(&self, settings: Settings) -> Option<BuildSnapshot> {
        let read = self
            .controller
            .read_input(settings)
            .map_err(BuildError::Output);
        let (snapshot, built_versions) = self.reported(read)?;
        let written = self
            .controller
            .convert_and_write(&snapshot, &built_versions);
        if let Some((_, site)) = self.reported(written) {
            self.preview.replace(site);
            self.listener.rebuilt();
        }
        Some(snapshot)
    }

    /// What `result` holds, or nothing once the listener is told what stopped the rebuild.
    fn reported<T>(&self, result: Result<T, BuildError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(build) => {
                self.listener.not_rebuilt(&RebuildError { build });
                None
            }
        }
    }
}
