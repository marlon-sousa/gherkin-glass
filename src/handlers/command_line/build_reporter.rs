//! Handler: the trait of a way to report how `ghg build` ended, which `--message-format` picks.

use super::Reply;
use crate::entities::BuildReport;
use crate::errors::BuildError;

/// Words how a build ended, as `ended` says, into what the run prints and the status it exits
/// with. The handler picks the reporter `--message-format` names and calls it without knowing
/// which it is, so that another format is another reporter, not a change to the build.
pub(super) trait BuildReporter {
    fn report(&self, ended: &Result<BuildReport, BuildError>) -> Reply;
}
