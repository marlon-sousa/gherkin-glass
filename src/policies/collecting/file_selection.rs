//! Policy: which paths of the input are read.

use std::path::Path;

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::entities::EntryKind;

pub(crate) const IGNORE_FILE: &str = ".ghg-ignore";

/// Gitignore rules, where the last matching rule wins: leave out everything,
/// but look into every folder and read every `.feature` file, and leave out
/// every hidden file or folder.
const DEFAULT_RULES: [&str; 4] = ["*", "!*/", "!*.feature", ".*"];

/// The rules in force in the folder being walked: the defaults, and the
/// `.ghg-ignore` rules of that folder and the folders above it.
pub(crate) struct FileSelection {
    defaults: Gitignore,
    /// Outermost first.
    rules: Vec<Gitignore>,
}

/// A `.ghg-ignore` file's rules, and each rule that is not valid. The valid rules still apply.
pub(crate) struct ParsedRules {
    pub(crate) rules: Gitignore,
    pub(crate) invalid: Vec<InvalidRule>,
}

/// A rule that is not valid, with its line, if the error is in one, and what is wrong with it.
#[derive(Debug)]
pub(crate) struct InvalidRule {
    pub(crate) line: Option<usize>,
    pub(crate) reason: String,
}

impl FileSelection {
    #[expect(clippy::expect_used, reason = "the default rules are valid")]
    pub(crate) fn new(input: &Path) -> Self {
        let mut builder = GitignoreBuilder::new(input);
        for rule in DEFAULT_RULES {
            builder
                .add_line(None, rule)
                .expect("the default rules are valid");
        }
        FileSelection {
            defaults: builder.build().expect("the default rules are valid"),
            rules: Vec::new(),
        }
    }

    /// Takes in the rules of a folder about to be walked, until `leave`.
    pub(crate) fn enter(&mut self, rules: Gitignore) {
        self.rules.push(rules);
    }

    pub(crate) fn leave(&mut self) {
        self.rules.pop();
    }

    /// The last matching rule of the nearest `.ghg-ignore` file with a
    /// matching rule decides; the defaults decide when none matches. A
    /// `.ghg-ignore` file itself is always left out.
    pub(crate) fn is_left_out(&self, path: &Path, kind: EntryKind) -> bool {
        let is_folder = kind == EntryKind::Folder;
        if !is_folder && path.file_name() == Some(IGNORE_FILE.as_ref()) {
            return true;
        }
        for rules in self.rules.iter().rev() {
            match rules.matched(path, is_folder) {
                Match::Ignore(_) => return true,
                Match::Whitelist(_) => return false,
                Match::None => {}
            }
        }
        self.defaults.matched(path, is_folder).is_ignore()
    }
}

/// Parses the text of the `.ghg-ignore` file at `path`, whose rules apply to
/// `folder` and below.
pub(crate) fn parse_rules(folder: &Path, path: &Path, text: &str) -> ParsedRules {
    let mut builder = GitignoreBuilder::new(folder);
    let mut invalid = Vec::new();
    // A byte order mark is not part of the first rule, as in git.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for (index, line) in text.lines().enumerate() {
        if let Err(error) = builder.add_line(Some(path.to_path_buf()), line) {
            invalid.push(InvalidRule {
                line: Some(index + 1),
                reason: error.to_string(),
            });
        }
    }
    let rules = builder.build().unwrap_or_else(|error| {
        invalid.push(InvalidRule {
            line: None,
            reason: error.to_string(),
        });
        Gitignore::empty()
    });
    ParsedRules { rules, invalid }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(rules: &[(&str, &str)]) -> FileSelection {
        let mut selection = FileSelection::new(Path::new("in"));
        for (folder, text) in rules {
            let folder = Path::new(folder);
            let parsed = parse_rules(folder, &folder.join(IGNORE_FILE), text);
            assert!(parsed.invalid.is_empty(), "{:?}", parsed.invalid);
            selection.enter(parsed.rules);
        }
        selection
    }

    #[test]
    fn the_defaults_read_feature_files_and_leave_out_hidden_ones() {
        let defaults = selection(&[]);
        assert!(!defaults.is_left_out(Path::new("in/a.feature"), EntryKind::File));
        assert!(!defaults.is_left_out(Path::new("in/sub"), EntryKind::Folder));
        assert!(defaults.is_left_out(Path::new("in/a.md"), EntryKind::File));
        assert!(defaults.is_left_out(Path::new("in/.a.feature"), EntryKind::File));
        assert!(defaults.is_left_out(Path::new("in/.git"), EntryKind::Folder));
    }

    #[test]
    fn the_nearest_ghg_ignore_with_a_matching_rule_decides() {
        let nested = selection(&[("in", "x.feature\n"), ("in/sub", "!x.feature\n")]);
        assert!(!nested.is_left_out(Path::new("in/sub/x.feature"), EntryKind::File));
        let outer_only = selection(&[("in", "x.feature\n"), ("in/sub", "y.feature\n")]);
        assert!(outer_only.is_left_out(Path::new("in/sub/x.feature"), EntryKind::File));
    }

    #[test]
    fn a_ghg_ignore_file_is_left_out_whatever_the_rules_say() {
        let everything = selection(&[("in", "!.*\n!*\n")]);
        assert!(everything.is_left_out(Path::new("in/.ghg-ignore"), EntryKind::File));
        assert!(!everything.is_left_out(Path::new("in/.hidden.feature"), EntryKind::File));
    }

    #[test]
    fn a_folder_named_like_a_ghg_ignore_file_follows_the_rules() {
        let selection = selection(&[("in", "!.ghg-ignore/\n")]);
        assert!(!selection.is_left_out(Path::new("in/.ghg-ignore"), EntryKind::Folder));
    }

    #[test]
    fn an_invalid_rule_is_reported_and_the_others_still_apply() {
        let parsed = parse_rules(
            Path::new("in"),
            Path::new("in/.ghg-ignore"),
            "\u{feff}a.feature\na{b\n",
        );
        assert_eq!(parsed.invalid.len(), 1);
        assert_eq!(parsed.invalid[0].line, Some(2));
        assert!(
            parsed
                .rules
                .matched(Path::new("in/a.feature"), false)
                .is_ignore()
        );
    }
}
