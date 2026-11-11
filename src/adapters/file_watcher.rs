//! Adapter: the change watcher port over the file system, with notify and its full debouncer.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use notify_debouncer_full::notify::{Error as NotifyError, Event, EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventHandler, DebounceEventResult, new_debouncer};

use crate::entities::{Burst, Changes, WatchFailure, Watched};
use crate::ports::ChangeWatcher;

/// How long the files must stay unchanged before a burst of changes ends.
const QUIET_FOR: Duration = Duration::from_millis(200);

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FileWatcher;

impl ChangeWatcher for FileWatcher {
    fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchFailure> {
        let places = watched
            .iter()
            .map(Place::of)
            .collect::<Result<Vec<_>, _>>()?;
        let (sender, bursts) = channel();
        let reporter = Reporter {
            places: places.clone(),
            sender,
        };
        let first = places.first().map(Place::folder).unwrap_or(Path::new("."));
        let mut debouncer = new_debouncer(QUIET_FOR, None, reporter).map_err(failure_at(first))?;
        // A folder watched alone again would stop its subfolders being
        // watched on macOS and Windows.
        for place in places.iter().filter(|place| place.alone(&places)) {
            debouncer
                .watch(place.folder(), place.mode())
                .map_err(failure_at(place.folder()))?;
        }
        Ok(Changes {
            bursts,
            watching: Box::new(debouncer),
        })
    }
}

/// One path watched: as it was given, and the canonical folder the system
/// watches for it, which every event's path compares with.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Place {
    Folder {
        given: PathBuf,
        folder: PathBuf,
    },
    File {
        given: PathBuf,
        folder: PathBuf,
        file: PathBuf,
    },
}

impl Place {
    fn of(watched: &Watched) -> Result<Place, WatchFailure> {
        match watched {
            Watched::Folder(given) => Ok(Place::Folder {
                given: given.clone(),
                folder: canonical(given)?,
            }),
            Watched::File(given) => match given.file_name() {
                Some(name) => {
                    let folder = canonical(parent(given))?;
                    Ok(Place::File {
                        given: given.clone(),
                        file: folder.join(name),
                        folder,
                    })
                }
                // Such as `..`: a folder, watched without its subfolders.
                None => {
                    let folder = canonical(given)?;
                    Ok(Place::File {
                        given: given.clone(),
                        file: folder.clone(),
                        folder,
                    })
                }
            },
        }
    }

    fn folder(&self) -> &Path {
        match self {
            Place::Folder { folder, .. } | Place::File { folder, .. } => folder,
        }
    }

    fn mode(&self) -> RecursiveMode {
        match self {
            Place::Folder { .. } => RecursiveMode::Recursive,
            Place::File { .. } => RecursiveMode::NonRecursive,
        }
    }

    /// Whether the system must watch this place's folder itself: a folder
    /// always, and a file's folder unless a folder watched holds it.
    fn alone(&self, places: &[Place]) -> bool {
        match self {
            Place::Folder { .. } => true,
            Place::File { folder, .. } => !places.iter().any(|place| place.holds(folder)),
        }
    }

    fn holds(&self, path: &Path) -> bool {
        matches!(self, Place::Folder { folder, .. } if path.starts_with(folder))
    }

    /// `path`, written from this place as it was given, when it is here.
    fn given(&self, path: &Path) -> Option<PathBuf> {
        match self {
            Place::Folder { given, folder } => {
                path.strip_prefix(folder).ok().map(|rest| given.join(rest))
            }
            Place::File { given, file, .. } => (path == file).then(|| given.clone()),
        }
    }
}

/// The debouncer's handler, on its own thread: sends each burst's changed
/// paths, written as they were given, on the one channel.
struct Reporter {
    places: Vec<Place>,
    sender: Sender<Burst>,
}

