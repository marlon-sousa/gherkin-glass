//! Controller: the orchestrator, building a site and serving it by running the services in order.

mod building;
mod watching;

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::park;

use watching::Watching;

use crate::entities::{
    BuildOptions, BuiltSite, Destination, ServeMode, ServeOptions, Served, ServedSite,
};
use crate::errors::{BuildError, RebuildError, ServeError, WatchError};
use crate::services::{
    ConverterApi, GherkinParserApi, InputCollectorApi, ServerApi, SettingsApi, SiteWriterApi,
};

/// What the handlers ask of gherkin-glass.
pub(crate) trait SiteApi {
    fn build(&self, options: &BuildOptions) -> Result<BuiltSite, BuildError>;

    /// Serves until ghg serve is stopped, so it returns only what stops it
    /// before it serves, or, under `--watch`, a watch that cannot start, or
    /// `Ok` once the watching ends.
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

    /// Serves `site` as `options` say, and tells `listener` where.
    fn serve_site(
        &self,
        site: ServedSite,
        options: &ServeOptions,
        listener: &dyn SiteListener,
    ) -> Result<Served, ServeError> {
        let served = self.server.serve(site, options)?;
        listener.listening(served.address);
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
        let settings = self
            .settings
            .load(&options.build, options.mode.destination())
            .map_err(BuildError::Config)?;
        match options.mode {
            ServeMode::Folder => {
                let site = ServedSite::Folder(settings.output.clone());
                self.serve_site(site, options, listener)?;
            }
            ServeMode::Build => {
                let built = self
                    .build_from_nothing(settings)
                    .map_err(|stopped| stopped.error)?;
                self.serve_site(built.site, options, listener)?;
            }
            ServeMode::Watch => {
                let first = self
                    .build_from_nothing(settings)
                    .map_err(|stopped| stopped.error)?;
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
                listener.listening(served.address);
                watching.rebuild_on(changes, first.snapshot);
                return Ok(());
            }
        }
        loop {
            park();
        }
    }
}

#[cfg(test)]
mod tests;
