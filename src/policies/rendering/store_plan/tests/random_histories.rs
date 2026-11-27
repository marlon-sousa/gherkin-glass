//! The store over random histories of versions and previews added, replaced and removed, each built from a suite drawn from a fixed set of features and settings.

use proptest::collection::vec;
use proptest::prelude::{Strategy, proptest};
use proptest::sample::select;

use super::*;
use crate::entities::{PageOptions, ParsedFeature};
use crate::policies::build_store;
use crate::policies::rendering::{
    DescriptorSettings, GHG_VERSION, StoreListing, marker_json, read_marker, render_store,
};
use crate::test_support::parsed_at;

/// The texts the page `a` may have: the first two differ only by comments, blank lines and
/// moved blocks, and the third by one step.
const A_TEXTS: [&str; 3] = [
    "Feature: A\n  Scenario: S\n    Given a\n",
    "# A comment.\n\nFeature: A\n\n  Scenario: S\n\n    Given a\n",
    "Feature: A\n  Scenario: S\n    Given b\n",
];
const B_TEXT: &str = "Feature: B\n  Scenario: T\n    Then c\n";
const NAMES: [&str; 3] = ["main", "v1.0.0", "v2.0.0"];

/// One command of a history: what it writes or removes, and, when it writes, the text of `a`,
/// if any, whether `b` is there, and the site's title, if any.
#[derive(Debug, Clone)]
struct Command {
    target: BuildTarget,
    removes: bool,
    a: Option<&'static str>,
    b: bool,
    title: Option<&'static str>,
}

fn command() -> impl Strategy<Value = Command> {
    (
        select(NAMES.to_vec()),
        select(vec![false, true]),
        select(vec![false, false, true]),
        select(vec![None, Some(0), Some(1), Some(2)]),
        select(vec![false, true]),
        select(vec![None, Some("Shop")]),
    )
        .prop_map(|(name, preview, removes, a, b, title)| Command {
            target: if preview {
                BuildTarget::Preview(crate::test_support::name(name))
            } else {
                BuildTarget::Version(crate::test_support::name(name))
            },
            removes,
            a: a.and_then(|text| A_TEXTS.get(text).copied()),
            b,
            title,
        })
}

/// The output folder, by its files' paths.
type Folder = BTreeMap<FilePath, Vec<u8>>;

impl Command {
    fn options(&self) -> PageOptions {
        PageOptions {
            title: self.title.map(str::to_string),
            ..PageOptions::default()
        }
    }

    /// The store's files this command renders.
    fn rendered(&self) -> StoreFiles {
        let parsed: Vec<ParsedFeature> = self
            .a
            .map(|text| parsed_at("a.feature", text))
            .into_iter()
            .chain(self.b.then(|| parsed_at("b.feature", B_TEXT)))
            .collect();
        let features: Vec<&ParsedFeature> = parsed.iter().collect();
        let options = self.options();
        render_store(
            &build_store(&features, &options),
            &DescriptorSettings {
                target: &self.target,
                options: &options,
                front_page: None,
            },
        )
    }

    /// Runs this command on `folder`, as the site writer does with the store's plan.
    fn run_on(&self, folder: &mut Folder) {
        let files: BTreeSet<FilePath> = folder.keys().cloned().collect();
        let listed = listed(folder);
        let published = published(folder);
        let page = VersionsPageWrite {
            record: VersionsPageRecord {
                ghg: GHG_VERSION.to_string(),
                title: "Shop".to_string(),
                accents: self.options().accents,
                front_page: None,
            },
            page: file(ROOT_PAGE),
        };
        let versions_page = match self.target {
            BuildTarget::Version(_) => Some(page),
            BuildTarget::Site | BuildTarget::Preview(_) => None,
        };
        let written = (!self.removes).then(|| WrittenTarget {
            store: self.rendered(),
            shell: file(&shell_path(&self.target)),
            anchor_map: file(&anchor_map_path(&self.target)),
            ghg: GHG_VERSION.to_string(),
        });
        let change = StoreChange {
            target: self.target.clone(),
            written,
            versions_page,
            assets: assets(GHG_VERSION),
        };
        let before = FolderBefore {
            listed: &listed,
            found: &files,
            published: &published,
        };
        let plan = plan_store(&before, change);
        folder.insert(OutputMarker::FILE.into(), marker_json(&plan.first_marker));
        folder.extend(
            plan.writes
                .iter()
                .map(|file| (file.path(), file.contents.to_vec())),
        );
        folder.retain(|path, _| !plan.removes.iter().any(|removed| is_in(path, removed)));
        folder.insert(OutputMarker::FILE.into(), marker_json(&plan.last_marker));
    }
}

/// Whether `path` is `removed` or is in the folder `removed`.
fn is_in(path: &str, removed: &str) -> bool {
    path == removed || path.starts_with(&format!("{removed}/"))
}

/// What the marker of `folder` lists.
fn listed(folder: &Folder) -> BTreeSet<FilePath> {
    let Some(bytes) = folder.get(OutputMarker::FILE) else {
        return BTreeSet::new();
    };
    match read_marker(&String::from_utf8_lossy(bytes)) {
        Some(OutputMarker::Store { files, .. }) => files,
        _ => panic!("the marker is the store's"),
    }
}

