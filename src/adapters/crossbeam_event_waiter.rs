//! Adapter: the serve event waiter port over crossbeam-channel's channels and its `select!`.

use std::iter::once;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;

use crossbeam_channel::{Receiver, Sender, select, unbounded};

use crate::entities::{Burst, BurstTeller, ChangeOrStop, StopTeller};
use crate::ports::{ChangeOrStopWaiter, ServeEventWaiter};

/// One unbounded channel for the requests to stop, and one for each watch's bursts. Telling of
/// either never blocks, so the signal handler never waits, and the watcher's thread is never
/// held up while a rebuild runs.
#[derive(Debug, Clone)]
pub(crate) struct CrossbeamEventWaiter {
    sender: Sender<()>,
    requests: Receiver<()>,
    asked: Arc<AtomicBool>,
}

impl Default for CrossbeamEventWaiter {
    fn default() -> Self {
        let (sender, requests) = unbounded();
        CrossbeamEventWaiter {
            sender,
            requests,
            asked: Arc::default(),
        }
    }
}

impl ServeEventWaiter for CrossbeamEventWaiter {
    fn stop_teller(&self) -> StopTeller {
        let sender = self.sender.clone();
        // Telling fails only once the waiter is gone, when no request is waited for any more.
        Box::new(move || {
            let _ = sender.send(());
        })
    }

    fn asked_to_stop(&self) -> bool {
        if !self.asked.load(Ordering::SeqCst) && self.requests.try_recv().is_ok() {
            self.asked.store(true, Ordering::SeqCst);
        }
        self.asked.load(Ordering::SeqCst)
    }

    fn wait_for_stop(&self) {
        if !self.asked_to_stop() {
            // The channel never closes, since the waiter holds a sender, so only a request ends
            // the wait.
            let _ = self.requests.recv();
            self.asked.store(true, Ordering::SeqCst);
        }
    }

    fn on_another_stop(&self, then: Box<dyn FnOnce() + Send>) {
        self.leave_those_that_came();
        let requests = self.requests.clone();
        spawn(move || {
            if requests.recv().is_ok() {
                then();
            }
        });
    }

    fn bursts(&self) -> (BurstTeller, Box<dyn ChangeOrStopWaiter>) {
        let (sender, bursts) = unbounded();
        // Telling fails only once the watch's waiter is dropped, when the burst is not needed.
        let tell = Box::new(move |burst| {
            let _ = sender.send(burst);
        });
        let waiter = WatchWaiter {
            stops: self.clone(),
            bursts,
        };
        (tell, Box::new(waiter))
    }
}

impl CrossbeamEventWaiter {
    /// Leaves the requests to stop that came already, so that only those to come count.
    fn leave_those_that_came(&self) {
        self.requests.try_iter().for_each(drop);
    }
}

/// One watch's bursts, waited for together with the requests to stop.
struct WatchWaiter {
    stops: CrossbeamEventWaiter,
    bursts: Receiver<Burst>,
}

impl ChangeOrStopWaiter for WatchWaiter {
    fn wait_for_change_or_stop(&self) -> ChangeOrStop {
        select! {
            recv(self.stops.requests) -> _ => {
                self.stops.asked.store(true, Ordering::SeqCst);
                ChangeOrStop::Stop
            }
            recv(self.bursts) -> burst => match burst {
                Ok(burst) => {
                    let waiting = self.bursts.try_iter();
                    ChangeOrStop::Changed(once(burst).chain(waiting).collect())
                }
                Err(_) => ChangeOrStop::WatchingEnded,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;
    use crate::test_support::changed;

    #[test]
    fn a_request_is_seen_once_and_stays_asked() {
        let waiter = CrossbeamEventWaiter::default();
        assert!(!waiter.asked_to_stop());
        waiter.stop_teller()();
        assert!(waiter.asked_to_stop());
        assert!(waiter.asked_to_stop());
        waiter.wait_for_stop();
    }

    #[test]
    fn the_wait_for_a_stop_ends_once_one_is_told() {
        let waiter = CrossbeamEventWaiter::default();
        let tell = waiter.stop_teller();
        spawn(tell);
        waiter.wait_for_stop();
        assert!(waiter.asked_to_stop());
    }

    #[test]
    fn every_burst_waiting_is_handed_back_in_order() {
        let waiter = CrossbeamEventWaiter::default();
        let (tell, bursts) = waiter.bursts();
        tell(changed("features/b.feature"));
        tell(Burst::Missed);
        assert_eq!(
            bursts.wait_for_change_or_stop(),
            ChangeOrStop::Changed(vec![changed("features/b.feature"), Burst::Missed])
        );
    }

    #[test]
    fn the_wait_ends_on_a_request_while_no_burst_comes() {
        let waiter = CrossbeamEventWaiter::default();
        let (_tell, bursts) = waiter.bursts();
        let stop = waiter.stop_teller();
        spawn(stop);
        assert_eq!(bursts.wait_for_change_or_stop(), ChangeOrStop::Stop);
        assert!(waiter.asked_to_stop());
    }

    #[test]
    fn the_bursts_of_an_earlier_watch_are_not_waited_for() {
        let waiter = CrossbeamEventWaiter::default();
        let (earlier, _earlier_bursts) = waiter.bursts();
        let (tell, bursts) = waiter.bursts();
        earlier(changed("features/a.feature"));
        tell(changed("features/b.feature"));
        assert_eq!(
            bursts.wait_for_change_or_stop(),
            ChangeOrStop::Changed(vec![changed("features/b.feature")])
        );
    }

    #[test]
    fn once_the_teller_is_dropped_the_watching_ends() {
        let waiter = CrossbeamEventWaiter::default();
        let (tell, bursts) = waiter.bursts();
        drop(tell);
        assert_eq!(
            bursts.wait_for_change_or_stop(),
            ChangeOrStop::WatchingEnded
        );
    }

    #[test]
    fn the_requests_that_came_are_left_before_another_is_waited_for() {
        let waiter = CrossbeamEventWaiter::default();
        let stop = waiter.stop_teller();
        stop();
        stop();
        waiter.leave_those_that_came();
        assert!(waiter.requests.is_empty());
    }

    #[test]
    fn another_request_runs_what_was_asked_for() {
        let waiter = CrossbeamEventWaiter::default();
        let stop = waiter.stop_teller();
        stop();
        waiter.wait_for_stop();
        let (ran, runs) = channel();
        waiter.on_another_stop(Box::new(move || ran.send(()).expect("sent")));
        stop();
        runs.recv().expect("ran");
    }
}
