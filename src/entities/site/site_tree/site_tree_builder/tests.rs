use std::collections::BTreeSet;

use super::*;
use crate::entities::PageOptions;
use crate::policies::{page_frame, site_tree};
use crate::test_support::parsed_at;

fn a_builder() -> SiteTreeBuilder {
    SiteTreeBuilder::new(page_frame(&PageOptions::default()), None)
}

/// Each node of `tree`, indented by its depth, by its kind and, for a folder or an asset, its name.
fn outline(tree: &SiteTree) -> Vec<String> {
    tree.tree
        .root()
        .descendants()
        .map(|node| {
            let depth = node.ancestors().count();
            let kind = match node.value() {
                SiteNode::Site(_) => "site".to_string(),
                SiteNode::Version(Some(name)) => format!("version {name}"),
                SiteNode::Version(None) => "version".to_string(),
                SiteNode::KeptVersion(name) => format!("kept version {name}"),
                SiteNode::InputFolder => "input folder".to_string(),
                SiteNode::Folder(name) => format!("folder {name}"),
                SiteNode::FolderTree(_) => "folder tree".to_string(),
                SiteNode::SearchDocuments(_) => "search documents".to_string(),
                SiteNode::Asset(name) => format!("asset {name}"),
                SiteNode::FeaturePage(page) => format!("feature page {}", page.address),
                SiteNode::FolderPage(page) => format!("folder page {}", page.address),
                SiteNode::Background(block) => format!("background {}", block.heading.anchor),
                SiteNode::Rule(block) => format!("rule {}", block.heading.anchor),
                SiteNode::Scenario(block) => format!("scenario {}", block.heading.anchor),
                SiteNode::Examples(block) => format!("examples {}", block.heading.anchor),
                SiteNode::ExamplesAsScenarios(heading) => {
                    format!("examples as scenarios {}", heading.anchor)
                }
                SiteNode::ExampleScenario(heading) => {
                    format!("example scenario {}", heading.anchor)
                }
                SiteNode::Step(step) => format!("step {}", step.keyword.trim()),
                SiteNode::StepTable(_) => "step table".to_string(),
                SiteNode::DocString(_) => "doc string".to_string(),
                SiteNode::SentencesPage(_) => "sentences page".to_string(),
                SiteNode::SubfolderLink(link) => format!("subfolder link {}", link.name),
                SiteNode::FeatureEntry(entry) => format!("feature entry {}", entry.address),
                SiteNode::SentencesGroup(group) => format!("sentences group {}", group.heading),
                SiteNode::Sentence(sentence) => format!("sentence {}", sentence.text),
                SiteNode::SentenceUse(found) => format!("sentence use {}", found.address),
                SiteNode::TagIndex(_) => "tag index".to_string(),
                SiteNode::TagIndexEntry(entry) => format!("tag index entry {}", entry.tag),
                SiteNode::TagPage(page) => format!("tag page {}", page.tag),
                SiteNode::TagFeature(feature) => format!("tag feature {}", feature.address),
                SiteNode::TagScenario(scenario) => format!("tag scenario {}", scenario.address),
                SiteNode::VersionsPage(_) => "versions page".to_string(),
                SiteNode::VersionsList(list) => format!("versions list {list:?}"),
                SiteNode::VersionEntry(entry) => format!("version entry {}", entry.name),
            };
            format!("{}{kind}", "  ".repeat(depth))
        })
        .collect()
}

#[test]
fn the_site_holds_its_version_which_holds_the_input_folder_then_its_other_files() {
    let mut builder = a_builder();
    builder.assets(["first.css".to_string()]);
    builder.folder_tree(Vec::new());
    builder.search_documents(Vec::new());
    builder.assets(["last.js".to_string()]);
    let tree = builder.build();
    assert_eq!(
        outline(&tree),
        [
            "site",
            "  version",
            "    input folder",
            "    asset first.css",
            "    folder tree",
            "    search documents",
            "    asset last.js",
        ]
    );
    assert_eq!(
        tree.files()
            .map(|node| node.value().clone())
            .collect::<Vec<_>>(),
        [
            SiteNode::Asset("first.css".to_string()),
            SiteNode::FolderTree(Vec::new()),
            SiteNode::SearchDocuments(Vec::new()),
            SiteNode::Asset("last.js".to_string()),
        ]
    );
}

#[test]
fn a_folder_is_made_once_inside_the_folders_above_it() {
    let mut builder = a_builder();
    let deep = builder.folder_at(&["b", "c"]);
    let b = builder.folder_at(&["b"]);
    let a = builder.folder_at(&["a"]);
    assert_eq!(builder.folder_at(&["b", "c"]), deep);
    assert_eq!(builder.folder_at(&[]), builder.input_folder().id());
    assert_eq!(builder.site_folder(deep).names(), ["b", "c"]);
    assert_eq!(builder.site_folder(a).name(), Some("a"));
    assert_eq!(builder.input_folder().name(), None);
    assert!(builder.input_folder().names().is_empty());
    let subfolders: Vec<_> = builder
        .input_folder()
        .subfolders()
        .map(|folder| folder.id())
        .collect();
    assert_eq!(subfolders, [b, a]);
    let every: Vec<_> = builder.folders().map(|folder| folder.names()).collect();
    assert_eq!(
        every,
        [
            vec![],
            vec!["b".to_string()],
            vec!["b".into(), "c".into()],
            vec!["a".into()]
        ]
    );
    assert_eq!(
        outline(&builder.build()),
        [
            "site",
            "  version",
            "    input folder",
            "      folder b",
            "        folder c",
            "      folder a",
        ]
    );
}

