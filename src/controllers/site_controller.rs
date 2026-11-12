//! Controller: the orchestrator, building a site and serving it by running the services in order.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::thread::park;

use crate::entities::{
    BuildOptions, BuiltSite, Preview, ServeMode, ServeOptions, ServedSite, Settings,
};
use crate::errors::{BuildError, InputErrors, RebuildError, ServeError};
use crate::ports::{SiteRecorder, SiteWriter};
use crate::services::{InputService, OutputService, ServerService, SettingsService, SiteService};

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
    settings: SettingsService,
    input: InputService,
    output: OutputService,
    on_disk: SiteService,
    in_memory: SiteService,
    memory: Arc<dyn SiteRecorder>,
    server: ServerService,
}

impl SiteController {
    /// `on_disk` writes a site into its output folder, once `output` has
    /// checked that folder; under `--watch`, the site is built into
    /// `memory`, which no output folder's check concerns.
    pub(crate) fn new(
        settings: SettingsService,
        input: InputService,
        output: OutputService,
        on_disk: Arc<dyn SiteWriter>,
        memory: Arc<dyn SiteRecorder>,
        server: ServerService,
    ) -> Self {
        SiteController {
            settings,
            on_disk: SiteService::new(input.clone(), on_disk),
            in_memory: SiteService::new(input.clone(), memory.clone()),
            input,
            output,
            memory,
            server,
        }
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
            Err(error) => listener.not_rebuilt(&error),
        }
    }

    fn rebuild(&self, options: &BuildOptions) -> Result<ServedSite, RebuildError> {
        let settings = self.settings.load(options).map_err(RebuildError::Config)?;
        self.build_in_memory(&settings).map_err(RebuildError::Input)
    }

    fn build_in_memory(&self, settings: &Settings) -> Result<ServedSite, InputErrors> {
        // A build that stopped may have written some files, which this drops.
        self.memory.take();
        let built = self.in_memory.build(settings, &settings.output);
        let files = self.memory.take();
        built.map(|_| ServedSite::Memory(files))
    }

    /// The build ghg build makes, into the output folder, served from there.
    fn build_into_output(&self, settings: &Settings) -> Result<ServedSite, BuildError> {
        self.output.build(&self.on_disk, settings)?;
        Ok(ServedSite::Folder(settings.output.clone()))
    }
}

impl SiteApi for SiteController {
    fn build(&self, options: &BuildOptions) -> Result<BuiltSite, BuildError> {
        let settings = self.settings.load(options).map_err(BuildError::Config)?;
        self.output.build(&self.on_disk, &settings)
    }

    fn serve(&self, options: &ServeOptions, listener: &dyn SiteListener) -> Result<(), ServeError> {
        let settings = self
            .settings
            .load(&options.build)
            .map_err(BuildError::Config)?;
        let config = SettingsService::config_file(&options.build);
        let selected = |input: &Path| self.input.paths(input);
        // Watching starts before the first build, so that a change made
        // during it is not missed; what stops the build is said first.
        let rebuilds = options
            .mode
            .watches()
            .then(|| self.server.watch(&settings.input, &config, &selected));
        let site = match options.mode {
            ServeMode::Folder => ServedSite::Folder(settings.output.clone()),
            ServeMode::Watch => self.build_in_memory(&settings).map_err(BuildError::Input)?,
            ServeMode::Build => self.build_into_output(&settings)?,
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
