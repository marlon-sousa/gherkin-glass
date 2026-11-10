//! Port: answering a browser's requests from the site being previewed.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::entities::{ListenFailure, Preview, ServerWords};

pub trait PreviewServer: Send + Sync {
    /// Listens at `host` and `port`, and answers every request, on threads
    /// of its own, from the version `preview` holds when the request comes.
    /// Returns the address it listens at, with the port chosen for port 0.
    fn start(
        &self,
        host: &str,
        port: u16,
        preview: Arc<Preview>,
        words: ServerWords,
    ) -> Result<SocketAddr, ListenFailure>;
}
