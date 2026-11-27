//! The store plan of a version or a preview removed from the output folder.

use super::*;

// Rule 257: the descriptor first, then the version's folder and the objects only it had.
#[test]
fn a_version_removed_takes_its_folder_and_its_objects_descriptor_first() {
    let change = StoreChange {
        target: version("main"),
        written: None,
        versions_page: Some(versions_page("2.0.0")),
        assets: assets("2.0.0"),
    };
    let plan = plan(&two_versions(), &two_versions_files(), change);
    assert_eq!(
        plan.removes,
        ["data/versions/main.json", "data/objects/old.json", "main"]
    );
    assert_eq!(
        paths(&plan.writes),
        ["data/versions.json", "data/pages.json", "index.html"]
    );
}

// Rule 257: the last version removed takes the versions page and the files of the versions.
#[test]
fn the_last_version_removed_takes_the_versions_page_with_it() {
    let mut published = two_versions();
    published.descriptors.remove(&version("v1.0.0"));
    let files: BTreeSet<FilePath> = two_versions_files()
        .into_iter()
        .filter(|file| !file.contains("v1.0.0"))
        .collect();
    let change = StoreChange {
        target: version("main"),
        written: None,
        versions_page: None,
        assets: assets("2.0.0"),
    };
    let plan = plan(&published, &files, change);
    assert!(plan.writes.is_empty());
    assert_eq!(plan.last_marker, BTreeSet::new());
    assert_eq!(
        plan.removes,
        [
            "data/versions/main.json",
            "assets/2.0.0",
            "data/objects/old.json",
            "data/objects/shared.json",
            "data/pages.json",
            "data/versions.json",
            "index.html",
            "main",
        ]
    );
}

// Rules 39 and 255: a ghg's assets that nothing links any more go as one folder, and a preview
// removed takes its folder.
#[test]
fn a_preview_removed_takes_its_folder_and_the_assets_nothing_links() {
    let preview = BuildTarget::Preview(name("pr-1"));
    let mut published = two_versions();
    published.descriptors.insert(
        preview.clone(),
        PublishedDescriptor {
            ghg: "2.1.0".into(),
            features: vec![feature("a", "shared")],
        },
    );
    let files: BTreeSet<FilePath> = two_versions_files()
        .into_iter()
        .chain(set(&[
            "previews/pr-1/index.html",
            "previews/pr-1/assets/anchors.json",
            "data/previews/pr-1.json",
            "assets/2.1.0/gherkin-glass.css",
            "assets/2.1.0/gherkin-glass.js",
            "assets/2.1.0/gherkin-glass.svg",
            "assets/2.1.0/THIRD-PARTY-LICENSES.txt",
        ]))
        .collect();
    let change = StoreChange {
        target: preview,
        written: None,
        versions_page: None,
        assets: assets("2.1.0"),
    };
    let plan = plan(&published, &files, change);
    assert!(plan.writes.is_empty());
    assert_eq!(
        plan.removes,
        ["data/previews/pr-1.json", "assets/2.1.0", "previews/pr-1"]
    );
    assert_eq!(plan.last_marker, two_versions_files());
}

#[test]
fn the_versions_once_changed_are_every_version_but_the_one_removed() {
    let store = two_versions();
    assert_eq!(
        versions_once_changed(&store, &version("v2"), false),
        [name("main"), name("v1.0.0"), name("v2")]
    );
    assert_eq!(
        versions_once_changed(&store, &version("main"), true),
        [name("v1.0.0")]
    );
    assert_eq!(
        versions_once_changed(&store, &BuildTarget::Site, false),
        [name("main"), name("v1.0.0")]
    );
}
