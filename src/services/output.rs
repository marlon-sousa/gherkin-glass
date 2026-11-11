//! Service: writing a build into its output folder, with or without versions, and the versions it holds.

use std::io::Error as IoError;
use std::iter::once;
use std::path::Path;
use std::sync::Arc;

use crate::entities::{
    ASSETS, BuildFolder, BuiltSite, FolderState, Marker, Settings, SiteAddress, SiteFile,
    VersionList, VersionName,
};
use crate::errors::{
    BuildError, InputError, InputErrorKind, InputErrors, OutputError, OutputErrorKind,
};
use crate::policies::{
    Moment, VERSIONS_PAGE, assets, folder_verdict, order_versions, render_versions_page,
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
    /// name, or without versions when they name none.
    pub(crate) fn build(
        &self,
        site: &SiteService,
        settings: &Settings,
    ) -> Result<BuiltSite, BuildError> {
        match &settings.version {
            Some(version) => self.build_version(site, settings, version),
            None => self.build_site(site, settings),
        }
    }

    /// Builds a site without versions into `settings.output`. The output
    /// folder is emptied only once the input has been read without an error.
    pub(crate) fn build_site(
        &self,
        site: &SiteService,
        settings: &Settings,
    ) -> Result<BuiltSite, BuildError> {
        let output = &settings.output;
        let folder = self.check_site(output, settings.replace)?;
        let site_files = site.files(settings)?;
        self.make_room(output, &folder)?;
        site.write(output, site_files.files)?;
        Ok(site_files.built)
    }

    /// Builds the version `version` of the site into its folder in
    /// `settings.output`, then writes the versions page. The version's folder
    /// is emptied only once the input has been read without an error.
    pub(crate) fn build_version(
        &self,
        site: &SiteService,
        settings: &Settings,
        version: &VersionName,
    ) -> Result<BuiltSite, BuildError> {
        let output = &settings.output;
        let folder = self.check(output, version, settings.replace)?;
        let site_files = site.files(settings)?;
        self.make_room(output, &folder)?;
        site.write(folder.path(), site_files.files)?;
        let page = render_versions_page(&self.versions(output)?, &settings.pages);
        // What is at the page's place and its assets' is the versions page's own.
        for place in [VERSIONS_PAGE, ASSETS] {
            self.folders.remove(&output.join(place)).map_err(|error| {
                InputErrors::new(vec![InputError::of_site(
                    output,
                    place,
                    InputErrorKind::CannotBeWritten {
                        reason: reason(&error),
                    },
                )])
            })?;
        }
        let root = once(SiteFile::text(SiteAddress::at_root(VERSIONS_PAGE), page)).chain(assets());
        site.write(output, root)?;
        Ok(site_files.built)
    }

    /// The output folder a site without versions is written into, once
    /// nothing stops it. Nothing is written.
    pub(crate) fn check_site(
        &self,
        output: &Path,
        replace: bool,
    ) -> Result<BuildFolder, OutputError> {
        let output_error = |kind| OutputError {
            output: output.to_path_buf(),
            version: None,
            kind,
        };
        let versions = self
            .folders
            .marked(output)
            .map_err(|error| output_error(cannot_be_read(&error)))?;
        if !versions.is_empty() {
            return Err(output_error(OutputErrorKind::HoldsVersions));
        }
        let state = self
            .folders
            .state(output, &Marker::Site)
            .map_err(|error| output_error(cannot_be_read(&error)))?;
        folder_verdict(state, &Marker::Site, Moment::Checking { replace }).map_err(output_error)?;
        Ok(BuildFolder::new(output, Marker::Site))
    }

    /// The folder the version `name` is written into, once nothing stops it.
    /// Nothing is written.
    pub(crate) fn check(
        &self,
        output: &Path,
        name: &VersionName,
        replace: bool,
    ) -> Result<BuildFolder, OutputError> {
        let marker = Marker::Version(name.clone());
        let folder = BuildFolder::new(output, marker.clone());
        let output_error = |version: Option<&VersionName>, kind| OutputError {
            output: output.to_path_buf(),
            version: version.cloned(),
            kind,
        };
        let site = self
            .folders
            .state(output, &Marker::Site)
            .map_err(|error| output_error(None, cannot_be_read(&error)))?;
        if site == FolderState::Marked {
            return Err(output_error(None, OutputErrorKind::HoldsSite));
        }
        let output_error = |kind| output_error(Some(name), kind);
        let state = self
            .folders
            .state(folder.path(), &marker)
            .map_err(|error| output_error(cannot_be_read(&error)))?;
        folder_verdict(state, &marker, Moment::Checking { replace }).map_err(output_error)?;
        Ok(folder)
    }

    /// Empties the folder a check gave, and marks it, ready for the site to
    /// be written into it. `output` is the one given to the check.
    pub(crate) fn make_room(&self, output: &Path, folder: &BuildFolder) -> Result<(), OutputError> {
        let version = match folder.marker() {
            Marker::Site => None,
            Marker::Version(name) => Some(name.clone()),
        };
        let output_error = |kind| OutputError {
            output: output.to_path_buf(),
            version: version.clone(),
            kind,
        };
        // What was checked may have changed while the input was read, and
        // nothing unmarked is ever emptied.
        let state = self
            .folders
            .state(folder.path(), folder.marker())
            .map_err(|error| output_error(cannot_be_read(&error)))?;
        folder_verdict(state, folder.marker(), Moment::Emptying).map_err(output_error)?;
        self.folders
            .clear(folder.path())
            .and_then(|()| self.folders.mark(folder.path(), folder.marker()))
            .map_err(|error| OutputError {
                output: output.to_path_buf(),
                version,
                kind: OutputErrorKind::CannotBeWritten {
                    reason: reason(&error),
                },
            })
    }

    /// Every version `output` holds, in the order of the versions page.
    pub(crate) fn versions(&self, output: &Path) -> Result<VersionList, OutputError> {
        self.folders
            .marked(output)
            .map(order_versions)
            .map_err(|error| OutputError {
                output: output.to_path_buf(),
                version: None,
                kind: OutputErrorKind::CannotBeListed {
                    reason: reason(&error),
                },
            })
    }
}

fn reason(error: &IoError) -> String {
    error.to_string()
}

fn cannot_be_read(error: &IoError) -> OutputErrorKind {
    OutputErrorKind::CannotBeRead {
        reason: reason(error),
    }
}

#[cfg(test)]
mod tests;
