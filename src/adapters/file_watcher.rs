//! Adapter: the change watcher port over the file system, with notify and its full debouncer.

use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use notify_debouncer_full::notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};

use crate::entities::{Changes, WatchFailure, Watched};
use crate::ports::ChangeWatcher;

/// How long the files must stay unchanged before a burst of changes ends.
const QUIET_FOR: Duration = Duration::from_millis(200);

#[derive(Debug, Default, Clone, Copy)]
pub struct FileWatcher;

impl ChangeWatcher for FileWatcher {
    fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchFailure> {
        let mut folders = Vec::new();
        let mut files = Vec::new();
        // Canonical paths, so that every event's path compares with them.
        for watched in watched {
            match watched {
                Watched::Folder(folder) => folders.push(canonical(folder)?),
                Watched::File(file) => {
                    let parent = match file.parent() {
                        Some(parent) if !parent.as_os_str().is_empty() => parent,
                        _ => Path::new("."),
                    };
                    match file.file_name() {
                        Some(name) => {
                            let parent = canonical(parent)?;
                            files.push((parent.clone(), parent.join(name)));
                        }
                        None => {
                            let folder = canonical(file)?;
                            files.push((folder.clone(), folder));
                        }
                    }
                }
            }
        }
        let (sender, bursts) = channel();
        let (watched_folders, watched_files) = (folders.clone(), files.clone());
        let handler = move |result: DebounceEventResult| {
            let changed = match result {
                // Reading a file is not changing it, and the build reads them all.
                Ok(events) => events.iter().any(|event| {
                    !matches!(event.kind, EventKind::Access(_))
                        && event.paths.iter().any(|path| {
                            watched_folders
                                .iter()
                                .any(|folder| path.starts_with(folder))
                                || watched_files.iter().any(|(_, file)| path == file)
                        })
                }),
                // A change was missed; a rebuild covers it.
                Err(_) => true,
            };
            if changed {
                // Nobody is listening once ghg serve stops.
                let _ = sender.send(());
            }
        };
        let failure = |path: &Path| {
            let path = path.to_path_buf();
            move |error: notify_debouncer_full::notify::Error| WatchFailure {
                path,
                reason: error.to_string(),
            }
        };
        let first = folders
            .first()
            .or(files.first().map(|(parent, _)| parent))
            .cloned()
            .unwrap_or_default();
        let mut debouncer = new_debouncer(QUIET_FOR, None, handler).map_err(failure(&first))?;
        for folder in &folders {
            debouncer
                .watch(folder, RecursiveMode::Recursive)
                .map_err(failure(folder))?;
        }
        // Watching a folder already watched with its subfolders, but alone,
        // would stop the subfolders being watched on macOS and Windows.
        for (parent, _) in &files {
            if folders.iter().any(|folder| parent.starts_with(folder)) {
                continue;
            }
            debouncer
                .watch(parent, RecursiveMode::NonRecursive)
                .map_err(failure(parent))?;
        }
        Ok(Changes {
            bursts,
            watching: Box::new(debouncer),
        })
    }
}

fn canonical(path: &Path) -> Result<PathBuf, WatchFailure> {
    path.canonicalize().map_err(|error| WatchFailure {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}
