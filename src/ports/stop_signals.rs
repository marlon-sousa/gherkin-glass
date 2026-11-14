//! Port: being told when ghg serve is asked to stop.

use crate::entities::StopTeller;
use crate::errors::SignalsError;

pub(crate) trait StopSignals: Send + Sync {
    /// Starts listening for the signals that ask ghg to stop, Control+C and the operating
    /// system's requests to end, so that they no longer end it at once; each one sent from then
    /// on is told to `tell` as one request. Called once in a run.
    fn listen(&self, tell: StopTeller) -> Result<(), SignalsError>;

    /// Ends ghg at once, with the status of an interrupted command.
    fn end_at_once(&self);
}
