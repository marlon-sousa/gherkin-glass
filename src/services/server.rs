//! Service: the server, serving a site until it is asked to stop and, under `--watch`, handing back the changes waiting, merged, until then.

use std::iter::once;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use i18n_embed_fl::fl;

use crate::entities::{
    ChangeOrStop, InputChanges, Preview, ServeOptions, Served, ServedSite, ServerWords,
    WatchKeeper, Watched,
};
use crate::errors::{ServeError, WatchError};
use crate::policies::{CATALOGUE, merged_bursts};
use crate::ports::{
    ChangeOrStopWaiter, ChangeWatcher, PreviewServer, ServeEventWaiter, StopSignals,
};

/// The address ghg serve listens on unless `--host` names another.
const DEFAULT_HOST: &str = "127.0.0.1";
/// The port ghg serve listens at unless `--port` names another.
const DEFAULT_PORT: u16 = 8000;

/// What the controller asks of the server.
pub(crate) trait ServerApi: Send + Sync {
    /// Starts listening for the requests to stop ghg serve, which from then on no longer end
    /// it at once. Called once in a run, before anything else.
    fn listen_for_stops(&self) -> Result<(), ServeError>;

    /// Whether ghg serve was asked to stop by now. Once asked, it stays asked.
    fn asked_to_stop(&self) -> bool;

    /// Waits until ghg serve is asked to stop.
    fn wait_for_stop(&self);

    /// Starts watching the folder `input` and `settings_files`, the configuration file and the
    /// front page's template. Each item is every burst of changes waiting, merged into one; it
    /// waits for one, and ends when the watching does or once ghg serve is asked to stop.
    /// Dropping it stops the watching, and drops the bursts it still holds.
    fn watch(&self, input: &Path, settings_files: &[PathBuf])
    -> Result<WatchedChanges, WatchError>;

    /// Starts watching the configuration file `config` alone, as [`ServerApi::watch`] does.
    fn watch_config(&self, config: &Path) -> Result<WatchedChanges, WatchError>;

    /// Serves `site` on threads of its own, as `options` say, until what it returns is dropped.
    fn serve(&self, site: ServedSite, options: &ServeOptions) -> Result<Served, ServeError>;

    /// Stops serving `served`, once the requests being answered are; when ghg serve was asked
    /// to stop, another request meanwhile ends ghg at once.
    fn stop(&self, served: Served);
}

/// The changes [`ServerApi::watch`] hands back, until ghg serve is asked to stop.
pub(crate) type WatchedChanges = Box<dyn Iterator<Item = InputChanges>>;

pub(crate) struct ServerService {
    server: Arc<dyn PreviewServer>,
    watcher: Arc<dyn ChangeWatcher>,
    signals: Arc<dyn StopSignals>,
    waiter: Arc<dyn ServeEventWaiter>,
}

impl ServerService {
    pub(crate) fn new(
        server: Arc<dyn PreviewServer>,
        watcher: Arc<dyn ChangeWatcher>,
        signals: Arc<dyn StopSignals>,
        waiter: Arc<dyn ServeEventWaiter>,
    ) -> Self {
        ServerService {
            server,
            watcher,
            signals,
            waiter,
        }
    }

    fn watched_changes(
        &self,
        watched: &[Watched],
        settings_files: &[PathBuf],
    ) -> Result<WatchedChanges, WatchError> {
        let (tell, waiter) = self.waiter.bursts();
        let watching = self.watcher.watch(watched, tell)?;
        Ok(Box::new(ChangesWaiting {
            watching,
            waiter,
            stops: self.waiter.clone(),
            settings_files: settings_files.to_vec(),
        }))
    }
}

impl ServerApi for ServerService {
    fn listen_for_stops(&self) -> Result<(), ServeError> {
        self.signals
            .listen(self.waiter.stop_teller())
            .map_err(ServeError::Signals)
    }

    fn asked_to_stop(&self) -> bool {
        self.waiter.asked_to_stop()
    }

    fn wait_for_stop(&self) {
        self.waiter.wait_for_stop();
    }

    fn watch(
        &self,
        input: &Path,
        settings_files: &[PathBuf],
    ) -> Result<WatchedChanges, WatchError> {
        let watched: Vec<_> = once(Watched::Folder(input.to_path_buf()))
            .chain(settings_files.iter().cloned().map(Watched::File))
            .collect();
        self.watched_changes(&watched, settings_files)
    }

    fn watch_config(&self, config: &Path) -> Result<WatchedChanges, WatchError> {
        let config = config.to_path_buf();
        self.watched_changes(&[Watched::File(config.clone())], &[config])
    }

    fn serve(&self, site: ServedSite, options: &ServeOptions) -> Result<Served, ServeError> {
        let preview = Arc::new(Preview::new(options.mode, site));
        let host = options.host.as_deref().unwrap_or(DEFAULT_HOST);
        let port = options.port.unwrap_or(DEFAULT_PORT);
        let listening = self
            .server
            .start(host, port, preview.clone(), words())
            .map_err(|kind| ServeError::Listen {
                host: host.to_string(),
                port,
                kind,
            })?;
        Ok(Served { listening, preview })
    }

    fn stop(&self, served: Served) {
        // Before the wait, so that the wait can always be cut short.
        if self.waiter.asked_to_stop() {
            let signals = self.signals.clone();
            self.waiter
                .on_another_stop(Box::new(move || signals.end_at_once()));
        }
        drop(served);
    }
}

/// One watch's changes: every burst waiting, merged, once one comes; nothing more once the
/// watching ends or ghg serve is asked to stop.
struct ChangesWaiting {
    #[expect(dead_code, reason = "it is held only to be dropped")]
    watching: WatchKeeper,
    waiter: Box<dyn ChangeOrStopWaiter>,
    stops: Arc<dyn ServeEventWaiter>,
    settings_files: Vec<PathBuf>,
}

impl Iterator for ChangesWaiting {
    type Item = InputChanges;

    fn next(&mut self) -> Option<InputChanges> {
        if self.stops.asked_to_stop() {
            return None;
        }
        match self.waiter.wait_for_change_or_stop() {
            // A request to stop that came with the bursts wins, so that no rebuild starts once
            // ghg serve is asked to stop.
            ChangeOrStop::Changed(_) if self.stops.asked_to_stop() => None,
            ChangeOrStop::Changed(bursts) => Some(merged_bursts(&bursts, &self.settings_files)),
            ChangeOrStop::Stop | ChangeOrStop::WatchingEnded => None,
        }
    }
}

fn words() -> ServerWords {
    ServerWords {
        not_found: fl!(CATALOGUE, "serve-not-found"),
        wrong_method: fl!(CATALOGUE, "serve-wrong-method"),
    }
}

#[cfg(test)]
mod tests;
