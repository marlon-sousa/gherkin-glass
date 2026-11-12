//! Policy: the index of every tag and each tag's page, built from the site's feature pages.

mod tag_carriers;

use std::slice::from_ref;

use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use crate::entities::{
    LinkAddress, Page, PageOptions, TagFeature, TagIndex, TagIndexEntry, TagNames, TagPage,
    TagScenario,
};
use crate::policies::{CATALOGUE, breadcrumb, outside_link, site_title};
use tag_carriers::{Through, carriers, writes};

/// Stands for the link in the words around it, to find where the link
/// goes; no message of the catalogue holds it.
const LINK_MARK: &str = "\u{0}";

/// From a tag page or the index, both in the folder of the tag pages, back to the site's root.
const ROOT: &str = "../";

/// The `lang` of the index and the tag pages: the catalogue's language.
pub(crate) fn tag_pages_language() -> String {
    CATALOGUE.current_language().to_string()
}

/// `tags` is every tag of the site in tag order, and `pages` every feature's page in path order.
pub(crate) fn build_tag_index(
    tags: &[String],
    pages: &[Page],
    names: &TagNames,
    options: &PageOptions,
) -> TagIndex {
    let entries = tags
        .iter()
        .map(|tag| {
            let count = pages
                .iter()
                .map(|page| carriers(page, tag).len())
                .sum::<usize>();
            let words = fl!(CATALOGUE, "tag-index-entry", tag = LINK_MARK, count = count);
            let (before, after) = around_mark(&words);
            TagIndexEntry {
                address: LinkAddress::to(&format!("{}.html", names.name(tag))),
                tag: tag.clone(),
                before,
                after,
            }
        })
        .collect();
    let title = fl!(CATALOGUE, "tag-index-title");
    TagIndex {
        // The index is the folder's own page, so its crumb reads the index's title.
        breadcrumb: breadcrumb(&site_title(options), from_ref(&title), None),
        title,
        entries,
        empty: fl!(CATALOGUE, "tag-index-empty"),
    }
}

pub(crate) fn build_tag_page(
    tag: &str,
    pages: &[Page],
    names: &TagNames,
    options: &PageOptions,
) -> TagPage {
    let language = tag_pages_language();
    let features = pages
        .iter()
        .filter(|page| writes(page, tag))
        .map(|page| {
            let page_address = format!("{ROOT}{}", page.address);
            TagFeature {
                address: LinkAddress::to(&page_address),
                heading: page.title.clone(),
                language: (page.language != language).then(|| page.language.clone()),
                scenarios: carriers(page, tag)
                    .into_iter()
                    .map(|(scenario, through)| {
                        let words = match through {
                            Through::Itself => LINK_MARK.to_string(),
                            Through::Examples => {
                                fl!(CATALOGUE, "tag-through-examples", scenario = LINK_MARK)
                            }
                            Through::Rule => {
                                fl!(CATALOGUE, "tag-through-rule", scenario = LINK_MARK)
                            }
                            Through::Feature => {
                                fl!(CATALOGUE, "tag-through-feature", scenario = LINK_MARK)
                            }
                        };
                        let (before, after) = around_mark(&words);
                        TagScenario {
                            address: LinkAddress::to(&page_address).at(&scenario.heading.anchor),
                            heading: scenario.heading.text(),
                            before,
                            after,
                        }
                    })
                    .collect(),
            }
        })
        .collect();
    let (heading_before, heading_after) =
        around_mark(&fl!(CATALOGUE, "tag-page-title", tag = LINK_MARK));
    let title = fl!(CATALOGUE, "tag-page-title", tag = tag);
    let file = format!("{}.html", names.name(tag));
    TagPage {
        address: names.address(tag),
        breadcrumb: breadcrumb(
            &site_title(options),
            &[fl!(CATALOGUE, "tag-index-title")],
            Some((&title, &file)),
        ),
        title,
        heading_before,
        tag: tag.to_string(),
        heading_after,
        outside: outside_link(tag, &options.tag_links),
        features,
        no_scenario: fl!(CATALOGUE, "tag-no-scenario"),
    }
}

