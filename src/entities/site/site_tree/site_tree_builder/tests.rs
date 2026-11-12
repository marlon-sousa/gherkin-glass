use super::*;
use crate::entities::PageOptions;
use crate::policies::page_frame;

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
                other => format!("{other:?}"),
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
        tree.files().cloned().collect::<Vec<_>>(),
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
        tree.root_files().cloned().collect::<Vec<_>>(),
        [SiteNode::Asset("shared.css".to_string())]
    );
    assert_eq!(
        tree.files().cloned().collect::<Vec<_>>(),
        [SiteNode::Asset("own.css".to_string())]
    );
}

#[test]
fn the_frame_is_the_site_s() {
    let frame = page_frame(&PageOptions::default());
    assert_eq!(a_builder().build().frame(), &frame);
}
