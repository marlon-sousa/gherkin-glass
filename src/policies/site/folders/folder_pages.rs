//! Policy: the page of every folder of feature files, the home page among them, from each folder node of the site tree and the folders and feature pages it holds.

use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use crate::entities::{
    FeatureEntry, FolderId, FolderLink, FolderPage, FolderPageContent, FolderWords, LinkAddress,
    PageOptions, SiteAddress, SiteFeaturePage, SiteFolder,
};
use crate::policies::site::{breadcrumb, site_title, tag_index_address};
use crate::policies::{CATALOGUE, FOLDER_PAGE, SENTENCES_PAGE};

/// The page of each of `folders`, the input folder's first, the home page,
/// each paired with its folder, made from the folders and feature pages the
/// folder holds and from the folders above it.
pub(crate) fn build_folder_pages<'a>(
    folders: impl Iterator<Item = SiteFolder<'a>>,
    options: &PageOptions,
) -> Vec<(FolderId, FolderPageContent)> {
    let title = site_title(options);
    folders.map(|folder| folder_page(&folder, &title)).collect()
}

/// The page of `folder`, in the site titled `title`, paired with the folder.
fn folder_page(folder: &SiteFolder<'_>, title: &str) -> (FolderId, FolderPageContent) {
    let names = folder.names();
    let home = folder.name().is_none();
    let mut subfolders: Vec<&str> = folder
        .subfolders()
        .filter_map(|subfolder| subfolder.name())
        .collect();
    subfolders.sort_unstable();
    let mut features: Vec<SiteFeaturePage<'_>> = folder.feature_pages().collect();
    features.sort_by(|a, b| file_name(a).cmp(file_name(b)));
    let page = FolderPage {
        address: SiteAddress::of(&names, FOLDER_PAGE),
        language: CATALOGUE.current_language().to_string(),
        title: if home {
            title.to_string()
        } else {
            fl!(
                CATALOGUE,
                "folder-page-title",
                folder = names.join("/"),
                site = title
            )
        },
        heading: names.last().cloned().unwrap_or_else(|| title.to_string()),
        breadcrumb: breadcrumb(title, &names, None),
        totals: if home { totals(folder) } else { Vec::new() },
        tags: home.then(|| FolderLink {
            address: LinkAddress::to(tag_index_address().as_str()),
            name: fl!(CATALOGUE, "tag-index-title"),
        }),
        sentences: home.then(|| FolderLink {
            address: LinkAddress::to(SENTENCES_PAGE),
            name: fl!(CATALOGUE, "sentences-title"),
        }),
        words: FolderWords {
            folders: fl!(CATALOGUE, "folders"),
            features: fl!(CATALOGUE, "features"),
        },
    };
    let content = FolderPageContent {
        page,
        folders: subfolders.into_iter().map(folder_link).collect(),
        features: features.iter().map(feature_entry).collect(),
    };
    (folder.id(), content)
}

/// The link to the page of the subfolder `name`.
fn folder_link(name: &str) -> FolderLink {
    FolderLink {
        address: LinkAddress::to(&format!("{name}/{FOLDER_PAGE}")),
        name: name.to_string(),
    }
}

fn file_name<'a>(feature: &SiteFeaturePage<'a>) -> &'a str {
    feature.page().path.rsplit('/').next().unwrap_or_default()
}

fn feature_entry(feature: &SiteFeaturePage<'_>) -> FeatureEntry {
    let page = feature.page();
    let count = feature.scenario_count();
    FeatureEntry {
        address: LinkAddress::to(page.address.file()),
        language: page.language.clone(),
        heading: page.heading.clone(),
        scenarios: fl!(CATALOGUE, "scenario-count", count = count),
        label: fl!(
            CATALOGUE,
            "feature-scenarios",
            count = count,
            feature = page.heading.as_str()
        ),
        contents: feature.contents(),
    }
}

/// The home page's, of every folder and feature inside the input folder
/// `input`: folders, features, rules and scenarios.
fn totals(input: &SiteFolder<'_>) -> Vec<String> {
    let pages: Vec<SiteFeaturePage<'_>> = input.every_feature_page().collect();
    let rules: usize = pages.iter().map(SiteFeaturePage::rule_count).sum();
    let scenarios: usize = pages.iter().map(SiteFeaturePage::scenario_count).sum();
    vec![
        fl!(
            CATALOGUE,
            "folder-count",
            count = input.every_subfolder().count()
        ),
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
    use crate::policies::site::site_with_feature_pages;

    /// The folder pages of the site of `files`, each a path and its text.
    fn site(files: &[(&str, &str)]) -> Vec<FolderPageContent> {
        let options = PageOptions::default();
        let features: Vec<_> = files
            .iter()
            .map(|(path, text)| {
                let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
                ParsedFeature::new(
                    PathBuf::from_iter(path.split('/')),
                    feature,
                    "en".to_string(),
                    BTreeMap::new(),
                )
            })
            .collect();
        let builder = site_with_feature_pages(&features, &options, &TagNames::default(), None);
        build_folder_pages(builder.folders(), &options)
            .into_iter()
            .map(|(_, page)| page)
            .collect()
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
        let addresses: Vec<_> = site
            .iter()
            .map(|content| content.page.address.as_str())
            .collect();
        assert_eq!(addresses, ["index.html", "b/index.html", "b/c/index.html"]);
        let b = &site[1];
        assert_eq!(
            (b.page.heading.as_str(), b.page.title.as_str()),
            ("b", "b, Feature files")
        );
        assert_eq!(site[2].page.title, "b/c, Feature files");
        assert!(b.features.is_empty());
        assert_eq!(b.folders[0].address.to_string(), "c/index.html");
        assert!(b.page.totals.is_empty());
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
            home.page.totals,
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
            site[0].page.totals,
            ["0 folders", "0 features", "0 rules", "0 scenarios"]
        );
        assert!(site[0].folders.is_empty() && site[0].features.is_empty());
    }
}
