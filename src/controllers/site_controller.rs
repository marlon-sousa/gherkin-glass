//! Controller: the orchestrator, building a site and serving it by running the services in order.

mod building;
mod watching;

use std::net::SocketAddr;
use std::sync::Arc;

use building::Built;
use watching::Watching;

use crate::entities::{
    BuildOptions, BuiltSite, Destination, ServeMode, ServeOptions, Served, ServedSite, Settings,
    StopRequests,
};
use crate::errors::{BuildError, RebuildError, ServeError, WatchError};
use crate::services::{
    ConverterApi, GherkinParserApi, InputCollectorApi, ServerApi, SettingsApi, SiteWriterApi,
};

/// What the handlers ask of gherkin-glass.
pub(crate) trait SiteApi {
    fn build(&self, options: &BuildOptions) -> Result<BuiltSite, BuildError>;

    /// Serves until ghg serve is asked to stop, or, under `--watch`, until the watching ends,
    /// then returns `Ok` once it no longer serves; or returns what stops it before it serves,
    /// or, under `--watch`, a watch that cannot start.
    fn serve(&self, options: &ServeOptions, listener: &dyn SiteListener) -> Result<(), ServeError>;
}

/// What happens while ghg serve serves, which the handler words.
pub(crate) trait SiteListener {
    fn listening(&self, address: SocketAddr);
    fn rebuilt(&self);
    fn not_rebuilt(&self, error: &RebuildError);
    fn not_watched(&self, error: &WatchError);
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

    /// The first build of ghg serve, from nothing with `settings`, or nothing when `stops` were
    /// asked while it ran.
    fn first_build(
        &self,
        settings: Settings,
        stops: &StopRequests,
    ) -> Result<Option<Built>, ServeError> {
        let first = self
            .build_from_nothing(settings)
            .map_err(|stopped| stopped.error)?;
        Ok((!stops.asked()).then_some(first))
    }

    /// Serves `site` as `options` say, tells `listener` where, and waits until `stops` are
    /// asked.
    fn serve_until_asked(
        &self,
        site: ServedSite,
        options: &ServeOptions,
        listener: &dyn SiteListener,
        stops: &StopRequests,
    ) -> Result<Served, ServeError> {
        let served = self.server.serve(site, options)?;
        listener.listening(served.listening.address);
        stops.wait();
        Ok(served)
    }

    /// Serves the site `first` built under `--watch`, and rebuilds on each change until
    /// `stops` are asked or the watching ends.
    fn serve_and_watch(
        &self,
        first: Built,
        options: &ServeOptions,
        listener: &dyn SiteListener,
        stops: &StopRequests,
    ) -> Result<Served, ServeError> {
        let served = self.server.serve(first.site, options)?;
        let watching = Watching {
            controller: self,
            options: &options.build,
            config: self.settings.config_file(&options.build),
            preview: &served.preview,
            listener,
            stops,
        };
        let changes = watching.start(&first.snapshot)?;
        // Said once watching, so that a change made from then on is seen.
        listener.listening(served.listening.address);
        watching.rebuild_on(changes, first.snapshot);
        Ok(served)
    }
}

impl SiteApi for SiteController {
    fn build(&self, options: &BuildOptions) -> Result<BuiltSite, BuildError> {
        let settings = self
            .settings
            .load(options, Destination::Folder)
            .map_err(BuildError::Config)?;
        let built = self
            .build_from_nothing(settings)
            .map_err(|stopped| stopped.error)?;
        Ok(built.built)
    }

    fn serve(&self, options: &ServeOptions, listener: &dyn SiteListener) -> Result<(), ServeError> {
        // Before the first build, so that a request to stop during it lets it finish.
        let stops = self.server.stop_requests()?;
        let settings = self
            .settings
            .load(&options.build, options.mode.destination())
            .map_err(BuildError::Config)?;
        // Each arm drops what it served once it returns, which stops the server.
        match options.mode {
            ServeMode::Folder => {
                let site = ServedSite::Folder(settings.output.clone());
                self.serve_until_asked(site, options, listener, &stops)?;
            }
            ServeMode::Build => {
                if let Some(first) = self.first_build(settings, &stops)? {
                    self.serve_until_asked(first.site, options, listener, &stops)?;
                }
            }
            ServeMode::Watch => {
                if let Some(first) = self.first_build(settings, &stops)? {
                    self.serve_and_watch(first, options, listener, &stops)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
