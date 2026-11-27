//! Service: the site writer's following of a plan of the store in a safe output folder: the marker first, then the files in their order, then what is removed, and the marker again.

use std::collections::BTreeSet;
use std::io::Error as IoError;
use std::sync::Arc;

use super::{SiteWriter, refusal, write_file, write_files};
use crate::entities::{
    BuildFolder, BuildTarget, FilePath, OutputMarker, SiteAddress, SiteFile, StorePlan,
};
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
            .try_for_each(|path| self.folders.remove(output, path))
            .map_err(|io| cannot_write(folder, target, &io))?;
        self.mark(folder, target, &last_marker)
    }

    /// Writes the marker of `folder`, listing `files`, for the command on `target`.
    fn mark(
        &self,
        folder: &BuildFolder,
        target: &BuildTarget,
        files: &BTreeSet<FilePath>,
    ) -> Result<(), OutputError> {
        let marker = SiteFile {
            address: SiteAddress::at_root(OutputMarker::FILE),
            contents: Arc::from(marker_json(files)),
        };
        write_file(self.folder_files.as_ref(), folder.output(), target, &marker)
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
