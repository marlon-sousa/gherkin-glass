//! Service: previewing a site, built and served, and rebuilt as its files change.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::park;

use crate::entities::{
    BuildFailure, Burst, Changes, ListenFailure, OutputProblem, Preview, Problems, ServeMode,
    ServeOptions, ServedSite, ServerWords, Settings, WatchFailure, Watched,
};

use crate::policies::{Read, calls_for_rebuild, remembered};
use crate::ports::{ChangeWatcher, ConfigReader, PreviewServer, SiteRecorder};
use crate::services::settings::CONFIG_FILE;
use crate::services::{ConfigProblems, InputService, OutputService, SettingsService, SiteService};

/// The address ghg serve listens on unless `--host` names another.
const DEFAULT_HOST: &str = "127.0.0.1";
/// The port ghg serve listens at unless `--port` names another.
const DEFAULT_PORT: u16 = 8000;

pub(crate) struct PreviewService {
    settings: SettingsService,
    on_disk: SiteService,
    output: OutputService,
    in_memory: SiteService,
    input: InputService,
    memory: Arc<dyn SiteRecorder>,
    server: Arc<dyn PreviewServer>,
    watcher: Arc<dyn ChangeWatcher>,
}

/// What ghg serve watches with, and the files the input held at the last burst.
struct Watching {
    changes: Changes,
    config: PathBuf,
    input: PathBuf,
    read: Vec<PathBuf>,
}

/// What stops ghg serve before it serves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServeFailure {
    Config(ConfigProblems),
    Input(Problems),
    Output(OutputProblem),
    Listen {
        host: String,
        port: u16,
        failure: ListenFailure,
    },
    Watch(WatchFailure),
}

/// What stops a rebuild, which leaves the last site that built served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RebuildFailure {
    Config(ConfigProblems),
    Input(Problems),
}

/// What happens while ghg serve serves, for the person at the terminal.
pub(crate) trait PreviewEvents {
    fn listening(&self, address: SocketAddr);
    fn rebuilt(&self);
    fn not_rebuilt(&self, failure: &RebuildFailure);
}

impl PreviewService {
    /// `on_disk` writes the site into its output folder, once `output` has
    /// checked that folder as ghg build does; under `--watch`, `input` is built
    /// into `memory`, which no output folder's check concerns.
    pub(crate) fn new(
        config_reader: Arc<dyn ConfigReader>,
        on_disk: SiteService,
        output: OutputService,
        input: InputService,
        memory: Arc<dyn SiteRecorder>,
        server: Arc<dyn PreviewServer>,
        watcher: Arc<dyn ChangeWatcher>,
    ) -> Self {
        PreviewService {
            settings: SettingsService::new(config_reader),
            on_disk,
            output,
            in_memory: SiteService::new(input.clone(), memory.clone()),
            input,
            memory,
            server,
            watcher,
        }
    }

    /// Serves until ghg serve is stopped, so it returns only what stops it
    /// before it serves, or, under `--watch`, `Ok` once the watching ends.
    pub(crate) fn serve(
        &self,
        options: &ServeOptions,
        words: ServerWords,
        events: &dyn PreviewEvents,
    ) -> Result<(), ServeFailure> {
        let settings = self
            .settings
            .load(&options.build)
            .map_err(ServeFailure::Config)?;
        let config = options
            .build
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE));
        // Watching starts before the first build, so that a change made
        // during it is not missed; what stops the build is said first.
        let changes = options.mode.watches().then(|| {
            let watched = [
                Watched::Folder(settings.input.clone()),
                Watched::File(config.clone()),
            ];
            self.watcher.watch(&watched)
        });
        let read_before = if options.mode.watches() {
            self.input.paths(&settings.input)
        } else {
            Vec::new()
        };
        let site = match options.mode {
            ServeMode::Folder => ServedSite::Folder(settings.output.clone()),
            ServeMode::Watch => self
                .build_in_memory(&settings)
                .map_err(ServeFailure::Input)?,
            ServeMode::Build => {
                self.build_into_output(&settings)
                    .map_err(|failure| match failure {
                        BuildFailure::Problems(problems) => ServeFailure::Input(problems),
                        BuildFailure::Output(problem) => ServeFailure::Output(problem),
                    })?
            }
        };
        let changes = changes.transpose().map_err(ServeFailure::Watch)?;
        let preview = Arc::new(Preview::new(options.mode, site));
        let host = options.host.as_deref().unwrap_or(DEFAULT_HOST);
        let port = options.port.unwrap_or(DEFAULT_PORT);
        let address = self
            .server
            .start(host, port, preview.clone(), words)
            .map_err(|failure| ServeFailure::Listen {
                host: host.to_string(),
                port,
                failure,
            })?;
        events.listening(address);
        let Some(changes) = changes else {
            loop {
                park();
            }
        };
        let watching = Watching {
            changes,
            config,
            input: settings.input,
            read: read_before,
        };
        self.rebuild_on_changes(watching, options, &preview, events);
        Ok(())
    }

    /// Reads each burst of changes on this thread, until the watching ends,
    /// and rebuilds when one may have changed what a build reads.
    fn rebuild_on_changes(
        &self,
        mut watching: Watching,
        options: &ServeOptions,
        preview: &Preview,
        events: &dyn PreviewEvents,
    ) {
        while let Ok(burst) = watching.changes.bursts.recv() {
            // Bursts that came during the last rebuild need only one more.
            let mut bursts = vec![burst];
            bursts.extend(watching.changes.bursts.try_iter());
            if self.calls_for_rebuild(&mut watching, &bursts) {
                self.rebuild_and_serve(options, preview, events);
            }
        }
    }

    fn calls_for_rebuild(&self, watching: &mut Watching, bursts: &[Burst]) -> bool {
        let after = self.input.paths(&watching.input);
        let read = Read {
            config: &watching.config,
            before: &watching.read,
            after: &after,
        };
        let rebuild = bursts.iter().any(|burst| calls_for_rebuild(burst, &read));
        watching.read = remembered(&watching.read, after, rebuild);
        rebuild
    }

    fn rebuild_and_serve(
        &self,
        options: &ServeOptions,
        preview: &Preview,
        events: &dyn PreviewEvents,
    ) {
        match self.rebuild(options) {
            Ok(site) => {
                preview.replace(site);
                events.rebuilt();
            }
            Err(failure) => events.not_rebuilt(&failure),
        }
    }

    fn rebuild(&self, options: &ServeOptions) -> Result<ServedSite, RebuildFailure> {
        let settings = self
            .settings
            .load(&options.build)
            .map_err(RebuildFailure::Config)?;
        self.build_in_memory(&settings)
            .map_err(RebuildFailure::Input)
    }

    fn build_in_memory(&self, settings: &Settings) -> Result<ServedSite, Problems> {
        // A build that stopped may have written some files, which this drops.
        self.memory.take();
        let built = self.in_memory.build(settings, &settings.output);
        let files = self.memory.take();
        built.map(|_| ServedSite::Memory(files))
    }

    /// The build ghg build makes, into the output folder, served from there.
    fn build_into_output(&self, settings: &Settings) -> Result<ServedSite, BuildFailure> {
        self.output.build(&self.on_disk, settings)?;
        Ok(ServedSite::Folder(settings.output.clone()))
    }
}

#[cfg(test)]
mod tests;
