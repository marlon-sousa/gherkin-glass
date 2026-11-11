//! Port: being told when files change.

use crate::entities::{Changes, Watched};
use crate::errors::WatchError;

pub(crate) trait ChangeWatcher: Send + Sync {
    /// Starts watching every one of `watched`, merging each burst of
    /// changes, such as an editor's when it saves, into one message that
    /// names the paths it changed. Opening or reading a file is not a
    /// change; which changes matter is for the caller to decide.
    fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchError>;
}
