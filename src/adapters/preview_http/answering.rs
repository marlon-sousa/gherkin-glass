//! Adapter: the preview server's threads answering requests, until what keeps them is dropped.

use std::sync::Arc;
use std::thread::{JoinHandle, spawn};

use tiny_http::Server;

use super::answer;
use crate::entities::{Preview, ServerWords};

/// How many requests are answered at once.
const THREADS: usize = 4;

/// The server and its threads answering. Dropped, it unblocks each thread, then waits for
/// them, so that the requests already taken are answered, and the server closes once the last
/// share of it is dropped.
pub(super) struct Answering {
    server: Arc<Server>,
    threads: Vec<JoinHandle<()>>,
}

impl Answering {
    /// Answers every request `server` takes, from the version `preview` holds when it comes.
    pub(super) fn start(
        server: Arc<Server>,
        preview: Arc<Preview>,
        words: Arc<ServerWords>,
    ) -> Self {
        let threads = (0..THREADS)
            .map(|_| answer_on_a_thread(&server, &preview, &words))
            .collect();
        Answering { server, threads }
    }
}

/// Answers the requests `server` takes on a thread of its own, until it is unblocked.
fn answer_on_a_thread(
    server: &Arc<Server>,
    preview: &Arc<Preview>,
    words: &Arc<ServerWords>,
) -> JoinHandle<()> {
    let (server, preview, words) = (server.clone(), preview.clone(), words.clone());
    spawn(move || {
        // `recv` fails once the thread is unblocked or the server is closed.
        while let Ok(request) = server.recv() {
            answer(request, &preview, &words);
        }
    })
}

impl Drop for Answering {
    fn drop(&mut self) {
        // tiny_http 0.12.0 queues each unblock behind the requests already taken, and each
        // unblock ends one thread's `recv`.
        for _ in &self.threads {
            self.server.unblock();
        }
        for thread in self.threads.drain(..) {
            // A thread that panicked has nothing left to answer.
            let _ = thread.join();
        }
    }
}
