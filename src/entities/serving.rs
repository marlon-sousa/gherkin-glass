//! Entity: ghg serve, the site it serves, what it watches and the changes it is told of.

mod input_changes;
mod preview;
mod watch;

pub(crate) use input_changes::InputChanges;
pub(crate) use preview::{Preview, Served, ServedSite, ServerWords, Version};
pub(crate) use watch::{Burst, Changes, Watched};
