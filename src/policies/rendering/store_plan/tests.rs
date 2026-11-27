mod random_histories;
mod removing;
mod writing;

use super::*;
use crate::entities::{Accents, StoreFiles};
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
/// descriptor, its shell and its map of anchors, by the ghg `ghg`.
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
        anchor_map: file(&anchor_map_path(target)),
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
        "main/assets/anchors.json",
        "v1.0.0/index.html",
        "v1.0.0/assets/anchors.json",
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
