//! Error: what stops a build writing its output, said of the output folder or of a version's folder.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{MAIN_SEPARATOR, PathBuf};

use i18n_embed_fl::fl;
use thiserror::Error;

use super::located;
use crate::entities::{FilePath, VersionName};
use crate::policies::CATALOGUE;

/// Its display is the folder, joined with `/` on every system, then what is wrong.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct OutputError {
    /// The output folder, as the settings give it.
    pub(crate) output: PathBuf,
    /// The version whose folder it is; `None` for the output folder itself.
    pub(crate) version: Option<VersionName>,
    pub(crate) kind: OutputErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum OutputErrorKind {
    /// The site, or the version, is already built there; displayed as the
    /// site's, since the version's name is the error's.
    #[error(fmt = Self::site_already_built)]
    AlreadyBuilt,
    /// The output folder of a build without a version holds versions.
    #[error(fmt = Self::holds_versions)]
    HoldsVersions,
    /// The output folder of a versioned build holds a site without versions.
    #[error(fmt = Self::holds_site)]
    HoldsSite,
    /// The output folder holds something and no marker, or is a file or a link.
    #[error(fmt = Self::not_a_site)]
    NotASite,
    /// Something in a marked output folder that ghg did not write; `found`
    /// is its path in the folder, the outermost when it is a folder.
    #[error(fmt = Self::not_safe)]
    NotSafe { found: FilePath },
    /// A version the marker lists whose folder is missing.
    #[error(fmt = Self::missing_version)]
    MissingVersion { name: VersionName },
    /// A folder whose name differs from the version's only in its capitals.
    #[error(fmt = Self::other_capitals)]
    OtherCapitals,
    /// `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_read)]
    CannotBeRead { reason: String },
    /// `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_written)]
    CannotBeWritten { reason: String },
    /// A file of the site, `file` its `/`-joined path below the folder;
    /// `reason` is the operating system's.
    #[error(fmt = Self::file_not_written)]
    FileNotWritten { file: FilePath, reason: String },
}

impl Display for OutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let output = self.output.to_string_lossy();
        let (folder, message) = match (&self.version, &self.kind) {
            (Some(name), OutputErrorKind::AlreadyBuilt) => (
                format!("{output}/{name}"),
                fl!(CATALOGUE, "version-already-built", name = name.as_str()),
            ),
            (_, OutputErrorKind::NotSafe { found }) => {
                (format!("{output}/{found}"), self.kind.to_string())
            }
            (_, OutputErrorKind::FileNotWritten { file, .. }) => {
                (file_in(&output, file), self.kind.to_string())
            }
            (_, OutputErrorKind::MissingVersion { name }) => {
                (format!("{output}/{name}"), self.kind.to_string())
            }
            (Some(name), kind) => (format!("{output}/{name}"), kind.to_string()),
            (None, kind) => (output.into_owned(), kind.to_string()),
        };
        located(f, &folder, None, &message)
    }
}

impl OutputErrorKind {
    fn site_already_built(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "site-already-built"))
    }

    fn holds_versions(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "site-holds-versions"))
    }

    fn holds_site(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "version-holds-site"))
    }

    fn not_a_site(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "site-not-a-site"))
    }

    fn not_safe(_found: &FilePath, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-not-safe"))
    }

    fn missing_version(_name: &VersionName, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "version-missing"))
    }

    fn other_capitals(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "version-other-capitals"))
    }

    fn cannot_be_read(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-cannot-be-read", reason = reason))
    }

    fn cannot_be_written(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-cannot-be-written", reason = reason))
    }

    fn file_not_written(_file: &FilePath, reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "cannot-be-written", reason = reason))
    }
}

/// The path of `file` in the folder `output`, as it was given, with no
/// separator added when it already ends in one.
fn file_in(output: &str, file: &str) -> String {
    if output.is_empty() || output.ends_with(['/', MAIN_SEPARATOR]) {
        format!("{output}{file}")
    } else {
        format!("{output}/{file}")
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn version(version: Option<&str>, kind: OutputErrorKind) -> String {
        OutputError {
            output: PathBuf::from("site/out"),
            version: version.map(|name| VersionName::new(name).expect("a version name")),
            kind,
        }
        .to_string()
    }

    #[rstest]
    #[case::already_built(
        Some("v1"),
        OutputErrorKind::AlreadyBuilt,
        "site/out/v1: the version v1 is already built, and replace_existing_version is false."
    )]
    #[case::not_safe(
        None,
        OutputErrorKind::NotSafe { found: "docs/notes.txt".into() },
        "site/out/docs/notes.txt: ghg did not write this, so the output folder is left as it is."
    )]
    #[case::missing_version(
        None,
        OutputErrorKind::MissingVersion { name: VersionName::new("v1").expect("a version name") },
        "site/out/v1: the marker lists this version, but its folder is missing, so the output folder is left as it is."
    )]
    #[case::other_capitals(
        Some("v1"),
        OutputErrorKind::OtherCapitals,
        "site/out/v1: the output folder holds a version whose name differs from this one only in its capitals, which some systems would put in the same folder, so it is left as it is."
    )]
    #[case::cannot_be_read(
        Some("v1"),
        OutputErrorKind::CannotBeRead { reason: "denied".into() },
        "site/out/v1: cannot be read: denied"
    )]
    #[case::cannot_be_written(
        Some("v1"),
        OutputErrorKind::CannotBeWritten { reason: "denied".into() },
        "site/out/v1: cannot be written: denied"
    )]
    #[case::a_file_of_the_site(
        None,
        OutputErrorKind::FileNotWritten { file: "sub/b.html".into(), reason: "denied".into() },
        "site/out/sub/b.html: cannot be written: denied"
    )]
    fn an_output_error_names_the_output_folder_or_the_version_s(
        #[case] version_name: Option<&str>,
        #[case] kind: OutputErrorKind,
        #[case] expected: &str,
    ) {
        assert_eq!(version(version_name, kind), expected);
    }
}
