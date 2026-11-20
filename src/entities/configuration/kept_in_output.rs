//! Entity: the patterns of what a build leaves alone in its output folder, in the gitignore syntax, and what they keep.

use std::path::Path;
use std::sync::Arc;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::{Error as IgnoreError, Match};

use crate::entities::EntryKind;

/// Every pattern, in the order given; the last that matches a path decides.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct KeptInOutput {
    patterns: Vec<KeptPattern>,
}

/// One pattern, as it was given, with the gitignore rule it is.
#[derive(Debug, Clone)]
pub(crate) struct KeptPattern {
    text: String,
    rule: Arc<Gitignore>,
}

impl KeptPattern {
    /// The pattern `text` is; `Err` with the `ignore` crate's words when it
    /// is not valid in the gitignore syntax.
    pub(crate) fn new(text: &str) -> Result<KeptPattern, String> {
        let mut builder = GitignoreBuilder::new(".");
        builder.add_line(None, text).map_err(glob_reason)?;
        let rule = builder.build().map_err(glob_reason)?;
        Ok(KeptPattern {
            text: text.to_string(),
            rule: Arc::new(rule),
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    /// The pattern with `true` when it keeps `path`, `false` when it gives
    /// it back to ghg, with `!`, and `None` when it does not match it.
    fn decision(&self, path: &str, is_folder: bool) -> Option<(&Self, bool)> {
        match self.rule.matched(Path::new(path), is_folder) {
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
    pub(crate) fn new(patterns: Vec<KeptPattern>) -> Self {
        KeptInOutput { patterns }
    }

    /// The pattern that keeps `path`, a path inside the output folder, or a
    /// folder it is in, `None` when it is ghg's to write. What a folder
    /// kept holds is kept whatever a later pattern says, as in gitignore.
    pub(crate) fn keeping(&self, path: &str, kind: EntryKind) -> Option<&KeptPattern> {
        folders_of(path)
            .find_map(|folder| self.keeping_itself(folder, true))
            .or_else(|| self.keeping_itself(path, kind == EntryKind::Folder))
    }

    /// The pattern that keeps `path` itself, the last that matches it.
    fn keeping_itself(&self, path: &str, is_folder: bool) -> Option<&KeptPattern> {
        let (pattern, keeps) = self
            .patterns
            .iter()
            .rev()
            .find_map(|pattern| pattern.decision(path, is_folder))?;
        keeps.then_some(pattern)
    }
}

/// The folders that hold `path`, from the outermost: `a` and `a/b` for `a/b/c`.
fn folders_of(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/')
        .filter_map(|(end, _)| path.get(..end))
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

    fn kept_in_output(patterns: &[&str]) -> KeptInOutput {
        KeptInOutput::new(
            patterns
                .iter()
                .map(|text| KeptPattern::new(text).expect("a valid pattern"))
                .collect(),
        )
    }

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
        assert_eq!(kept.keeping(path, kind).map(KeptPattern::as_str), keeping);
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
