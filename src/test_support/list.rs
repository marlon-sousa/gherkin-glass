//! Test support: the list of `--only` held in memory, and input collectors that read it.

use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::path::Path;
use std::sync::Arc;

use crate::ports::{FileReader, InputFolders, StandardInput};
use crate::services::InputCollector;

/// The list `--only` names, its file and standard input alike: `text`, or, when it is `None`, not
/// found as a file and not text on standard input.
struct FakeList {
    text: Option<String>,
}

impl FileReader for FakeList {
    fn read(&self, _: &Path) -> IoResult<String> {
        self.text
            .clone()
            .ok_or_else(|| IoError::new(ErrorKind::NotFound, "not found"))
    }
}

impl StandardInput for FakeList {
    fn read(&self) -> IoResult<String> {
        self.text
            .clone()
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "not UTF-8"))
    }
}

/// The input collector over `input_folders`, whose list of `--only` holds `list`, or cannot be
/// read when it is `None`.
pub(crate) fn listing_collector(
    input_folders: Arc<dyn InputFolders>,
    list: Option<&str>,
) -> InputCollector {
    let list = Arc::new(FakeList {
        text: list.map(str::to_string),
    });
    InputCollector::new(input_folders, list.clone(), list)
}

/// The input collector over `input_folders`, for a build without `--only`.
pub(crate) fn input_collector(input_folders: Arc<dyn InputFolders>) -> InputCollector {
    listing_collector(input_folders, None)
}
