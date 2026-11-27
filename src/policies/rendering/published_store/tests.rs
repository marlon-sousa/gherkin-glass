use super::*;
use crate::test_support::{name, set};

fn descriptor(format: u32, object: &str) -> String {
    format!(
        r#"{{"format":{format},"ghg":"2.0.0","features":[{{"path":"a.feature","page":"a","object":"{object}","lines":[1]}}]}}"#
    )
}

fn listing() -> StoreListing {
    StoreListing::of(&set(&[
        "data/versions/v1.0.json",
        "data/versions/main.json",
        "data/versions/bad name.json",
        "data/site.json",
        "data/objects/ab12.json",
        "data/objects/sub/cd.json",
        "data/pages.json",
    ]))
}

#[test]
fn the_listing_finds_the_versions_descriptors_and_the_objects() {
    let listing = listing();
    let paths: Vec<_> = listing.descriptors().map(String::as_str).collect();
    assert_eq!(
        paths,
        ["data/versions/main.json", "data/versions/v1.0.json"]
    );
    assert_eq!(listing.objects, BTreeSet::from(["ab12".to_string()]));
}

// Rule 255: what each descriptor names is read from it.
#[test]
fn the_store_holds_what_each_descriptor_names() {
    let published = listing()
        .published(&[descriptor(1, "ab12"), descriptor(1, "ef34")])
        .expect("descriptors");
    let named: Vec<_> = published
        .versions
        .iter()
        .flat_map(|(version, features)| {
            features
                .iter()
                .map(move |f| (version.as_str(), f.object.as_str()))
        })
        .collect();
    assert_eq!(named, [("main", "ab12"), ("v1.0", "ef34")]);
    assert_eq!(published.versions.get(&name("main")).map(Vec::len), Some(1));
}

#[test]
fn a_descriptor_of_another_format_not_json_or_missing_is_none() {
    assert_eq!(
        listing().published(&[descriptor(1, "a"), descriptor(2, "b")]),
        None
    );
    assert_eq!(
        listing().published(&[descriptor(1, "a"), "{".to_string()]),
        None
    );
    assert_eq!(listing().published(&[descriptor(1, "a")]), None);
}
