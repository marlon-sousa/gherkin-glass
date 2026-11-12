//! Service: the site writer, writing a site into its output folder, with or without versions, once the folder is safe, or into memory.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Error as IoError;
use std::iter::once;
use std::path::Path;
use std::sync::Arc;

use crate::entities::{
    BuildFolder, BuildTarget, Destination, FilePath, Record, ServedSite, Settings, SiteAddress,
    SiteFile, SiteFiles, VersionName,
};
use crate::errors::{OutputError, OutputErrorKind};
use crate::policies::{
    Moment, VERSIONS_PAGE, assets, order_versions, output_safety, render_versions_page,
};
use crate::ports::{FileWriter, OutputFolders};

pub(crate) struct SiteWriter {
    folders: Arc<dyn OutputFolders>,
    folder_files: Arc<dyn FileWriter>,
    memory_files: Arc<dyn FileWriter>,
}

impl SiteWriter {
    /// `folder_files` writes into the output folder, and `memory_files` keeps
    /// the site in memory; the destination of the settings chooses between them.
    pub(crate) fn new(
        folders: Arc<dyn OutputFolders>,
        folder_files: Arc<dyn FileWriter>,
        memory_files: Arc<dyn FileWriter>,
    ) -> Self {
        SiteWriter {
            folders,
            folder_files,
            memory_files,
        }
    }

    /// Checks, before the input is read, that the site may be built into the
    /// output folder; a site built in memory needs no check.
    pub(crate) fn check(&self, settings: &Settings) -> Result<(), OutputError> {
        if settings.destination == Destination::Memory {
            return Ok(());
        }
        let moment = Moment::Checking {
            replace: settings.replace,
        };
        self.check_folder(&settings.output, &target(settings), moment)
            .map(drop)
    }

    /// Writes `site` where the settings say, and returns it as it is served.
    /// Into the output folder, it is written as the version its settings name,
    /// or without versions when they name none, once the folder is checked
    /// again; in memory, as a site without versions.
    pub(crate) fn write(
        &self,
        settings: &Settings,
        site: SiteFiles,
    ) -> Result<ServedSite, OutputError> {
        let output = &settings.output;
        if settings.destination == Destination::Memory {
            // A site that was not taken, if any, is dropped, so that nothing of it is served.
            self.memory_files.site(output);
            write_files(self.memory_files.as_ref(), output, &site.files)?;
            return Ok(self.memory_files.site(output));
        }
        let folder = self.check_folder(output, &target(settings), Moment::Writing)?;
        let files = paths(&site.files);
        match folder.target() {
            BuildTarget::Site => {
                self.clear(&folder)?;
                self.mark(&folder, &Record::Site(files))?;
                write_files(self.folder_files.as_ref(), output, &site.files)?;
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
                write_files(
                    self.folder_files.as_ref(),
                    &output.join(name.as_str()),
                    &site.files,
                )?;
                write_files(self.folder_files.as_ref(), output, &page_files)?;
            }
        }
        Ok(self.folder_files.site(output))
    }

    /// The output folder, once nothing stops `target` being written into it
    /// at `moment`, with what its marker records. Nothing is written.
    fn check_folder(
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

/// What the build writes: the version its settings name, or a site without versions.
fn target(settings: &Settings) -> BuildTarget {
    match &settings.version {
        Some(version) => BuildTarget::Version(version.clone()),
        None => BuildTarget::Site,
    }
}

/// Writes `files` into `output`, stopping at the first that cannot be written.
fn write_files(
    writer: &dyn FileWriter,
    output: &Path,
    files: &[SiteFile],
) -> Result<(), OutputError> {
    files
        .iter()
        .try_for_each(|file| write_file(writer, output, file))
}

fn write_file(writer: &dyn FileWriter, output: &Path, file: &SiteFile) -> Result<(), OutputError> {
    writer
        .write(output, file.address.as_str(), &file.contents)
        .map_err(|io| file_not_written(output, file, &io))
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

fn file_not_written(output: &Path, file: &SiteFile, error: &IoError) -> OutputError {
    OutputError {
        output: output.to_path_buf(),
        version: None,
        kind: OutputErrorKind::FileNotWritten {
            file: file.address.as_str().to_string(),
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
