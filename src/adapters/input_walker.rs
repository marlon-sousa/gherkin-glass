//! Adapter: the input collector port, walking the input folder on disk with its `.ghg-ignore` files.

use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

use crate::entities::{CollectedInput, Problem};
use crate::policies::{FileSelection, IGNORE_FILE, parse_rules};
use crate::ports::InputCollector;

#[derive(Debug, Default, Clone, Copy)]
pub struct InputWalker;

impl InputCollector for InputWalker {
    fn collect(&self, input: &Path) -> CollectedInput {
        let mut walk = Walk {
            input,
            selection: FileSelection::new(input),
            found: CollectedInput::default(),
        };
        walk.visit(input);
        walk.found.paths.sort();
        walk.found
    }
}

struct Walk<'a> {
    input: &'a Path,
    selection: FileSelection,
    found: CollectedInput,
}

impl Walk<'_> {
    fn visit(&mut self, folder: &Path) {
        let Some(entries) = self.entries(folder) else {
            return;
        };
        let has_rules = entries
            .iter()
            .any(|(path, is_folder)| !is_folder && path.file_name() == Some(IGNORE_FILE.as_ref()));
        let entered = has_rules && self.enter_rules(folder);
        for (path, is_folder) in entries {
            if self.selection.is_left_out(&path, is_folder) {
                continue;
            }
            if is_folder {
                self.visit(&path);
            } else {
                self.found.paths.push(path);
            }
        }
        if entered {
            self.selection.leave();
        }
    }

    /// The files and folders in `folder`, each with whether it is a folder.
    /// Anything else, a symbolic link among them, is left out, so a link is
    /// never followed.
    fn entries(&mut self, folder: &Path) -> Option<Vec<(PathBuf, bool)>> {
        let listing = read_dir(folder).and_then(|listing| {
            listing
                .filter_map(|entry| {
                    entry
                        .and_then(|entry| Ok((entry.path(), entry.file_type()?)))
                        .map(|(path, kind)| {
                            (kind.is_file() || kind.is_dir()).then(|| (path, kind.is_dir()))
                        })
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()
        });
        match listing {
            Ok(entries) => Some(entries),
            Err(error) => {
                self.problem(folder, None, format!("cannot be listed: {error}"));
                None
            }
        }
    }

    /// Whether the rules of `folder`'s `.ghg-ignore` file were taken in.
    fn enter_rules(&mut self, folder: &Path) -> bool {
        let path = folder.join(IGNORE_FILE);
        let text = match read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                self.problem(&path, None, format!("cannot be read: {error}"));
                return false;
            }
        };
        let parsed = parse_rules(folder, &path, &text);
        for (line, message) in parsed.invalid {
            self.problem(&path, line, message);
        }
        self.selection.enter(parsed.rules);
        true
    }

    fn problem(&mut self, path: &Path, line: Option<usize>, message: String) {
        self.found
            .problems
            .push(Problem::new(self.input, path, line, message));
    }
}
