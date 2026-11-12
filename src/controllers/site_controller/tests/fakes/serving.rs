//! The fakes of serving: the server service, and a listener that records what it is told.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};

use super::*;
use crate::entities::{InputChanges, Preview, Served};
use crate::errors::WatchError;
use crate::services::WatchedChanges;

/// Serves at 127.0.0.1, port 8000, keeping the preview, unless `listen_failing`; watches,
/// unless `watch_failing`, and hands the test the sender of the changes of the last watch it
/// started, so that sending to an earlier one changes nothing. The configuration file alone is
/// watched even while `watch_failing`, unless `config_watch_failing`.
#[derive(Default)]
pub(crate) struct FakeServerService {
    pub(super) calls: Arc<Calls>,
    pub(crate) listen_failing: AtomicBool,
    pub(crate) watch_failing: AtomicBool,
    pub(crate) config_watch_failing: AtomicBool,
    pub(super) sender: Mutex<Option<Sender<InputChanges>>>,
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
        *self.sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(sender);
        self.calls.note(call);
        Ok(Box::new(changes.into_iter()))
    }

    /// Sends `changes` to the last watch started.
    pub(crate) fn change(&self, changes: InputChanges) {
        self.sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .expect("watching")
            .send(changes)
            .expect("sent");
    }

    /// Ends the watching.
    pub(crate) fn stop_watching(&self) {
        self.sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
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
