//! Service: the site writer's following of a plan of the store in a safe output folder: the marker first, then the files in their order, then what is removed, and the marker again.

use std::collections::BTreeSet;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::Arc;

use super::{SiteWriter, file_not_written, refusal, write_files};
use crate::entities::{BuildFolder, BuildTarget, FilePath, OutputMarker, StorePlan};
use crate::errors::{OutputError, OutputErrorKind};
use crate::policies::marker_json;

impl SiteWriter {
    /// Follows `plan` of the command on `target` in `folder`, so that a command stopped half way
    /// leaves only what the marker lists.
    pub(super) fn follow(
        &self,
        folder: &BuildFolder,
        target: &BuildTarget,
        plan: StorePlan,
    ) -> Result<(), OutputError> {
        let output = folder.output();
        let StorePlan {
            first_marker,
            writes,
            removes,
            last_marker,
        } = plan;
        self.mark(folder, target, &first_marker)?;
        write_files(self.folder_files.as_ref(), output, target, &writes)?;
        removes
            .iter()
            .try_for_each(|path| self.remove_file(output, target, path))?;
        self.mark(folder, target, &last_marker)
    }

    /// Removes the file or folder at `path` in `output` for the command on `target`.
    fn remove_file(
        &self,
        output: &Path,
        target: &BuildTarget,
        path: &str,
    ) -> Result<(), OutputError> {
        self.folders
            .remove(output, path)
            .map_err(|io| file_not_written(output, target, path, &io))
    }

    /// Writes the marker of `folder`, listing `files`, for the command on `target`.
    fn mark(
        &self,
        folder: &BuildFolder,
        target: &BuildTarget,
        files: &BTreeSet<FilePath>,
    ) -> Result<(), OutputError> {
        let marker = Arc::<[u8]>::from(marker_json(files));
        self.folder_files
            .write(folder.output(), OutputMarker::FILE, &marker)
            .map_err(|io| cannot_write(folder, target, &io))
    }
}

fn cannot_write(folder: &BuildFolder, target: &BuildTarget, error: &IoError) -> OutputError {
    refusal(
        folder.output(),
        target,
        OutputErrorKind::CannotBeWritten {
            reason: error.to_string(),
        },
    )
}
