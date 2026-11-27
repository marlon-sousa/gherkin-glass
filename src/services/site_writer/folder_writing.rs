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
    /// Follows `plan` in `folder`, so that a command stopped half way leaves only what the
    /// marker lists.
    pub(super) fn follow(&self, folder: &BuildFolder, plan: StorePlan) -> Result<(), OutputError> {
        let output = folder.output();
        let StorePlan {
            first_marker,
            writes,
            removes,
            last_marker,
        } = plan;
        self.mark(folder, &first_marker)?;
        write_files(self.folder_files.as_ref(), output, &writes)?;
        removes
            .iter()
            .try_for_each(|path| self.folders.remove(output, path))
            .map_err(|io| cannot_write(folder, &io))?;
        self.mark(folder, &last_marker)
    }

    /// Writes the marker of `folder`, listing `files`.
    fn mark(&self, folder: &BuildFolder, files: &BTreeSet<FilePath>) -> Result<(), OutputError> {
        let marker = SiteFile {
            address: SiteAddress::at_root(OutputMarker::FILE),
            contents: Arc::from(marker_json(files)),
        };
        write_file(self.folder_files.as_ref(), folder.output(), &marker)
    }
}

fn cannot_write(folder: &BuildFolder, error: &IoError) -> OutputError {
    refusal(
        folder.output(),
        &BuildTarget::Site,
        OutputErrorKind::CannotBeWritten {
            reason: error.to_string(),
        },
    )
}
