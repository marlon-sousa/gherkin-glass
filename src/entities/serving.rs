//! Entity: ghg serve, the site it serves, and what it watches.

mod preview;
mod watch;

pub(crate) use preview::{Preview, Served, ServedSite, ServerWords, Version};
pub(crate) use watch::{Burst, Changes, Watched};
