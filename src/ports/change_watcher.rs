//! Port: being told when files change.

use crate::entities::{Changes, WatchFailure, Watched};

pub trait ChangeWatcher: Send + Sync {
    /// Starts watching every one of `watched`, merging each burst of
    /// changes, such as an editor's when it saves, into one message.
    fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchFailure>;
}
