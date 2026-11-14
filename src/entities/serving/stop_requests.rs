//! Entity: the requests to stop ghg serve, such as Control+C, as they come.

use std::cell::Cell;
use std::sync::mpsc::{Receiver, TryRecvError};

/// One message for each time ghg serve is asked to stop. Once asked, it stays asked.
#[derive(Debug)]
pub(crate) struct StopRequests {
    requests: Receiver<()>,
    asked: Cell<bool>,
}

impl StopRequests {
    pub(crate) fn new(requests: Receiver<()>) -> Self {
        StopRequests {
            requests,
            asked: Cell::new(false),
        }
    }

    /// Whether ghg serve was asked to stop by now. A closed channel counts as asked, since
    /// nothing could ask then, and nothing else would end the wait.
    pub(crate) fn asked(&self) -> bool {
        if !self.asked.get() {
            self.asked.set(!matches!(
                self.requests.try_recv(),
                Err(TryRecvError::Empty)
            ));
        }
        self.asked.get()
    }

    /// Waits until ghg serve is asked to stop.
    pub(crate) fn wait(&self) {
        if !self.asked() {
            // Either a request or a closed channel ends the wait, as `asked` says.
            let _ = self.requests.recv();
            self.asked.set(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn a_request_is_seen_once_and_stays_asked() {
        let (sender, requests) = channel();
        let requests = StopRequests::new(requests);
        assert!(!requests.asked());
        sender.send(()).expect("sent");
        assert!(requests.asked());
        assert!(requests.asked());
        requests.wait();
    }

    #[test]
    fn a_closed_channel_counts_as_asked() {
        let (sender, requests) = channel::<()>();
        let requests = StopRequests::new(requests);
        drop(sender);
        requests.wait();
        assert!(requests.asked());
    }
}
