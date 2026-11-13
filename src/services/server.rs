//! Service: the server, serving a site and, under `--watch`, handing back the changes waiting, merged.

use std::iter::{from_fn, once};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use i18n_embed_fl::fl;

use crate::entities::{
    Changes, InputChanges, Preview, ServeOptions, Served, ServedSite, ServerWords, Watched,
};
use crate::errors::{ServeError, WatchError};
use crate::policies::{CATALOGUE, merged_bursts};
use crate::ports::{ChangeWatcher, PreviewServer};

/// The address ghg serve listens on unless `--host` names another.
const DEFAULT_HOST: &str = "127.0.0.1";
/// The port ghg serve listens at unless `--port` names another.
const DEFAULT_PORT: u16 = 8000;

/// What the controller asks of the server.
pub(crate) trait ServerApi: Send + Sync {
    /// Starts watching the folder `input` and `settings_files`, the configuration file and the
    /// front page's template. Each item is every burst of changes waiting, merged into one; it
    /// waits for one, and ends when the watching does. Dropping it stops the watching, and
    /// drops the bursts it still holds.
    fn watch(&self, input: &Path, settings_files: &[PathBuf])
    -> Result<WatchedChanges, WatchError>;

    /// Starts watching the configuration file `config` alone, as [`ServerApi::watch`] does.
    fn watch_config(&self, config: &Path) -> Result<WatchedChanges, WatchError>;

    /// Serves `site` on threads of its own, as `options` say, until ghg serve is stopped.
    fn serve(&self, site: ServedSite, options: &ServeOptions) -> Result<Served, ServeError>;
}

/// The changes [`ServerApi::watch`] hands back.
pub(crate) type WatchedChanges = Box<dyn Iterator<Item = InputChanges>>;

pub(crate) struct ServerService {
    server: Arc<dyn PreviewServer>,
    watcher: Arc<dyn ChangeWatcher>,
}

impl ServerService {
    pub(crate) fn new(server: Arc<dyn PreviewServer>, watcher: Arc<dyn ChangeWatcher>) -> Self {
        ServerService { server, watcher }
    }

    fn watched_changes(
        &self,
        watched: &[Watched],
        settings_files: &[PathBuf],
    ) -> Result<WatchedChanges, WatchError> {
        let changes = self.watcher.watch(watched)?;
        let settings_files = settings_files.to_vec();
        Ok(Box::new(from_fn(move || {
            changes_waiting(&changes, &settings_files)
        })))
    }
}

impl ServerApi for ServerService {
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
        let address = self
            .server
            .start(host, port, preview.clone(), words())
            .map_err(|kind| ServeError::Listen {
                host: host.to_string(),
                port,
                kind,
            })?;
        Ok(Served { address, preview })
    }
}

/// Every burst of `changes` waiting, merged, once one comes; nothing once the watching ends.
fn changes_waiting(changes: &Changes, settings_files: &[PathBuf]) -> Option<InputChanges> {
    let burst = changes.bursts.recv().ok()?;
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
