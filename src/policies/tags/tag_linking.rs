//! Policy: the links from tags to other sites.

use std::cell::RefCell;

use regex_automata::meta::Regex;
use regex_automata::util::interpolate;
use regex_automata::{Input, PatternID};
use regex_syntax::hir::{Hir, Look};
use regex_syntax::{Error as SyntaxError, ParserBuilder};

use crate::entities::{OutsideLink, TagLink, percent_encoded};

/// Why a pattern cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternProblem {
    /// The `regex-syntax` crate's short description, such as `unclosed group`.
    Syntax(String),
    /// Over the size limit the `regex` crate sets too.
    TooLarge,
}

/// `source`, in the `regex` crate's syntax, made to match a whole tag only.
pub(crate) fn whole_tag_pattern(source: &str) -> Result<Regex, PatternProblem> {
    let hir = ParserBuilder::new()
        .build()
        .parse(source)
        .map_err(|error| match error {
            SyntaxError::Parse(error) => error.kind().to_string(),
            SyntaxError::Translate(error) => error.kind().to_string(),
            other => other.to_string(),
        })
        .map_err(PatternProblem::Syntax)?;
    // Anchoring the parsed expression, rather than its text, keeps a
    // trailing comment of the `x` flag from swallowing the anchor.
    let whole = Hir::concat(vec![Hir::look(Look::Start), hir, Hir::look(Look::End)]);
    Regex::builder()
        .build_from_hir(&whole)
        .map_err(|error| match error.size_limit() {
            Some(_) => PatternProblem::TooLarge,
            None => PatternProblem::Syntax(error.to_string()),
        })
}

/// The first group `replacement` names that `pattern` does not have, as
/// the replacement writes it.
pub(crate) fn unknown_group(replacement: &str, pattern: &Regex) -> Option<String> {
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

/// The link of the first of `links` whose pattern matches `tag`, written with its `@`.
pub(crate) fn outside_link(tag: &str, links: &[TagLink]) -> Option<OutsideLink> {
    links.iter().find_map(|link| {
        let mut captures = link.pattern.create_captures();
        link.pattern
            .search_captures(&Input::new(tag), &mut captures);
        if !captures.is_match() {
            return None;
        }
        let names = link.pattern.group_info();
        let fill = |template: &str, encode: bool| {
            let mut filled = String::new();
            interpolate::string(
                template,
                |index, out| {
                    let text = captures.get_group(index).map_or("", |span| &tag[span]);
                    if encode {
                        out.push_str(&percent_encoded(text));
                    } else {
                        out.push_str(text);
                    }
                },
                |name| names.to_index(PatternID::ZERO, name),
                &mut filled,
            );
            filled
        };
        let address = fill(&link.address, true);
        let text = fill(&link.text, false);
        // A link with no text, or nowhere to go, would be heard as nothing.
        (!address.is_empty() && !text.trim().is_empty()).then_some(OutsideLink { address, text })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(pattern: &str, address: &str, text: &str) -> TagLink {
        TagLink {
            source: pattern.into(),
            pattern: whole_tag_pattern(pattern).expect("a regular expression"),
            address: address.into(),
            text: text.into(),
        }
    }

    #[test]
    fn the_first_pattern_that_matches_the_whole_tag_gives_the_link() {
        let links = [
            link("@issue-(\\d+)", "https://x/issues/$1", "Issue $1 on GitHub"),
            link("@issue-(?<n>.+)", "https://y/${n}", "Other $n"),
        ];
        assert_eq!(
            outside_link("@issue-12", &links),
            Some(OutsideLink {
                address: "https://x/issues/12".into(),
                text: "Issue 12 on GitHub".into(),
            })
        );
        assert_eq!(
            outside_link("@issue-1 2#ç", &links),
            Some(OutsideLink {
                address: "https://y/1%202%23%C3%A7".into(),
                text: "Other 1 2#ç".into(),
            })
        );
        assert_eq!(outside_link("@my-issue-12", &links[..1]), None);
        assert_eq!(outside_link("@issue-12a", &links[..1]), None);
    }

    #[test]
    fn a_link_whose_text_or_address_comes_out_empty_is_not_shown() {
        let links = [
            link("@x(.*)", "https://x/$1", "$1"),
            link("@y(.*)", "$1", "Y $1"),
        ];
        assert_eq!(outside_link("@x", &links), None);
        assert!(outside_link("@xa", &links).is_some());
        assert_eq!(outside_link("@y", &links), None);
    }

    #[test]
    fn a_whole_match_is_found_even_when_a_shorter_alternative_comes_first() {
        let links = [link("@a|@ab", "x", "$0 $$ $ end $")];
        assert_eq!(
            outside_link("@ab", &links).expect("a match").text,
            "@ab $ $ end $"
        );
    }

    #[test]
    fn a_trailing_comment_does_not_undo_the_whole_match() {
        let links = [link("(?x)@a # a comment", "x", "y")];
        assert!(outside_link("@a", &links).is_some());
        assert!(outside_link("@ab", &links).is_none());
    }

    #[test]
    fn a_pattern_that_is_not_a_regular_expression_has_the_regex_crate_s_reason() {
        assert_eq!(
            whole_tag_pattern("@issue-(\\d+").err(),
            Some(PatternProblem::Syntax("unclosed group".into()))
        );
        assert_eq!(
            whole_tag_pattern("@(a{1000}){1000}").err(),
            Some(PatternProblem::TooLarge)
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
