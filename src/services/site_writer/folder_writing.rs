//! Service: the site writer's writes into a safe output folder, of a site without versions or of a version, each marked before its files are written.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Error as IoError;

use super::{SiteWriter, write_files};
use crate::entities::{
    BuildFolder, BuildTarget, FilePath, Record, SiteFiles, StorePlan, VersionName,
};
use crate::errors::{OutputError, OutputErrorKind};

use super::written_paths::{paths, top_of};

impl SiteWriter {
    /// Writes `site` and the store by `plan` into `folder`, as its target says.
    pub(super) fn write_folder(
        &self,
        folder: &BuildFolder,
        site: &SiteFiles,
        plan: StorePlan,
    ) -> Result<(), OutputError> {
        match folder.target() {
            BuildTarget::Site => self.write_site_folder(folder, site, plan),
            BuildTarget::Version(name) => self.write_version(folder, name, site, plan),
        }
    }

    /// Empties the folder, marks it, and writes the site and the store.
    fn write_site_folder(
        &self,
        folder: &BuildFolder,
        site: &SiteFiles,
        plan: StorePlan,
    ) -> Result<(), OutputError> {
        let output = folder.output();
        self.clear(folder)?;
        let written = paths(&site.files).into_iter().chain(plan.files).collect();
        self.mark(folder, &Record::Site(written))?;
        write_files(self.folder_files.as_ref(), output, &site.files)?;
        write_files(self.folder_files.as_ref(), output, &plan.writes)
    }

    /// Removes the version's folder and the versions page, marks the folder with what is
    /// about to be there and what is still to be removed, writes the version, the versions
    /// page and the store, removes what nothing references, and marks the folder again.
    fn write_version(
        &self,
        folder: &BuildFolder,
        name: &VersionName,
        site: &SiteFiles,
        plan: StorePlan,
    ) -> Result<(), OutputError> {
        let output = folder.output();
        self.clear_version(folder, name)?;
        let StorePlan {
            writes,
            removes,
            files: data,
        } = plan;
        let page = paths(&site.root_files);
        let versions = versions(folder, name, &paths(&site.files));
        let record = Record::Versions {
            page: page.clone(),
            versions: versions.clone(),
            data: data.union(&removes).cloned().collect(),
        };
        self.mark(folder, &record)?;
        write_files(
            self.folder_files.as_ref(),
            &output.join(name.as_str()),
            &site.files,
        )?;
        write_files(self.folder_files.as_ref(), output, &site.root_files)?;
        write_files(self.folder_files.as_ref(), output, &writes)?;
        removes
            .iter()
            .try_for_each(|file| self.folders.remove(&output.join(file)))
            .map_err(|io| cannot_write(folder, None, &io))?;
        let record = Record::Versions {
            page,
            versions,
            data,
        };
        self.mark(folder, &record)
    }

    /// Empties the output folder of a site without versions of what ghg
    /// wrote there, which is all it holds but what is kept, and which holds
    /// nothing kept.
    fn clear(&self, folder: &BuildFolder) -> Result<(), OutputError> {
        let Some(Record::Site(files)) = folder.record() else {
            return Ok(());
        };
        let output = folder.output();
        let tops: BTreeSet<_> = files.iter().filter_map(|file| top_of(file)).collect();
        tops.into_iter()
            .try_for_each(|top| self.folders.remove(&output.join(top)))
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

/// Each version the output folder's marker records, with its files.
pub(super) fn recorded_versions(folder: &BuildFolder) -> BTreeMap<VersionName, BTreeSet<FilePath>> {
    match folder.record() {
        Some(Record::Versions { versions, .. }) => versions.clone(),
        _ => BTreeMap::new(),
    }
}

/// The versions the output folder holds once the version `name`, with
/// `files`, is written into it.
fn versions(
    folder: &BuildFolder,
    name: &VersionName,
    files: &BTreeSet<FilePath>,
) -> BTreeMap<VersionName, BTreeSet<FilePath>> {
    let mut versions = recorded_versions(folder);
    versions.insert(name.clone(), files.clone());
    versions
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
