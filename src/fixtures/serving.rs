//! Test support: a server and a watcher for ghg serve, which touch no network and no folder.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use crate::entities::{Burst, Changes, Preview, ServerWords, Watched};
use crate::errors::{ListenErrorKind, WatchError};
use crate::ports::{ChangeWatcher, PreviewServer};

/// Records the preview it was given, and listens at port 1 for port 0,
/// unless the port is `taken`.
#[derive(Default)]
pub(crate) struct FakeServer {
    pub(crate) started: Mutex<Option<(String, u16, Arc<Preview>)>>,
    pub(crate) taken: bool,
}

impl PreviewServer for FakeServer {
    fn start(
        &self,
        host: &str,
        port: u16,
        preview: Arc<Preview>,
        _: ServerWords,
    ) -> Result<SocketAddr, ListenErrorKind> {
        if self.taken {
            return Err(ListenErrorKind::PortTaken);
        }
        *self.started.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((host.to_string(), port, preview));
        Ok(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::LOCALHOST,
            port.max(1),
        )))
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
