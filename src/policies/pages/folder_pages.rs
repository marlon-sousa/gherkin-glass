//! Policy: the page of every folder of feature files, the home page among them, from the features' pages.

use std::collections::BTreeSet;

use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use crate::entities::{
    FeatureEntry, FolderLink, FolderPage, FolderWords, LinkAddress, Page, PageOptions, SiteAddress,
};
use crate::policies::{
    CATALOGUE, FOLDER_PAGE, SENTENCES_PAGE, breadcrumb, page_frame, site_title, tag_index_address,
};

/// `pages` are every feature's, in any order; the folder pages come
/// outermost first, the home page first of all, then folder by folder.
pub(crate) fn build_folder_pages(pages: &[Page], options: &PageOptions) -> Vec<FolderPage> {
    let title = site_title(options);
    let located: Vec<(Vec<String>, String, &Page)> = pages.iter().map(located).collect();
    let mut folders: BTreeSet<Vec<String>> = BTreeSet::from([Vec::new()]);
    for (parts, _, _) in &located {
        for depth in 1..=parts.len() {
            folders.insert(parts.get(..depth).unwrap_or_default().to_vec());
        }
    }
    folders
        .iter()
        .map(|folder| {
            let subfolders = folders
                .iter()
                .filter(|other| other.len() == folder.len() + 1 && other.starts_with(folder))
                .filter_map(|other| other.last())
                .map(|name| FolderLink {
                    address: LinkAddress::to(&format!("{name}/{FOLDER_PAGE}")),
                    name: name.clone(),
                })
                .collect();
            let mut features: Vec<(&String, &Page)> = located
                .iter()
                .filter(|(parts, _, _)| parts == folder)
                .map(|(_, file, page)| (file, *page))
                .collect();
            features.sort_by(|a, b| file_name(a.1).cmp(file_name(b.1)));
            let totals = if folder.is_empty() {
                totals(pages, folders.len() - 1)
            } else {
                Vec::new()
            };
            FolderPage {
                address: SiteAddress::of(folder, FOLDER_PAGE),
                language: CATALOGUE.current_language().to_string(),
                title: if folder.is_empty() {
                    title.clone()
                } else {
                    fl!(
                        CATALOGUE,
                        "folder-page-title",
                        folder = folder.join("/"),
                        site = title.as_str()
                    )
                },
                heading: folder.last().cloned().unwrap_or_else(|| title.clone()),
                frame: page_frame(options),
                breadcrumb: breadcrumb(&title, folder, None),
                totals,
                tags: folder.is_empty().then(|| FolderLink {
                    address: LinkAddress::to(tag_index_address().as_str()),
                    name: fl!(CATALOGUE, "tag-index-title"),
                }),
                sentences: folder.is_empty().then(|| FolderLink {
                    address: LinkAddress::to(SENTENCES_PAGE),
                    name: fl!(CATALOGUE, "sentences-title"),
                }),
                folders: subfolders,
                features: features
                    .into_iter()
                    .map(|(file, page)| feature_entry(file, page))
                    .collect(),
                words: FolderWords {
                    folders: fl!(CATALOGUE, "folders"),
                    features: fl!(CATALOGUE, "features"),
                },
            }
        })
        .collect()
}

/// The page's folders, from the outermost, its file's name, and the page.
fn located(page: &Page) -> (Vec<String>, String, &Page) {
    (
        page.address
            .folders()
            .into_iter()
            .map(str::to_string)
            .collect(),
        page.address.file().to_string(),
        page,
    )
}

fn file_name(page: &Page) -> &str {
    page.path.rsplit('/').next().unwrap_or_default()
}

fn feature_entry(file: &str, page: &Page) -> FeatureEntry {
    let count = page.scenario_count();
    FeatureEntry {
        address: LinkAddress::to(file),
        language: page.language.clone(),
        heading: page.heading.clone(),
        scenarios: fl!(CATALOGUE, "scenario-count", count = count),
        label: fl!(
            CATALOGUE,
            "feature-scenarios",
            count = count,
            feature = page.heading.as_str()
        ),
        contents: page.contents.clone(),
    }
}

