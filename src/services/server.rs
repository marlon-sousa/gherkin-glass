//! Service: the server, serving a site until it is asked to stop and, under `--watch`, handing back the changes waiting, merged, until then.

use std::iter::{from_fn, once};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use i18n_embed_fl::fl;

use crate::entities::{
    Changes, InputChanges, Preview, ServeOptions, Served, ServedSite, ServerWords, StopRequests,
    Watched,
};
use crate::errors::{ServeError, WatchError};
use crate::policies::{CATALOGUE, merged_bursts};
use crate::ports::{ChangeWatcher, PreviewServer, StopSignals};

/// The address ghg serve listens on unless `--host` names another.
const DEFAULT_HOST: &str = "127.0.0.1";
/// The port ghg serve listens at unless `--port` names another.
const DEFAULT_PORT: u16 = 8000;
/// How long a wait for changes lasts before it looks again whether ghg serve was asked to stop.
const STOP_LOOKED_FOR_EVERY: Duration = Duration::from_millis(100);

/// What the controller asks of the server.
pub(crate) trait ServerApi: Send + Sync {
    /// Starts listening for the requests to stop ghg serve, which from then on no longer end
    /// it at once. Called once in a run, before anything else.
    fn stop_requests(&self) -> Result<StopRequests, ServeError>;

    /// Starts watching the folder `input` and `settings_files`, the configuration file and the
    /// front page's template. Each item is every burst of changes waiting, merged into one; it
    /// waits for one, and ends when the watching does or once `stops` are asked. Dropping it
    /// stops the watching, and drops the bursts it still holds.
    fn watch<'a>(
        &self,
        input: &Path,
        settings_files: &[PathBuf],
        stops: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError>;

    /// Starts watching the configuration file `config` alone, as [`ServerApi::watch`] does.
    fn watch_config<'a>(
        &self,
        config: &Path,
        stops: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError>;

    /// Serves `site` on threads of its own, as `options` say, until what it returns is dropped.
    fn serve(&self, site: ServedSite, options: &ServeOptions) -> Result<Served, ServeError>;
}

/// The changes [`ServerApi::watch`] hands back, until the stop requests they look at are asked.
pub(crate) type WatchedChanges<'a> = Box<dyn Iterator<Item = InputChanges> + 'a>;

pub(crate) struct ServerService {
    server: Arc<dyn PreviewServer>,
    watcher: Arc<dyn ChangeWatcher>,
    signals: Arc<dyn StopSignals>,
}

impl ServerService {
    pub(crate) fn new(
        server: Arc<dyn PreviewServer>,
        watcher: Arc<dyn ChangeWatcher>,
        signals: Arc<dyn StopSignals>,
    ) -> Self {
        ServerService {
            server,
            watcher,
            signals,
        }
    }

    fn watched_changes<'a>(
        &self,
        watched: &[Watched],
        settings_files: &[PathBuf],
        stops: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError> {
        let changes = self.watcher.watch(watched)?;
        let settings_files = settings_files.to_vec();
        Ok(Box::new(from_fn(move || {
            changes_waiting(&changes, &settings_files, stops)
        })))
    }
}

impl ServerApi for ServerService {
    fn stop_requests(&self) -> Result<StopRequests, ServeError> {
        self.signals.listen().map_err(ServeError::Signals)
    }

    fn watch<'a>(
        &self,
        input: &Path,
        settings_files: &[PathBuf],
        stops: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError> {
        let watched: Vec<_> = once(Watched::Folder(input.to_path_buf()))
            .chain(settings_files.iter().cloned().map(Watched::File))
            .collect();
        self.watched_changes(&watched, settings_files, stops)
    }

    fn watch_config<'a>(
        &self,
        config: &Path,
        stops: &'a StopRequests,
    ) -> Result<WatchedChanges<'a>, WatchError> {
        let config = config.to_path_buf();
        self.watched_changes(&[Watched::File(config.clone())], &[config], stops)
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
}

/// Every burst of `changes` waiting, merged, once one comes; nothing once the watching ends or
/// `stops` are asked.
fn changes_waiting(
    changes: &Changes,
    settings_files: &[PathBuf],
    stops: &StopRequests,
) -> Option<InputChanges> {
    let burst = loop {
        if stops.asked() {
            return None;
        }
        match changes.bursts.recv_timeout(STOP_LOOKED_FOR_EVERY) {
            // A burst that came with a request to stop starts no rebuild.
            Ok(_) if stops.asked() => return None,
            Ok(burst) => break burst,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return None,
        }
    };
    let bursts: Vec<_> = once(burst).chain(changes.bursts.try_iter()).collect();
    Some(merged_bursts(&bursts, settings_files))
}

fn words() -> ServerWords {
    ServerWords {
        not_found: fl!(CATALOGUE, "serve-not-found"),
        wrong_method: fl!(CATALOGUE, "serve-wrong-method"),
    }
}

#[cfg(test)]
mod tests;
