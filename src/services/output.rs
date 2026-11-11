//! Service: writing a build into its output folder, with or without versions, once the folder is safe.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Error as IoError;
use std::iter::once;
use std::path::Path;
use std::sync::Arc;

use crate::entities::{
    BuildFolder, BuildTarget, BuiltSite, FilePath, Record, Settings, SiteAddress, SiteFile,
    VersionName,
};
use crate::errors::{BuildError, OutputError, OutputErrorKind};
use crate::policies::{
    Moment, VERSIONS_PAGE, assets, order_versions, output_safety, render_versions_page,
};
use crate::ports::OutputFolders;
use crate::services::SiteService;

#[derive(Clone)]
pub(crate) struct OutputService {
    folders: Arc<dyn OutputFolders>,
}

impl OutputService {
    pub(crate) fn new(folders: Arc<dyn OutputFolders>) -> Self {
        OutputService { folders }
    }

    /// Builds the site into `settings.output`, as the version its settings
    /// name, or without versions when they name none. The output folder is
    /// checked before the input is read, and again before anything is
    /// written, once the input has been read without an error.
    pub(crate) fn build(
        &self,
        site: &SiteService,
        settings: &Settings,
    ) -> Result<BuiltSite, BuildError> {
        let output = &settings.output;
        let target = match &settings.version {
            Some(version) => BuildTarget::Version(version.clone()),
            None => BuildTarget::Site,
        };
        self.check(
            output,
            &target,
            Moment::Checking {
                replace: settings.replace,
            },
        )?;
        let site_files = site.files(settings)?;
        let folder = self.check(output, &target, Moment::Writing)?;
        let files = paths(&site_files.files);
        match folder.target() {
            BuildTarget::Site => {
                self.clear(&folder)?;
                self.mark(&folder, &Record::Site(files))?;
                site.write(output, site_files.files)?;
            }
            BuildTarget::Version(name) => {
                let versions = versions(&folder, name, &files);
                let page = render_versions_page(
                    &order_versions(versions.keys().cloned()),
                    &settings.pages,
                );
                let page_files: Vec<_> =
                    once(SiteFile::text(SiteAddress::at_root(VERSIONS_PAGE), page))
                        .chain(assets())
                        .collect();
                self.clear_version(&folder, name)?;
                let record = Record::Versions {
                    page: paths(&page_files),
                    versions,
                };
                self.mark(&folder, &record)?;
                site.write(&output.join(name.as_str()), site_files.files)?;
                site.write(output, page_files)?;
            }
        }
        Ok(site_files.built)
    }

    /// The output folder, once nothing stops `target` being written into it
    /// at `moment`, with what its marker records. Nothing is written.
    pub(crate) fn check(
        &self,
        output: &Path,
        target: &BuildTarget,
        moment: Moment,
    ) -> Result<BuildFolder, OutputError> {
        let error = |kind| refusal(output, target, kind);
        let contents = self
            .folders
            .contents(output)
            .map_err(|io| error(cannot_be_read(&io)))?;
        let record = output_safety(contents, target, moment).map_err(error)?;
        Ok(BuildFolder::new(output, target.clone(), record))
    }

    /// Empties the output folder of a site without versions.
    fn clear(&self, folder: &BuildFolder) -> Result<(), OutputError> {
        self.folders
            .clear(folder.output())
            .map_err(|io| cannot_write(folder, None, &io))
    }

    /// Removes the folder of the version `name`, when it is already built,
    /// and the versions page, which are written again.
    fn clear_version(&self, folder: &BuildFolder, name: &VersionName) -> Result<(), OutputError> {
        let output = folder.output();
        self.folders
            .remove(&output.join(name.as_str()))
            .map_err(|io| cannot_write(folder, Some(name), &io))?;
        let Some(Record::Versions { page, .. }) = folder.record() else {
            return Ok(());
        };
        page.iter()
            .try_for_each(|file| self.folders.remove(&output.join(file)))
            .map_err(|io| cannot_write(folder, None, &io))
    }

    /// Writes the marker of the output folder, before the site's files, so
    /// that it lists them even when one of them cannot be written.
    fn mark(&self, folder: &BuildFolder, record: &Record) -> Result<(), OutputError> {
        self.folders
            .record(folder.output(), record)
            .map_err(|io| cannot_write(folder, None, &io))
    }
}

/// The versions the output folder holds once the version `name`, with
/// `files`, is written into it.
fn versions(
    folder: &BuildFolder,
    name: &VersionName,
    files: &BTreeSet<FilePath>,
) -> BTreeMap<VersionName, BTreeSet<FilePath>> {
    let mut versions = match folder.record() {
        Some(Record::Versions { versions, .. }) => versions.clone(),
        _ => BTreeMap::new(),
    };
    versions.insert(name.clone(), files.clone());
    versions
}

/// The path of each of `files`.
fn paths(files: &[SiteFile]) -> BTreeSet<FilePath> {
    files
        .iter()
        .map(|file| file.address.as_str().to_string())
        .collect()
}

/// What stops writing `target` into `output`, said of the version's folder
/// when it is about that version.
fn refusal(output: &Path, target: &BuildTarget, kind: OutputErrorKind) -> OutputError {
    let version = match (target, &kind) {
        (
            BuildTarget::Version(name),
            OutputErrorKind::AlreadyBuilt | OutputErrorKind::OtherCapitals,
        ) => Some(name.clone()),
        _ => None,
    };
    OutputError {
        output: output.to_path_buf(),
        version,
        kind,
    }
}

fn cannot_write(
    folder: &BuildFolder,
    version: Option<&VersionName>,
    error: &IoError,
) -> OutputError {
    OutputError {
        output: folder.output().to_path_buf(),
        version: version.cloned(),
        kind: OutputErrorKind::CannotBeWritten {
            reason: error.to_string(),
        },
    }
}

fn cannot_be_read(error: &IoError) -> OutputErrorKind {
    OutputErrorKind::CannotBeRead {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
