//! Service: the site writer, reading the front page's template, rendering the site tree, keeping from the last site the file of every node that has not changed, rendering the site's store beside it, and writing their files into its output folder, with or without versions, once the folder is safe, by the plan of the store, or into memory.

mod folder_writing;
mod published;
mod written_paths;

use std::collections::BTreeSet;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::Arc;

use bon::Builder;

use crate::entities::{
    BuildFolder, BuildTarget, ConvertedSite, Destination, FrontPageTemplate, KeptInOutput,
    OutputContents, PublishedStore, RenderedSite, ServedSite, Settings, SiteFile, SiteFiles,
    StoreFiles, VersionName,
};
use crate::errors::{BuildError, FrontPageError, FrontPageErrorKind, OutputError, OutputErrorKind};
use crate::policies::{
    DescriptorSettings, Moment, output_safety, plan_store, render_site, render_store, store_paths,
};
use crate::ports::{FileReader, FileWriter, OutputFolders};

use folder_writing::recorded_versions;
use published::published_store;
use written_paths::planned;

/// What the controller asks of the site writer.
pub(crate) trait SiteWriterApi: Send + Sync {
    /// Checks, before the input is read, that the site may be built into the
    /// output folder, and returns the versions it already holds, none when
    /// it holds a site without versions; a site built in memory needs no
    /// check and has no versions.
    fn check(&self, settings: &Settings) -> Result<BTreeSet<VersionName>, OutputError>;

    /// Reads the front page's template the settings name, if any, then
    /// renders every file of `converted`'s site tree, except one whose node is
    /// unchanged since `last`, the site the last build rendered, which it
    /// keeps from there, and the files of its store, and writes them where
    /// the settings say, once every file is rendered. It returns the site as
    /// it is served, and the site tree with the files rendered from it, which
    /// hold what the build has to say of the site. Into the output folder, it
    /// is written as the version its settings name, or without versions when
    /// they name none, once the folder is checked again; in memory, as a site
    /// without versions.
    fn write(
        &self,
        settings: &Settings,
        converted: ConvertedSite,
        last: Option<&RenderedSite>,
    ) -> Result<(ServedSite, RenderedSite), BuildError>;
}

#[derive(Builder)]
pub(crate) struct SiteWriter {
    /// Reads the front page's template.
    file_reader: Arc<dyn FileReader>,
    folders: Arc<dyn OutputFolders>,
    /// Writes into the output folder; the destination of the settings chooses between it and `memory_files`.
    folder_files: Arc<dyn FileWriter>,
    memory_files: Arc<dyn FileWriter>,
}

impl SiteWriter {
    /// The front page's template the settings name, read; `None` when they name none.
    fn front_page(&self, settings: &Settings) -> Result<Option<FrontPageTemplate>, FrontPageError> {
        let Some(file) = &settings.front_page else {
            return Ok(None);
        };
        let text = self
            .file_reader
            .read(file)
            .map_err(|io| unreadable_template(file, &io))?;
        Ok(Some(FrontPageTemplate {
            file: file.clone(),
            text,
        }))
    }

    /// Writes `site` and `store` where the settings say, and returns the site as it is served.
    fn write_site(
        &self,
        settings: &Settings,
        site: &SiteFiles,
        store: StoreFiles,
        front_page: Option<&str>,
    ) -> Result<ServedSite, OutputError> {
        let files = &site.files;
        let output = &settings.output;
        if settings.destination == Destination::Memory {
            let plan = plan_store(
                &PublishedStore::default(),
                store,
                None,
                &settings.pages,
                front_page,
            );
            // A site that was not taken, if any, is dropped, so that nothing of it is served.
            self.memory_files.site(output);
            write_files(self.memory_files.as_ref(), output, files)?;
            write_files(self.memory_files.as_ref(), output, &plan.writes)?;
            return Ok(self.memory_files.site(output));
        }
        let target = target(settings);
        let mut planned = planned(&target, site);
        planned.extend(store_paths(&store));
        let moment = Moment::Writing { site: &planned };
        let folder = self.check_folder(output, &target, moment, &settings.keep_in_output)?;
        let version = match folder.target() {
            BuildTarget::Site => None,
            BuildTarget::Version(name) => Some(name),
        };
        let plan = plan_store(
            folder.published(),
            store,
            version,
            &settings.pages,
            front_page,
        );
        self.write_folder(&folder, site, plan)?;
        Ok(self.folder_files.site(output))
    }

    /// The output folder, once nothing stops `target` being written into it
    /// at `moment`, leaving alone what `kept` keeps, with what its marker
    /// records. Nothing is written.
    fn check_folder(
        &self,
        output: &Path,
        target: &BuildTarget,
        moment: Moment<'_>,
        kept: &KeptInOutput,
    ) -> Result<BuildFolder, OutputError> {
        let error = |kind| refusal(output, target, kind);
        let contents = self
            .folders
            .contents(output, target.left_alone(), kept)
            .map_err(|io| error(cannot_be_read(&io)))?;
        let files = match (&contents, target) {
            (OutputContents::Folder { files, .. }, BuildTarget::Version(_)) => files.clone(),
            _ => BTreeSet::new(),
        };
        let record = output_safety(contents, target, moment, kept).map_err(error)?;
        let published = published_store(self.folders.as_ref(), output, &files).map_err(error)?;
        Ok(BuildFolder::new(output, target.clone(), record, published))
    }
}

impl SiteWriterApi for SiteWriter {
    fn check(&self, settings: &Settings) -> Result<BTreeSet<VersionName>, OutputError> {
        if settings.destination == Destination::Memory {
            return Ok(BTreeSet::new());
        }
        let moment = Moment::Checking {
            replace: settings.replace,
        };
        let folder = self.check_folder(
            &settings.output,
            &target(settings),
            moment,
            &settings.keep_in_output,
        )?;
        Ok(built_versions(&folder))
    }

    fn write(
        &self,
        settings: &Settings,
        converted: ConvertedSite,
        last: Option<&RenderedSite>,
    ) -> Result<(ServedSite, RenderedSite), BuildError> {
        let ConvertedSite { site_tree, store } = converted;
        let front_page = self.front_page(settings)?;
        let files = render_site(&site_tree, last, &settings.pages, front_page.as_ref())?;
        let template = front_page.as_ref().map(|template| template.text.as_str());
        let version = match target(settings) {
            BuildTarget::Version(name) if settings.destination == Destination::Folder => Some(name),
            _ => None,
        };
        let store = render_store(
            &store,
            &DescriptorSettings {
                version: version.as_ref(),
                options: &settings.pages,
                front_page: template,
            },
        );
        let served = self.write_site(settings, &files, store, template)?;
        Ok((
            served,
            RenderedSite {
                tree: site_tree,
                files,
            },
        ))
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

/// The versions the output folder holds, as its marker records them.
fn built_versions(folder: &BuildFolder) -> BTreeSet<VersionName> {
    recorded_versions(folder).into_keys().collect()
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

fn unreadable_template(file: &Path, error: &IoError) -> FrontPageError {
    FrontPageError {
        file: file.to_path_buf(),
        line: None,
        kind: FrontPageErrorKind::CannotBeRead {
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
