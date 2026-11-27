use rstest::rstest;

use super::*;
use crate::entities::{PageOptions, VersionName};
use crate::policies::build_store;
use crate::policies::rendering::{GHG_VERSION, sha256_hex};
use crate::test_support::{name, parsed_at};

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
