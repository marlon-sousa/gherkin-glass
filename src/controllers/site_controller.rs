//! Controller: the orchestrator, building a site and serving it by running the services in order.

mod building;
mod watching;

use std::net::SocketAddr;
use std::sync::Arc;

use building::{Built, IntoFolder};
use watching::Watching;

use crate::entities::{
    BuildOptions, BuildReport, Destination, NothingToBuild, ServeEnd, ServeMode, ServeOptions,
    Served, ServedSite, Settings,
};
use crate::errors::{BuildError, RebuildError, ServeError, WatchError};
use crate::services::{
    ConverterApi, GherkinParserApi, InputCollectorApi, ServerApi, SettingsApi, SiteWriterApi,
};

/// What the handlers ask of gherkin-glass.
pub(crate) trait SiteApi {
    fn build(&self, options: &BuildOptions) -> Result<BuildReport, BuildError>;

    /// Serves until ghg serve is asked to stop, or, under `--watch`, until the watching ends,
    /// then returns `Ok` once it no longer serves; or, building into the output folder, returns
    /// that it had nothing to build when its first build read no feature file, serving nothing;
    /// or returns what stops it before it serves, or, under `--watch`, a watch that cannot
    /// start.
    fn serve(
        &self,
        options: &ServeOptions,
        listener: &dyn SiteListener,
    ) -> Result<ServeEnd, ServeError>;
}

/// What happens while ghg serve serves, which the handler words.
pub(crate) trait SiteListener {
    fn listening(&self, address: SocketAddr);
    fn rebuilt(&self);
    fn not_rebuilt(&self, error: &RebuildError);
    fn not_watched(&self, error: &WatchError);
    /// The site served under `--watch` was built from no feature file.
    fn empty(&self, nothing: &NothingToBuild);
}

pub(crate) struct SiteController {
    settings: Arc<dyn SettingsApi>,
    input_collector: Arc<dyn InputCollectorApi>,
    gherkin_parser: Arc<dyn GherkinParserApi>,
    converter: Arc<dyn ConverterApi>,
    site_writer: Arc<dyn SiteWriterApi>,
    server: Arc<dyn ServerApi>,
}

impl SiteController {
    pub(crate) fn new(
        settings: Arc<dyn SettingsApi>,
        input_collector: Arc<dyn InputCollectorApi>,
        gherkin_parser: Arc<dyn GherkinParserApi>,
        converter: Arc<dyn ConverterApi>,
        site_writer: Arc<dyn SiteWriterApi>,
        server: Arc<dyn ServerApi>,
    ) -> Self {
        SiteController {
            settings,
            input_collector,
            gherkin_parser,
            converter,
            site_writer,
            server,
        }
    }

    /// The first build of ghg serve, from nothing with `settings`, or nothing when ghg serve
    /// was asked to stop while it ran.
    fn first_build(&self, settings: Arc<Settings>) -> Result<Option<Built>, ServeError> {
        let first = self
            .build_from_nothing(settings)
            .map_err(|stopped| stopped.error)?;
        Ok(self.unless_asked_to_stop(first))
    }

    /// `first`, the first build of ghg serve, or nothing when ghg serve was asked to stop while
    /// it ran.
    fn unless_asked_to_stop(&self, first: Built) -> Option<Built> {
        (!self.server.asked_to_stop()).then_some(first)
    }

    /// Serves `site` as `options` say, tells `listener` where, and waits until ghg serve is
    /// asked to stop.
    fn serve_until_asked(
        &self,
        site: ServedSite,
        options: &ServeOptions,
        listener: &dyn SiteListener,
    ) -> Result<Served, ServeError> {
        let served = self.server.serve(site, options)?;
        listener.listening(served.listening.address);
        self.server.wait_for_stop();
        Ok(served)
    }

    /// Serves the site `first` built under `--watch`, and rebuilds on each change until ghg
    /// serve is asked to stop or the watching ends.
    fn serve_and_watch(
        &self,
        first: Built,
        options: &ServeOptions,
        listener: &dyn SiteListener,
    ) -> Result<Served, ServeError> {
        let served = self.server.serve(first.site, options)?;
        let watching = Watching {
            controller: self,
            options: &options.build,
            config: self.settings.config_file(&options.build),
            preview: &served.preview,
            listener,
        };
        let changes = watching.start(&first.snapshot)?;
        // Said once watching, so that a change made from then on is seen.
        listener.listening(served.listening.address);
        let snapshot = &first.snapshot;
        if let Some(nothing) = NothingToBuild::found(&snapshot.settings, &snapshot.input_tree) {
            listener.empty(&nothing);
        }
        watching.rebuild_on(changes, first.snapshot);
        Ok(served)
    }
}

impl SiteApi for SiteController {
    fn build(&self, options: &BuildOptions) -> Result<BuildReport, BuildError> {
        let settings = Arc::new(
            self.settings
                .load(options, Destination::Folder)
                .map_err(BuildError::Config)?,
        );
        let built = self
            .build_into_folder(Arc::clone(&settings))
            .map_err(|stopped| stopped.error)?;
        Ok(match built {
            IntoFolder::Built(built) => BuildReport::Written {
                site: built.built,
                output: settings.output.clone(),
                version: settings.version.clone(),
            },
            IntoFolder::NothingToBuild(nothing) => BuildReport::NothingToBuild(nothing),
        })
    }

    fn serve(
        &self,
        options: &ServeOptions,
        listener: &dyn SiteListener,
    ) -> Result<ServeEnd, ServeError> {
        // Before the first build, so that a request to stop during it lets it finish.
        self.server.listen_for_stops()?;
        let settings = Arc::new(
            self.settings
                .load(&options.build, options.mode.destination())
                .map_err(BuildError::Config)?,
        );
        let served = match options.mode {
            ServeMode::Folder => {
                let site = ServedSite::Folder(settings.output.clone());
                Some(self.serve_until_asked(site, options, listener)?)
            }
            ServeMode::Build => {
                let first = match self
                    .build_into_folder(Arc::clone(&settings))
                    .map_err(|stopped| stopped.error)?
                {
                    IntoFolder::Built(first) => first,
                    IntoFolder::NothingToBuild(nothing) => {
                        return Ok(ServeEnd::NothingToBuild(nothing));
                    }
                };
                self.unless_asked_to_stop(first)
                    .map(|first| self.serve_until_asked(first.site, options, listener))
                    .transpose()?
            }
            ServeMode::Watch => self
                .first_build(settings)?
                .map(|first| self.serve_and_watch(first, options, listener))
                .transpose()?,
        };
        if let Some(served) = served {
            self.server.stop(served);
        }
        Ok(ServeEnd::NoLongerServed)
    }
}

#[cfg(test)]
mod tests;
