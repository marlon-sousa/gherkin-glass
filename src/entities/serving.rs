//! Entity: ghg serve, the site it serves, what it watches, the changes it is told of, the requests to stop it and how it ended.

mod input_changes;
mod preview;
mod serve_end;
mod serve_events;
mod watch;

pub(crate) use input_changes::InputChanges;
pub(crate) use preview::{Listening, Preview, Served, ServedSite, ServerWords, Version};
pub(crate) use serve_end::ServeEnd;
pub(crate) use serve_events::{BurstTeller, ChangeOrStop, StopTeller, WatchKeeper};
pub(crate) use watch::{Burst, Watched};
