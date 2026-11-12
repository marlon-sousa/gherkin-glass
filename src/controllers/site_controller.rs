//! Controller: the orchestrator, building a site and serving it by running the services in order.

mod watching;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::park;

use watching::Watching;

use crate::entities::{
    BuildOptions, BuildSnapshot, BuiltSite, Destination, ServeMode, ServeOptions, Served,
    ServedSite, Settings, VersionName,
};
use crate::errors::{BuildError, OutputError, RebuildError, ServeError, WatchError};
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

    /// Builds the site from nothing, as `settings` say: what the next build is judged against,
    /// what the build says of the site, and the site as it is served.
    fn build_from_nothing(
        &self,
        settings: Settings,
    ) -> Result<(BuildSnapshot, BuiltSite, ServedSite), BuildError> {
        let (snapshot, built_versions) = self.read_input(settings)?;
        let (built, site) = self.convert_and_write(&snapshot, &built_versions)?;
        Ok((snapshot, built, site))
    }

    /// Checks the output folder, then collects and parses the input, as `settings` say: the
    /// input tree, which keeps every error the input holds, and the versions the output folder
    /// already holds.
    fn read_input(
        &self,
        settings: Settings,
    ) -> Result<(BuildSnapshot, BTreeSet<VersionName>), OutputError> {
        let built_versions = self.site_writer.check(&settings)?;
        let mut input_tree = self.input_collector.collect(&settings.input);
        let files = input_tree.files_to_parse();
        let parsed = self
            .gherkin_parser
            .parse(&settings.input, files, settings.unnamed);
        input_tree.put_parsed(parsed);
        let snapshot = BuildSnapshot {
            settings,
            input_tree,
        };
        Ok((snapshot, built_versions))
    }

    /// Converts the input of `snapshot` and writes its site, beside `built_versions`, unless the
    /// input holds an error.
    fn convert_and_write(
        &self,
        snapshot: &BuildSnapshot,
        built_versions: &BTreeSet<VersionName>,
    ) -> Result<(BuiltSite, ServedSite), BuildError> {
        let BuildSnapshot {
            settings,
            input_tree,
        } = snapshot;
        if let Some(errors) = input_tree.input_errors() {
            return Err(BuildError::Input(errors));
        }
        let site_tree = self.converter.convert(input_tree, settings, built_versions);
        Ok(self.site_writer.write(settings, &site_tree)?)
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
        let (_, built, _) = self.build_from_nothing(settings)?;
        Ok(built)
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
                let (_, _, site) = self.build_from_nothing(settings)?;
                self.serve_site(site, options, listener)?;
            }
            ServeMode::Watch => {
                let (last, _, site) = self.build_from_nothing(settings)?;
                let served = self.server.serve(site, options)?;
                let watching = Watching {
                    controller: self,
                    options: &options.build,
                    config: self.settings.config_file(&options.build),
                    preview: &served.preview,
                    listener,
                };
                let changes = watching.start(&last)?;
                // Said once watching, so that a change made from then on is seen.
                listener.listening(served.address);
                watching.rebuild_on(changes, last);
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
