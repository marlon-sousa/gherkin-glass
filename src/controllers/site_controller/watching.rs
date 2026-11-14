//! Controller: ghg serve's watch, rebuilding from the last build for the paths that changed, and from nothing, watching again on the input the settings name, when everything changed or the settings differ, until ghg serve is asked to stop.

use std::iter::once;
use std::path::PathBuf;

use super::building::{BuildOutcome, Built, Stopped};
use super::{SiteController, SiteListener};
use crate::entities::{
    BuildOptions, BuildSnapshot, BuildStart, Destination, InputChanges, Preview, Settings,
    StopRequests,
};
use crate::errors::{BuildError, RebuildError, ServeError, WatchError};
use crate::services::WatchedChanges;

/// What ghg serve watches with: the controller, the options it was started with, the
/// configuration file they name, the site it serves, the listener it tells and the requests to
/// stop that end the watching.
pub(super) struct Watching<'a> {
    pub(super) controller: &'a SiteController,
    pub(super) options: &'a BuildOptions,
    pub(super) config: PathBuf,
    pub(super) preview: &'a Preview,
    pub(super) listener: &'a dyn SiteListener,
    pub(super) stops: &'a StopRequests,
}

impl<'a> Watching<'a> {
    /// Starts watching the input of `first`, the first build, the configuration file and the
    /// front page's template.
    pub(super) fn start(&self, first: &BuildSnapshot) -> Result<WatchedChanges<'a>, ServeError> {
        self.watch(&first.settings).map_err(ServeError::Watch)
    }

    /// Starts watching the input `settings` name, the configuration file and the front page's
    /// template they name.
    fn watch(&self, settings: &Settings) -> Result<WatchedChanges<'a>, WatchError> {
        let settings_files: Vec<_> = once(self.config.clone())
            .chain(settings.front_page.clone())
            .collect();
        self.controller
            .server
            .watch(&settings.input, &settings_files, self.stops)
    }

    /// Rebuilds for each of `changes` that calls for it, `last` being the first build, until
    /// the watching ends or ghg serve is asked to stop.
    pub(super) fn rebuild_on(&self, mut changes: WatchedChanges<'a>, mut last: BuildSnapshot) {
        while let Some(input_changes) = changes.next() {
            let settings = match input_changes {
                InputChanges::Everything => self.read_settings(),
                InputChanges::Paths(paths) => {
                    let start = BuildStart::FromLast {
                        last: &last,
                        paths: &paths,
                    };
                    match self.controller.build_from(start, self.options) {
                        BuildOutcome::SettingsChanged { left, settings } => {
                            last = left;
                            Some(settings)
                        }
                        outcome => {
                            last = self.served(outcome, last);
                            None
                        }
                    }
                }
            };
            let Some(settings) = settings else {
                continue;
            };
            let settings = match self.watched_again(settings) {
                Ok(Some((watched, read))) => {
                    changes = watched;
                    read
                }
                // The configuration stops the rebuild, so the watching stays as it was.
                Ok(None) => continue,
                Err(error) => {
                    self.listener.not_watched(&error);
                    changes = self.config_watched_alone(changes);
                    continue;
                }
            };
            let outcome = self
                .controller
                .build_from(BuildStart::FromNothing(settings), self.options);
            last = self.served(outcome, last);
        }
    }

    /// Serves the site `outcome` built and tells the listener, or tells it what stopped the
    /// build; what the next change is judged against, `last` when the build left nothing.
    fn served(&self, outcome: BuildOutcome, last: BuildSnapshot) -> BuildSnapshot {
        match outcome {
            BuildOutcome::Built(Built { snapshot, site, .. }) => {
                self.preview.replace(site);
                self.listener.rebuilt();
                snapshot
            }
            BuildOutcome::Stopped(Stopped { left, error }) => {
                self.listener.not_rebuilt(&RebuildError { build: error });
                left.unwrap_or(last)
            }
            BuildOutcome::NothingToBuild(left) | BuildOutcome::SettingsChanged { left, .. } => left,
        }
    }

    /// The settings read again, or nothing once the listener is told what stops the rebuild.
    fn read_settings(&self) -> Option<Settings> {
        self.reported(self.load_settings())
    }

    fn load_settings(&self) -> Result<Settings, BuildError> {
        self.controller
            .settings
            .load(self.options, Destination::Memory)
            .map_err(BuildError::Config)
    }

    /// A new watch of the input and the front page's template `settings` name, and of the
    /// configuration file, with the settings read once it has started, watching again while
    /// they name another input or template, so that a change to the configuration file saved
    /// before the watch started is never lost; or nothing, once the listener is told what
    /// stops the rebuild, the new watch dropped.
    fn watched_again(
        &self,
        mut settings: Settings,
    ) -> Result<Option<(WatchedChanges<'a>, Settings)>, WatchError> {
        loop {
            let watched = self.watch(&settings)?;
            match self.load_settings() {
                Ok(read) if watches_other_files(&read, &settings) => settings = read,
                Ok(read) => return Ok(Some((watched, read))),
                Err(build) => {
                    // Dropped before the listener is told, so that only the last watch is left.
                    drop(watched);
                    self.listener.not_rebuilt(&RebuildError { build });
                    return Ok(None);
                }
            }
        }
    }

    /// The configuration file watched alone, or, when even it cannot be watched, `changes`, the
    /// only ones left to tell of a change.
    fn config_watched_alone(&self, changes: WatchedChanges<'a>) -> WatchedChanges<'a> {
        match self
            .controller
            .server
            .watch_config(&self.config, self.stops)
        {
            Ok(watched) => watched,
            Err(error) => {
                self.listener.not_watched(&error);
                changes
            }
        }
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

/// Whether `read` names another input or front page's template than `watched`.
fn watches_other_files(read: &Settings, watched: &Settings) -> bool {
    read.input != watched.input || read.front_page != watched.front_page
}
