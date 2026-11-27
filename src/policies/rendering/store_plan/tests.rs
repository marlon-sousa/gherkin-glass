use super::*;
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

/// What a build renders: an object for each of `objects`, each the feature `a`, and a
/// descriptor at `descriptor`.
fn built(objects: &[&str], descriptor: &str) -> StoreFiles {
    StoreFiles {
        objects: objects
            .iter()
            .map(|name| file(&object_path(name)))
            .collect(),
        descriptor: file(descriptor),
        features: objects.iter().map(|name| feature("a", name)).collect(),
    }
}

fn written(plan: &StorePlan) -> Vec<&str> {
    plan.writes
        .iter()
        .map(|file| file.address.as_str())
        .collect()
}

/// The versions `main`, of the objects `shared` and `old`, and `v1.0.0`, of `shared`.
fn published() -> PublishedStore {
    PublishedStore {
        versions: BTreeMap::from([
            (
                name("main"),
                vec![feature("a", "shared"), feature("b", "old")],
            ),
            (name("v1.0.0"), vec![feature("a", "shared")]),
        ]),
        objects: ["old", "shared"].map(str::to_string).into(),
    }
}

#[test]
fn a_site_without_versions_writes_its_objects_and_its_descriptor() {
    let plan = plan_store(
        &PublishedStore::default(),
        built(&["one"], "data/site.json"),
        None,
        &PageOptions::default(),
        None,
    );
    assert_eq!(written(&plan), ["data/objects/one.json", "data/site.json"]);
    assert!(plan.removes.is_empty());
    assert_eq!(
        plan.files,
        set(&["data/objects/one.json", "data/site.json"])
    );
}

// Rule 247: an object already in the store is not written again.
#[test]
fn a_version_writes_only_the_objects_the_store_lacks() {
    let plan = plan_store(
        &published(),
        built(&["shared", "new"], "data/versions/v2.0.0.json"),
        Some(&name("v2.0.0")),
        &PageOptions::default(),
        None,
    );
    assert_eq!(
        written(&plan),
        [
            "data/objects/new.json",
            "data/versions/v2.0.0.json",
            "data/versions.json",
            "data/pages.json"
        ]
    );
    assert!(plan.removes.is_empty());
}

// Rule 255: a replaced version leaves the objects no descriptor names, which are removed, and
// the store's files are every descriptor's, the objects they name and both indexes.
#[test]
fn a_replaced_version_removes_what_nothing_references() {
    let plan = plan_store(
        &published(),
        built(&["new"], "data/versions/main.json"),
        Some(&name("main")),
        &PageOptions::default(),
        None,
    );
    assert_eq!(plan.removes, set(&["data/objects/old.json"]));
    assert_eq!(
        plan.files,
        set(&[
            "data/objects/new.json",
            "data/objects/shared.json",
            "data/pages.json",
            "data/versions.json",
            "data/versions/main.json",
            "data/versions/v1.0.0.json",
        ])
    );
}

// Rule 254: the index of pages is written from every version's descriptor, the new one in place
// of the one it replaces.
#[test]
fn the_index_of_pages_holds_the_new_version_in_place_of_the_old() {
    let plan = plan_store(
        &published(),
        built(&["new"], "data/versions/main.json"),
        Some(&name("main")),
        &PageOptions::default(),
        None,
    );
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

#[test]
fn the_store_s_paths_are_what_a_build_may_write() {
    assert_eq!(
        store_paths(&built(&["one"], "data/versions/main.json")),
        [
            "data/objects/one.json",
            "data/versions/main.json",
            "data/versions.json",
            "data/pages.json"
        ]
    );
}
