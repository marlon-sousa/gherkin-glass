//! Policy: the tags of the input, their order, and the name of each tag's page, which the feature pages link to and the tag pages are written at.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};

use gherkin::Feature;

use crate::entities::{ParsedFeature, TagNames};
use crate::policies::TAG_INDEX_NAME;
use crate::policies::site::{anchor_of, unique};

/// The name of a tag that leaves nothing after the anchor rule.
const FALLBACK: &str = "tag";

/// Every tag written in `features`, in tag order, each named.
pub(crate) fn name_tags(features: &[ParsedFeature]) -> TagNames {
    name_each(&tags_of(features))
}

/// Tag order: by the text lower-cased, then as written, each by character codes.
fn tag_order(a: &str, b: &str) -> Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}

/// Every tag written in `features`, once each, with its `@`, in tag order.
fn tags_of(features: &[ParsedFeature]) -> Vec<String> {
    let mut tags: Vec<String> = features
        .iter()
        .flat_map(|parsed| feature_tags(&parsed.feature))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|tag| format!("@{tag}"))
        .collect();
    tags.sort_by(|a, b| tag_order(a, b));
    tags
}

/// `tags` in tag order, each with its `@`.
fn name_each(tags: &[String]) -> TagNames {
    let mut taken = HashSet::from([TAG_INDEX_NAME.to_string()]);
    let mut names = TagNames::default();
    for tag in tags {
        let mut name = anchor_of(tag.strip_prefix('@').unwrap_or(tag));
        if name.is_empty() {
            name = FALLBACK.to_string();
        }
        names.insert(tag.clone(), unique(name, &mut taken));
    }
    names
}

/// Without their `@`, as the `gherkin` crate keeps them.
fn feature_tags(feature: &Feature) -> Vec<&str> {
    let scenarios = feature
        .scenarios
        .iter()
        .chain(feature.rules.iter().flat_map(|rule| &rule.scenarios));
    let examples = scenarios
        .clone()
        .flat_map(|scenario| &scenario.examples)
        .flat_map(|examples| &examples.tags);
    feature
        .tags
        .iter()
        .chain(feature.rules.iter().flat_map(|rule| &rule.tags))
        .chain(scenarios.flat_map(|scenario| &scenario.tags))
        .chain(examples)
        .map(String::as_str)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::GherkinEnv;

    use super::*;

    fn parsed(source: &str) -> ParsedFeature {
        let feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
        ParsedFeature::new(
            PathBuf::from("a.feature"),
            feature,
            "en".into(),
            BTreeMap::new(),
        )
    }

    #[test]
    fn every_tag_is_found_once_in_the_index_s_order() {
        let features = [
            parsed(
                "@b @Slow\nFeature: F\n  @r\n  Rule: R\n    @slow\n    Scenario: S\n      Given x\n\n      @ex\n      Examples:\n        | a |\n        | 1 |\n",
            ),
            parsed("@b\nFeature: G\n  @A\n  Scenario: T\n    Given y\n"),
        ];
        assert_eq!(
            tags_of(&features),
            ["@A", "@b", "@ex", "@r", "@Slow", "@slow"]
        );
    }

    #[test]
    fn names_follow_the_anchor_rule_with_index_taken_and_repeats_numbered() {
        let tags: Vec<String> = ["@index", "@needs:db", "@Slow", "@slow", "@🚀", "@???"]
            .map(String::from)
            .to_vec();
        let names = name_each(&tags);
        let addresses: Vec<_> = tags
            .iter()
            .map(|tag| names.address(tag).to_string())
            .collect();
        assert_eq!(
            addresses,
            [
                "tags/index-2.html",
                "tags/needs-db.html",
                "tags/slow.html",
                "tags/slow-2.html",
                "tags/tag.html",
                "tags/tag-2.html",
            ]
        );
    }
}
