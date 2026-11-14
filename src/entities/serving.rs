//! Entity: ghg serve, the site it serves, what it watches, the changes it is told of and the requests to stop it.

mod input_changes;
mod preview;
mod stop_requests;
mod watch;

pub(crate) use input_changes::InputChanges;
pub(crate) use preview::{Listening, Preview, Served, ServedSite, ServerWords, Version};
pub(crate) use stop_requests::StopRequests;
pub(crate) use watch::{Burst, Changes, Watched};
