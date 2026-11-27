mod random_histories;

use super::*;
use crate::entities::Accents;
use crate::test_support::{name, set};

fn feature(page: &str, object: &str) -> PublishedFeature {
    PublishedFeature {
        page: page.to_string(),
        object: object.to_string(),
    }
}

fn file(path: &str) -> SiteFile {
    json_file(path, b"{}".to_vec())
}

fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(name(text))
}

/// What a build of `target` renders: an object for each of `objects`, each the page `a`, its
/// descriptor and its shell, by the ghg `ghg`.
fn written(target: &BuildTarget, objects: &[&str], ghg: &str) -> WrittenTarget {
    WrittenTarget {
        store: StoreFiles {
            objects: objects
                .iter()
                .map(|name| file(&object_path(name)))
                .collect(),
            descriptor: file(&descriptor_path(target)),
            features: objects.iter().map(|name| feature("a", name)).collect(),
            large_site_data: None,
        },
        shell: file(&shell_path(target)),
        ghg: ghg.to_string(),
    }
}

fn record(ghg: &str) -> VersionsPageRecord {
    VersionsPageRecord {
        ghg: ghg.to_string(),
        title: "Shop".to_string(),
        accents: Accents::DEFAULT,
        front_page: None,
    }
}

fn versions_page(ghg: &str) -> VersionsPageWrite {
    VersionsPageWrite {
        record: record(ghg),
        page: file(ROOT_PAGE),
    }
}

fn assets(ghg: &str) -> Vec<SiteFile> {
    asset_paths(ghg).map(|path| file(&path)).collect()
}

fn paths(files: &[SiteFile]) -> Vec<&str> {
    files.iter().map(|file| file.address.as_str()).collect()
}

/// The versions `main`, of the objects `shared` and `old`, and `v1.0.0`, of `shared`, written
/// by the ghg 2.0.0, with the versions page.
fn two_versions() -> PublishedStore {
    PublishedStore {
        descriptors: BTreeMap::from([
            (
                version("main"),
                PublishedDescriptor {
                    ghg: "2.0.0".into(),
                    features: vec![feature("a", "shared"), feature("b", "old")],
                },
            ),
            (
                version("v1.0.0"),
                PublishedDescriptor {
                    ghg: "2.0.0".into(),
                    features: vec![feature("a", "shared")],
                },
            ),
        ]),
        objects: ["old", "shared"].map(str::to_string).into(),
        versions_page: Some(record("2.0.0")),
    }
}

/// Every file of `two_versions()`, as its marker lists it and the folder holds it.
fn two_versions_files() -> BTreeSet<FilePath> {
    set(&[
        "index.html",
        "main/index.html",
        "v1.0.0/index.html",
        "data/versions/main.json",
        "data/versions/v1.0.0.json",
        "data/objects/shared.json",
        "data/objects/old.json",
        "data/versions.json",
        "data/pages.json",
        "assets/2.0.0/gherkin-glass.css",
        "assets/2.0.0/gherkin-glass.js",
        "assets/2.0.0/gherkin-glass.svg",
        "assets/2.0.0/THIRD-PARTY-LICENSES.txt",
    ])
}

fn plan(published: &PublishedStore, files: &BTreeSet<FilePath>, change: StoreChange) -> StorePlan {
    let before = FolderBefore {
        listed: files,
        found: files,
        published,
    };
    plan_store(&before, change)
}

// Rules 38, 39 and 252: into an empty folder, the objects, the assets, the shell and the
// descriptor, in that order, the marker listing them before and after.
#[test]
fn a_site_into_an_empty_folder_writes_its_store_its_assets_and_its_shell() {
    let change = StoreChange {
        target: BuildTarget::Site,
        written: Some(written(&BuildTarget::Site, &["one"], "2.0.0")),
        versions_page: None,
        assets: assets("2.0.0"),
    };
    let plan = plan(&PublishedStore::default(), &BTreeSet::new(), change);
    assert_eq!(
        paths(&plan.writes),
        [
            "data/objects/one.json",
            "assets/2.0.0/gherkin-glass.css",
            "assets/2.0.0/gherkin-glass.js",
            "assets/2.0.0/gherkin-glass.svg",
            "assets/2.0.0/THIRD-PARTY-LICENSES.txt",
            "index.html",
            "data/site.json",
        ]
    );
    assert!(plan.removes.is_empty());
    let every: BTreeSet<FilePath> = plan.writes.iter().map(SiteFile::path).collect();
    assert_eq!(plan.first_marker, every);
    assert_eq!(plan.last_marker, every);
}

// Rules 122, 126 and 247: an object or an asset already there is not written again, and the
// versions page, the list of versions and the index of pages are written after the version.
#[test]
fn a_version_writes_only_what_the_store_lacks_then_the_files_of_the_versions() {
    let change = StoreChange {
        target: version("v2.0.0"),
        written: Some(written(&version("v2.0.0"), &["shared", "new"], "2.0.0")),
        versions_page: Some(versions_page("2.0.0")),
        assets: assets("2.0.0"),
    };
    let plan = plan(&two_versions(), &two_versions_files(), change);
    assert_eq!(
        paths(&plan.writes),
        [
            "data/objects/new.json",
            "v2.0.0/index.html",
            "data/versions/v2.0.0.json",
            "data/versions.json",
            "data/pages.json",
            "index.html",
        ]
    );
    assert!(plan.removes.is_empty());
}

// Rule 255: a replaced version leaves the objects no descriptor names, which are removed after
// everything is written, the marker listing them only until then.
#[test]
fn a_replaced_version_removes_what_nothing_references() {
    let change = StoreChange {
        target: version("main"),
        written: Some(written(&version("main"), &["new"], "2.0.0")),
        versions_page: Some(versions_page("2.0.0")),
        assets: assets("2.0.0"),
    };
    let plan = plan(&two_versions(), &two_versions_files(), change);
    assert_eq!(plan.removes, ["data/objects/old.json"]);
    assert!(plan.first_marker.contains("data/objects/old.json"));
    assert!(!plan.last_marker.contains("data/objects/old.json"));
    assert!(plan.last_marker.contains("data/objects/new.json"));
}

// Rule 254: the index of pages is written from every version's descriptor, the new one in place
// of the one it replaces.
#[test]
fn the_index_of_pages_holds_the_new_version_in_place_of_the_old() {
    let change = StoreChange {
        target: version("main"),
        written: Some(written(&version("main"), &["new"], "2.0.0")),
        versions_page: Some(versions_page("2.0.0")),
        assets: assets("2.0.0"),
    };
    let plan = plan(&two_versions(), &two_versions_files(), change);
    let index = plan
        .writes
        .iter()
        .find(|file| file.address.as_str() == PAGES_INDEX)
        .expect("the index");
    let expected = pages_json(&BTreeMap::from([
        (name("main"), vec![feature("a", "new")]),
        (name("v1.0.0"), vec![feature("a", "shared")]),
    ]));
    assert_eq!(index.contents.as_ref(), expected.as_slice());
}

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
