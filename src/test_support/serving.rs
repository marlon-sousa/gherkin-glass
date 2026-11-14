//! Test support: a server, a watcher and stop signals for ghg serve, which touch no network, no folder and no signal.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use crate::entities::{Burst, Changes, Listening, Preview, ServerWords, StopRequests, Watched};
use crate::errors::{ListenErrorKind, SignalsError, WatchError};
use crate::ports::{ChangeWatcher, PreviewServer, StopSignals};

/// Records the preview it was given, and listens at port 1 for port 0,
/// unless the port is `taken`; counts the sites no longer served.
#[derive(Default)]
pub(crate) struct FakeServer {
    pub(crate) started: Mutex<Option<(String, u16, Arc<Preview>)>>,
    pub(crate) taken: bool,
    pub(crate) no_longer_served: Arc<AtomicUsize>,
}

impl PreviewServer for FakeServer {
    fn start(
        &self,
        host: &str,
        port: u16,
        preview: Arc<Preview>,
        _: ServerWords,
    ) -> Result<Listening, ListenErrorKind> {
        if self.taken {
            return Err(ListenErrorKind::PortTaken);
        }
        *self.started.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((host.to_string(), port, preview));
        Ok(Listening {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port.max(1))),
            answering: Box::new(Counted {
                dropped: self.no_longer_served.clone(),
            }),
        })
    }
}

impl FakeServer {
    /// The host, the port and the preview it was started with.
    pub(crate) fn started(&self) -> Option<(String, u16, Arc<Preview>)> {
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Hands the test the sender of its changes, and records what it
/// watches; `waiting` bursts are already sent when watching starts, and
/// watching cannot start while it is `failing`.
#[derive(Default)]
pub(crate) struct FakeWatcher {
    pub(crate) sender: Mutex<Option<Sender<Burst>>>,
    pub(crate) watched: Mutex<Vec<Watched>>,
    pub(crate) waiting: usize,
    pub(crate) failing: bool,
}

impl ChangeWatcher for FakeWatcher {
    fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchError> {
        *self.watched.lock().unwrap_or_else(PoisonError::into_inner) = watched.to_vec();
        if self.failing {
            return Err(WatchError {
                path: PathBuf::from("features"),
                reason: "too many folders".into(),
            });
        }
        let (sender, bursts) = channel();
        for _ in 0..self.waiting {
            sender.send(Burst::Missed).expect("sent");
        }
        *self.sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(sender);
        Ok(Changes {
            bursts,
            watching: Box::new(()),
        })
    }
}

impl FakeWatcher {
    /// The sender of the changes, which ends the watching once dropped.
    pub(crate) fn sender(&self) -> Option<Sender<Burst>> {
        self.sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }
}

/// A burst that changed only `path`.
pub(crate) fn changed(path: &str) -> Burst {
    Burst::Changed(vec![PathBuf::from(path)])
}

/// Keeps the sender of the requests to stop, for the test to ask with, unless it is `failing`;
/// asks at once as it starts listening while `asking_at_once`; once told to end on another
/// request, notes in `ending` how many sites `no_longer_served` counted then.
#[derive(Default)]
pub(crate) struct FakeSignals {
    pub(crate) sender: Mutex<Option<Sender<()>>>,
    pub(crate) failing: AtomicBool,
    pub(crate) asking_at_once: AtomicBool,
    pub(crate) no_longer_served: Arc<AtomicUsize>,
    pub(crate) ending: Mutex<Option<usize>>,
}

impl StopSignals for FakeSignals {
    fn listen(&self) -> Result<StopRequests, SignalsError> {
        if self.failing.load(Ordering::SeqCst) {
            return Err(SignalsError {
                reason: "not permitted".into(),
            });
        }
        let (sender, requests) = channel();
        *self.sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(sender);
        if self.asking_at_once.load(Ordering::SeqCst) {
            self.ask_to_stop();
        }
        Ok(StopRequests::new(requests))
    }

    fn end_on_another(&self, _: StopRequests) {
        *self.ending.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(self.no_longer_served.load(Ordering::SeqCst));
    }
}

impl FakeSignals {
    /// Asks ghg serve to stop, once it listens.
    pub(crate) fn ask_to_stop(&self) {
        let sender = self.sender.lock().unwrap_or_else(PoisonError::into_inner);
        sender.as_ref().expect("listening").send(()).expect("sent");
    }
}

/// Counted in `dropped` once dropped.
pub(crate) struct Counted {
    pub(crate) dropped: Arc<AtomicUsize>,
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
