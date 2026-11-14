//! Port: being told when files change.

use crate::entities::{BurstTeller, Watched, Watching};
use crate::errors::WatchError;

pub(crate) trait ChangeWatcher: Send + Sync {
    /// Starts watching every one of `watched`, telling `tell` of each burst of changes, such
    /// as an editor's when it saves, merged into one that names the paths it changed, until
    /// what it returns is dropped. Opening or reading a file is not a change; which changes
    /// matter is for the caller to decide.
    fn watch(&self, watched: &[Watched], tell: BurstTeller) -> Result<Watching, WatchError>;
}
