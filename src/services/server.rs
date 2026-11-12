//! Service: the server, serving a site and, under `--watch`, saying which changes call for a rebuild.

mod rebuilds;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use i18n_embed_fl::fl;
use rebuilds::Rebuilds;

use crate::entities::{Burst, Preview, ServeOptions, Served, ServedSite, ServerWords, Watched};
use crate::errors::{ServeError, WatchError};
use crate::policies::CATALOGUE;
use crate::ports::{ChangeWatcher, PreviewServer};

/// The address ghg serve listens on unless `--host` names another.
const DEFAULT_HOST: &str = "127.0.0.1";
/// The port ghg serve listens at unless `--port` names another.
const DEFAULT_PORT: u16 = 8000;

/// What the controller asks of the server.
pub(crate) trait ServerApi: Send + Sync {
    /// Starts watching the folder `input` and the file `config`, and takes
    /// the files of `input` that `selected` gives as those the site is built
    /// from, so that a change made from now on is not missed. Each item is
    /// the bursts of changes, one or more, that call for one rebuild; it waits
    /// for them, and ends when the watching does.
    fn watch<'a>(
        &self,
        input: &Path,
        config: &Path,
        selected: &'a dyn Fn(&Path) -> Vec<PathBuf>,
    ) -> Result<Box<dyn Iterator<Item = Vec<Burst>> + 'a>, WatchError>;

    /// Serves `site` on threads of its own, as `options` say, until ghg serve is stopped.
    fn serve(&self, site: ServedSite, options: &ServeOptions) -> Result<Served, ServeError>;
}

pub(crate) struct ServerService {
    server: Arc<dyn PreviewServer>,
    watcher: Arc<dyn ChangeWatcher>,
}

impl ServerService {
    pub(crate) fn new(server: Arc<dyn PreviewServer>, watcher: Arc<dyn ChangeWatcher>) -> Self {
        ServerService { server, watcher }
    }
}

impl ServerApi for ServerService {
    fn watch<'a>(
        &self,
        input: &Path,
        config: &Path,
        selected: &'a dyn Fn(&Path) -> Vec<PathBuf>,
    ) -> Result<Box<dyn Iterator<Item = Vec<Burst>> + 'a>, WatchError> {
        let watched = [
            Watched::Folder(input.to_path_buf()),
            Watched::File(config.to_path_buf()),
        ];
        let changes = self.watcher.watch(&watched)?;
        Ok(Box::new(Rebuilds::new(changes, input, config, selected)))
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

fn words() -> ServerWords {
    ServerWords {
        not_found: fl!(CATALOGUE, "serve-not-found"),
        wrong_method: fl!(CATALOGUE, "serve-wrong-method"),
    }
}

#[cfg(test)]
mod tests;
