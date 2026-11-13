use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};
use rstest::rstest;

use super::*;
use crate::entities::ParsedFeature;
use crate::test_support::{site_from_nothing, site_of};

/// Every file of `tree`, with ghg's own front page.
fn files_of(tree: &SiteTree, last: Option<&RenderedSite>, options: &PageOptions) -> SiteFiles {
    render_site(tree, last, options, None).expect("ghg's own front page renders")
}

/// The site of one feature named `name`.
fn built(name: &str) -> BuiltSite {
    let feature =
        Feature::parse(format!("Feature: {name}\n"), GherkinEnv::default()).expect("parses");
    let parsed = ParsedFeature::new(
        PathBuf::from("a.feature"),
        feature,
        "en".to_string(),
        BTreeMap::new(),
    );
    let options = PageOptions::default();
    files_of(&site_from_nothing(&[parsed], &options), None, &options).built
}

#[rstest]
#[case::small(1, false)]
#[case::large(LARGE_SEARCH_DOCUMENTS, true)]
fn only_search_documents_above_the_size_are_reported(
    #[case] name_length: usize,
    #[case] reported: bool,
) {
    let built = built(&"a".repeat(name_length));
    assert_eq!(built.large_search_documents.is_some(), reported);
    assert!(
        built
            .large_search_documents
            .is_none_or(|size| is_large(&size))
    );
}

/// `files` with every file that is rendered marked as kept, its contents replaced by `KEPT`.
fn marked_as_kept(files: SiteFiles) -> SiteFiles {
    let mark = |file: SiteFile| SiteFile {
        contents: Arc::from(KEPT),
        ..file
    };
    SiteFiles {
        files: files.files.into_iter().map(mark).collect(),
        root_files: files.root_files.into_iter().map(mark).collect(),
        ..files
    }
}

const KEPT: &[u8] = b"kept";

#[test]
fn a_file_whose_node_is_unchanged_is_kept_from_the_last_site_and_every_other_is_rendered() {
    let options = PageOptions::default();
    let site = |b: &str| {
        site_of(
            &[("a.feature", "Feature: A\n"), ("b/b.feature", b)],
            &options,
        )
    };
    let tree = site("Feature: B\n");
    let before = files_of(&tree, None, &options).files;
    let last = RenderedSite {
        files: marked_as_kept(files_of(&tree, None, &options)),
        tree,
    };
    let changed = site("Feature: B changed\n");
    let rendered = files_of(&changed, Some(&last), &options).files;
    let fresh = files_of(&changed, None, &options).files;
    let contents = |address: &str| {
        rendered
            .iter()
            .find(|file| file.address.as_str() == address)
            .map(|file| &*file.contents)
    };
    assert_eq!(contents("a.html"), Some(KEPT));
    assert_ne!(contents("b/b.html"), Some(KEPT));
    assert_eq!((rendered.len(), before.len()), (fresh.len(), fresh.len()));
    for ((file, fresh), before) in rendered.iter().zip(&fresh).zip(&before) {
        assert_eq!(
            (&file.address, &before.address),
            (&fresh.address, &fresh.address)
        );
        // A file that changed is never kept, and one that is not kept is rendered afresh.
        if *file.contents == *KEPT {
            assert_eq!(fresh.contents, before.contents, "{} kept", file.address);
        } else {
            assert_eq!(file.contents, fresh.contents, "{} rendered", file.address);
        }
    }
}

/// The front page's template written `text`.
fn template(text: &str) -> FrontPageTemplate {
    FrontPageTemplate {
        file: PathBuf::from("front.md"),
        text: text.to_string(),
    }
}

#[test]
fn the_home_page_of_a_site_without_versions_is_rendered_from_the_template() {
    // Rule 183: the home page takes the template, and no other folder page does.
    let options = PageOptions::default();
    let tree = site_of(&[("b/b.feature", "Feature: B\n")], &options);
    let template = template("# Our suite\n\nRead on.\n\n<!-- ghg:contents -->\n");
    let files = render_site(&tree, None, &options, Some(&template))
        .expect("the template renders")
        .files;
    let page = |address: &str| {
        files
            .iter()
            .find(|file| file.address.as_str() == address)
            .map(|file| String::from_utf8_lossy(&file.contents).into_owned())
            .unwrap_or_default()
    };
    let home = page("index.html");
    assert!(home.contains("<title>Our suite</title>"), "{home}");
    assert!(
        home.contains("<h1>Our suite</h1>\n<p>Read on.</p>\n<ul class=\"ghg-totals\">"),
        "{home}"
    );
    assert!(home.contains("<h2>Folders</h2>"), "{home}");
    let folder = page("b/index.html");
    assert!(folder.contains("<h1>b</h1>"), "{folder}");
    assert!(!folder.contains("Read on."), "{folder}");
}

#[test]
fn a_template_that_breaks_a_rule_stops_the_rendering() {
    let options = PageOptions::default();
    let tree = site_of(&[("a.feature", "Feature: A\n")], &options);
    let error = render_site(&tree, None, &options, Some(&template("# A\n")))
        .err()
        .map(|error| error.to_string());
    assert_eq!(
        error.as_deref(),
        Some(
            "front.md: has no marker <!-- ghg:contents --> on a line of its own, where ghg puts its part of the front page."
        )
    );
}