#[test]
fn a_version_is_named_by_the_build() {
    let name = VersionName::new("v1.0.0").expect("a version name");
    let builder = SiteTreeBuilder::new(page_frame(&PageOptions::default()), Some(name));
    assert_eq!(outline(&builder.build())[1], "  version v1.0.0");
}

#[test]
fn the_kept_versions_and_the_versions_page_s_assets_sit_beside_the_version() {
    let name = VersionName::new("v2").expect("a version name");
    let mut builder = SiteTreeBuilder::new(page_frame(&PageOptions::default()), Some(name));
    builder.kept_versions([VersionName::new("v1").expect("a version name")]);
    builder.versions_page_assets(["shared.css".to_string()]);
    builder.assets(["own.css".to_string()]);
    let tree = builder.build();
    assert_eq!(
        outline(&tree),
        [
            "site",
            "  version v2",
            "    input folder",
            "    asset own.css",
            "  kept version v1",
            "  asset shared.css",
        ]
    );
    assert_eq!(
        tree.root_files()
            .map(|node| node.value().clone())
            .collect::<Vec<_>>(),
        [SiteNode::Asset("shared.css".to_string())]
    );
    assert_eq!(
        tree.files()
            .map(|node| node.value().clone())
            .collect::<Vec<_>>(),
        [SiteNode::Asset("own.css".to_string())]
    );
}

#[test]
fn the_frame_is_the_site_s() {
    let frame = page_frame(&PageOptions::default());
    assert_eq!(a_builder().build().frame(), &frame);
}

#[test]
fn a_feature_page_holds_its_blocks_each_block_its_steps_and_each_step_its_table_or_doc_string() {
    let text = "\
Feature: F
  Background:
    Given a
      | x |
  Scenario: S
    Given b
      \"\"\"
      text
      \"\"\"
  Rule: R
    Background: B
      Given c
    Scenario Outline: O
      Given <n>
      Examples:
        | n |
        | 1 |
";
    let parsed = parsed_at("f.feature", text);
    let tree = site_tree(&[parsed], &PageOptions::default(), None, &BTreeSet::new());
    let lines = outline(&tree);
    let start = lines
        .iter()
        .position(|line| line.ends_with("feature page f.html"))
        .expect("the feature page");
    assert_eq!(
        lines[start..start + 16],
        [
            "      feature page f.html",
            "        background background",
            "          step Given",
            "            step table",
            "        scenario s",
            "          step Given",
            "            doc string",
            "        rule r",
            "          background b",
            "            step Given",
            "          scenario o",
            "            step Given",
            "            examples examples",
            "            examples as scenarios o_scenarios",
            "              example scenario o-row-1",
            "                step Given",
        ]
        .map(String::from)
    );
}

/// Whether the line of `outline` is not one of the nodes the test of the
/// other pages leaves out: the assets, the feature page's blocks and steps,
/// the folder tree's data and the search documents.
fn lists_an_entry(line: &str) -> bool {
    const LEFT_OUT: [&str; 5] = [
        "asset",
        "step",
        "scenario s",
        "folder tree",
        "search documents",
    ];
    LEFT_OUT.iter().all(|part| !line.contains(part))
}

#[test]
fn each_other_page_holds_its_entries_and_each_entry_what_it_lists() {
    let parsed = parsed_at(
        "d/f.feature",
        "@t\nFeature: F\n  Scenario: S\n    Given a\n",
    );
    let version = VersionName::new("v1.0.0").expect("a version name");
    let tree = site_tree(
        &[parsed],
        &PageOptions::default(),
        Some(&version),
        &BTreeSet::new(),
    );
    let kept: Vec<_> = outline(&tree)
        .into_iter()
        .filter(|line| lists_an_entry(line))
        .collect();
    assert_eq!(
        kept,
        [
            "site",
            "  version v1.0.0",
            "    input folder",
            "      folder d",
            "        feature page d/f.html",
            "        folder page d/index.html",
            "          feature entry f.html",
            "      folder page index.html",
            "        subfolder link d",
            "    sentences page",
            "      sentences group Given",
            "        sentence a",
            "          sentence use d/f.html#s",
            "    tag index",
            "      tag index entry @t",
            "    tag page @t",
            "      tag feature ../d/f.html",
            "        tag scenario ../d/f.html#s",
            "  versions page",
            "    versions list Development",
            "    versions list Releases",
            "      version entry v1.0.0",
            "    versions list PreReleases",
        ]
        .map(String::from)
    );
}