/// The home page's: folders, features, rules and scenarios.
fn totals(pages: &[Page], folders: usize) -> Vec<String> {
    let rules: usize = pages.iter().map(Page::rule_count).sum();
    let scenarios: usize = pages.iter().map(Page::scenario_count).sum();
    vec![
        fl!(CATALOGUE, "folder-count", count = folders),
        fl!(CATALOGUE, "feature-count", count = pages.len()),
        fl!(CATALOGUE, "rule-count", count = rules),
        fl!(CATALOGUE, "scenario-count", count = scenarios),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{ParsedFeature, TagNames};
    use crate::policies::anchor_feature;
    use crate::policies::build_page;

    fn site(files: &[(&str, &str)]) -> Vec<FolderPage> {
        let options = PageOptions::default();
        let pages: Vec<Page> = files
            .iter()
            .map(|(path, text)| {
                let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
                let parsed = ParsedFeature::new(
                    PathBuf::from_iter(path.split('/')),
                    feature,
                    "en".to_string(),
                    BTreeMap::new(),
                );
                build_page(&anchor_feature(&parsed), &options, &TagNames::default())
            })
            .collect();
        build_folder_pages(&pages, &options)
    }

    const COUNTED: &str = "\
Feature: Counted
  Background:
    Given x
  Scenario: One
    Given x
  Scenario Outline: Two
    Given <n>
    Examples:
      | n |
      | 1 |
      | 2 |
  Rule: R
    Scenario: Three
      Given x
  Rule: Empty
";

    #[test]
    fn every_folder_above_a_feature_has_a_page_the_home_page_first() {
        let site = site(&[
            ("b/c/deep.feature", "Feature: Deep\n"),
            ("a.feature", "Feature: A\n"),
        ]);
        let addresses: Vec<_> = site.iter().map(|page| page.address.as_str()).collect();
        assert_eq!(addresses, ["index.html", "b/index.html", "b/c/index.html"]);
        let b = &site[1];
        assert_eq!(
            (b.heading.as_str(), b.title.as_str()),
            ("b", "b, Feature files")
        );
        assert_eq!(site[2].title, "b/c, Feature files");
        assert!(b.features.is_empty());
        assert_eq!(b.folders[0].address.to_string(), "c/index.html");
        assert!(b.totals.is_empty());
    }

    #[test]
    fn subfolders_and_features_are_sorted_by_their_character_codes() {
        let site = site(&[
            ("b.feature", "Feature: B\n"),
            ("a.feature", "Feature: A\n"),
            ("Z.feature", "Feature: Z\n"),
            ("y/x.feature", "Feature: X\n"),
            ("W/x.feature", "Feature: X\n"),
        ]);
        let home = &site[0];
        let folders: Vec<_> = home.folders.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(folders, ["W", "y"]);
        let features: Vec<_> = home
            .features
            .iter()
            .map(|f| f.address.to_string())
            .collect();
        assert_eq!(features, ["Z.html", "a.html", "b.html"]);
        // By the files' names, not the pages'.
        let site = super::tests::site(&[("a.c.feature", "Feature: C\n"), ("a.b", "Feature: B\n")]);
        let features: Vec<_> = site[0]
            .features
            .iter()
            .map(|f| f.address.to_string())
            .collect();
        assert_eq!(features, ["a.html", "a.c.html"]);
    }

    #[test]
    fn outlines_count_once_rules_scenarios_count_and_backgrounds_do_not() {
        let site = site(&[
            ("x/counted.feature", COUNTED),
            ("y.feature", "Feature: Y\n"),
        ]);
        let home = &site[0];
        assert_eq!(
            home.totals,
            ["1 folder", "2 features", "2 rules", "3 scenarios"]
        );
        let counted = &site[1].features[0];
        assert_eq!(counted.scenarios, "3 scenarios");
        assert_eq!(counted.label, "3 scenarios of Feature: Counted");
        assert_eq!(counted.contents.len(), 4);
        let y = &home.features[0];
        assert_eq!((y.scenarios.as_str(), y.contents.len()), ("0 scenarios", 0));
    }

    #[test]
    fn an_empty_input_has_a_home_page_with_nothing_in_it() {
        let site = site(&[]);
        assert_eq!(site.len(), 1);
        assert_eq!(
            site[0].totals,
            ["0 folders", "0 features", "0 rules", "0 scenarios"]
        );
        assert!(site[0].folders.is_empty() && site[0].features.is_empty());
    }
}
