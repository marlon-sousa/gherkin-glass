//! Service: collecting the paths of the files to read, folder by folder.

use std::path::{Path, PathBuf};

use crate::entities::{EntryKind, Problem};
use crate::policies::{FileSelection, IGNORE_FILE, parse_rules};
use crate::ports::InputFolder;

/// The files to read under `input`, sorted by path, and the problems met.
pub(super) fn collect(folder: &dyn InputFolder, input: &Path) -> (Vec<PathBuf>, Vec<Problem>) {
    let mut walk = Walk {
        folder,
        input,
        selection: FileSelection::new(input),
        files: Vec::new(),
        problems: Vec::new(),
    };
    walk.visit(input);
    walk.files.sort();
    (walk.files, walk.problems)
}

struct Walk<'a> {
    folder: &'a dyn InputFolder,
    input: &'a Path,
    selection: FileSelection,
    files: Vec<PathBuf>,
    problems: Vec<Problem>,
}

impl Walk<'_> {
    fn visit(&mut self, path: &Path) {
        let entries = match self.folder.entries(path) {
            Ok(entries) => entries,
            Err(error) => {
                self.problem(path, None, format!("cannot be listed: {error}"));
                return;
            }
        };
        let has_rules = entries.iter().any(|entry| {
            entry.kind == EntryKind::File && entry.path.file_name() == Some(IGNORE_FILE.as_ref())
        });
        let entered = has_rules && self.enter_rules(path);
        for entry in entries {
            let is_folder = match entry.kind {
                EntryKind::File => false,
                EntryKind::Folder => true,
                EntryKind::Other => continue,
            };
            if self.selection.is_left_out(&entry.path, is_folder) {
                continue;
            }
            if is_folder {
                self.visit(&entry.path);
            } else {
                self.files.push(entry.path);
            }
        }
        if entered {
            self.selection.leave();
        }
    }

    /// Whether the rules of `path`'s `.ghg-ignore` file were taken in.
    fn enter_rules(&mut self, path: &Path) -> bool {
        let rules_path = path.join(IGNORE_FILE);
        let text = match self.folder.read_text(&rules_path) {
            Ok(text) => text,
            Err(error) => {
                self.problem(&rules_path, None, format!("cannot be read: {error}"));
                return false;
            }
        };
        let parsed = parse_rules(path, &rules_path, &text);
        for (line, message) in parsed.invalid {
            self.problem(&rules_path, line, message);
        }
        self.selection.enter(parsed.rules);
        true
    }

    fn problem(&mut self, path: &Path, line: Option<usize>, message: String) {
        self.problems
            .push(Problem::new(self.input, path, line, message));
    }
}
