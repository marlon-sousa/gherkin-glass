use super::*;
use crate::test_support::name;

fn feature(page: &str, object: &str) -> PublishedFeature {
    PublishedFeature {
        page: page.to_string(),
        object: object.to_string(),
    }
}

fn versions() -> BTreeMap<VersionName, Vec<PublishedFeature>> {
    BTreeMap::from([
        (name("main"), vec![feature("b", "o2"), feature("a/x", "o1")]),
        (
            name("v1.0.0"),
            vec![
                feature("b", "o3"),
                feature("a/x", "o1"),
                feature("a-b/x", "o5"),
            ],
        ),
        (name("1.0.0"), vec![feature("a", "o4")]),
        (name("v2.0.0-rc.1"), vec![feature("b", "o2")]),
    ])
}

// Rule 253: the three groups of the versions page, each version number's precedence, and what
// the versions page is written with.
#[test]
fn the_list_of_versions_holds_its_groups_and_their_precedence() {
    let list = String::from_utf8(versions_json(&versions(), &PageOptions::default(), None))
        .expect("UTF-8");
    assert_eq!(
        list,
        format!(
            r##"{{"format":1,"development":["main"],"releases":["1.0.0","v1.0.0"],"pre_releases":["v2.0.0-rc.1"],"precedence":[["v2.0.0-rc.1"],["1.0.0","v1.0.0"]],"versions_page":{{"ghg":"{GHG_VERSION}","title":"Feature files","accent_light":"#0b4f9c","accent_dark":"#8cc4ff","front_page":null}}}}"##
        )
    );
}

// Rule 254: each page by name, folder by folder, so `a/x` before `a-b/x`, each of its objects
// by name, with the versions that have it in the order of the list of versions.
#[test]
fn the_index_of_pages_lists_each_page_s_objects_and_versions() {
    let index = String::from_utf8(pages_json(&versions())).expect("UTF-8");
    assert_eq!(
        index,
        r#"{"format":1,"pages":[{"page":"a","objects":[{"object":"o4","versions":["1.0.0"]}]},{"page":"a/x","objects":[{"object":"o1","versions":["main","v1.0.0"]}]},{"page":"a-b/x","objects":[{"object":"o5","versions":["v1.0.0"]}]},{"page":"b","objects":[{"object":"o2","versions":["main","v2.0.0-rc.1"]},{"object":"o3","versions":["v1.0.0"]}]}]}"#
    );
}