/// What the store of `folder` holds, read from its descriptors and its list of versions.
fn published(folder: &Folder) -> PublishedStore {
    let listing = StoreListing::of(&folder.keys().cloned().collect());
    let texts: Vec<String> = listing
        .to_read()
        .filter_map(|path| folder.get(path))
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();
    listing.published(&texts).expect("every descriptor")
}

/// Whether `path` is one of the files of `target`: its descriptor, or a file in its folder.
fn is_of(path: &str, target: &BuildTarget) -> bool {
    path == descriptor_path(target)
        || target
            .folder()
            .is_some_and(|folder| path.starts_with(&format!("{folder}/")))
}

/// Every hash the object of the page `a` of `target` holds, in its order, or `None` when the
/// target has no such page.
fn hashes_of_a(folder: &Folder, target: &BuildTarget) -> Option<Vec<String>> {
    let store = published(folder);
    let object = store
        .descriptors
        .get(target)?
        .features
        .iter()
        .find(|feature| feature.page == "a")?
        .object
        .clone();
    let bytes = folder.get(&object_path(&object))?;
    let value: serde_json::Value = serde_json::from_slice(bytes).expect("an object is JSON");
    let mut hashes = Vec::new();
    collect_hashes(&value, &mut hashes);
    Some(hashes)
}

fn collect_hashes(value: &serde_json::Value, hashes: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(members) => {
            members
                .iter()
                .for_each(|(key, member)| match (key.as_str(), member.as_str()) {
                    ("hash", Some(hash)) => hashes.push(hash.to_string()),
                    _ => collect_hashes(member, hashes),
                })
        }
        serde_json::Value::Array(items) => {
            items.iter().for_each(|item| collect_hashes(item, hashes))
        }
        _ => {}
    }
}

/// Whether two of `A_TEXTS` say the same of their blocks: only the last changes a step.
fn same_blocks(a: &str, b: &str) -> bool {
    let changed = A_TEXTS.last().copied();
    (Some(a) == changed) == (Some(b) == changed)
}

/// Checks the store of `folder` after any command.
fn assert_store(folder: &Folder) {
    let store = published(folder);
    let referenced: BTreeSet<String> = store
        .descriptors
        .values()
        .flat_map(|descriptor| &descriptor.features)
        .map(|feature| feature.object.clone())
        .collect();
    // Rule 255: the objects are exactly those referenced.
    assert_eq!(store.objects, referenced);
    // Rule 38: the marker lists every file there but itself, and nothing else.
    let mut there: BTreeSet<FilePath> = folder.keys().cloned().collect();
    there.remove(OutputMarker::FILE);
    assert_eq!(listed(folder), there);
    // Rules 253 and 257: the files of the versions are there only beside a version.
    let versions = versions_of(&store.descriptors);
    for path in [VERSIONS_LIST, PAGES_INDEX, ROOT_PAGE] {
        assert_eq!(folder.contains_key(path), !versions.is_empty(), "{path}");
    }
    if let Some(page) = &store.versions_page {
        // Rules 253 and 254: they are what the descriptors give.
        assert_eq!(
            folder.get(VERSIONS_LIST).map(Vec::as_slice),
            Some(index_data::versions_json(&versions, page).as_slice())
        );
        assert_eq!(
            folder.get(PAGES_INDEX).map(Vec::as_slice),
            Some(index_data::pages_json(&versions).as_slice())
        );
    }
}

proptest! {
    // Rules 38, 247, 253 to 258 and 261, over any history of versions and previews.
    #[test]
    fn every_history_leaves_the_store_its_descriptors_give(commands in vec(command(), 1..8)) {
        let mut folder = Folder::new();
        // The text of `a` each target built holds, and `None` for one without `a`.
        let mut texts_of_a: BTreeMap<BuildTarget, Option<&str>> = BTreeMap::new();
        for command in &commands {
            let before_command = folder.clone();
            command.run_on(&mut folder);
            assert_store(&folder);
            // No other target's files change.
            for (path, bytes) in before_command.iter().filter(|(path, _)| {
                texts_of_a.keys().any(|target| *target != command.target && is_of(path, target))
            }) {
                assert_eq!(folder.get(path), Some(bytes), "{path}");
            }
            if command.removes {
                texts_of_a.remove(&command.target);
            } else {
                texts_of_a.insert(command.target.clone(), command.a);
            }
            // Rule 261: two targets' blocks are the same exactly when their texts say the same
            // of them, a block that only moved lines included.
            for (first, first_text) in &texts_of_a {
                for (second, second_text) in &texts_of_a {
                    if let (Some(a), Some(b)) = (first_text, second_text) {
                        assert_eq!(
                            hashes_of_a(&folder, first) == hashes_of_a(&folder, second),
                            same_blocks(a, b),
                            "{first:?} and {second:?}"
                        );
                    }
                }
            }
            if !command.removes {
                // Rule 247: the target's descriptor is what building it alone gives.
                let alone = command.rendered();
                assert_eq!(
                    folder.get(&alone.descriptor.path()).map(Vec::as_slice),
                    Some(alone.descriptor.contents.as_ref())
                );
            }
            // Running the same command twice changes nothing.
            let before = folder.clone();
            command.run_on(&mut folder);
            assert_eq!(folder, before);
            // A change of setting alone makes no object.
            let retitled = Command { title: Some("Other"), ..command.clone() };
            assert_eq!(retitled.rendered().objects, command.rendered().objects);
        }
    }
}
