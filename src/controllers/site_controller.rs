//! Controller: the orchestrator, building a site and serving it by running the services in order.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::park;

use crate::entities::{
    BuildOptions, BuiltSite, Destination, Preview, ServeMode, ServeOptions, ServedSite, Settings,
};
use crate::errors::{BuildError, RebuildError, ServeError};
use crate::services::{
    ConverterApi, GherkinParserApi, InputCollectorApi, ServerApi, SettingsApi, SiteWriterApi,
};

/// What the handlers ask of gherkin-glass.
pub(crate) trait SiteApi {
    fn build(&self, options: &BuildOptions) -> Result<BuiltSite, BuildError>;

    /// Serves until ghg serve is stopped, so it returns only what stops it
    /// before it serves, or, under `--watch`, `Ok` once the watching ends.
    fn serve(&self, options: &ServeOptions, listener: &dyn SiteListener) -> Result<(), ServeError>;
}

/// What happens while ghg serve serves, which the handler words.
pub(crate) trait SiteListener {
    fn listening(&self, address: SocketAddr);
    fn rebuilt(&self);
    fn not_rebuilt(&self, error: &RebuildError);
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

    /// Builds the site from nothing, as `settings` say, and returns what the build says of it
    /// and the site as it is served.
    fn build_from_nothing(
        &self,
        settings: &Settings,
    ) -> Result<(BuiltSite, ServedSite), BuildError> {
        let built_versions = self.site_writer.check(settings)?;
        let mut input_tree = self.input_collector.collect(&settings.input);
        let files = input_tree.files_to_parse();
        let parsed = self
            .gherkin_parser
            .parse(&settings.input, files, settings.unnamed);
        input_tree.put_parsed(parsed);
        if let Some(errors) = input_tree.input_errors() {
            return Err(BuildError::Input(errors));
        }
        let site_tree = self
            .converter
            .convert(&input_tree, settings, &built_versions);
        Ok(self.site_writer.write(settings, &site_tree)?)
    }

    /// The paths of the files a build from nothing would read from the folder `input`.
    fn allowed_files(&self, input: &Path) -> Vec<PathBuf> {
        self.input_collector.collect(input).allowed_files()
    }

    fn rebuild_and_serve(
        &self,
        options: &BuildOptions,
        preview: &Preview,
        listener: &dyn SiteListener,
    ) {
        match self.rebuild(options) {
            Ok(site) => {
                preview.replace(site);
                listener.rebuilt();
            }
            Err(build) => listener.not_rebuilt(&RebuildError { build }),
        }
    }

    /// Builds the site again in memory, reading the configuration again.
    fn rebuild(&self, options: &BuildOptions) -> Result<ServedSite, BuildError> {
        let settings = self
            .settings
            .load(options, Destination::Memory)
            .map_err(BuildError::Config)?;
        Ok(self.build_from_nothing(&settings)?.1)
    }
}

impl SiteApi for SiteController {
    fn build(&self, options: &BuildOptions) -> Result<BuiltSite, BuildError> {
        let settings = self
            .settings
            .load(options, Destination::Folder)
            .map_err(BuildError::Config)?;
        Ok(self.build_from_nothing(&settings)?.0)
    }

    fn serve(&self, options: &ServeOptions, listener: &dyn SiteListener) -> Result<(), ServeError> {
        let settings = self
            .settings
            .load(&options.build, options.mode.destination())
            .map_err(BuildError::Config)?;
        let config = self.settings.config_file(&options.build);
        let selected = |input: &Path| self.allowed_files(input);
        // Watching starts before the first build, so that a change made
        // during it is not missed; what stops the build is said first.
        let rebuilds = options
            .mode
            .watches()
            .then(|| self.server.watch(&settings.input, &config, &selected));
        let site = match options.mode {
            ServeMode::Folder => ServedSite::Folder(settings.output.clone()),
            ServeMode::Build | ServeMode::Watch => self.build_from_nothing(&settings)?.1,
        };
        let rebuilds = rebuilds.transpose().map_err(ServeError::Watch)?;
        let served = self.server.serve(site, options)?;
        listener.listening(served.address);
        let Some(rebuilds) = rebuilds else {
            loop {
                park();
            }
        };
        for _bursts in rebuilds {
            self.rebuild_and_serve(&options.build, &served.preview, listener);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
