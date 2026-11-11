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
mod tests {
    use std::io::{Error as IoError, ErrorKind, Result as IoResult};
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc::{Sender, channel};
    use std::sync::{Mutex, PoisonError};
    use std::thread::{scope, sleep};
    use std::time::{Duration, Instant};

    use super::*;
    use crate::entities::{
        Changes, CollectedInput, FolderState, Marker, OutputProblemKind, ParseFailure, Problem,
        ProblemKind, RawFeature, VersionName,
    };
    use crate::ports::{FeatureParser, InputCollector, OutputFolders, SiteWriter};
    use std::collections::BTreeMap;

    /// No configuration file.
    struct NoFile;

    impl ConfigReader for NoFile {
        fn read(&self, _: &Path) -> IoResult<String> {
            Err(IoError::new(ErrorKind::NotFound, "not found"))
        }
    }

    /// An input with no features, or, while `failing`, one problem;
    /// `collected` counts how often it was walked.
    #[derive(Default)]
    struct Input {
        failing: AtomicBool,
        collected: AtomicUsize,
    }

    impl InputCollector for Input {
        fn collect(&self, input: &Path) -> CollectedInput {
            self.collected.fetch_add(1, Ordering::SeqCst);
            let problems = if self.failing.load(Ordering::SeqCst) {
                let kind = ProblemKind::Changed;
                vec![Problem::new(input, &input.join("a.feature"), None, kind)]
            } else {
                Vec::new()
            };
            CollectedInput {
                paths: Vec::new(),
                problems,
            }
        }
    }

    impl FeatureParser for Input {
        fn parse(&self, _: &Path) -> Result<RawFeature, ParseFailure> {
            unreachable!("no feature is collected")
        }
    }

    /// Records the preview it was given, and listens at port 1 for port 0.
    #[derive(Default)]
    struct Server {
        started: Mutex<Option<(String, u16, Arc<Preview>)>>,
    }

