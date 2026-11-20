//! Entity: the patterns of what a build leaves alone in its output folder, in the gitignore syntax, and what they keep.

use std::path::Path;
use std::sync::Arc;

use derive_more::Constructor;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::{Error as IgnoreError, Match};

use crate::entities::EntryKind;

/// Every pattern, in the order given; the last that matches a path decides.
#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor)]
pub(crate) struct KeptInOutput {
    patterns: Vec<KeptPattern>,
}

/// One pattern, as it was given, with the gitignore rule it is, and the same
/// rule matching letters whatever their case.
#[derive(Debug, Clone)]
pub(crate) struct KeptPattern {
    text: String,
    rule: Arc<Gitignore>,
    rule_whatever_the_case: Arc<Gitignore>,
}

/// How a pattern's letters are matched: as written, as git does, or whatever
/// their case, as a file system that does not tell capitals apart would.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LetterCase {
    AsWritten,
    Whatever,
}

impl KeptPattern {
    /// The pattern `text` is; `Err` with the `ignore` crate's words when it
    /// is not valid in the gitignore syntax.
    pub(crate) fn new(text: &str) -> Result<KeptPattern, String> {
        Ok(KeptPattern {
            text: text.to_string(),
            rule: Arc::new(rule(text, LetterCase::AsWritten)?),
            rule_whatever_the_case: Arc::new(rule(text, LetterCase::Whatever)?),
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    /// The pattern with `true` when it keeps `path`, `false` when it gives
    /// it back to ghg, with `!`, and `None` when it does not match it.
    fn decision(&self, path: &str, is_folder: bool, case: LetterCase) -> Option<(&Self, bool)> {
        let rule = match case {
            LetterCase::AsWritten => &self.rule,
            LetterCase::Whatever => &self.rule_whatever_the_case,
        };
        match rule.matched(Path::new(path), is_folder) {
            Match::Ignore(_) => Some((self, true)),
            Match::Whitelist(_) => Some((self, false)),
            Match::None => None,
        }
    }
}

/// Two patterns are the same when they are written the same, which makes
/// the same rule.
impl PartialEq for KeptPattern {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for KeptPattern {}

impl KeptInOutput {
    /// The pattern that keeps `path`, a path inside the output folder, or a
    /// folder it is in, its letters matched as `case` says, `None` when it is
    /// ghg's to write. What a folder kept holds is kept whatever a later
    /// pattern says, as in gitignore.
    pub(crate) fn keeping(
        &self,
        path: &str,
        kind: EntryKind,
        case: LetterCase,
    ) -> Option<&KeptPattern> {
        folders_of(path)
            .find_map(|folder| self.keeping_itself(folder, true, case))
            .or_else(|| self.keeping_itself(path, kind == EntryKind::Folder, case))
    }

    /// The pattern that keeps `path` itself, the last that matches it.
    fn keeping_itself(
        &self,
        path: &str,
        is_folder: bool,
        case: LetterCase,
    ) -> Option<&KeptPattern> {
        let (pattern, keeps) = self
            .patterns
            .iter()
            .rev()
            .find_map(|pattern| pattern.decision(path, is_folder, case))?;
        keeps.then_some(pattern)
    }
}

/// The folders that hold `path`, from the outermost: `a` and `a/b` for `a/b/c`.
fn folders_of(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/')
        .filter_map(|(end, _)| path.get(..end))
}

/// The rule of the pattern `text`, its letters matched as `case` says.
fn rule(text: &str, case: LetterCase) -> Result<Gitignore, String> {
    let mut builder = GitignoreBuilder::new(".");
    builder
        .case_insensitive(case == LetterCase::Whatever)
        .map_err(glob_reason)?;
    builder.add_line(None, text).map_err(glob_reason)?;
    builder.build().map_err(glob_reason)
}

/// The words of `error`: for a glob, only what is wrong with it, since the
/// pattern is shown beside them.
fn glob_reason(error: IgnoreError) -> String {
    match error {
        IgnoreError::Glob { err, .. } => err,
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_support::kept_in_output;

    // Rule 225.
    #[rstest]
    #[case::a_name_anywhere(&[".nojekyll"], "docs/.nojekyll", EntryKind::File, Some(".nojekyll"))]
    #[case::a_name_at_the_root(&["/CNAME"], "CNAME", EntryKind::File, Some("/CNAME"))]
    #[case::a_rooted_name_below_the_root(&["/CNAME"], "docs/CNAME", EntryKind::File, None)]
    #[case::a_folder_pattern_and_a_file(&["drafts/"], "drafts", EntryKind::File, None)]
    #[case::a_folder_pattern_and_a_folder(&["drafts/"], "drafts", EntryKind::Folder, Some("drafts/"))]
    #[case::what_a_kept_folder_holds(&["drafts/"], "drafts/a/b.txt", EntryKind::File, Some("drafts/"))]
    #[case::a_star(&["*.txt"], "a/notes.txt", EntryKind::File, Some("*.txt"))]
    #[case::the_last_pattern_decides(&["*.txt", "!notes.txt"], "notes.txt", EntryKind::File, None)]
    #[case::a_kept_folder_whatever_a_later_pattern_says(&["drafts", "!drafts/a.txt"], "drafts/a.txt", EntryKind::File, Some("drafts"))]
    #[case::a_comment(&["#CNAME"], "#CNAME", EntryKind::File, None)]
    #[case::no_pattern(&[], "CNAME", EntryKind::File, None)]
    fn a_pattern_keeps_what_it_matches_as_gitignore_does(
        #[case] patterns: &[&str],
        #[case] path: &str,
        #[case] kind: EntryKind,
        #[case] keeping: Option<&str>,
    ) {
        let kept = kept_in_output(patterns);
        let keeping_as_written = kept.keeping(path, kind, LetterCase::AsWritten);
        assert_eq!(keeping_as_written.map(KeptPattern::as_str), keeping);
    }

    #[test]
    fn letters_match_as_written_or_whatever_their_case() {
        let kept = kept_in_output(&["Assets"]);
        let folder = EntryKind::Folder;
        assert!(
            kept.keeping("assets", folder, LetterCase::AsWritten)
                .is_none()
        );
        assert!(
            kept.keeping("assets", folder, LetterCase::Whatever)
                .is_some()
        );
    }

    // Rule 226.
    #[test]
    fn a_pattern_that_is_not_valid_says_what_is_wrong_with_it() {
        let reason = KeptPattern::new("{draft").expect_err("not valid");
        assert!(reason.starts_with("unclosed alternate group"), "{reason}");
    }

    #[test]
    fn patterns_written_the_same_are_the_same() {
        assert_eq!(kept_in_output(&["a", "b"]), kept_in_output(&["a", "b"]));
        assert_ne!(kept_in_output(&["a", "b"]), kept_in_output(&["b", "a"]));
    }
}
