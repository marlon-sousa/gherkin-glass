//! Entity: what an output folder's marker records: the files ghg wrote there, with the store's format and the version of the ghg that last wrote it, or that gherkin-glass 1 marked it.

use std::collections::BTreeSet;

/// A file's path below the folder it is in, folders separated by `/`.
pub(crate) type FilePath = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OutputMarker {
    /// A marker of gherkin-glass 2.
    Store {
        format: u32,
        /// The version of the ghg that last wrote the output folder.
        ghg: String,
        /// Every file ghg wrote there, but the marker.
        files: BTreeSet<FilePath>,
    },
    /// A marker of gherkin-glass 1, which gherkin-glass 2 does not add to.
    GherkinGlassOne,
}

impl OutputMarker {
    /// The marker, at the output folder's root.
    pub(crate) const FILE: &str = "ghg-site.json";
    /// The marker gherkin-glass 1 wrote at the root of an output folder of versions.
    pub(crate) const GHERKIN_GLASS_ONE_VERSIONS: &str = "ghg-versions.json";
}
