//! Port: being told when ghg serve is asked to stop.

use crate::entities::StopRequests;
use crate::errors::SignalsError;

pub(crate) trait StopSignals: Send + Sync {
    /// Starts listening for the signals that ask ghg to stop, Control+C and the operating
    /// system's requests to end, so that they no longer end it at once; each one sent from then
    /// on is one request. Called once in a run.
    fn listen(&self) -> Result<StopRequests, SignalsError>;

    /// From now on, ends ghg at once, with the status of an interrupted command, when another
    /// request comes in `requests`; those that came already do not count.
    fn end_on_another(&self, requests: StopRequests);
}
