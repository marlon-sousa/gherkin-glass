use rstest::rstest;

use super::*;
use crate::entities::PageOptions;
use crate::policies::build_store;
use crate::test_support::{name, parsed_at, set};

fn store_of(files: &[(&str, &str)]) -> SiteStore {
    let parsed: Vec<_> = files
        .iter()
        .map(|(path, text)| parsed_at(path, text))
        .collect();
    let features: Vec<_> = parsed.iter().collect();
    build_store(&features, &PageOptions::default())
}

fn rendered(files: &[(&str, &str)], version: Option<&VersionName>) -> StoreFiles {
    let settings = DescriptorSettings {
        version,
        options: &PageOptions::default(),
        front_page: Some("# Home"),
    };
    render_store(&store_of(files), &settings)
}

fn text(file: &SiteFile) -> String {
    String::from_utf8(file.contents.to_vec()).expect("UTF-8")
}

/// The hash of the block whose `hash` member follows `after` in `json`.
fn hash_after<'a>(json: &'a str, after: &str) -> &'a str {
    let start = json.find(after).expect("the block") + after.len();
    let rest = json.get(start..).expect("its members");
    let hash = rest.find("\"hash\":\"").expect("its hash") + 8;
    rest.get(hash..hash + 64).expect("64 digits")
}

// Rules 247 to 249: an object's members in their order, compact, and its feature's hash of
// rule 251.
#[test]
fn an_object_is_its_feature_as_compact_json() {
    let files = rendered(&[("a.feature", "Feature: F\n")], None);
    let object = files.objects.first().expect("an object");
    let hash = sha256_hex(br#"["feature","Feature","F","",[],[],"en"]"#);
    assert_eq!(
        text(object),
        format!(
            r#"{{"format":1,"language":"en","lang":"en","feature":{{"kind":"feature","keyword":"Feature","note":false,"name":"F","description":"","tags":[],"anchor":"","hash":"{hash}","blocks":[],"steps":[],"examples":[],"place":null,"table":null,"runs":[]}}}}"#
        )
    );
}

// Rule 247: an object is named by the SHA-256 of its bytes, and two files of the same content
// share it.
#[test]
fn an_object_is_named_by_its_bytes_and_shared() {
    let files = rendered(
        &[
            ("a.feature", "Feature: F\n"),
            ("b/c.feature", "Feature: F\n"),
        ],
        None,
    );
    let [object] = files.objects.as_slice() else {
        panic!("{:?}", files.objects.len());
    };
    let address = object_path(&sha256_hex(&object.contents));
    assert_eq!(object.address, SiteAddress::at_root(&address));
    let named: Vec<_> = files.features.iter().map(|f| f.object.as_str()).collect();
    assert_eq!(
        named,
        [sha256_hex(&object.contents), sha256_hex(&object.contents)]
    );
}

// Rule 247: only `"`, `\` and the characters below U+0020 are escaped.
#[test]
fn text_escapes_only_quotes_backslashes_and_control_characters() {
    let written = json_bytes(&"é \"a\" \\ \u{1}\t\n<&>");
    assert_eq!(written, "\"é \\\"a\\\" \\\\ \\u0001\\t\\n<&>\"".as_bytes());
}

const STEPS: &str = "\
Feature: F
  Scenario: S
    Given a
      | x | y |
";

// Rule 251: what ghg drops as it reads a file never changes a block's hash, and a step does.
#[rstest]
#[case::comments_and_blank_lines(
    "# language: en\nFeature: F\n\n  # a comment\n\n  Scenario: S\n    Given a\n      |x|  y  |\n",
    true
)]
#[case::moved_lines(
    "\n\n\nFeature: F\n\n\n  Scenario: S\n\n    Given a\n      | x | y |\n",
    true
)]
#[case::another_step("Feature: F\n  Scenario: S\n    Given b\n      | x | y |\n", false)]
fn a_scenario_s_hash_is_what_its_page_shows(#[case] other: &str, #[case] same: bool) {
    let hashes: Vec<String> = [STEPS, other]
        .iter()
        .map(|source| {
            text(
                rendered(&[("a.feature", source)], None)
                    .objects
                    .first()
                    .expect("an object"),
            )
        })
        .map(|json| hash_after(&json, "\"kind\":\"scenario\"").to_string())
        .collect();
    assert_eq!(hashes.first() == hashes.get(1), same, "{hashes:?}");
}

