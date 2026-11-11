//! Error: what stops a build writing its output, said of the output folder or of a version's folder.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use i18n_embed_fl::fl;
use thiserror::Error;

use super::located;
use crate::entities::VersionName;
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
    /// The output folder of a build without a version holds something
    /// else, or is a file or a link.
    #[error(fmt = Self::not_a_site)]
    NotASite,
    /// A file, a symbolic link, or a folder with something in it and no marker.
    #[error(fmt = Self::not_a_version)]
    NotAVersion,
    /// A folder whose name differs from the version's only in its capitals.
    #[error(fmt = Self::other_capitals)]
    OtherCapitals,
    /// `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_read)]
    CannotBeRead { reason: String },
    /// `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_written)]
    CannotBeWritten { reason: String },
    /// The output folder's; `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_listed)]
    CannotBeListed { reason: String },
}

impl Display for OutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let output = self.output.to_string_lossy();
        let (folder, message) = match (&self.version, &self.kind) {
            (Some(name), OutputErrorKind::AlreadyBuilt) => (
                format!("{output}/{name}"),
                fl!(CATALOGUE, "version-already-built", name = name.as_str()),
            ),
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

    fn not_a_version(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "version-not-a-version"))
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

    fn cannot_be_listed(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-cannot-be-listed", reason = reason))
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
    #[case::not_a_version(
        Some("v1"),
        OutputErrorKind::NotAVersion,
        "site/out/v1: this is not a version ghg built, so it is left as it is."
    )]
    #[case::other_capitals(
        Some("v1"),
        OutputErrorKind::OtherCapitals,
        "site/out/v1: a folder whose name differs from this one only in its capitals is in its place, so it is left as it is."
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
    #[case::cannot_be_listed(
        None,
        OutputErrorKind::CannotBeListed { reason: "denied".into() },
        "site/out: cannot be listed: denied"
    )]
    fn an_output_error_names_the_output_folder_or_the_version_s(
        #[case] version_name: Option<&str>,
        #[case] kind: OutputErrorKind,
        #[case] expected: &str,
    ) {
        assert_eq!(version(version_name, kind), expected);
    }
}
