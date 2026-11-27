//! Error: what stops a command writing its output folder, or removing from it, said of the output folder or of the folder of a version or a preview.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{MAIN_SEPARATOR, PathBuf};

use i18n_embed_fl::fl;
use thiserror::Error;

use super::LocatedError;
use crate::entities::{BuildTarget, FilePath};
use crate::policies::CATALOGUE;

/// Its display is the folder, joined with `/` on every system, then what is wrong.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct OutputError {
    /// The output folder, as the settings give it.
    pub(crate) output: PathBuf,
    /// What the command writes or removes, whose folder an error about it is said of.
    pub(crate) target: BuildTarget,
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
    /// Something ghg writes, or something in a folder ghg writes, that the
    /// pattern `pattern` keeps; `found` is its path in the folder.
    #[error(fmt = Self::kept_clash)]
    KeptClash { found: FilePath, pattern: String },
    /// gherkin-glass 1 marked the output folder.
    #[error(fmt = Self::gherkin_glass_one)]
    GherkinGlassOne,
    /// The store is in a newer format than this ghg's; `ghg` is the version its marker names.
    #[error(fmt = Self::newer_store)]
    NewerStore { ghg: String },
    /// A version the store holds whose folder is missing; `name` is its name.
    #[error(fmt = Self::missing_version)]
    MissingVersion { name: String },
    /// A version or a preview whose name differs from one the store holds only in its
    /// capitals; displayed as a version's, since the target is the error's.
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

impl OutputError {
    /// The error as its line of the report, said of the output folder, or of the folder or file
    /// in it that it concerns.
    pub(crate) fn problem(&self) -> LocatedError {
        let output = self.output.to_string_lossy();
        let target = (self.target.folder(), &self.target);
        let (path, message) = match (target, &self.kind) {
            ((Some(folder), BuildTarget::Version(name)), OutputErrorKind::AlreadyBuilt) => (
                format!("{output}/{folder}"),
                fl!(CATALOGUE, "version-already-built", name = name.as_str()),
            ),
            ((Some(folder), BuildTarget::Preview(_)), OutputErrorKind::OtherCapitals) => (
                format!("{output}/{folder}"),
                fl!(CATALOGUE, "preview-other-capitals"),
            ),
            ((Some(folder), _), OutputErrorKind::OtherCapitals) => {
                (format!("{output}/{folder}"), self.kind.to_string())
            }
            (
                _,
                OutputErrorKind::NotSafe { found }
                | OutputErrorKind::KeptClash { found, .. }
                | OutputErrorKind::MissingVersion { name: found },
            ) => (format!("{output}/{found}"), self.kind.to_string()),
            (_, OutputErrorKind::FileNotWritten { file, .. }) => {
                (file_in(&output, file), self.kind.to_string())
            }
            (_, kind) => (output.into_owned(), kind.to_string()),
        };
        LocatedError {
            path,
            line: None,
            message,
        }
    }
}

impl Display for OutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.problem().fmt(f)
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

    fn kept_clash(_found: &FilePath, pattern: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-kept-clash", pattern = pattern))
    }

    fn gherkin_glass_one(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-gherkin-glass-one"))
    }

    fn newer_store(ghg: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "output-newer-store", ghg = ghg))
    }

    fn missing_version(_name: &str, f: &mut Formatter<'_>) -> FmtResult {
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
    use crate::test_support::name;

    fn said(target: BuildTarget, kind: OutputErrorKind) -> String {
        OutputError {
            output: PathBuf::from("site/out"),
            target,
            kind,
        }
        .to_string()
    }

    #[rstest]
    #[case::already_built(
        BuildTarget::Version(name("v1")),
        OutputErrorKind::AlreadyBuilt,
        "site/out/v1: the version v1 is already built, and replace_existing_version is false."
    )]
    #[case::site_already_built(
        BuildTarget::Site,
        OutputErrorKind::AlreadyBuilt,
        "site/out: a site is already built here, and replace_existing_version is false."
    )]
    #[case::not_safe(
        BuildTarget::Site,
        OutputErrorKind::NotSafe { found: "docs/notes.txt".into() },
        "site/out/docs/notes.txt: ghg did not write this, so the output folder is left as it is."
    )]
    #[case::kept_clash(
        BuildTarget::Site,
        OutputErrorKind::KeptClash { found: "assets".into(), pattern: "assets".into() },
        "site/out/assets: the pattern assets keeps this, but ghg writes it or the folder it is in, so the output folder is left as it is."
    )]
    // Rule 122.
    #[case::missing_version(
        BuildTarget::Version(name("v2")),
        OutputErrorKind::MissingVersion { name: "v1".into() },
        "site/out/v1: the site's data holds this version, but its folder is missing, so the output folder is left as it is."
    )]
    // Rule 124.
    #[case::other_capitals(
        BuildTarget::Version(name("v1")),
        OutputErrorKind::OtherCapitals,
        "site/out/v1: the output folder holds a version whose name differs from this one only in its capitals, which some systems would put in the same folder, so it is left as it is."
    )]
    #[case::a_preview_s_other_capitals(
        BuildTarget::Preview(name("pr-1")),
        OutputErrorKind::OtherCapitals,
        "site/out/previews/pr-1: the output folder holds a preview whose name differs from this one only in its capitals, which some systems would put in the same folder, so it is left as it is."
    )]
    // Rule 256.
    #[case::gherkin_glass_one(
        BuildTarget::Version(name("v1")),
        OutputErrorKind::GherkinGlassOne,
        "site/out: a site of gherkin-glass 1 is built here, which gherkin-glass 2 cannot add to, so it is left as it is. Build into an empty or new folder."
    )]
    // Rule 246.
    #[case::newer_store(
        BuildTarget::Site,
        OutputErrorKind::NewerStore { ghg: "3.0.0".into() },
        "site/out: the site's data was written by ghg 3.0.0, in a newer format than this ghg reads, so it is left as it is. Use ghg 3.0.0 or later."
    )]
    #[case::cannot_be_read(
        BuildTarget::Version(name("v1")),
        OutputErrorKind::CannotBeRead { reason: "denied".into() },
        "site/out: cannot be read: denied"
    )]
    #[case::cannot_be_written(
        BuildTarget::Version(name("v1")),
        OutputErrorKind::CannotBeWritten { reason: "denied".into() },
        "site/out: cannot be written: denied"
    )]
    #[case::a_file_of_the_site(
        BuildTarget::Site,
        OutputErrorKind::FileNotWritten { file: "data/site.json".into(), reason: "denied".into() },
        "site/out/data/site.json: cannot be written: denied"
    )]
    fn an_output_error_names_the_output_folder_or_its_target_s(
        #[case] target: BuildTarget,
        #[case] kind: OutputErrorKind,
        #[case] expected: &str,
    ) {
        assert_eq!(said(target, kind), expected);
    }
}