// Rule 251: a scenario's hash is its kind, keyword, name, description, tags and steps.
#[test]
fn a_scenario_s_hash_is_the_list_of_rule_251() {
    let json = text(
        rendered(&[("a.feature", STEPS)], None)
            .objects
            .first()
            .expect("an object"),
    );
    let listed = r#"["scenario","Scenario","S","",[],[["Given ","a",[["x","y"]],null]]]"#;
    assert_eq!(
        hash_after(&json, "\"kind\":\"scenario\""),
        sha256_hex(listed.as_bytes())
    );
}

// Rule 252: a site's descriptor holds its front page and no version; a version's the reverse.
#[rstest]
#[case::a_site(
    None,
    "data/site.json",
    r#""version":null"#,
    r##""front_page":"# Home""##
)]
#[case::a_version(
    Some("v1.0"),
    "data/versions/v1.0.json",
    r#""version":"v1.0""#,
    r#""front_page":null"#
)]
fn a_descriptor_names_its_version_and_its_front_page(
    #[case] version: Option<&str>,
    #[case] path: &str,
    #[case] named: &str,
    #[case] front_page: &str,
) {
    let version = version.map(name);
    let files = rendered(&[("a.feature", "Feature: F\n")], version.as_ref());
    assert_eq!(files.descriptor.address, SiteAddress::at_root(path));
    let descriptor = text(&files.descriptor);
    assert!(descriptor.contains(named), "{descriptor}");
    assert!(descriptor.contains(front_page), "{descriptor}");
}

// Rule 252: a descriptor's members in their order, its accents as `#` and six lower-case
// digits.
#[test]
fn a_descriptor_holds_the_build_s_settings() {
    let files = rendered(
        &[("a.feature", "@t\nFeature: F\n  Scenario: S\n    Given a\n")],
        None,
    );
    let object = files
        .features
        .first()
        .map(|f| f.object.clone())
        .expect("a feature");
    assert_eq!(
        text(&files.descriptor),
        format!(
            r##"{{"format":1,"ghg":"{GHG_VERSION}","version":null,"preview":null,"title":"Feature files","only":false,"start":{{"collapse_rules":false,"collapse_scenarios":false,"show_tree":true,"inline_backgrounds":true}},"features":[{{"path":"a.feature","page":"a","object":"{object}","lines":[2,3],"source":null}}],"tags":[{{"tag":"@t","page":"tags/t","link":null}}],"sentences":{{"given":["a"],"when":[],"then":[]}},"front_page":"# Home","settings":{{"title":null,"source_link":null,"tag_links":[],"accent_light":"#0b4f9c","accent_dark":"#8cc4ff","collapse_rules":false,"collapse_scenarios":false,"show_tree":true,"inline_backgrounds":true}}}}"##
        )
    );
}

// Rule 252: what a descriptor names is read back from it, in its store's format only.
#[test]
fn a_descriptor_s_features_are_read_back() {
    let files = rendered(&[("a.feature", "Feature: F\n")], None);
    let descriptor = text(&files.descriptor);
    assert_eq!(published_features(&descriptor), Some(files.features));
    let newer = descriptor.replacen("\"format\":1", "\"format\":2", 1);
    assert_eq!(published_features(&newer), None);
    assert_eq!(published_features("not JSON"), None);
}

#[test]
fn the_listing_finds_the_versions_descriptors_and_the_objects() {
    let listing = store_listing(&set(&[
        "data/versions/v1.0.json",
        "data/versions/bad name.json",
        "data/site.json",
        "data/objects/ab12.json",
        "data/objects/sub/cd.json",
        "data/pages.json",
    ]));
    assert_eq!(
        listing.descriptors,
        [(name("v1.0"), "data/versions/v1.0.json".to_string())]
    );
    assert_eq!(listing.objects, BTreeSet::from(["ab12".to_string()]));
}

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
            vec![feature("b", "o3"), feature("a/x", "o1")],
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

// Rule 254: each page by name, each of its objects by name, with the versions that have it in
// the order of the list of versions.
#[test]
fn the_index_of_pages_lists_each_page_s_objects_and_versions() {
    let index = String::from_utf8(pages_json(&versions())).expect("UTF-8");
    assert_eq!(
        index,
        r#"{"format":1,"pages":[{"page":"a","objects":[{"object":"o4","versions":["1.0.0"]}]},{"page":"a/x","objects":[{"object":"o1","versions":["main","v1.0.0"]}]},{"page":"b","objects":[{"object":"o2","versions":["main","v2.0.0-rc.1"]},{"object":"o3","versions":["v1.0.0"]}]}]}"#
    );
}
