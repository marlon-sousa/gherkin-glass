//! Entity: what the server makes of the bursts of changes waiting under `--watch`, merged into one.

use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InputChanges {
    /// The configuration file changed, or changes may have been missed, so a build reads
    /// everything again.
    Everything,
    /// The paths the bursts named.
    Paths(BTreeSet<PathBuf>),
}
