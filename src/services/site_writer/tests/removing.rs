//! Removing a version or a preview from the output folder.

use super::*;

// Rule 257: a version removed takes its folder, its descriptor and the objects only it had, and
// the versions page names those left.
#[test]
fn a_version_removed_leaves_the_others_and_their_objects() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = into(&output);
    add(&site_writer, version("v1"), "a").expect("built");
    add(&site_writer, version("v2"), "b").expect("built");
    assert_eq!(remove(&site_writer, version("v1")), Ok(true));
    let paths = output.paths();
    assert!(!paths.iter().any(|path| path.contains("v1")), "{paths:?}");
    assert!(paths.contains(&"v2/index.html".to_string()), "{paths:?}");
    let objects = paths
        .iter()
        .filter(|path| path.starts_with("data/objects/"));
    assert_eq!(objects.count(), 1);
    let page = output.text("index.html").expect("the versions page");
    assert!(!page.contains("v1/index.html"), "{page}");
    let mut listed = listed(&output);
    listed.sort();
    let others: Vec<String> = paths
        .into_iter()
        .filter(|path| path != "ghg-site.json")
        .collect();
    assert_eq!(listed, others);
}

// Rule 257: the last version removed leaves only the marker, which lists nothing.
#[test]
fn the_last_version_removed_leaves_an_empty_store() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = into(&output);
    add(&site_writer, version("v1"), "a").expect("built");
    assert_eq!(remove(&site_writer, version("v1")), Ok(true));
    assert_eq!(output.paths(), ["ghg-site.json"]);
    assert!(listed(&output).is_empty());
}

// Rule 258: a preview removed takes its folder and leaves the site beside it.
#[test]
fn a_preview_removed_leaves_the_site_beside_it() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = into(&output);
    add(&site_writer, BuildTarget::Site, "a").expect("built");
    let site = output.paths();
    add(&site_writer, preview("pr-1"), "b").expect("built");
    assert!(
        output
            .paths()
            .contains(&"previews/pr-1/index.html".to_string())
    );
    assert_eq!(remove(&site_writer, preview("pr-1")), Ok(true));
    assert_eq!(output.paths(), site);
}

// Rules 257 and 258: what the folder does not hold is not removed, and nothing is written.
#[test]
fn what_is_not_there_is_not_removed() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = into(&output);
    add(&site_writer, version("v1"), "a").expect("built");
    let before = output.paths();
    assert_eq!(remove(&site_writer, version("v2")), Ok(false));
    assert_eq!(remove(&site_writer, preview("v1")), Ok(false));
    assert_eq!(output.paths(), before);
}

// Rules 38 and 125: what cannot be removed is reported with its path.
#[test]
fn what_cannot_be_removed_is_reported_with_its_path() {
    let output = Arc::new(MemoryOutput {
        fails: Some("v1"),
        ..MemoryOutput::default()
    });
    let site_writer = into(&output);
    add(&site_writer, version("v1"), "a").expect("built");
    add(&site_writer, version("v2"), "b").expect("built");
    let Err(BuildError::Output(error)) = remove(&site_writer, version("v1")) else {
        panic!("removed");
    };
    assert_eq!(error.to_string(), "site/v1: cannot be written: disk full");
}
