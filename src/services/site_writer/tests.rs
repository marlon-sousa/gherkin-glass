mod fakes;
mod removing;

use rstest::rstest;

use super::*;
use crate::adapters::MemoryFiles;
use fakes::*;

/// A site writer into a folder held in memory, with no template.
fn into(output: &Arc<MemoryOutput>) -> SiteWriter {
    site_writer(
        output,
        Arc::new(MemoryFiles::default()),
        FakeTemplate::default(),
    )
}

/// The paths the marker of `output` lists.
fn listed(output: &MemoryOutput) -> Vec<String> {
    let marker = output.text("ghg-site.json").expect("a marker");
    let json: serde_json::Value = serde_json::from_str(&marker).expect("JSON");
    json["files"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|file| file.as_str().expect("a path").to_string())
        .collect()
}

// Rules 38, 39 and 241: a site writes its store, its assets and its shell, and its marker lists
// every file but itself.
#[test]
fn a_site_into_a_missing_folder_writes_its_store_and_lists_every_file() {
    let output = Arc::new(MemoryOutput::default());
    add(&into(&output), BuildTarget::Site, "a").expect("built");
    let paths = output.paths();
    for path in ["index.html", "data/site.json", "ghg-site.json"] {
        assert!(paths.contains(&path.to_string()), "{paths:?}");
    }
    assert!(paths.iter().any(|path| path.starts_with("data/objects/")));
    assert!(paths.iter().any(|path| path.starts_with("assets/")));
    let mut listed = listed(&output);
    listed.sort();
    let others: Vec<String> = paths
        .into_iter()
        .filter(|path| path != "ghg-site.json")
        .collect();
    assert_eq!(listed, others);
}

// Rules 247, 253 and 254: versions share the store, and the versions page, the list of
// versions and the index of pages name every one.
#[test]
fn versions_share_the_store_beside_the_versions_page() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = into(&output);
    add(&site_writer, version("v1.0.0"), "a").expect("built");
    add(&site_writer, version("main"), "a").expect("built");
    let paths = output.paths();
    for path in [
        "main/index.html",
        "v1.0.0/index.html",
        "data/versions.json",
        "data/pages.json",
    ] {
        assert!(paths.contains(&path.to_string()), "{paths:?}");
    }
    let objects = paths
        .iter()
        .filter(|path| path.starts_with("data/objects/"));
    assert_eq!(objects.count(), 1, "the same feature is stored once");
    let versions = output.text("data/versions.json").expect("the list");
    assert!(
        versions.contains("\"main\"") && versions.contains("\"v1.0.0\""),
        "{versions}"
    );
    let page = output.text("index.html").expect("the versions page");
    assert!(
        page.contains("<a href=\"v1.0.0/index.html\">v1.0.0</a>"),
        "{page}"
    );
}

// Rule 123: a version already built is refused unless it may be replaced.
#[test]
fn a_version_already_built_is_kept_unless_replaced() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = into(&output);
    add(&site_writer, version("v1"), "a").expect("built");
    let mut settings = settings(Destination::Folder, version("v1"));
    settings.replace = false;
    let refused = site_writer.check(&settings).map_err(|error| error.kind);
    assert_eq!(refused, Err(OutputErrorKind::AlreadyBuilt));
}

#[test]
fn a_file_that_cannot_be_written_is_reported_with_its_path() {
    let output = Arc::new(MemoryOutput {
        fails: Some("data/site.json"),
        ..MemoryOutput::default()
    });
    let Err(BuildError::Output(error)) = add(&into(&output), BuildTarget::Site, "a") else {
        panic!("written");
    };
    assert_eq!(
        error.to_string(),
        "site/data/site.json: cannot be written: disk full"
    );
}

// Rules 38 and 125: a marker that cannot be written is reported as the output folder.
#[test]
fn a_marker_that_cannot_be_written_is_reported_with_the_output_folder() {
    let output = Arc::new(MemoryOutput {
        fails: Some("ghg-site.json"),
        ..MemoryOutput::default()
    });
    let Err(BuildError::Output(error)) = add(&into(&output), BuildTarget::Site, "a") else {
        panic!("written");
    };
    assert_eq!(error.to_string(), "site: cannot be written: disk full");
}

#[test]
fn an_output_folder_that_cannot_be_looked_into_is_reported_as_such() {
    let output = Arc::new(MemoryOutput {
        unreadable: true,
        ..MemoryOutput::default()
    });
    let outcome = add(&into(&output), BuildTarget::Site, "a");
    assert!(matches!(
        outcome,
        Err(BuildError::Output(OutputError {
            kind: OutputErrorKind::CannotBeRead { .. },
            ..
        }))
    ));
}

// Rule 38: a folder that changed while the input was read is checked again, and left alone.
#[test]
fn a_folder_that_changed_while_the_input_was_read_is_left_as_it_is() {
    let output = Arc::new(MemoryOutput {
        changes: Some("notes.txt"),
        ..MemoryOutput::default()
    });
    let outcome = add(&into(&output), BuildTarget::Site, "a");
    assert!(matches!(
        outcome,
        Err(BuildError::Output(OutputError {
            kind: OutputErrorKind::NotASite,
            ..
        }))
    ));
    assert!(output.paths().is_empty());
}

// Rule 66: a site in memory checks no folder, and is a site without versions.
#[rstest]
#[case::a_site(BuildTarget::Site)]
#[case::a_version(version("v1"))]
fn a_site_in_memory_checks_no_folder(#[case] target: BuildTarget) {
    let output = Arc::new(MemoryOutput {
        unreadable: true,
        ..MemoryOutput::default()
    });
    let memory = Arc::new(MemoryFiles::default());
    let site_writer = site_writer(&output, memory, FakeTemplate::default());
    let settings = settings(Destination::Memory, target);
    site_writer.check(&settings).expect("no check");
    let (served, _) = site_writer.write(&settings, &store("a")).expect("written");
    let ServedSite::Memory(files) = served else {
        panic!("a site in memory");
    };
    assert!(files.contains_key("index.html") && files.contains_key("data/site.json"));
    assert!(!files.contains_key("ghg-site.json"));
}

// Rule 190: a template that cannot be read stops the build before anything is written.
#[test]
fn a_front_page_s_template_that_cannot_be_read_stops_the_build() {
    let output = Arc::new(MemoryOutput::default());
    let site_writer = site_writer(
        &output,
        Arc::new(MemoryFiles::default()),
        FakeTemplate::default(),
    );
    let mut settings = settings(Destination::Folder, BuildTarget::Site);
    settings.front_page = Some("front.html".into());
    let outcome = site_writer.write(&settings, &store("a"));
    assert!(
        matches!(outcome, Err(BuildError::FrontPage(_))),
        "{outcome:?}"
    );
    assert!(output.paths().is_empty());
}
