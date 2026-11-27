//! Service: the site writer, reading the front page's template, having the target's files rendered and planned by the policies of rendering, and writing them into the output folder, once it is safe, by the plan of the store, or into memory; and removing a version or a preview from the output folder by that plan.

mod folder_writing;
mod published;

use std::io::Error as IoError;
use std::path::Path;
use std::sync::Arc;

use bon::Builder;

use crate::entities::{
    BuildFolder, BuildTarget, BuiltSite, Destination, FrontPageTemplate, KeptInOutput, ServedSite,
    Settings, SiteFile, SiteStore,
};
use crate::errors::{BuildError, FrontPageError, FrontPageErrorKind, OutputError, OutputErrorKind};
use crate::policies::{Change, Moment, TargetFiles, describe_site, output_safety, removal_plan};
use crate::ports::{FileReader, FileWriter, OutputFolders};

use published::published_store;

/// What the controller asks of the site writer.
pub(crate) trait SiteWriterApi: Send + Sync {
    /// Checks, before the input is read, that the target of `settings` may be written into the
    /// output folder; a site built in memory needs no check, and `remove` checks the folder
    /// itself.
    fn check(&self, settings: &Settings) -> Result<(), OutputError>;

    /// Reads the front page's template the settings name, if any, has the files of the settings'
    /// target rendered from `store` with `TargetFiles`, and writes them where the settings say:
    /// into the output folder once it is checked again, by the plan of the store, or in memory,
    /// as a site without versions. It returns the site as it is served, and what the build says
    /// of it.
    fn write(
        &self,
        settings: &Settings,
        store: &SiteStore,
    ) -> Result<(ServedSite, BuiltSite), BuildError>;

    /// Removes the target of `settings`, a version or a preview, from the output folder, with
    /// what no descriptor references any more, writing the versions page again from what the
    /// store says it was last written with; `false` when the output folder holds none of that
    /// name, so that nothing is written or removed.
    fn remove(&self, settings: &Settings) -> Result<bool, BuildError>;
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

    /// The output folder, once nothing stops `change` of `target` in it at `moment`, leaving
    /// alone what `kept` keeps, with what its marker lists and its store holds. Nothing is
    /// written.
    fn check_folder(
        &self,
        output: &Path,
        target: &BuildTarget,
        change: Change,
        moment: Moment<'_>,
        kept: &KeptInOutput,
    ) -> Result<BuildFolder, OutputError> {
        let error = |kind| refusal(output, target, kind);
        let contents = self
            .folders
            .contents(output, kept)
            .map_err(|io| error(cannot_be_read(&io)))?;
        let safe = output_safety(contents, target, change, moment, kept).map_err(error)?;
        let held = safe.listed.intersection(&safe.found).cloned().collect();
        let published = published_store(self.folders.as_ref(), output, &held).map_err(error)?;
        Ok(BuildFolder::new(output, safe.listed, safe.found, published))
    }
}

impl SiteWriterApi for SiteWriter {
    fn check(&self, settings: &Settings) -> Result<(), OutputError> {
        if settings.destination == Destination::Memory {
            return Ok(());
        }
        let moment = Moment::Checking {
            replace: settings.replace,
        };
        self.check_folder(
            &settings.output,
            &settings.target,
            Change::Add,
            moment,
            &settings.keep_in_output,
        )
        .map(drop)
    }

    fn write(
        &self,
        settings: &Settings,
        store: &SiteStore,
    ) -> Result<(ServedSite, BuiltSite), BuildError> {
        let template = self.front_page(settings)?;
        let files = TargetFiles::render(settings, store, template.as_ref())?;
        let built = BuiltSite {
            large_site_data: files.large_site_data(),
            ..describe_site(store, &settings.pages)
        };
        let output = &settings.output;
        if settings.destination == Destination::Memory {
            // A site that was not taken, if any, is dropped, so that nothing of it is served.
            self.memory_files.site(output);
            write_files(
                self.memory_files.as_ref(),
                output,
                &BuildTarget::Site,
                &files.in_memory(),
            )?;
            return Ok((self.memory_files.site(output), built));
        }
        let written = files.paths();
        let moment = Moment::Writing { site: &written };
        let folder = self.check_folder(
            output,
            &settings.target,
            Change::Add,
            moment,
            &settings.keep_in_output,
        )?;
        let plan = files.plan(&folder, settings, template.as_ref())?;
        self.follow(&folder, &settings.target, plan)?;
        Ok((self.folder_files.site(output), built))
    }

    fn remove(&self, settings: &Settings) -> Result<bool, BuildError> {
        let target = &settings.target;
        let moment = Moment::Checking { replace: true };
        let folder = self.check_folder(
            &settings.output,
            target,
            Change::Remove,
            moment,
            &settings.keep_in_output,
        )?;
        if !folder.published().descriptors.contains_key(target) {
            return Ok(false);
        }
        let plan = removal_plan(&folder, settings)?;
        self.follow(&folder, &settings.target, plan)?;
        Ok(true)
    }
}

/// Writes `files` into `output`, stopping at the first that cannot be written.
fn write_files(
    writer: &dyn FileWriter,
    output: &Path,
    target: &BuildTarget,
    files: &[SiteFile],
) -> Result<(), OutputError> {
    files
        .iter()
        .try_for_each(|file| write_file(writer, output, target, file))
}

/// Writes `file` into `output` for the command on `target`.
fn write_file(
    writer: &dyn FileWriter,
    output: &Path,
    target: &BuildTarget,
    file: &SiteFile,
) -> Result<(), OutputError> {
    let path = file.address.as_str();
    writer
        .write(output, path, &file.contents)
        .map_err(|io| file_not_written(output, target, path, &io))
}

/// What stops a command on `target` in `output`.
fn refusal(output: &Path, target: &BuildTarget, kind: OutputErrorKind) -> OutputError {
    OutputError {
        output: output.to_path_buf(),
        target: target.clone(),
        kind,
    }
}

fn file_not_written(
    output: &Path,
    target: &BuildTarget,
    file: &str,
    error: &IoError,
) -> OutputError {
    refusal(
        output,
        target,
        OutputErrorKind::FileNotWritten {
            file: file.to_string(),
            reason: error.to_string(),
        },
    )
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
