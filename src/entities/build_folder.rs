//! Entity: a folder that nothing stops a build from emptying and writing into.

use std::path::{Path, PathBuf};

use crate::entities::Marker;

/// Outside the crate, made only by [`crate::services::OutputService`]'s checks, so
/// that no other folder is ever emptied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildFolder {
    path: PathBuf,
    marker: Marker,
}

impl BuildFolder {
    /// The folder of `marker` in `output`: `output` itself for a site, or
    /// the version's folder inside it.
    pub(crate) fn new(output: &Path, marker: Marker) -> Self {
        let path = match &marker {
            Marker::Site => output.to_path_buf(),
            Marker::Version(name) => output.join(name.as_str()),
        };
        BuildFolder { path, marker }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn marker(&self) -> &Marker {
        &self.marker
    }
}
