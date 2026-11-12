//! Policy: the patterns of the configuration's `tag_links`, each made to match a whole tag, and the groups their replacements name.

use std::cell::RefCell;

use regex_automata::PatternID;
use regex_automata::meta::Regex;
use regex_automata::util::interpolate;
use regex_syntax::hir::{Hir, Look};
use regex_syntax::{Error as SyntaxError, ParserBuilder};

use crate::errors::PatternErrorKind;

/// `source`, in the `regex` crate's syntax, made to match a whole tag only.
pub(super) fn whole_tag_pattern(source: &str) -> Result<Regex, PatternErrorKind> {
    let hir = ParserBuilder::new()
        .build()
        .parse(source)
        .map_err(|error| match error {
            SyntaxError::Parse(error) => error.kind().to_string(),
            SyntaxError::Translate(error) => error.kind().to_string(),
            other => other.to_string(),
        })
        .map_err(PatternErrorKind::Syntax)?;
    // Anchoring the parsed expression, rather than its text, keeps a
    // trailing comment of the `x` flag from swallowing the anchor.
    let whole = Hir::concat(vec![Hir::look(Look::Start), hir, Hir::look(Look::End)]);
    Regex::builder()
        .build_from_hir(&whole)
        .map_err(|error| match error.size_limit() {
            Some(_) => PatternErrorKind::TooLarge,
            None => PatternErrorKind::Syntax(error.to_string()),
        })
}

/// The first group `replacement` names that `pattern` does not have, as
/// the replacement writes it.
pub(super) fn unknown_group(replacement: &str, pattern: &Regex) -> Option<String> {
    let groups = pattern.group_info();
    let count = groups.group_len(PatternID::ZERO);
    let unknown = RefCell::new(None);
    let note = |group: String| {
        unknown.borrow_mut().get_or_insert(group);
    };
    interpolate::string(
        replacement,
        |index, _| {
            if index >= count {
                note(index.to_string());
            }
        },
        |name| {
            let index = groups.to_index(PatternID::ZERO, name);
            if index.is_none() {
                note(name.to_string());
            }
            index
        },
        &mut String::new(),
    );
    unknown.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trailing_comment_does_not_undo_the_whole_match() {
        let pattern = whole_tag_pattern("(?x)@a # a comment").expect("valid");
        assert!(pattern.is_match("@a"));
        assert!(!pattern.is_match("@ab"));
    }

    #[test]
    fn a_pattern_that_is_not_a_regular_expression_has_the_regex_crate_s_reason() {
        assert_eq!(
            whole_tag_pattern("@issue-(\\d+").err(),
            Some(PatternErrorKind::Syntax("unclosed group".into()))
        );
        assert_eq!(
            whole_tag_pattern("@(a{1000}){1000}").err(),
            Some(PatternErrorKind::TooLarge)
        );
    }

    #[test]
    fn a_group_the_pattern_lacks_is_found() {
        let pattern = whole_tag_pattern("@(?<kind>[a-z]+)-(\\d+)").expect("valid");
        assert_eq!(unknown_group("$0 $1 ${2} $kind $$3 $ end", &pattern), None);
        assert_eq!(unknown_group("$3", &pattern).as_deref(), Some("3"));
        assert_eq!(unknown_group("$1a", &pattern).as_deref(), Some("1a"));
        assert_eq!(
            unknown_group("${other}", &pattern).as_deref(),
            Some("other")
        );
    }
}
