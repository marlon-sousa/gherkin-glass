use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};
use rstest::rstest;

use super::*;
use crate::entities::ParsedFeature;
use crate::test_support::{site_from_nothing, site_of};

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
    render_site(&site_from_nothing(&[parsed], &options), None, &options).built
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
    let before = render_site(&tree, None, &options).files;
    let last = RenderedSite {
        files: marked_as_kept(render_site(&tree, None, &options)),
        tree,
    };
    let changed = site("Feature: B changed\n");
    let rendered = render_site(&changed, Some(&last), &options).files;
    let fresh = render_site(&changed, None, &options).files;
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
