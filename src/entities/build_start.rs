//! Entity: where a build starts, from nothing with the settings, or from the last build and the paths that changed since, so that everything changed means starting from nothing.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::entities::{BuildSnapshot, Settings};

#[derive(Debug, Clone)]
pub(crate) enum BuildStart<'a> {
    FromNothing(Arc<Settings>),
    FromLast {
        last: &'a BuildSnapshot,
        paths: &'a BTreeSet<PathBuf>,
    },
}
