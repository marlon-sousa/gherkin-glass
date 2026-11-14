//! The fakes of serving: the server service, and a listener that records what it is told.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{IntoIter, Sender, channel};

use super::*;
use crate::entities::{InputChanges, Listening, Preview, Served, StopRequests};
use crate::errors::WatchError;
use crate::ports::StopSignals;
use crate::services::WatchedChanges;
use crate::test_support::FakeSignals;

/// Listens for the requests to stop with `signals`; serves at 127.0.0.1, port 8000, keeping
/// the preview, unless `listen_failing`, and counts the sites no longer served; watches, unless
/// `watch_failing`, keeping the sender of each watch's changes, and counts the watches dropped.
/// The configuration file alone is watched even while `watch_failing`, unless
/// `config_watch_failing`.
#[derive(Default)]
pub(crate) struct FakeServerService {
    pub(super) calls: Arc<Calls>,
    pub(crate) signals: FakeSignals,
    pub(crate) no_longer_served: Arc<AtomicUsize>,
    pub(crate) listen_failing: AtomicBool,
    pub(crate) watch_failing: AtomicBool,
    pub(crate) config_watch_failing: AtomicBool,
    pub(super) senders: Mutex<Vec<Sender<InputChanges>>>,
    pub(crate) dropped: Arc<AtomicUsize>,
    pub(super) preview: Mutex<Option<Arc<Preview>>>,
}

impl ServerApi for FakeServerService {
    fn stop_requests(&self) -> Result<StopRequests, ServeError> {
        self.calls.note("server.stop_requests".into());
        self.signals.listen().map_err(ServeError::Signals)
    }

    fn watch<'a>(
        &self,
        input: &Path,
        settings_files: &[PathBuf],
        _: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError> {
        let files: Vec<_> = settings_files
            .iter()
            .map(|file| file.display().to_string())
            .collect();
        let call = format!("server.watch {} {}", input.display(), files.join(" "));
        self.watched(call, input, &self.watch_failing)
    }

    fn watch_config<'a>(
        &self,
        config: &Path,
        _: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError> {
        let call = format!("server.watch_config {}", config.display());
        self.watched(call, config, &self.config_watch_failing)
    }

    fn serve(&self, site: ServedSite, options: &ServeOptions) -> Result<Served, ServeError> {
        self.calls.note(format!("server.serve {site:?}"));
        if self.listen_failing.load(Ordering::SeqCst) {
            return Err(listen_error());
        }
        let preview = Arc::new(Preview::new(options.mode, site));
        *self.preview.lock().unwrap_or_else(PoisonError::into_inner) = Some(preview.clone());
        Ok(Served {
            listening: Listening {
                address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8000)),
                answering: Box::new(Counted {
                    dropped: self.no_longer_served.clone(),
                }),
            },
            preview,
        })
    }
}

impl FakeServerService {
    /// Notes `call` once the watch of `path` has started, so that a test that has seen it can
    /// send to it, or has failed, while `failing`.
    fn watched(
        &self,
        call: String,
        path: &Path,
        failing: &AtomicBool,
    ) -> Result<WatchedChanges<'static>, WatchError> {
        if failing.load(Ordering::SeqCst) {
            self.calls.note(call);
            return Err(watch_error(path));
        }
        let (sender, changes) = channel();
        self.senders
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(sender);
        self.calls.note(call);
        Ok(Box::new(CountedChanges {
            changes: changes.into_iter(),
            counted: Counted {
                dropped: self.dropped.clone(),
            },
        }))
    }

    /// Sends `changes` to the last watch started that is not dropped.
    pub(crate) fn change(&self, changes: InputChanges) {
        let senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        let sent = senders.iter().rev().any(|sender| sent_to(sender, &changes));
        assert!(sent, "a watch not dropped");
    }

    /// Ends the watching.
    pub(crate) fn stop_watching(&self) {
        self.senders
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    /// The preview served, once a site is.
    pub(crate) fn preview(&self) -> Arc<Preview> {
        self.preview
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .expect("served")
    }
}

/// The error of a watch of `path` that cannot start.
pub(crate) fn watch_error(path: &Path) -> WatchError {
    WatchError {
        path: path.to_path_buf(),
        reason: "too many folders".into(),
    }
}

/// Whether `changes` went to the watch of `sender`, which they cannot once it is dropped.
fn sent_to(sender: &Sender<InputChanges>, changes: &InputChanges) -> bool {
    sender.send(changes.clone()).is_ok()
}

/// A watch's changes, counted once dropped.
struct CountedChanges {
    changes: IntoIter<InputChanges>,
    #[expect(dead_code, reason = "it is held only to be dropped")]
    counted: Counted,
}

impl Iterator for CountedChanges {
    type Item = InputChanges;

    fn next(&mut self) -> Option<InputChanges> {
        self.changes.next()
    }
}

/// Counted in `dropped` once dropped.
struct Counted {
    dropped: Arc<AtomicUsize>,
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

/// Records each event it is told of.
#[derive(Default)]
pub(crate) struct Recorded {
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
    fn not_watched(&self, error: &WatchError) {
        self.push(format!("not watched: {error}"));
    }
}

impl Recorded {
    fn push(&self, event: String) {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }

    /// The events so far, once there are `count` of them, or after ten seconds.
    pub(crate) fn wait_for(&self, count: usize) -> Vec<String> {
        waited_for(count, || self.taken())
    }

    fn taken(&self) -> Vec<String> {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
