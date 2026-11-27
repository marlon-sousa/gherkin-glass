use super::*;
use crate::test_support::{parsed_at, tag_link};

fn store(files: &[(&str, &str)], options: &PageOptions) -> SiteStore {
    let parsed: Vec<_> = files
        .iter()
        .map(|(path, text)| parsed_at(path, text))
        .collect();
    let features: Vec<_> = parsed.iter().collect();
    build_store(&features, options)
}

// Rule 252: the features in the order of rule 15, name by name, so the folder `a` before
// `a.feature`.
#[test]
fn the_features_are_in_the_order_of_their_paths() {
    let built = store(
        &[
            ("b.feature", "Feature: B\n"),
            ("a/z.feature", "Feature: Z\n"),
            ("a.feature", "Feature: A\n"),
        ],
        &PageOptions::default(),
    );
    let pages: Vec<_> = built
        .features
        .iter()
        .map(|feature| feature.page.as_str())
        .collect();
    assert_eq!(pages, ["a/z", "a", "b"]);
}

// Rule 252: the source is the link rule 47 makes, at the feature's line, and none without
// `source_link`.
#[test]
fn a_feature_s_source_is_its_link_at_its_line() {
    let files = [("a.feature", "# a comment\nFeature: A\n")];
    let linked = PageOptions {
        source_link: Some("https://example.com/{path}#L{line}".to_string()),
        ..PageOptions::default()
    };
    let sources: Vec<_> = [&linked, &PageOptions::default()]
        .into_iter()
        .map(|options| store(&files, options))
        .filter_map(|built| built.features.into_iter().next())
        .map(|feature| feature.source)
        .collect();
    assert_eq!(
        sources,
        [Some("https://example.com/a.feature#L2".to_string()), None]
    );
}

// Rule 252: each tag with the whole name of its page and its link to another site.
#[test]
fn each_tag_has_its_page_and_its_link() {
    let options = PageOptions {
        tag_links: vec![tag_link("@JIRA-(\\d+)", "https://jira/{1}", "Ticket {1}")],
        ..PageOptions::default()
    };
    let built = store(
        &[("a.feature", "@JIRA-7 @needs:db\nFeature: A\n")],
        &options,
    );
    let tags: Vec<_> = built
        .tags
        .iter()
        .map(|tag| (tag.tag.as_str(), tag.page.as_str(), tag.link.is_some()))
        .collect();
    assert_eq!(
        tags,
        [
            ("@JIRA-7", "tags/jira-7", true),
            ("@needs:db", "tags/needs-db", false)
        ]
    );
}

// Rule 252: the sentences of each kind, each once, in the order of rule 138, from every
// feature, but not from an outline's runs.
#[test]
fn the_sentences_are_grouped_by_kind_each_once_in_order() {
    let built = store(
        &[
            (
                "a.feature",
                "Feature: A\n  Scenario: S\n    Given the Shop\n    When I pay 5\n    Then done\n",
            ),
            (
                "b.feature",
                "Feature: B\n  Scenario Outline: O\n    Given an apple\n    And the Shop\n    \
                 Then <n> left\n\n    Examples:\n      | n |\n      | 1 |\n",
            ),
        ],
        &PageOptions::default(),
    );
    let sentences = &built.sentences;
    assert_eq!(sentences.given, ["an apple", "the Shop"]);
    assert_eq!(sentences.when, ["I pay 5"]);
    assert_eq!(sentences.then, ["\"…\" left", "done"]);
}
