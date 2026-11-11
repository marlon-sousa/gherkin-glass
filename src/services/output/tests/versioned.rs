//! The output service's versioned builds, over the fakes.

use super::*;
use fakes::*;

#[test]
fn a_versioned_build_replaces_its_folder_and_the_page_and_records_every_version() {
    let folders = FakeFolders::holding(two_versions());
    let writer = Arc::new(FakeWriter::default());
    build(
        vec!["a.feature"],
        Some("v1.0.0"),
        folders.clone(),
        writer.clone(),
    )
    .expect("built");
    assert_eq!(
        folders.done(),
        [
            "remove site/v1.0.0",
            "remove site/index.html",
            "record site"
        ]
    );
    let written = written(&writer);
    assert_eq!(
        written.first().map(String::as_str),
        Some("site/v1.0.0/a.html")
    );
    assert!(written.contains(&"site/index.html".to_string()));
    assert!(written.contains(&"site/assets/gherkin-glass.css".to_string()));
    let Some(Record::Versions { page, versions }) = folders.recorded.lock().expect("lock").clone()
    else {
        panic!("no versions recorded");
    };
    assert!(page.contains("index.html") && page.contains("assets/gherkin-glass.css"));
    assert_eq!(versions.get(&name("main")), Some(&set(&["index.html"])));
    assert!(versions[&name("v1.0.0")].contains("a.html"));
}

#[test]
fn input_with_an_error_removes_nothing_and_writes_nothing() {
    let folders = FakeFolders::holding(two_versions());
    let writer = Arc::new(FakeWriter::default());
    let outcome = build(
        vec!["bad.feature"],
        Some("v1.0.0"),
        folders.clone(),
        writer.clone(),
    );
    assert!(matches!(outcome, Err(BuildError::Input(_))));
    assert!(folders.done().is_empty());
    assert!(written(&writer).is_empty());
}

#[test]
fn a_page_that_cannot_be_written_leaves_the_versions_page_unwritten() {
    let folders = FakeFolders::holding(OutputContents::Missing);
    let writer = Arc::new(FakeWriter {
        fails: Some("b.html"),
        ..FakeWriter::default()
    });
    let outcome = build(
        vec!["a.feature", "b.feature"],
        Some("v1.0.0"),
        folders.clone(),
        writer.clone(),
    );
    assert!(matches!(outcome, Err(BuildError::Input(_))));
    assert_eq!(folders.done(), ["remove site/v1.0.0", "record site"]);
    assert_eq!(written(&writer), ["site/v1.0.0/a.html"]);
}
