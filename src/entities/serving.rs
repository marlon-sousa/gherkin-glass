//! Entity: ghg serve, the site it serves, what it watches, and why it cannot listen.

mod listen_failure;
mod preview;
mod watch;

pub(crate) use listen_failure::ListenFailure;
pub(crate) use preview::{Preview, ServedSite, ServerWords, Version};
pub(crate) use watch::{Burst, Changes, WatchFailure, Watched};