impl DebounceEventHandler for Reporter {
    fn handle_event(&mut self, result: DebounceEventResult) {
        let burst = match result {
            // The system dropped changes, such as when its queue overflowed.
            Ok(events) if events.iter().any(|event| event.need_rescan()) => Burst::Missed,
            Ok(events) => {
                let paths = self.changed(events.iter().map(|event| &event.event));
                if paths.is_empty() {
                    // Nothing watched changed, so there is nothing to report.
                    return;
                }
                Burst::Changed(paths)
            }
            // The watcher failed, so changes may have been missed.
            Err(_) => Burst::Missed,
        };
        // Sending fails only when the receiver is gone, which happens once
        // the service has stopped watching; the burst is then not needed.
        let _ = self.sender.send(burst);
    }
}

impl Reporter {
    fn changed<'a>(&self, events: impl Iterator<Item = &'a Event>) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = events
            .filter(|event| is_change(event))
            .flat_map(|event| &event.paths)
            .flat_map(|path| self.given(path))
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    fn given<'a>(&'a self, path: &'a Path) -> impl Iterator<Item = PathBuf> + 'a {
        self.places
            .iter()
            .filter_map(move |place| place.given(path))
    }
}

/// Opening or reading a file is not changing it, and every build reads them all.
fn is_change(event: &Event) -> bool {
    !matches!(event.kind, EventKind::Access(_))
}

fn parent(file: &Path) -> &Path {
    match file.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

fn canonical(path: &Path) -> Result<PathBuf, WatchFailure> {
    path.canonicalize().map_err(|error| WatchFailure {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

fn failure_at(path: &Path) -> impl FnOnce(NotifyError) -> WatchFailure + use<> {
    let path = path.to_path_buf();
    move |error| WatchFailure {
        path,
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(given: &str, folder: &str) -> Place {
        Place::Folder {
            given: given.into(),
            folder: folder.into(),
        }
    }

    fn file(given: &str, folder: &str) -> Place {
        Place::File {
            given: given.into(),
            folder: folder.into(),
            file: Path::new(folder).join("ghg.toml"),
        }
    }

    #[test]
    fn a_folder_watched_with_its_subfolders_is_never_watched_alone() {
        let places = [
            folder("features", "/p/features"),
            file("ghg.toml", "/p"),
            file("features/ghg.toml", "/p/features"),
            file("features/sub/ghg.toml", "/p/features/sub"),
        ];
        let alone: Vec<_> = places.iter().map(|place| place.alone(&places)).collect();
        assert_eq!(alone, [true, true, false, false]);
    }

    #[test]
    fn changed_paths_are_written_as_given_and_only_where_watched() {
        let reporter = Reporter {
            places: vec![folder("features", "/p/features"), file("ghg.toml", "/p")],
            sender: channel().0,
        };
        let event = |kind, path: &str| Event::new(kind).add_path(path.into());
        let modified = EventKind::Modify(notify_debouncer_full::notify::event::ModifyKind::Any);
        let opened = EventKind::Access(notify_debouncer_full::notify::event::AccessKind::Any);
        let events = [
            event(modified, "/p/features/sub/a.feature"),
            event(modified, "/p/ghg.toml"),
            event(modified, "/p/notes.txt"),
            event(opened, "/p/features/b.feature"),
            event(modified, "/p/features/sub/a.feature"),
        ];
        assert_eq!(
            reporter.changed(events.iter()),
            [
                PathBuf::from("features/sub/a.feature"),
                PathBuf::from("ghg.toml")
            ]
        );
    }

    #[test]
    fn changes_the_system_dropped_are_reported_as_missed() {
        let (sender, bursts) = channel();
        let mut reporter = Reporter {
            places: vec![folder("features", "/p/features")],
            sender,
        };
        let rescan = Event::new(EventKind::Other)
            .set_flag(notify_debouncer_full::notify::event::Flag::Rescan);
        let debounced =
            notify_debouncer_full::DebouncedEvent::new(rescan, std::time::Instant::now());
        reporter.handle_event(Ok(vec![debounced]));
        assert_eq!(bursts.try_recv(), Ok(Burst::Missed));
        reporter.handle_event(Ok(Vec::new()));
        assert!(bursts.try_recv().is_err());
    }
}
