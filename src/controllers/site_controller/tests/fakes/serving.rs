//! The fakes of serving: the server service, and a listener that records what it is told.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::*;
use crate::entities::{Burst, Served};
use crate::errors::WatchError;
use crate::test_support::changed;

/// Serves at 127.0.0.1, port 8000, keeping the preview, unless `listen_failing`; watches,
/// unless `watch_failing`, walking the input once with the function it is handed, and hands the
/// test the sender of the changes that call for a rebuild.
#[derive(Default)]
pub(crate) struct FakeServerService {
    pub(super) calls: Arc<Calls>,
    pub(crate) listen_failing: AtomicBool,
    pub(crate) watch_failing: AtomicBool,
    pub(super) sender: Mutex<Option<Sender<Vec<Burst>>>>,
    pub(super) preview: Mutex<Option<Arc<Preview>>>,
}

impl ServerApi for FakeServerService {
    fn watch<'a>(
        &self,
        input: &Path,
        config: &Path,
        selected: &'a dyn Fn(&Path) -> Vec<PathBuf>,
    ) -> Result<Box<dyn Iterator<Item = Vec<Burst>> + 'a>, WatchError> {
        self.calls.note(format!(
            "server.watch {} {}",
            input.display(),
            config.display()
        ));
        selected(input);
        if self.watch_failing.load(Ordering::SeqCst) {
            return Err(WatchError {
                path: input.to_path_buf(),
                reason: "too many folders".into(),
            });
        }
        let (sender, rebuilds) = channel();
        *self.sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(sender);
        Ok(Box::new(rebuilds.into_iter()))
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
    /// Sends one rebuild's changes, a change to `ghg.toml`.
    pub(crate) fn change(&self) {
        self.sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .expect("watching")
            .send(vec![changed("ghg.toml")])
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
