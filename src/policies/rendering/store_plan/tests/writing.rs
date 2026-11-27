//! The store plan of a target written into the output folder.

use super::*;

// Rules 38, 39, 252 and 263: into an empty folder, the objects, the assets, the shell, the map of
// anchors and the descriptor, in that order, the marker listing them before and after.
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
            "assets/anchors.json",
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
            "v2.0.0/assets/anchors.json",
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
