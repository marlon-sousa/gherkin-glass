//! The first step of the build: collect the paths of the files to read.
//!
//! docs/BEHAVIOUR.md, "Which files are read". The folders are walked here
//! rather than with the `ignore` crate's `WalkBuilder`, and only the crate's
//! gitignore matcher is used. The walker reads `.gitignore`, `.ignore`, git's
//! exclude files and the ignore files above the input unless each is turned
//! off, and takes its lowest-precedence rules only from a file on disk; a walk
//! of our own reads `.ghg-ignore` files and nothing else, so there is nothing
//! to turn off.

use std::fs::{self, FileType};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::problem::Problem;

/// The name of the files whose rules decide what is read.
const IGNORE_FILE: &str = ".ghg-ignore";

/// The rules that apply when no `.ghg-ignore` rule matches, in gitignore
/// syntax, where the last matching rule wins: leave out everything, but look
/// into every folder and read every `.feature` file, and leave out every
/// hidden file or folder.
const DEFAULT_RULES: [&str; 4] = ["*", "!*/", "!*.feature", ".*"];

/// Collects the files to read under `input`, sorted by path, and the problems
/// met on the way.
pub(crate) fn collect(input: &Path) -> (Vec<PathBuf>, Vec<Problem>) {
    let mut walk = Walk {
        input,
        defaults: default_rules(input),
        rules: Vec::new(),
        files: Vec::new(),
        problems: Vec::new(),
    };
    walk.folder(input);
    walk.files.sort();
    (walk.files, walk.problems)
}

fn default_rules(input: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(input);
    for rule in DEFAULT_RULES {
        builder
            .add_line(None, rule)
            .expect("the default rules are valid");
    }
    builder.build().expect("the default rules are valid")
}

struct Walk<'a> {
    input: &'a Path,
    defaults: Gitignore,
    /// The rules of the `.ghg-ignore` files of the folders being walked,
    /// outermost first.
    rules: Vec<Gitignore>,
    files: Vec<PathBuf>,
    problems: Vec<Problem>,
}

impl Walk<'_> {
    fn folder(&mut self, folder: &Path) {
        let Some(entries) = self.entries(folder) else {
            return;
        };
        let has_rules = entries
            .iter()
            .any(|(path, kind)| kind.is_file() && path.file_name() == Some(IGNORE_FILE.as_ref()));
        let pushed = has_rules && self.read_rules(folder);
        for (path, kind) in entries {
            // A symbolic link is neither a file nor a folder here, so it is
            // never followed. Other kinds of entries are skipped too.
            // A `.ghg-ignore` file is never read as a feature, whatever its rules say.
            let is_rules = path.file_name() == Some(IGNORE_FILE.as_ref());
            if !(kind.is_file() || kind.is_dir())
                || is_rules
                || self.is_left_out(&path, kind.is_dir())
            {
                continue;
            }
            if kind.is_dir() {
                self.folder(&path);
            } else {
                self.files.push(path);
            }
        }
        if pushed {
            self.rules.pop();
        }
    }

    /// The entries of `folder`, or `None` if it cannot be listed.
    fn entries(&mut self, folder: &Path) -> Option<Vec<(PathBuf, FileType)>> {
        let listing = match fs::read_dir(folder) {
            Ok(listing) => listing,
            Err(error) => {
                self.problem(folder, None, format!("cannot be listed: {error}"));
                return None;
            }
        };
        let mut entries = Vec::new();
        for entry in listing {
            match entry.and_then(|entry| Ok((entry.path(), entry.file_type()?))) {
                Ok(entry) => entries.push(entry),
                Err(error) => {
                    self.problem(folder, None, format!("cannot be listed: {error}"));
                    return None;
                }
            }
        }
        Some(entries)
    }

    /// Reads the `.ghg-ignore` file of `folder` and adds its rules, unless it
    /// cannot be read. A rule that is not valid is reported, and the others
    /// still apply, so that the walk does not report more than is wrong.
    /// Returns whether rules were added.
    fn read_rules(&mut self, folder: &Path) -> bool {
        let path = folder.join(IGNORE_FILE);
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                self.problem(&path, None, format!("cannot be read: {error}"));
                return false;
            }
        };
        let mut builder = GitignoreBuilder::new(folder);
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line_number = index + 1;
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    self.problem(&path, Some(line_number), format!("cannot be read: {error}"));
                    return false;
                }
            };
            // A byte order mark at the start is not part of the first rule, as in git.
            let rule = if index == 0 {
                line.trim_start_matches('\u{feff}')
            } else {
                &line
            };
            if let Err(error) = builder.add_line(Some(path.clone()), rule) {
                self.problem(&path, Some(line_number), error.to_string());
            }
        }
        match builder.build() {
            Ok(rules) => {
                self.rules.push(rules);
                true
            }
            Err(error) => {
                self.problem(&path, None, error.to_string());
                false
            }
        }
    }

    /// Whether `path` is left out: the last matching rule of the nearest
    /// `.ghg-ignore` file with a matching rule decides, and the defaults
    /// decide when none matches.
    fn is_left_out(&self, path: &Path, is_dir: bool) -> bool {
        for rules in self.rules.iter().rev() {
            match rules.matched(path, is_dir) {
                Match::Ignore(_) => return true,
                Match::Whitelist(_) => return false,
                Match::None => {}
            }
        }
        self.defaults.matched(path, is_dir).is_ignore()
    }

    fn problem(&mut self, path: &Path, line: Option<usize>, message: String) {
        self.problems
            .push(Problem::new(self.input, path, line, message));
    }
}
