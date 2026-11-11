//! Policy: the folder tree's items, from the folder pages, and the JSON the pages' script reads them from.

use std::collections::BTreeMap;

use crate::entities::{ContentsEntry, FolderPage, LinkAddress, TreeItem};
use crate::policies::FOLDER_PAGE;

/// `folders` are every folder page, the home page among them. The home
/// page's item comes first, then the input folder's, as its page lists them.
pub(crate) fn build_folder_tree(folders: &[FolderPage]) -> Vec<TreeItem> {
    let by_address: BTreeMap<&str, &FolderPage> = folders
        .iter()
        .map(|folder| (folder.address.as_str(), folder))
        .collect();
    let Some(home) = by_address.get(FOLDER_PAGE) else {
        return Vec::new();
    };
    let mut items = vec![TreeItem {
        text: home.heading.clone(),
        address: LinkAddress::to(FOLDER_PAGE),
        lang: None,
        items: Vec::new(),
    }];
    items.extend(folder_items(&by_address, ""));
    items
}

/// `folder` is the folder's path in the site, as written, ending in `/`, or
/// empty for the input folder.
fn folder_items(by_address: &BTreeMap<&str, &FolderPage>, folder: &str) -> Vec<TreeItem> {
    let Some(page) = by_address.get(format!("{folder}{FOLDER_PAGE}").as_str()) else {
        return Vec::new();
    };
    let encoded = LinkAddress::to(folder);
    let subfolders = page.folders.iter().map(|subfolder| {
        let inner = format!("{folder}{}/", subfolder.name);
        TreeItem {
            text: subfolder.name.clone(),
            address: encoded.then(&subfolder.address),
            lang: None,
            items: folder_items(by_address, &inner),
        }
    });
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

/// One line, ending in a line break.
#[expect(clippy::expect_used, reason = "strings and lists always serialise")]
pub(crate) fn render_folder_tree(items: &[TreeItem]) -> String {
    let mut json = serde_json::to_string(items).expect("strings and lists always serialise");
    json.push('\n');
    json
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{PageOptions, ParsedFeature, TagNames};
    use crate::policies::anchor_feature;
    use crate::policies::{build_folder_pages, build_page};

    fn tree(files: &[(&str, &str)]) -> Vec<TreeItem> {
        let options = PageOptions {
            title: Some("Site".into()),
            ..PageOptions::default()
        };
        let pages: Vec<_> = files
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
        build_folder_tree(&build_folder_pages(&pages, &options))
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
    fn only_a_feature_s_item_has_a_lang_and_empty_lists_are_left_out_of_the_json() {
        let items = tree(&[("a/b.feature", "Feature: B\n  Scenario: S\n")]);
        assert_eq!(
            render_folder_tree(&items),
            "[{\"text\":\"Site\",\"address\":\"index.html\"},\
             {\"text\":\"a\",\"address\":\"a/index.html\",\"items\":[\
             {\"text\":\"Feature: B\",\"address\":\"a/b.html\",\"lang\":\"en\",\"items\":[\
             {\"text\":\"Scenario: S\",\"address\":\"a/b.html#s\"}]}]}]\n"
        );
    }

    #[test]
    fn an_empty_input_has_only_the_home_page() {
        let items = tree(&[]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].address.to_string(), "index.html");
    }
}
