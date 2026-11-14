//! Entity: what ghg serve waits for, a change or a request to stop, and where it is told of each.

use std::any::Any;

use super::Burst;

/// Where the signals tell of each request to stop ghg serve, such as Control+C, as it comes.
pub(crate) type StopTeller = Box<dyn Fn() + Send>;

/// Where the watcher tells of each burst of changes, as it comes.
pub(crate) type BurstTeller = Box<dyn Fn(Burst) + Send>;

/// Whatever keeps a watch going; dropping it ends the watch.
pub(crate) type Watching = Box<dyn Any + Send>;

/// What a wait for a change or a request to stop ended with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChangeOrStop {
    /// Every burst of changes waiting, in the order they came.
    Changed(Vec<Burst>),
    /// ghg serve was asked to stop; the bursts waiting, if any, are left.
    Stop,
    /// The watch ended, so no change can come.
    WatchingEnded,
}
