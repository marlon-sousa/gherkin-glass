//! Adapter: the stop signals port over the operating system's signals, with ctrlc.

use std::process::exit;
use std::sync::mpsc::channel;
use std::thread::spawn;

use ctrlc::{Error as CtrlcError, set_handler};

use crate::entities::StopRequests;
use crate::errors::SignalsError;
use crate::ports::StopSignals;

/// Control+C, `SIGTERM` and `SIGHUP` on Unix, through ctrlc's `termination` feature, and
/// Control+C and Control+Break on Windows. Windows ends the process as soon as ctrlc 3.5.2
/// has been told of a closed console, before a request is seen.
/// The status a shell gives a command that Control+C ended, 128 and `SIGINT`'s number.
const INTERRUPTED: i32 = 130;

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

    fn end_on_another(&self, requests: StopRequests) {
        spawn(move || {
            if requests.another() {
                exit(INTERRUPTED);
            }
        });
    }
}

fn signals_error(error: CtrlcError) -> SignalsError {
    let reason = match error {
        // ctrlc 3.5.2 words it "Unexpected system error", hiding the operating system's words.
        CtrlcError::System(error) => error.to_string(),
        error => error.to_string(),
    };
    SignalsError { reason }
}
