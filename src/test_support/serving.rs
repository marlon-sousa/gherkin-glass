//! Test support: a server, a watcher, stop signals and a waiter for ghg serve, which touch no network, no folder and no signal.

use std::collections::VecDeque;
use std::mem::take;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use crate::entities::{
    Burst, BurstTeller, ChangeOrStop, Listening, Preview, ServerWords, StopTeller, Watched,
    Watching,
};
use crate::errors::{ListenErrorKind, SignalsError, WatchError};
use crate::ports::{
    ChangeOrStopWaiter, ChangeWatcher, PreviewServer, ServeEventWaiter, StopSignals,
};

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

/// Keeps the teller of its bursts, for the test to tell with, and records what it watches;
/// `waiting` bursts are told as watching starts, and watching cannot start while it is
/// `failing`.
#[derive(Default)]
pub(crate) struct FakeWatcher {
    pub(crate) teller: Mutex<Option<BurstTeller>>,
    pub(crate) watched: Mutex<Vec<Watched>>,
    pub(crate) waiting: usize,
    pub(crate) failing: bool,
}

impl ChangeWatcher for FakeWatcher {
    fn watch(&self, watched: &[Watched], tell: BurstTeller) -> Result<Watching, WatchError> {
        *self.watched.lock().unwrap_or_else(PoisonError::into_inner) = watched.to_vec();
        if self.failing {
            return Err(WatchError {
                path: PathBuf::from("features"),
                reason: "too many folders".into(),
            });
        }
        for _ in 0..self.waiting {
            tell(Burst::Missed);
        }
        *self.teller.lock().unwrap_or_else(PoisonError::into_inner) = Some(tell);
        Ok(Box::new(()))
    }
}

impl FakeWatcher {
    /// Tells of `burst`, once watching.
    pub(crate) fn tell(&self, burst: Burst) {
        let teller = self.teller.lock().unwrap_or_else(PoisonError::into_inner);
        teller.as_ref().expect("watching")(burst);
    }
}

/// A burst that changed only `path`.
pub(crate) fn changed(path: &str) -> Burst {
    Burst::Changed(vec![PathBuf::from(path)])
}

/// Keeps the teller of the requests to stop, for the test to ask with, unless it is `failing`;
/// asks at once as it starts listening while `asking_at_once`; once told to end ghg at once,
/// notes in `ending` how many sites `no_longer_served` counted then.
#[derive(Default)]
pub(crate) struct FakeSignals {
    pub(crate) teller: Mutex<Option<StopTeller>>,
    pub(crate) failing: AtomicBool,
    pub(crate) asking_at_once: AtomicBool,
    pub(crate) no_longer_served: Arc<AtomicUsize>,
    pub(crate) ending: Mutex<Option<usize>>,
}

impl StopSignals for FakeSignals {
    fn listen(&self, tell: StopTeller) -> Result<(), SignalsError> {
        if self.failing.load(Ordering::SeqCst) {
            return Err(SignalsError {
                reason: "not permitted".into(),
            });
        }
        *self.teller.lock().unwrap_or_else(PoisonError::into_inner) = Some(tell);
        if self.asking_at_once.load(Ordering::SeqCst) {
            self.ask_to_stop();
        }
        Ok(())
    }

    fn end_at_once(&self) {
        *self.ending.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(self.no_longer_served.load(Ordering::SeqCst));
    }
}

impl FakeSignals {
    /// Asks ghg serve to stop, once it listens.
    pub(crate) fn ask_to_stop(&self) {
        let teller = self.teller.lock().unwrap_or_else(PoisonError::into_inner);
        teller.as_ref().expect("listening")();
    }
}

/// Waits on standard channels: for a request to stop, until one is told, and for a change or
/// a request to stop, by handing back the bursts told to the last watch, or, when none was,
/// the next of `script`, or the end of the watching once `script` is empty. Runs at once what
/// another request to stop would run, as if one came at once.
pub(crate) struct FakeWaiter {
    sender: Sender<()>,
    requests: Mutex<Receiver<()>>,
    asked: AtomicBool,
    pub(crate) script: Arc<Mutex<VecDeque<ChangeOrStop>>>,
}

impl Default for FakeWaiter {
    fn default() -> Self {
        let (sender, requests) = channel();
        FakeWaiter {
            sender,
            requests: Mutex::new(requests),
            asked: AtomicBool::default(),
            script: Arc::default(),
        }
    }
}

impl ServeEventWaiter for FakeWaiter {
    fn stop_teller(&self) -> StopTeller {
        let sender = self.sender.clone();
        Box::new(move || sender.send(()).expect("waited for"))
    }

    fn asked_to_stop(&self) -> bool {
        let requests = self.requests.lock().unwrap_or_else(PoisonError::into_inner);
        if requests.try_recv().is_ok() {
            self.asked.store(true, Ordering::SeqCst);
        }
        self.asked.load(Ordering::SeqCst)
    }

    fn wait_for_stop(&self) {
        if !self.asked_to_stop() {
            let requests = self.requests.lock().unwrap_or_else(PoisonError::into_inner);
            requests.recv().expect("asked");
            self.asked.store(true, Ordering::SeqCst);
        }
    }

    fn on_another_stop(&self, then: Box<dyn FnOnce() + Send>) {
        then();
    }

    fn bursts(&self) -> (BurstTeller, Box<dyn ChangeOrStopWaiter>) {
        let told = Arc::new(Mutex::new(Vec::new()));
        let telling = told.clone();
        let tell = Box::new(move |burst| {
            telling
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(burst);
        });
        let waiter = ScriptedWaiter {
            told,
            script: self.script.clone(),
        };
        (tell, Box::new(waiter))
    }
}

/// One watch's waiter, as [`FakeWaiter`] says.
struct ScriptedWaiter {
    told: Arc<Mutex<Vec<Burst>>>,
    script: Arc<Mutex<VecDeque<ChangeOrStop>>>,
}

impl ChangeOrStopWaiter for ScriptedWaiter {
    fn wait_for_change_or_stop(&self) -> ChangeOrStop {
        let told = take(&mut *self.told.lock().unwrap_or_else(PoisonError::into_inner));
        if !told.is_empty() {
            return ChangeOrStop::Changed(told);
        }
        self.script
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .unwrap_or(ChangeOrStop::WatchingEnded)
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
