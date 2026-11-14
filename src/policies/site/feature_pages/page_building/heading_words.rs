//! Policy: the words of a block's heading, in parts, from a message of the catalogue with feature text in it.

use i18n_embed_fl::fl;

use crate::entities::{HeadingPart, plain_parts};
use crate::policies::CATALOGUE;

/// Stand for feature text in a heading's words, to find where it goes; no
/// message of the catalogue holds them.
pub(super) const TEXT_MARKS: [&str; 2] = ["\u{1}", "\u{2}"];

/// The kinds of block a heading names in gherkin-glass's own words, whatever
/// keyword the file writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Feature,
    Background,
    Rule,
    Scenario,
    /// A scenario with examples, whatever its keyword.
    ScenarioOutline,
    Examples,
}

/// The heading of a block of `kind`: its kind, then `: ` and its name, or its
/// kind alone, then the keyword as written when it is another word than the
/// kind's, other than in its case; the name and the keyword are isolated.
pub(super) fn heading(kind: Kind, keyword: &str, name: &str) -> Vec<HeadingPart> {
    let id = kind.id();
    let keyword = keyword.trim();
    let name = name.trim();
    let kind_words = fl!(CATALOGUE, "unnamed-heading", kind = id);
    let named = if name.is_empty() {
        plain_parts(&kind_words)
    } else {
        let words = fl!(CATALOGUE, "named-heading", kind = id, name = TEXT_MARKS[0]);
        worded_parts(&words, &[isolated(name)])
    };
    if keyword.to_lowercase() == kind_words.to_lowercase() {
        return named;
    }
    let words = fl!(
        CATALOGUE,
        "written-as",
        heading = TEXT_MARKS[0],
        keyword = TEXT_MARKS[1]
    );
    worded_parts(&words, &[named, isolated(keyword)])
}

impl Kind {
    /// The kind as the catalogue's messages select it.
    fn id(self) -> &'static str {
        match self {
            Kind::Feature => "feature",
            Kind::Background => "background",
            Kind::Rule => "rule",
            Kind::Scenario => "scenario",
            Kind::ScenarioOutline => "scenario-outline",
            Kind::Examples => "examples",
        }
    }
}

/// `words`, worded with `TEXT_MARKS` in place of `texts`, in parts, each
/// mark replaced by its text's parts.
pub(super) fn worded_parts(words: &str, texts: &[Vec<HeadingPart>]) -> Vec<HeadingPart> {
    debug_assert!(texts.len() <= TEXT_MARKS.len());
    let mut parts = Vec::new();
    let mut rest = words;
    while let Some((before, text, after)) = first_mark(rest, texts) {
        parts.push(plain(before));
        parts.extend(text.iter().cloned());
        rest = after;
    }
    parts.push(plain(rest));
    parts.retain(|part| !part.text.is_empty());
    parts
}

/// The words before the first mark in `rest`, the text that goes in its place, and the words after it.
fn first_mark<'a, 'b>(
    rest: &'a str,
    texts: &'b [Vec<HeadingPart>],
) -> Option<(&'a str, &'b [HeadingPart], &'a str)> {
    TEXT_MARKS
        .iter()
        .zip(texts)
        .filter_map(|(mark, text)| split_at_mark(rest, mark, text))
        .min_by_key(|(before, _, _)| before.len())
}

fn split_at_mark<'a, 'b>(
    rest: &'a str,
    mark: &str,
    text: &'b [HeadingPart],
) -> Option<(&'a str, &'b [HeadingPart], &'a str)> {
    let (before, after) = rest.split_once(mark)?;
    Some((before, text, after))
}

pub(super) fn plain(text: &str) -> HeadingPart {
    HeadingPart {
        text: text.to_string(),
        isolated: false,
    }
}

pub(super) fn isolated(text: &str) -> Vec<HeadingPart> {
    vec![HeadingPart {
        text: text.to_string(),
        isolated: true,
    }]
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// The parts of `parts`, each as its text and whether it is isolated.
    fn shown(parts: &[HeadingPart]) -> Vec<(&str, bool)> {
        parts
            .iter()
            .map(|part| (part.text.as_str(), part.isolated))
            .collect()
    }

    #[rstest]
    #[case::outline_written_as_outline(
        Kind::ScenarioOutline,
        "Scenario Outline",
        " Withdraw ",
        &[("Scenario outline: ", false), ("Withdraw", true)]
    )]
    #[case::examples_written_as_scenarios(
        Kind::Examples,
        "Scenarios",
        "Small",
        &[
            ("Examples: ", false),
            ("Small", true),
            (" (written as ", false),
            ("Scenarios", true),
            (")", false)
        ]
    )]
    #[case::unnamed_background_in_portuguese(
        Kind::Background,
        "Contexto",
        "",
        &[
            ("Background", false),
            (" (written as ", false),
            ("Contexto", true),
            (")", false)
        ]
    )]
    #[case::unnamed_rule(Kind::Rule, "Rule", "", &[("Rule", false)])]
    fn a_heading_names_its_kind_then_the_keyword_only_when_it_is_another_word(
        #[case] kind: Kind,
        #[case] keyword: &str,
        #[case] name: &str,
        #[case] expected: &[(&str, bool)],
    ) {
        assert_eq!(shown(&heading(kind, keyword, name)), expected);
    }

    #[test]
    fn worded_parts_put_each_text_where_its_mark_is_whatever_the_order() {
        let words = format!("{}, then {}.", TEXT_MARKS[1], TEXT_MARKS[0]);
        let parts = worded_parts(&words, &[isolated("a\u{1}"), isolated("b")]);
        let parts: Vec<_> = parts
            .iter()
            .map(|part| (part.text.as_str(), part.isolated))
            .collect();
        assert_eq!(
            parts,
            [
                ("b", true),
                (", then ", false),
                ("a\u{1}", true),
                (".", false)
            ]
        );
    }
}
