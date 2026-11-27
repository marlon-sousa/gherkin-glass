//! The site writer's versioned builds, over the fakes.

use super::*;
use fakes::*;
use std::collections::BTreeMap;

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
            "record site",
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
    let Some(Record::Versions { page, versions, .. }) =
        folders.recorded.lock().expect("lock").clone()
    else {
        panic!("no versions recorded");
    };
    assert!(page.contains("index.html") && page.contains("assets/gherkin-glass.css"));
    assert_eq!(versions.get(&name("main")), Some(&set(&["index.html"])));
    assert!(versions[&name("v1.0.0")].contains("a.html"));
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
    assert!(
        matches!(&outcome, Err(BuildError::Output(error)) if error.to_string() == "site/v1.0.0/b.html: cannot be written: disk full"),
        "{outcome:?}"
    );
    assert_eq!(folders.done(), ["remove site/v1.0.0", "record site"]);
    assert_eq!(written(&writer), ["site/v1.0.0/a.html"]);
}

/// The versions `main` and `v1.0.0` with their store, each naming an object of its own.
fn two_stored_versions() -> Arc<FakeFolders> {
    let store = [
        "data/objects/gone.json",
        "data/objects/kept.json",
        "data/versions/main.json",
        "data/versions/v1.0.0.json",
    ];
    let OutputContents::Folder {
        mut files,
        mut folders,
        record: Some(Record::Versions { page, versions, .. }),
        ..
    } = two_versions()
    else {
        panic!("two versions");
    };
    files.extend(set(&store));
    folders.extend(set(&["data", "data/objects", "data/versions"]));
    let contents = OutputContents::Folder {
        files,
        folders,
        kept: BTreeMap::new(),
        record: Some(Record::Versions {
            page,
            versions,
            data: set(&store),
        }),
    };
    let descriptor =
        |object: &str| format!(r#"{{"format":1,"features":[{{"page":"a","object":"{object}"}}]}}"#);
    Arc::new(FakeFolders {
        texts: vec![
            ("data/versions/main.json", descriptor("kept")),
            ("data/versions/v1.0.0.json", descriptor("gone")),
        ],
        ..Arc::into_inner(FakeFolders::holding(contents)).expect("one owner")
    })
}

// Rules 38 and 255: the marker lists what is about to be there and what is to be removed, the
// store is written, what nothing references is removed, and the marker is written again
// without it.
#[test]
fn a_replaced_version_removes_the_object_nothing_references_between_two_markers() {
    let folders = two_stored_versions();
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
            "record site",
            "remove site/data/objects/gone.json",
            "record site"
        ]
    );
    let written = written(&writer);
    let unwritten: Vec<_> = [
        "site/data/versions/v1.0.0.json",
        "site/data/versions.json",
        "site/data/pages.json",
    ]
    .into_iter()
    .filter(|file| !written.contains(&file.to_string()))
    .collect();
    assert!(unwritten.is_empty(), "{unwritten:?} not in {written:?}");
    let Some(Record::Versions { data, .. }) = folders.recorded.lock().expect("lock").clone() else {
        panic!("no versions recorded");
    };
    assert!(data.contains("data/objects/kept.json") && !data.contains("data/objects/gone.json"));
    assert!(data.contains("data/versions/main.json"));
}

// Rule 252: a descriptor that cannot be read is no site ghg built.
#[test]
fn a_descriptor_that_cannot_be_parsed_stops_the_build() {
    let folders = two_stored_versions();
    let unparsed = Arc::new(FakeFolders {
        texts: vec![("data/versions/main.json", "{".to_string())],
        looks: Mutex::new(folders.looks.lock().expect("lock").clone()),
        ..FakeFolders::default()
    });
    let outcome = build(
        vec!["a.feature"],
        Some("v1.0.0"),
        unparsed.clone(),
        Arc::new(FakeWriter::default()),
    );
    assert!(
        matches!(&outcome, Err(BuildError::Output(error)) if error.kind == OutputErrorKind::NotASite),
        "{outcome:?}"
    );
    assert!(unparsed.done().is_empty());
}
