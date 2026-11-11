//! Entity: how a run ends, what stops a build and the status ghg exits with.

mod build_failure;
mod exit_status;

pub(crate) use build_failure::BuildFailure;
pub(crate) use exit_status::ExitStatus;
