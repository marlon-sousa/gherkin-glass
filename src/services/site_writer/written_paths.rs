//! Service: the site writer's paths of what a build writes, by their place in the output folder.

use std::collections::BTreeSet;

use crate::entities::{BuildTarget, FilePath, SiteFile, SiteFiles, VersionName};

/// Every file the build of `target` writes, by its path in the output folder.
pub(super) fn planned(target: &BuildTarget, site: &SiteFiles) -> BTreeSet<FilePath> {
    match target {
        BuildTarget::Site => paths(&site.files),
        BuildTarget::Version(name) => site
            .files
            .iter()
            .map(|file| in_version(name, file))
            .chain(paths(&site.root_files))
            .collect(),
    }
}

fn in_version(name: &VersionName, file: &SiteFile) -> FilePath {
    format!("{name}/{}", file.address.as_str())
}

/// The name at the top of the output folder that `file` is, or is in.
pub(super) fn top_of(file: &str) -> Option<&str> {
    file.split('/').next()
}

/// The path of each of `files`.
pub(super) fn paths(files: &[SiteFile]) -> BTreeSet<FilePath> {
    files
        .iter()
        .map(|file| file.address.as_str().to_string())
        .collect()
}