    impl PreviewServer for Server {
        fn start(
            &self,
            host: &str,
            port: u16,
            preview: Arc<Preview>,
            _: ServerWords,
        ) -> Result<SocketAddr, ListenFailure> {
            *self.started.lock().unwrap_or_else(PoisonError::into_inner) =
                Some((host.to_string(), port, preview));
            Ok(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::LOCALHOST,
                port.max(1),
            )))
        }
    }

    /// Hands the test the sender of its changes, and records what it
    /// watches; `waiting` bursts are already sent when watching starts.
    #[derive(Default)]
    struct Watcher {
        sender: Mutex<Option<Sender<Burst>>>,
        watched: Mutex<Vec<Watched>>,
        waiting: usize,
    }

    impl ChangeWatcher for Watcher {
        fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchFailure> {
            let (sender, bursts) = channel();
            for _ in 0..self.waiting {
                sender.send(Burst::Missed).expect("sent");
            }
            *self.sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(sender);
            *self.watched.lock().unwrap_or_else(PoisonError::into_inner) = watched.to_vec();
            Ok(Changes {
                bursts,
                watching: Box::new(()),
            })
        }
    }

    #[derive(Default)]
    struct Recorded(Mutex<Vec<String>>);

    impl PreviewEvents for Recorded {
        fn listening(&self, address: SocketAddr) {
            self.push(format!("listening at {address}"));
        }
        fn rebuilt(&self) {
            self.push("rebuilt".into());
        }
        fn not_rebuilt(&self, failure: &RebuildFailure) {
            self.push(format!("not rebuilt: {failure:?}"));
        }
    }

    impl Recorded {
        fn push(&self, event: String) {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
        }

        /// The events so far, once there are `count` of them.
        fn wait_for(&self, count: usize) -> Vec<String> {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let events = self
                    .0
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone();
                if events.len() >= count || Instant::now() > deadline {
                    return events;
                }
                sleep(Duration::from_millis(5));
            }
        }
    }

    /// Keeps what is written until it is taken.
    #[derive(Default)]
    struct Recorder {
        files: Mutex<BTreeMap<String, Vec<u8>>>,
    }

    impl SiteWriter for Recorder {
        fn write(&self, _: &Path, file: &str, contents: &[u8]) -> IoResult<()> {
            self.files
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(file.to_string(), contents.to_vec());
            Ok(())
        }
    }

    impl SiteRecorder for Recorder {
        fn take(&self) -> BTreeMap<String, Vec<u8>> {
            std::mem::take(&mut *self.files.lock().unwrap_or_else(PoisonError::into_inner))
        }
    }

    /// Builds of `input`, into a recorder of their own.
    fn site(input: &Arc<Input>) -> SiteService {
        SiteService::new(reading(input), Arc::new(Recorder::default()))
    }

    /// Output folders that are all in one state, and hold no version.
    struct Folders(FolderState);

    impl OutputFolders for Folders {
        fn state(&self, _: &Path, _: &Marker) -> IoResult<FolderState> {
            Ok(self.0)
        }

        fn clear(&self, _: &Path) -> IoResult<()> {
            Ok(())
        }

        fn remove(&self, _: &Path) -> IoResult<()> {
            Ok(())
        }

        fn mark(&self, _: &Path, _: &Marker) -> IoResult<()> {
            Ok(())
        }

        fn marked(&self, _: &Path) -> IoResult<Vec<VersionName>> {
            Ok(Vec::new())
        }
    }

    fn output(state: FolderState) -> OutputService {
        OutputService::new(Arc::new(Folders(state)))
    }

    fn reading(input: &Arc<Input>) -> InputService {
        InputService::new(input.clone(), input.clone())
    }

    fn words() -> ServerWords {
        ServerWords {
            not_found: "not found".into(),
            wrong_method: "wrong method".into(),
        }
    }

    fn changed(path: &str) -> Burst {
        Burst::Changed(vec![PathBuf::from(path)])
    }

    fn wait_until(done: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !done() && Instant::now() < deadline {
            sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn watching_rebuilds_on_each_burst_and_keeps_the_last_good_site() {
        let input = Arc::new(Input::default());
        let server = Arc::new(Server::default());
        let watcher = Arc::new(Watcher::default());
        // A file left from an earlier build, which no version may serve.
        let recorder = Arc::new(Recorder::default());
        recorder
            .write(Path::new("site"), "stale.html", b"")
            .expect("written");
        let service = PreviewService::new(
            Arc::new(NoFile),
            site(&input),
            output(FolderState::Missing),
            reading(&input),
            recorder,
            server.clone(),
            watcher.clone(),
        );
        let options = ServeOptions {
            mode: ServeMode::Watch,
            port: Some(0),
            ..ServeOptions::default()
        };
        let events = Recorded::default();
        scope(|scope| {
            let serving = scope.spawn(|| service.serve(&options, words(), &events));
            assert_eq!(events.wait_for(1), ["listening at 127.0.0.1:1"]);
            let sender = watcher
                .sender
                .lock()
                .expect("lock")
                .take()
                .expect("watching");
            let (host, port, preview) = server
                .started
                .lock()
                .expect("lock")
                .clone()
                .expect("started");
            assert_eq!((host.as_str(), port), ("127.0.0.1", 0));
            assert!(preview.mode.watches());
            assert_eq!(preview.current().number, 1);
            let ServedSite::Memory(files) = &preview.current().site else {
                panic!("a site in memory");
            };
            assert!(
                files.contains_key("assets/gherkin-glass.js"),
                "{:?}",
                files.keys()
            );
            assert!(!files.contains_key("stale.html"));

            // A change no build reads is looked at, and rebuilds nothing.
            let walked = input.collected.load(Ordering::SeqCst);
            sender.send(changed("features/notes.txt")).expect("sent");
            wait_until(|| input.collected.load(Ordering::SeqCst) > walked);
            assert_eq!(preview.current().number, 1);

            sender.send(changed("ghg.toml")).expect("sent");
            assert_eq!(events.wait_for(2)[1], "rebuilt");
            assert_eq!(preview.current().number, 2);

            input.failing.store(true, Ordering::SeqCst);
            sender.send(changed("ghg.toml")).expect("sent");
            assert!(events.wait_for(3)[2].starts_with("not rebuilt: Input("));
            assert_eq!(preview.current().number, 2);

            drop(sender);
            assert_eq!(serving.join().expect("no panic"), Ok(()));
        });
        assert_eq!(
            *watcher.watched.lock().expect("lock"),
            [
                Watched::Folder(PathBuf::from("features")),
                Watched::File(PathBuf::from("ghg.toml"))
            ]
        );
    }

    #[test]
    fn a_first_build_that_stops_serves_nothing() {
        let input = Arc::new(Input::default());
        input.failing.store(true, Ordering::SeqCst);
        let server = Arc::new(Server::default());
        let service = PreviewService::new(
            Arc::new(NoFile),
            site(&input),
            output(FolderState::Missing),
            reading(&input),
            Arc::new(Recorder::default()),
            server.clone(),
            Arc::new(Watcher::default()),
        );
        for mode in [ServeMode::Build, ServeMode::Watch] {
            let options = ServeOptions {
                mode,
                ..ServeOptions::default()
            };
            let failure = service.serve(&options, words(), &Recorded::default());
            assert!(
                matches!(failure, Err(ServeFailure::Input(_))),
                "{failure:?}"
            );
        }
        assert!(server.started.lock().expect("lock").is_none());
    }

    #[test]
    fn bursts_that_wait_need_one_rebuild_and_the_defaults_are_127_0_0_1_at_8000() {
        let input = Arc::new(Input::default());
        let server = Arc::new(Server::default());
        let watcher = Arc::new(Watcher {
            waiting: 3,
            ..Watcher::default()
        });
        let service = PreviewService::new(
            Arc::new(NoFile),
            site(&input),
            output(FolderState::Missing),
            reading(&input),
            Arc::new(Recorder::default()),
            server.clone(),
            watcher.clone(),
        );
        let options = ServeOptions {
            mode: ServeMode::Watch,
            ..ServeOptions::default()
        };
        let events = Recorded::default();
        scope(|scope| {
            let serving = scope.spawn(|| service.serve(&options, words(), &events));
            assert_eq!(events.wait_for(2).len(), 2);
            drop(watcher.sender.lock().expect("lock").take());
            assert_eq!(serving.join().expect("no panic"), Ok(()));
        });
        assert_eq!(
            *events.0.lock().expect("lock"),
            ["listening at 127.0.0.1:8000", "rebuilt"]
        );
        let (host, port, preview) = server
            .started
            .lock()
            .expect("lock")
            .clone()
            .expect("started");
        assert_eq!(
            (host.as_str(), port, preview.current().number),
            ("127.0.0.1", 8000, 2)
        );
    }

    #[test]
    fn a_build_into_a_folder_ghg_did_not_build_serves_nothing_unless_it_is_in_memory() {
        let input = Arc::new(Input::default());
        let server = Arc::new(Server::default());
        let watcher = Arc::new(Watcher::default());
        let service = PreviewService::new(
            Arc::new(NoFile),
            site(&input),
            output(FolderState::Other),
            reading(&input),
            Arc::new(Recorder::default()),
            server.clone(),
            watcher.clone(),
        );
        let failure = service.serve(&ServeOptions::default(), words(), &Recorded::default());
        assert!(
            matches!(
                &failure,
                Err(ServeFailure::Output(problem)) if problem.kind == OutputProblemKind::NotASite
            ),
            "{failure:?}"
        );
        assert!(server.started.lock().expect("lock").is_none());
        let options = ServeOptions {
            mode: ServeMode::Watch,
            ..ServeOptions::default()
        };
        let events = Recorded::default();
        scope(|scope| {
            let serving = scope.spawn(|| service.serve(&options, words(), &events));
            assert_eq!(events.wait_for(1).len(), 1);
            drop(watcher.sender.lock().expect("lock").take());
            assert_eq!(serving.join().expect("no panic"), Ok(()));
        });
        assert!(server.started.lock().expect("lock").is_some());
    }
}
