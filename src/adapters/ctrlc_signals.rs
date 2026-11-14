//! Adapter: the stop signals port over the operating system's signals, with ctrlc.

use std::sync::mpsc::channel;

use ctrlc::{Error as CtrlcError, set_handler};

use crate::entities::StopRequests;
use crate::errors::SignalsError;
use crate::ports::StopSignals;

/// Control+C, `SIGTERM` and `SIGHUP` on Unix, and Control+C, Control+Break and a closed
/// console on Windows, through ctrlc's `termination` feature.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct CtrlcSignals;

impl StopSignals for CtrlcSignals {
    fn listen(&self) -> Result<StopRequests, SignalsError> {
        let (sender, requests) = channel();
        set_handler(move || {
            // A run that no longer waits for a request needs none.
            let _ = sender.send(());
        })
        .map_err(signals_error)?;
        Ok(StopRequests::new(requests))
    }
}

fn signals_error(error: CtrlcError) -> SignalsError {
    let reason = match error {
        // ctrlc's own words for it hide the operating system's.
        CtrlcError::System(error) => error.to_string(),
        error => error.to_string(),
    };
    SignalsError { reason }
}
