use super::*;
use crate::test_support::{name, set};

fn descriptor(format: u32, object: &str) -> String {
    format!(
        r#"{{"format":{format},"ghg":"2.0.0","version":null,"features":[{{"path":"a.feature","page":"a","object":"{object}","lines":[1]}}]}}"#
    )
}

const VERSIONS_TEXT: &str = r##"{"format":1,"development":["main"],"releases":[],"pre_releases":[],"precedence":[],"versions_page":{"ghg":"2.0.0","title":"T","accent_light":"#0b4f9c","accent_dark":"#8cc4ff","front_page":null}}"##;

fn listing() -> StoreListing {
    StoreListing::of(&set(&[
        "data/versions/v1.0.json",
        "data/versions/main.json",
        "data/versions/bad name.json",
        "data/previews/pr-1.json",
        "data/site.json",
        "data/objects/ab12.json",
        "data/objects/sub/cd.json",
        "data/pages.json",
        "data/versions.json",
    ]))
}

#[test]
fn the_listing_finds_every_descriptor_the_objects_and_the_list_of_versions() {
    let listing = listing();
    assert_eq!(
        listing.to_read().collect::<Vec<_>>(),
        [
            "data/previews/pr-1.json",
            "data/site.json",
            "data/versions/main.json",
            "data/versions/v1.0.json",
            "data/versions.json",
        ]
    );
    assert_eq!(listing.objects, BTreeSet::from(["ab12".to_string()]));
}

// Rule 255: what each descriptor names is read from it.
#[test]
fn the_store_holds_what_each_descriptor_names() {
    let texts = [
        descriptor(1, "p"),
        descriptor(1, "s"),
        descriptor(1, "ab12"),
        descriptor(1, "ef34"),
        VERSIONS_TEXT.to_string(),
    ];
    let published = listing().published(&texts).expect("descriptors");
    let named: Vec<_> = published
        .descriptors
        .iter()
        .map(|(target, descriptor)| (target.clone(), descriptor.features[0].object.as_str()))
        .collect();
    assert_eq!(
        named,
        [
            (BuildTarget::Site, "s"),
            (BuildTarget::Version(name("main")), "ab12"),
            (BuildTarget::Version(name("v1.0")), "ef34"),
            (BuildTarget::Preview(name("pr-1")), "p"),
        ]
    );
    let page = published.versions_page.expect("the versions page");
    assert_eq!((page.ghg.as_str(), page.title.as_str()), ("2.0.0", "T"));
    assert_eq!(page.accents, Accents::DEFAULT);
}

#[test]
fn a_file_of_another_format_not_json_or_missing_is_none() {
    let good = || descriptor(1, "a");
    let texts = |first: String, last: &str| [first, good(), good(), good(), last.to_string()];
    assert_eq!(
        listing().published(&texts(descriptor(2, "b"), VERSIONS_TEXT)),
        None
    );
    assert_eq!(
        listing().published(&texts("{".to_string(), VERSIONS_TEXT)),
        None
    );
    assert_eq!(
        listing().published(&texts(good(), &VERSIONS_TEXT.replace(":1,", ":2,"))),
        None
    );
    assert_eq!(listing().published(&[good()]), None);
}
