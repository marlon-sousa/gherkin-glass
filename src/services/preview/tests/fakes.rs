use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Mutex, PoisonError};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::*;
use crate::entities::{Changes, CollectedInput, OutputContents, RawFeature, Record};
use crate::errors::{InputError, InputErrorKind, ListenErrorKind, ParseError, WatchError};
use crate::ports::{FeatureParser, InputCollector, OutputFolders, SiteWriter};
use std::collections::{BTreeMap, BTreeSet};

/// No configuration file.
pub(super) struct NoFile;

impl ConfigReader for NoFile {
    fn read(&self, _: &Path) -> IoResult<String> {
        Err(IoError::new(ErrorKind::NotFound, "not found"))
    }
}

/// An input with no features, or, while `failing`, one error;
/// `collected` counts how often it was walked.
#[derive(Default)]
pub(super) struct Input {
    pub(super) failing: AtomicBool,
    pub(super) collected: AtomicUsize,
}

impl InputCollector for Input {
    fn collect(&self, input: &Path) -> CollectedInput {
        self.collected.fetch_add(1, Ordering::SeqCst);
        let errors = if self.failing.load(Ordering::SeqCst) {
            let kind = InputErrorKind::Changed;
            vec![InputError::new(input, &input.join("a.feature"), None, kind)]
        } else {
            Vec::new()
        };
        CollectedInput {
            paths: Vec::new(),
            errors,
        }
    }
}

impl FeatureParser for Input {
    fn parse(&self, _: &Path) -> Result<RawFeature, ParseError> {
        unreachable!("no feature is collected")
    }
}

/// Records the preview it was given, and listens at port 1 for port 0.
#[derive(Default)]
pub(super) struct Server {
    pub(super) started: Mutex<Option<(String, u16, Arc<Preview>)>>,
}

impl PreviewServer for Server {
    fn start(
        &self,
        host: &str,
        port: u16,
        preview: Arc<Preview>,
        _: ServerWords,
    ) -> Result<SocketAddr, ListenErrorKind> {
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
pub(super) struct Watcher {
    pub(super) sender: Mutex<Option<Sender<Burst>>>,
    pub(super) watched: Mutex<Vec<Watched>>,
    pub(super) waiting: usize,
}

impl ChangeWatcher for Watcher {
    fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchError> {
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
pub(super) struct Recorded(pub(super) Mutex<Vec<String>>);

impl PreviewEvents for Recorded {
    fn listening(&self, address: SocketAddr) {
        self.push(format!("listening at {address}"));
    }
    fn rebuilt(&self) {
        self.push("rebuilt".into());
    }
    fn not_rebuilt(&self, error: &RebuildError) {
        self.push(format!("not rebuilt: {error:?}"));
    }
}

impl Recorded {
    pub(super) fn push(&self, event: String) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }

    /// The events so far, once there are `count` of them.
    pub(super) fn wait_for(&self, count: usize) -> Vec<String> {
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
pub(super) struct Recorder {
    pub(super) files: Mutex<BTreeMap<String, Vec<u8>>>,
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
pub(super) fn site(input: &Arc<Input>) -> SiteService {
    SiteService::new(reading(input), Arc::new(Recorder::default()))
}

/// Output folders that all hold the same contents.
pub(super) struct Folders(OutputContents);

impl OutputFolders for Folders {
    fn contents(&self, _: &Path) -> IoResult<OutputContents> {
        Ok(self.0.clone())
    }

    fn clear(&self, _: &Path) -> IoResult<()> {
        Ok(())
    }

    fn remove(&self, _: &Path) -> IoResult<()> {
        Ok(())
    }

    fn record(&self, _: &Path, _: &Record) -> IoResult<()> {
        Ok(())
    }
}

pub(super) fn output(contents: OutputContents) -> OutputService {
    OutputService::new(Arc::new(Folders(contents)))
}

/// A folder holding a file ghg did not write, and no marker.
pub(super) fn not_a_site() -> OutputContents {
    OutputContents::Folder {
        files: ["notes.txt".to_string()].into(),
        folders: BTreeSet::new(),
        record: None,
    }
}

pub(super) fn reading(input: &Arc<Input>) -> InputService {
    InputService::new(input.clone(), input.clone())
}

pub(super) fn words() -> ServerWords {
    ServerWords {
        not_found: "not found".into(),
        wrong_method: "wrong method".into(),
    }
}

pub(super) fn changed(path: &str) -> Burst {
    Burst::Changed(vec![PathBuf::from(path)])
}

pub(super) fn wait_until(done: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done() && Instant::now() < deadline {
        sleep(Duration::from_millis(5));
    }
}
