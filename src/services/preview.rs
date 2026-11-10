//! Service: previewing a site, built and served, and rebuilt as its files change.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::park;

use crate::entities::{
    ListenFailure, Preview, Problems, ServeOptions, ServedSite, ServerWords, Settings,
    WatchFailure, Watched,
};

use crate::ports::{ChangeWatcher, ConfigReader, PreviewServer, SiteRecorder};
use crate::services::settings::CONFIG_FILE;
use crate::services::{ConfigProblems, InputService, SettingsService, SiteService};

/// The address ghg serve listens on unless `--host` names another.
const DEFAULT_HOST: &str = "127.0.0.1";
/// The port ghg serve listens at unless `--port` names another.
const DEFAULT_PORT: u16 = 8000;

pub struct PreviewService {
    settings: SettingsService,
    on_disk: SiteService,
    in_memory: SiteService,
    memory: Arc<dyn SiteRecorder>,
    server: Arc<dyn PreviewServer>,
    watcher: Arc<dyn ChangeWatcher>,
}

/// What stops ghg serve before it serves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServeFailure {
    Config(ConfigProblems),
    Input(Problems),
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
    /// `on_disk` writes the site into its output folder; under `--watch`,
    /// `input` is built into `memory`.
    pub fn new(
        config_reader: Arc<dyn ConfigReader>,
        on_disk: SiteService,
        input: InputService,
        memory: Arc<dyn SiteRecorder>,
        server: Arc<dyn PreviewServer>,
        watcher: Arc<dyn ChangeWatcher>,
    ) -> Self {
        PreviewService {
            settings: SettingsService::new(config_reader),
            on_disk,
            in_memory: SiteService::new(input, memory.clone()),
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
        // Watching starts before the first build, so that a change made
        // during it is not missed; what stops the build is said first.
        let changes = options.watch.then(|| {
            let config = options
                .build
                .config
                .clone()
                .unwrap_or_else(|| PathBuf::from(CONFIG_FILE));
            let watched = [
                Watched::Folder(settings.input.clone()),
                Watched::File(config),
            ];
            self.watcher.watch(&watched)
        });
        let site = if options.no_build {
            ServedSite::Folder(settings.output.clone())
        } else if options.watch {
            self.build_in_memory(&settings)
                .map_err(ServeFailure::Input)?
        } else {
            self.build_into_output(&settings)
                .map_err(ServeFailure::Input)?
        };
        let changes = changes.transpose().map_err(ServeFailure::Watch)?;
        let preview = Arc::new(Preview::new(options.watch, site));
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
        while changes.bursts.recv().is_ok() {
            // Bursts that came during the last rebuild need only this one.
            while changes.bursts.try_recv().is_ok() {}
            match self.rebuild(options) {
                Ok(site) => {
                    preview.replace(site);
                    events.rebuilt();
                }
                Err(failure) => events.not_rebuilt(&failure),
            }
        }
        Ok(())
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
        built.map(|()| ServedSite::Memory(files))
    }

    fn build_into_output(&self, settings: &Settings) -> Result<ServedSite, Problems> {
        self.on_disk.build(settings, &settings.output)?;
        Ok(ServedSite::Folder(settings.output.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Error as IoError, ErrorKind, Result as IoResult};
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{Sender, channel};
    use std::sync::{Mutex, PoisonError};
    use std::thread::{scope, sleep};
    use std::time::{Duration, Instant};

    use super::*;
    use crate::entities::{
        Changes, CollectedInput, ParseFailure, Problem, ProblemKind, RawFeature,
    };
    use crate::ports::{FeatureParser, InputCollector, SiteWriter};
    use std::collections::BTreeMap;

    /// No configuration file.
    struct NoFile;

    impl ConfigReader for NoFile {
        fn read(&self, _: &Path) -> IoResult<String> {
            Err(IoError::new(ErrorKind::NotFound, "not found"))
        }
    }

    /// An input with no features, or, while `failing`, one problem.
    #[derive(Default)]
    struct Input {
        failing: AtomicBool,
    }

    impl InputCollector for Input {
        fn collect(&self, input: &Path) -> CollectedInput {
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
        sender: Mutex<Option<Sender<()>>>,
        watched: Mutex<Vec<Watched>>,
        waiting: usize,
    }

    impl ChangeWatcher for Watcher {
        fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchFailure> {
            let (sender, bursts) = channel();
            for _ in 0..self.waiting {
                sender.send(()).expect("sent");
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

    fn reading(input: &Arc<Input>) -> InputService {
        InputService::new(input.clone(), input.clone())
    }

    fn words() -> ServerWords {
        ServerWords {
            not_found: "not found".into(),
            wrong_method: "wrong method".into(),
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
            reading(&input),
            recorder,
            server.clone(),
            watcher.clone(),
        );
        let options = ServeOptions {
            watch: true,
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
            assert!(preview.watching);
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

            sender.send(()).expect("sent");
            assert_eq!(events.wait_for(2)[1], "rebuilt");
            assert_eq!(preview.current().number, 2);

            input.failing.store(true, Ordering::SeqCst);
            sender.send(()).expect("sent");
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
            reading(&input),
            Arc::new(Recorder::default()),
            server.clone(),
            Arc::new(Watcher::default()),
        );
        for watch in [false, true] {
            let options = ServeOptions {
                watch,
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
            reading(&input),
            Arc::new(Recorder::default()),
            server.clone(),
            watcher.clone(),
        );
        let options = ServeOptions {
            watch: true,
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
}
