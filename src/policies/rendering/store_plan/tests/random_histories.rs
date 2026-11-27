//! The store over random histories of versions added and replaced, each built from a suite drawn from a fixed set of features and settings.

use std::collections::BTreeMap;

use proptest::collection::vec;
use proptest::prelude::{Strategy, proptest};
use proptest::sample::select;

use super::*;
use crate::entities::ParsedFeature;
use crate::policies::build_store;
use crate::policies::rendering::{DescriptorSettings, StoreListing, render_store};
use crate::test_support::parsed_at;

/// The texts the page `a` may have: the first two differ only by comments, blank lines and
/// moved blocks, and the third by one step.
const A_TEXTS: [&str; 3] = [
    "Feature: A\n  Scenario: S\n    Given a\n",
    "# A comment.\n\nFeature: A\n\n  Scenario: S\n\n    Given a\n",
    "Feature: A\n  Scenario: S\n    Given b\n",
];
const B_TEXT: &str = "Feature: B\n  Scenario: T\n    Then c\n";
const VERSIONS: [&str; 3] = ["main", "v1.0.0", "v2.0.0"];

/// One build of a history: the version, the text of `a`, if any, whether `b` is there, and the
/// site's title, if any.
#[derive(Debug, Clone)]
struct Build {
    version: &'static str,
    a: Option<&'static str>,
    b: bool,
    title: Option<&'static str>,
}

fn build() -> impl Strategy<Value = Build> {
    (
        select(VERSIONS.to_vec()),
        select(vec![None, Some(0), Some(1), Some(2)]),
        select(vec![false, true]),
        select(vec![None, Some("Shop")]),
    )
        .prop_map(|(version, a, b, title)| Build {
            version,
            a: a.and_then(|text| A_TEXTS.get(text).copied()),
            b,
            title,
        })
}

/// The output folder's store, by its files' paths.
type Folder = BTreeMap<FilePath, Vec<u8>>;

impl Build {
    fn options(&self) -> PageOptions {
        PageOptions {
            title: self.title.map(str::to_string),
            ..PageOptions::default()
        }
    }

    /// The store's files this build renders.
    fn rendered(&self) -> StoreFiles {
        let parsed: Vec<ParsedFeature> = self
            .a
            .map(|text| parsed_at("a.feature", text))
            .into_iter()
            .chain(self.b.then(|| parsed_at("b.feature", B_TEXT)))
            .collect();
        let features: Vec<&ParsedFeature> = parsed.iter().collect();
        let version = name(self.version);
        let options = self.options();
        render_store(
            &build_store(&features, &options),
            &DescriptorSettings {
                version: Some(&version),
                options: &options,
                front_page: None,
            },
        )
    }

    /// Builds this version into `folder`, as the site writer does with the store's plan.
    fn build_into(&self, folder: &mut Folder) {
        let plan = plan_store(
            &published(folder),
            self.rendered(),
            Some(&name(self.version)),
            &self.options(),
            None,
        );
        folder.extend(
            plan.writes
                .iter()
                .map(|file| (file.path(), file.contents.to_vec())),
        );
        folder.retain(|path, _| !plan.removes.contains(path));
        assert_eq!(folder.keys().cloned().collect::<BTreeSet<_>>(), plan.files);
    }
}

/// What the store of `folder` holds, read from its descriptors.
fn published(folder: &Folder) -> PublishedStore {
    let listing = StoreListing::of(&folder.keys().cloned().collect());
    let texts: Vec<String> = listing
        .descriptors()
        .filter_map(|path| folder.get(path))
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();
    listing.published(&texts).expect("every descriptor")
}

/// The objects every descriptor of `store` names.
fn referenced(store: &PublishedStore) -> BTreeSet<String> {
    store
        .versions
        .values()
        .flatten()
        .map(|feature| feature.object.clone())
        .collect()
}

/// Checks the store of `folder` once `last` is built into it.
fn assert_store_after(folder: &Folder, last: &Build) {
    let store = published(folder);
    // Rule 255: the objects are exactly those referenced.
    assert_eq!(store.objects, referenced(&store));
    // Rule 247: the version's descriptor and objects are what building it alone gives.
    let alone = last.rendered();
    assert_eq!(
        folder.get(&alone.descriptor.path()).map(Vec::as_slice),
        Some(alone.descriptor.contents.as_ref())
    );
    let objects_alone: Vec<_> = alone
        .objects
        .iter()
        .map(|file| folder.get(&file.path()))
        .collect();
    assert!(objects_alone.iter().all(Option::is_some));
    // Rules 253 and 254: the list of versions and the index of pages are what the descriptors
    // give.
    assert_eq!(
        folder.get(VERSIONS_LIST).map(Vec::as_slice),
        Some(index_data::versions_json(&store.versions, &last.options(), None).as_slice())
    );
    assert_eq!(
        folder.get(PAGES_INDEX).map(Vec::as_slice),
        Some(index_data::pages_json(&store.versions).as_slice())
    );
}

proptest! {
    // Rules 247 and 253 to 255, over any history of versions added and replaced.
    #[test]
    fn every_history_leaves_the_store_its_descriptors_give(builds in vec(build(), 1..8)) {
        let mut folder = Folder::new();
        for build in &builds {
            build.build_into(&mut folder);
            assert_store_after(&folder, build);
            // Building the same version twice changes nothing.
            let before = folder.clone();
            build.build_into(&mut folder);
            assert_eq!(folder, before);
            // A change of setting alone makes no object.
            let retitled = Build { title: Some("Other"), ..build.clone() };
            assert_eq!(
                retitled.rendered().objects,
                build.rendered().objects
            );
        }
    }
}
