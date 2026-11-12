//! The fakes of serving: the server service, and a listener that records what it is told.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{IntoIter, Sender, channel};

use super::*;
use crate::entities::{InputChanges, Preview, Served};
use crate::errors::WatchError;
use crate::services::WatchedChanges;

/// Serves at 127.0.0.1, port 8000, keeping the preview, unless `listen_failing`; watches,
/// unless `watch_failing`, keeping the sender of each watch's changes, and counts the watches
/// dropped. The configuration file alone is watched even while `watch_failing`, unless
/// `config_watch_failing`.
#[derive(Default)]
pub(crate) struct FakeServerService {
    pub(super) calls: Arc<Calls>,
    pub(crate) listen_failing: AtomicBool,
    pub(crate) watch_failing: AtomicBool,
    pub(crate) config_watch_failing: AtomicBool,
    pub(super) senders: Mutex<Vec<Sender<InputChanges>>>,
    pub(crate) dropped: Arc<AtomicUsize>,
    pub(super) preview: Mutex<Option<Arc<Preview>>>,
}

impl ServerApi for FakeServerService {
    fn watch(&self, input: &Path, config: &Path) -> Result<WatchedChanges, WatchError> {
        let call = format!("server.watch {} {}", input.display(), config.display());
        self.watched(call, input, &self.watch_failing)
    }

    fn watch_config(&self, config: &Path) -> Result<WatchedChanges, WatchError> {
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
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8000)),
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
    ) -> Result<WatchedChanges, WatchError> {
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
            dropped: self.dropped.clone(),
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

/// A watch's changes, counted in `dropped` once dropped.
struct CountedChanges {
    changes: IntoIter<InputChanges>,
    dropped: Arc<AtomicUsize>,
}

impl Iterator for CountedChanges {
    type Item = InputChanges;

    fn next(&mut self) -> Option<InputChanges> {
        self.changes.next()
    }
}

impl Drop for CountedChanges {
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
