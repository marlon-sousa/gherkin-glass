//! Adapter: the preview server's threads, one taking connections with mio and a few answering them, until what keeps them is dropped.

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, unbounded};
use mio::event::Event;
use mio::{Events, Interest, Poll, Token, Waker};

use super::connection::{PATIENCE, answer_connection};
use crate::entities::{Preview, ServerWords};

/// How many connections are answered at once.
const ANSWERING_THREADS: usize = 8;
/// How many events one wait reports at most; the others wait for the next.
const EVENTS: usize = 64;
const LISTENER: Token = Token(0);
const STOP: Token = Token(1);
const FIRST_CONNECTION: usize = 2;
/// How long to wait before taking connections again once the system refused one, such as
/// when ghg serve has as many files open as it may.
const AFTER_REFUSAL: Duration = Duration::from_millis(100);

/// The thread taking connections, which owns the threads answering them, and the waker that
/// stops it.
pub(super) struct Answering {
    stop: Waker,
    taking: Option<JoinHandle<()>>,
}

impl Answering {
    /// Answers every connection `listener` takes, from the version `preview` holds when its
    /// request comes.
    pub(super) fn start(
        listener: TcpListener,
        preview: Arc<Preview>,
        words: Arc<ServerWords>,
    ) -> io::Result<Self> {
        listener.set_nonblocking(true)?;
        let poll = Poll::new()?;
        let mut listener = mio::net::TcpListener::from_std(listener);
        poll.registry()
            .register(&mut listener, LISTENER, Interest::READABLE)?;
        let stop = Waker::new(poll.registry(), STOP)?;
        let (connections, taken) = unbounded();
        let answering = (0..ANSWERING_THREADS)
            .map(|_| answer_on_a_thread(&taken, &preview, &words))
            .collect::<io::Result<Vec<_>>>()?;
        let taking = Taking {
            poll,
            listener,
            waiting: HashMap::new(),
            next_token: FIRST_CONNECTION,
            connections,
        };
        let taking = Builder::new().spawn(move || {
            taking.run();
            answering.into_iter().for_each(wait_for);
        })?;
        Ok(Answering {
            stop,
            taking: Some(taking),
        })
    }
}

/// Answers the connections `taken` brings on a thread of its own, until it is closed.
fn answer_on_a_thread(
    taken: &Receiver<TcpStream>,
    preview: &Arc<Preview>,
    words: &Arc<ServerWords>,
) -> io::Result<JoinHandle<()>> {
    let (taken, preview, words) = (taken.clone(), preview.clone(), words.clone());
    Builder::new().spawn(move || {
        taken
            .iter()
            .for_each(|stream| answer_connection(stream, &preview, &words));
    })
}

fn wait_for(thread: JoinHandle<()>) {
    // A thread that panicked has nothing left to answer.
    let _ = thread.join();
}

/// What the thread taking connections holds: the listener, and the connections whose
/// request has not started, which it hands to the answering threads once it does, so that
/// a connection a browser opens ahead of need holds no answering thread.
struct Taking {
    poll: Poll,
    listener: mio::net::TcpListener,
    waiting: HashMap<Token, Waiting>,
    next_token: usize,
    connections: Sender<TcpStream>,
}

/// A connection whose request has not started, and when it was taken.
struct Waiting {
    stream: mio::net::TcpStream,
    taken: Instant,
}

impl Waiting {
    /// How long it may still send nothing, at `now`, before it is closed.
    fn patience_left(&self, now: Instant) -> Duration {
        PATIENCE.saturating_sub(now.duration_since(self.taken))
    }
}

impl Taking {
    /// Takes connections until the stop, or until the system cannot wait.
    fn run(mut self) {
        let mut events = Events::with_capacity(EVENTS);
        let mut timeout = None;
        loop {
            match self.poll.poll(&mut events, timeout) {
                Ok(()) => {}
                // A signal, such as the Control+C that stops ghg serve, interrupts the wait.
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return,
            }
            if events.iter().any(is_stop) {
                return;
            }
            for token in events.iter().map(Event::token) {
                self.hand_over(token);
            }
            timeout = [self.take_waiting(), self.close_silent()]
                .into_iter()
                .flatten()
                .min();
        }
    }

    /// Takes every connection waiting, since mio tells of them only once more come after
    /// all were taken. When the system refuses one, it says how long to wait before taking
    /// again.
    fn take_waiting(&mut self) -> Option<Duration> {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => self.hold(stream),
                Err(error) if error.kind() == ErrorKind::WouldBlock => return None,
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::ConnectionAborted | ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Some(AFTER_REFUSAL),
            }
        }
    }

    /// Holds `stream` until its request starts. A connection mio cannot watch is closed.
    fn hold(&mut self, mut stream: mio::net::TcpStream) {
        let token = Token(self.next_token);
        self.next_token += 1;
        if self
            .poll
            .registry()
            .register(&mut stream, token, Interest::READABLE)
            .is_ok()
        {
            let taken = Instant::now();
            self.waiting.insert(token, Waiting { stream, taken });
        }
    }

    /// Closes the connections that sent nothing for `PATIENCE`, and says how long until the
    /// next would have.
    fn close_silent(&mut self) -> Option<Duration> {
        let now = Instant::now();
        self.waiting
            .retain(|_, waiting| !waiting.patience_left(now).is_zero());
        self.waiting
            .values()
            .map(|waiting| waiting.patience_left(now))
            .min()
    }

    /// Hands the connection of `token`, if one waits, to the answering threads.
    fn hand_over(&mut self, token: Token) {
        let Some(Waiting { mut stream, .. }) = self.waiting.remove(&token) else {
            return;
        };
        let _ = self.poll.registry().deregister(&mut stream);
        let stream = TcpStream::from(stream);
        // mio's connections are non-blocking; the answering threads read and write blocking.
        if stream.set_nonblocking(false).is_ok() {
            // The channel closes only once this thread drops its sender.
            let _ = self.connections.send(stream);
        }
    }
}

fn is_stop(event: &Event) -> bool {
    event.token() == STOP
}

impl Drop for Answering {
    fn drop(&mut self) {
        // A waker that cannot wake leaves the threads as they are, for the process to end.
        if self.stop.wake().is_ok()
            && let Some(taking) = self.taking.take()
        {
            wait_for(taking);
        }
    }
}
