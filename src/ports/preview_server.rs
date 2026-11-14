//! Port: answering a browser's requests from the site being previewed.

use std::sync::Arc;

use crate::entities::{Listening, Preview, ServerWords};
use crate::errors::ListenErrorKind;

pub(crate) trait PreviewServer: Send + Sync {
    /// Listens at `host` and `port`, and answers every request, on threads
    /// of its own, from the version `preview` holds when the request comes,
    /// until what it returns is dropped. Returns the address it listens at,
    /// with the port chosen for port 0.
    fn start(
        &self,
        host: &str,
        port: u16,
        preview: Arc<Preview>,
        words: ServerWords,
    ) -> Result<Listening, ListenErrorKind>;
}
