//! Port: waiting for what ghg serve waits for, a request to stop it or a change.

use crate::entities::{BurstTeller, ChangeOrStop, StopTeller};

pub(crate) trait ServeEventWaiter: Send + Sync {
    /// Where the signals tell of each request to stop ghg serve.
    fn stop_teller(&self) -> StopTeller;

    /// Whether ghg serve was asked to stop by now. Once asked, it stays asked.
    fn asked_to_stop(&self) -> bool;

    /// Waits until ghg serve is asked to stop.
    fn wait_for_stop(&self);

    /// From now on, runs `then` once another request to stop comes; those that came already
    /// do not count.
    fn on_another_stop(&self, then: Box<dyn FnOnce() + Send>);

    /// A new watch's bursts: where the watcher tells of them, and what waits for them or a
    /// request to stop. Bursts told to an earlier watch's teller are never waited for here.
    fn bursts(&self) -> (BurstTeller, Box<dyn ChangeOrStopWaiter>);
}

/// Waits for one watch's bursts or a request to stop, as [`ServeEventWaiter::bursts`] hands it.
pub(crate) trait ChangeOrStopWaiter: Send {
    /// Waits until a burst of changes or a request to stop comes, whichever is first, and hands
    /// back every burst waiting then. When both have come, either may be handed back.
    fn wait_for_change_or_stop(&self) -> ChangeOrStop;
}
