use std::collections::{BTreeMap, BTreeSet};
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::*;
use crate::entities::{EntryKind, FolderEntry, OutputContents, RawFeature, Record};
use crate::errors::{InputErrorKind, ParseError};
use crate::fixtures::{FakeServer, FakeWatcher};
use crate::ports::{ConfigReader, FeatureParser, FileWriter, InputFolders, OutputFolders};

/// Configuration files held in memory; any other path is not found.
struct ConfigFiles {
    files: BTreeMap<PathBuf, String>,
}

impl ConfigReader for ConfigFiles {
    fn read(&self, path: &Path) -> IoResult<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| IoError::new(ErrorKind::NotFound, "not found"))
    }
}

/// An input with no features, or, while `failing`, one file, `a.feature`,
/// in whatever folder is read, which fails with that folder in its message;
/// `listed` counts how often a folder was listed.
#[derive(Default)]
pub(super) struct Input {
    pub(super) failing: AtomicBool,
    pub(super) listed: AtomicUsize,
}

impl InputFolders for Input {
    fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
        self.listed.fetch_add(1, Ordering::SeqCst);
        if !self.failing.load(Ordering::SeqCst) {
            return Ok(Vec::new());
        }
        Ok(vec![FolderEntry {
            path: folder.join("a.feature"),
            kind: EntryKind::File,
        }])
    }

    fn read(&self, _: &Path) -> IoResult<String> {
        Err(IoError::new(ErrorKind::NotFound, "not found"))
    }
}

impl FeatureParser for Input {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError> {
        Err(ParseError {
            line: Some(1),
            kind: InputErrorKind::Parser {
                message: format!("read in {}", path.parent().unwrap_or(path).display()),
            },
        })
    }
}

/// Output folders that all hold the same contents.
struct Folders {
    contents: OutputContents,
}

impl OutputFolders for Folders {
    fn contents(&self, _: &Path) -> IoResult<OutputContents> {
        Ok(self.contents.clone())
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

/// Keeps what is written in memory until the site is taken.
#[derive(Default)]
pub(super) struct Recorder {
    pub(super) files: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl FileWriter for Recorder {
    fn write(&self, _: &Path, file: &str, contents: &[u8]) -> IoResult<()> {
        self.files
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(file.to_string(), contents.to_vec());
        Ok(())
    }

    fn site(&self, _: &Path) -> ServedSite {
        ServedSite::Memory(take(
            &mut *self.files.lock().unwrap_or_else(PoisonError::into_inner),
        ))
    }
}

/// Writes nothing, as if into the output folder.
pub(super) struct NoFolder;

impl FileWriter for NoFolder {
    fn write(&self, _: &Path, _: &str, _: &[u8]) -> IoResult<()> {
        Ok(())
    }

    fn site(&self, output: &Path) -> ServedSite {
        ServedSite::Folder(output.to_path_buf())
    }
}

/// What the controller is wired to: its input, its server, its watcher and its memory.
#[derive(Default)]
pub(super) struct World {
    pub(super) input: Arc<Input>,
    pub(super) server: Arc<FakeServer>,
    pub(super) watcher: Arc<FakeWatcher>,
    pub(super) memory: Arc<Recorder>,
}

impl World {
    /// A controller over this world, with the configuration `files`, and
    /// output folders that all hold `contents`.
    pub(super) fn controller(
        &self,
        files: &[(&str, &str)],
        contents: OutputContents,
    ) -> SiteController {
        let files = files
            .iter()
            .map(|(path, text)| (PathBuf::from(path), text.to_string()))
            .collect();
        SiteController::new(
            SettingsService::new(Arc::new(ConfigFiles { files })),
            InputService::new(self.input.clone(), self.input.clone()),
            Converter,
            SiteWriter::new(
                Arc::new(Folders { contents }),
                Arc::new(NoFolder),
                self.memory.clone(),
            ),
            ServerService::new(self.server.clone(), self.watcher.clone()),
        )
    }
}

/// Records each event it is told of.
#[derive(Default)]
pub(super) struct Recorded {
    events: Mutex<Vec<String>>,
}

impl SiteListener for Recorded {
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
    fn push(&self, event: String) {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }

    /// The events so far, once there are `count` of them.
    pub(super) fn wait_for(&self, count: usize) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let events = self
                .events
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

/// A folder holding a file ghg did not write, and no marker.
pub(super) fn not_a_site() -> OutputContents {
    OutputContents::Folder {
        files: ["notes.txt".to_string()].into(),
        folders: BTreeSet::new(),
        record: None,
    }
}

pub(super) fn wait_until(done: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done() && Instant::now() < deadline {
        sleep(Duration::from_millis(5));
    }
}
