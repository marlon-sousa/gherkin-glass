//! Policy: the folder tree's items, from the input folder's node of the site tree and the folders and pages it holds.

use crate::entities::{ContentsEntry, FolderLink, LinkAddress, SiteFolder, TreeItem};
use crate::policies::FOLDER_PAGE;

/// The items of the input folder `input`, once each folder holds its page.
/// The home page's item comes first, then the input folder's, as its page lists them.
pub(crate) fn build_folder_tree(input: SiteFolder<'_>) -> Vec<TreeItem> {
    let Some(home) = input.folder_page() else {
        return Vec::new();
    };
    let mut items = vec![TreeItem {
        text: home.heading.clone(),
        address: LinkAddress::to(FOLDER_PAGE),
        lang: None,
        items: Vec::new(),
    }];
    items.extend(folder_items(input, ""));
    items
}

/// The items inside `folder`, as its page lists them. `path` is the folder's
/// path in the site, as written, ending in `/`, or empty for the input folder.
fn folder_items(folder: SiteFolder<'_>, path: &str) -> Vec<TreeItem> {
    let Some(page) = folder.folder_page() else {
        return Vec::new();
    };
    let encoded = LinkAddress::to(path);
    let subfolders = page
        .folders
        .iter()
        .map(|link| subfolder_item(folder, path, &encoded, link));
    let features = page.features.iter().map(|feature| {
        let address = encoded.then(&feature.address);
        TreeItem {
            text: feature.heading.clone(),
            lang: Some(feature.language.clone()),
            items: block_items(&feature.contents, &address),
            address,
        }
    });
    subfolders.chain(features).collect()
}

/// The item of the subfolder `link` lists, inside `folder`, whose path is
/// `path` and, encoded, `encoded`.
fn subfolder_item(
    folder: SiteFolder<'_>,
    path: &str,
    encoded: &LinkAddress,
    link: &FolderLink,
) -> TreeItem {
    TreeItem {
        text: link.name.clone(),
        address: encoded.then(&link.address),
        lang: None,
        items: folder
            .subfolder(&link.name)
            .map(|subfolder| folder_items(subfolder, &format!("{path}{}/", link.name)))
            .unwrap_or_default(),
    }
}

fn block_items(contents: &[ContentsEntry], page: &LinkAddress) -> Vec<TreeItem> {
    contents
        .iter()
        .map(|entry| TreeItem {
            text: entry.heading.clone(),
            address: page.at(&entry.anchor),
            lang: None,
            items: block_items(&entry.entries, page),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{PageOptions, ParsedFeature, SiteNode};
    use crate::policies::site_tree;

    /// The folder tree's items of the site of `files`, each a path and its text.
    fn tree(files: &[(&str, &str)]) -> Vec<TreeItem> {
        let options = PageOptions {
            title: Some("Site".into()),
            ..PageOptions::default()
        };
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
        site_tree(&features, &options, None, &BTreeSet::new())
            .files()
            .find_map(|file| match file.value() {
                SiteNode::FolderTree(items) => Some(items.clone()),
                _ => None,
            })
            .expect("a site has its folder tree's data")
    }

    /// Each item's text and address, indented by its depth.
    fn outline(items: &[TreeItem], depth: usize, lines: &mut Vec<String>) {
        for item in items {
            lines.push(format!(
                "{}{} {}",
                "  ".repeat(depth),
                item.text,
                item.address
            ));
            outline(&item.items, depth + 1, lines);
        }
    }

    #[test]
    fn the_home_page_comes_first_then_folders_before_features_as_their_pages_list_them() {
        let items = tree(&[
            (
                "z.feature",
                "Feature: Z\n  Scenario: Out\n    Given x\n  Rule: R\n    Scenario: In\n      Given x\n",
            ),
            ("b c/d.feature", "Feature: D\n"),
            ("a/x/e.feature", "Feature: E\n  Scenario: S\n"),
        ]);
        let mut lines = Vec::new();
        outline(&items, 0, &mut lines);
        assert_eq!(
            lines,
            [
                "Site index.html",
                "a a/index.html",
                "  x a/x/index.html",
                "    Feature: E a/x/e.html",
                "      Scenario: S a/x/e.html#s",
                "b c b%20c/index.html",
                "  Feature: D b%20c/d.html",
                "Feature: Z z.html",
                "  Scenario: Out z.html#out",
                "  Rule: R z.html#r",
                "    Scenario: In z.html#in",
            ]
        );
    }

    #[test]
    fn an_empty_input_has_only_the_home_page() {
        let items = tree(&[]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].address.to_string(), "index.html");
    }
}