fn around_mark(words: &str) -> (String, String) {
    let (before, after) = words.split_once(LINK_MARK).unwrap_or((words, ""));
    (before.to_string(), after.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::ParsedFeature;
    use crate::policies::anchor_feature;
    use crate::policies::{build_page, name_tags, tags_of};

    const FEATURE: &str = "\
@feature
Feature: F

  @own
  Scenario: Own
    Given x

  Scenario Outline: Through examples
    Given <a>

    @ex
    Examples:
      | a |
      | 1 |

  @rule
  Rule: R
    @own
    Scenario: In the rule
      Given y

  @empty
  Rule: Nothing
";

    fn site(sources: &[(&str, &str)]) -> (Vec<String>, Vec<Page>, TagNames) {
        let parsed: Vec<_> = sources
            .iter()
            .map(|(path, source)| {
                let feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
                ParsedFeature::new(PathBuf::from(path), feature, "en".into(), BTreeMap::new())
            })
            .collect();
        let tags = tags_of(&parsed);
        let names = name_tags(&tags);
        let pages = parsed
            .iter()
            .map(|p| build_page(&anchor_feature(p), &PageOptions::default(), &names))
            .collect();
        (tags, pages, names)
    }

    fn listed(page: &TagPage) -> Vec<(String, String)> {
        page.features
            .iter()
            .flat_map(|feature| &feature.scenarios)
            .map(|s| {
                (
                    s.address.to_string(),
                    format!("{}{}{}", s.before, s.heading, s.after),
                )
            })
            .collect()
    }

    #[test]
    fn a_scenario_carries_a_tag_itself_or_through_its_examples_rule_or_feature() {
        let (_, pages, names) = site(&[("a b.feature", FEATURE)]);
        let page = |tag| build_tag_page(tag, &pages, &names, &PageOptions::default());
        assert_eq!(
            listed(&page("@feature")),
            [
                (
                    "../a%20b.html#own".into(),
                    "Scenario: Own, through its feature".into()
                ),
                (
                    "../a%20b.html#through-examples".into(),
                    "Scenario Outline: Through examples, through its feature".into()
                ),
                (
                    "../a%20b.html#in-the-rule".into(),
                    "Scenario: In the rule, through its feature".into()
                ),
            ]
        );
        assert_eq!(
            listed(&page("@own")),
            [
                ("../a%20b.html#own".into(), "Scenario: Own".into()),
                (
                    "../a%20b.html#in-the-rule".into(),
                    "Scenario: In the rule".into()
                ),
            ]
        );
        assert_eq!(
            listed(&page("@ex"))[0].1,
            "Scenario Outline: Through examples, through its examples"
        );
        assert_eq!(
            listed(&page("@rule"))[0].1,
            "Scenario: In the rule, through its rule"
        );
        let empty = page("@empty");
        assert_eq!(empty.features.len(), 1);
        assert!(empty.features[0].scenarios.is_empty());
        assert_eq!(empty.title, "Tag @empty");
        assert_eq!(empty.address.as_str(), "tags/empty.html");
    }

    #[test]
    fn the_index_counts_the_scenarios_that_carry_each_tag() {
        let (tags, pages, names) =
            site(&[("a.feature", FEATURE), ("b.feature", "@own\nFeature: G\n")]);
        let index = build_tag_index(&tags, &pages, &names, &PageOptions::default());
        let entries: Vec<_> = index
            .entries
            .iter()
            .map(|e| format!("{} {}{}{}", e.address, e.before, e.tag, e.after))
            .collect();
        assert_eq!(
            entries,
            [
                "empty.html @empty, no scenarios",
                "ex.html @ex, one scenario",
                "feature.html @feature, 3 scenarios",
                "own.html @own, 2 scenarios",
                "rule.html @rule, one scenario",
            ]
        );
        let own = build_tag_page("@own", &pages, &names, &PageOptions::default());
        assert_eq!(own.features.len(), 2);
        assert!(own.features[1].scenarios.is_empty());
    }
}
