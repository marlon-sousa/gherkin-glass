use std::collections::{BTreeMap, BTreeSet};

use rstest::rstest;

use super::*;

const KEEP: Moment = Moment::Checking { replace: false };
const REPLACE: Moment = Moment::Checking { replace: true };

fn name(text: &str) -> VersionName {
    VersionName::new(text).unwrap_or_else(|| panic!("{text} is a version name"))
}

fn set(paths: &[&str]) -> BTreeSet<FilePath> {
    paths.iter().map(|path| path.to_string()).collect()
}

fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(name(text))
}

/// A site without versions of `index.html` and `sub/page.html`.
fn site() -> Record {
    Record::Site(set(&["index.html", "sub/page.html"]))
}

/// The versions `main` and `v1.0.0`, each of one `index.html`, and the versions page.
fn versions() -> Record {
    Record::Versions {
        page: set(&["index.html", "assets/ghg.css"]),
        versions: BTreeMap::from([
            (name("main"), set(&["index.html"])),
            (name("v1.0.0"), set(&["index.html", "sub/old.html"])),
        ]),
    }
}

/// What ghg wrote for `site()`, with `files` and `folders` added.
fn site_folder(files: &[&str], folders: &[&str]) -> OutputContents {
    OutputContents::Folder {
        files: set(&["ghg-site.json", "index.html", "sub/page.html"])
            .into_iter()
            .chain(set(files))
            .collect(),
        folders: set(&["sub"]).into_iter().chain(set(folders)).collect(),
        record: Some(site()),
    }
}

/// What ghg wrote for `versions()`, with `files` and `folders` added and `gone` taken away.
fn versions_folder(files: &[&str], folders: &[&str], gone: &[&str]) -> OutputContents {
    let gone = set(gone);
    let all = |made: &[&str], added: &[&str]| -> BTreeSet<FilePath> {
        set(made)
            .into_iter()
            .chain(set(added))
            .filter(|path| !gone.contains(path))
            .collect()
    };
    OutputContents::Folder {
        files: all(
            &[
                "ghg-versions.json",
                "index.html",
                "assets/ghg.css",
                "main/index.html",
                "v1.0.0/index.html",
                "v1.0.0/sub/old.html",
            ],
            files,
        ),
        folders: all(&["assets", "main", "v1.0.0", "v1.0.0/sub"], folders),
        record: Some(versions()),
    }
}

#[rstest]
#[case::missing(OutputContents::Missing, BuildTarget::Site)]
#[case::missing_for_a_version(OutputContents::Missing, version("main"))]
#[case::empty(
    OutputContents::Folder { files: BTreeSet::new(), folders: BTreeSet::new(), record: None },
    version("main")
)]
fn a_missing_or_empty_folder_is_marked(
    #[case] contents: OutputContents,
    #[case] target: BuildTarget,
) {
    assert_eq!(output_safety(contents, &target, REPLACE), Ok(None));
}

#[rstest]
#[case::site(site_folder(&[], &[]), BuildTarget::Site, site())]
#[case::new_version(versions_folder(&[], &[], &[]), version("v2.0.0"), versions())]
#[case::listed_version(versions_folder(&[], &[], &[]), version("main"), versions())]
#[case::a_written_file_gone(
    versions_folder(&[], &[], &["v1.0.0/sub/old.html", "v1.0.0/sub"]),
    version("main"),
    versions()
)]
fn a_safe_folder_is_built_into(
    #[case] contents: OutputContents,
    #[case] target: BuildTarget,
    #[case] record: Record,
) {
    assert_eq!(output_safety(contents, &target, REPLACE), Ok(Some(record)));
}

#[rstest]
#[case::a_file(
    OutputContents::NotAFolder,
    BuildTarget::Site,
    OutputErrorKind::NotASite
)]
#[case::unmarked(
    OutputContents::Folder { files: set(&["notes.txt"]), folders: BTreeSet::new(), record: None },
    version("main"),
    OutputErrorKind::NotASite
)]
#[case::unmarked_folder(
    OutputContents::Folder { files: BTreeSet::new(), folders: set(&["docs"]), record: None },
    BuildTarget::Site,
    OutputErrorKind::NotASite
)]
#[case::versions_for_a_site(versions_folder(&[], &[], &[]), BuildTarget::Site, OutputErrorKind::HoldsVersions)]
#[case::a_site_for_a_version(site_folder(&[], &[]), version("main"), OutputErrorKind::HoldsSite)]
#[case::a_stray_file(
    site_folder(&["notes.txt"], &[]),
    BuildTarget::Site,
    OutputErrorKind::NotSafe { found: "notes.txt".into() }
)]
#[case::a_stray_file_in_a_made_folder(
    versions_folder(&["v1.0.0/extra.html"], &[], &[]),
    version("main"),
    OutputErrorKind::NotSafe { found: "v1.0.0/extra.html".into() }
)]
#[case::a_stray_folder_named_outermost(
    versions_folder(&["docs/a/notes.txt", "zz.txt"], &["docs", "docs/a"], &[]),
    version("main"),
    OutputErrorKind::NotSafe { found: "docs".into() }
)]
#[case::an_unlisted_version(
    versions_folder(&["v0.9.0/index.html"], &["v0.9.0"], &[]),
    version("main"),
    OutputErrorKind::NotSafe { found: "v0.9.0".into() }
)]
#[case::a_listed_version_missing(
    versions_folder(&[], &[], &["main/index.html", "main"]),
    version("v1.0.0"),
    OutputErrorKind::MissingVersion { name: name("main") }
)]
#[case::other_capitals(versions_folder(&[], &[], &[]), version("Main"), OutputErrorKind::OtherCapitals)]
fn an_unsafe_folder_is_refused(
    #[case] contents: OutputContents,
    #[case] target: BuildTarget,
    #[case] refused: OutputErrorKind,
) {
    assert_eq!(output_safety(contents, &target, REPLACE), Err(refused));
}

#[rstest]
#[case::site(site_folder(&[], &[]), BuildTarget::Site, KEEP, Err(OutputErrorKind::AlreadyBuilt))]
#[case::listed_version(versions_folder(&[], &[], &[]), version("main"), KEEP, Err(OutputErrorKind::AlreadyBuilt))]
#[case::new_version(versions_folder(&[], &[], &[]), version("v2.0.0"), KEEP, Ok(Some(versions())))]
#[case::site_when_writing(site_folder(&[], &[]), BuildTarget::Site, Moment::Writing, Ok(Some(site())))]
#[case::version_when_writing(
    versions_folder(&[], &[], &[]),
    version("main"),
    Moment::Writing,
    Ok(Some(versions()))
)]
fn a_build_replaces_only_when_allowed(
    #[case] contents: OutputContents,
    #[case] target: BuildTarget,
    #[case] moment: Moment,
    #[case] outcome: Result<Option<Record>, OutputErrorKind>,
) {
    assert_eq!(output_safety(contents, &target, moment), outcome);
}

#[rstest]
#[case::at_the_root("index.html", &[])]
#[case::nested("a/b/c.html", &["a", "a/b"])]
fn the_folders_of_a_file_are_listed_outermost_first(#[case] file: &str, #[case] folders: &[&str]) {
    assert_eq!(folders_of(file).collect::<Vec<_>>(), folders);
}

/// A folder marked as a site of the one file `listed`, holding only its marker.
fn listing(listed: &str) -> OutputContents {
    OutputContents::Folder {
        files: set(&["ghg-site.json"]),
        folders: BTreeSet::new(),
        record: Some(Record::Site(set(&[listed]))),
    }
}

#[rstest]
#[case::the_folder_above("../victim.txt")]
#[case::deeper_above("sub/../../victim.txt")]
#[case::the_root("/home/user")]
#[case::empty("")]
#[case::an_empty_name("a//b.html")]
#[case::this_folder("./index.html")]
fn a_marker_listing_a_path_outside_the_folder_is_no_marker(#[case] listed: &str) {
    assert_eq!(
        output_safety(listing(listed), &BuildTarget::Site, REPLACE),
        Err(OutputErrorKind::NotASite)
    );
}

#[rstest]
#[case::a_version_s_file(Record::Versions {
    page: set(&["index.html"]),
    versions: BTreeMap::from([(name("main"), set(&["../../victim.txt"]))]),
})]
#[case::a_page_file(Record::Versions {
    page: set(&["../victim.txt"]),
    versions: BTreeMap::new(),
})]
fn a_versions_marker_listing_a_path_outside_the_folder_is_no_marker(#[case] record: Record) {
    let contents = OutputContents::Folder {
        files: set(&["ghg-versions.json"]),
        folders: BTreeSet::new(),
        record: Some(record),
    };
    assert_eq!(
        output_safety(contents, &version("main"), REPLACE),
        Err(OutputErrorKind::NotASite)
    );
}

#[test]
fn the_first_thing_found_is_named_folder_by_folder() {
    let contents = OutputContents::Folder {
        files: set(&["ghg-site.json", "a/index.html", "a/b.txt", "a-c.txt"]),
        folders: set(&["a"]),
        record: Some(Record::Site(set(&["a/index.html"]))),
    };
    assert_eq!(
        output_safety(contents, &BuildTarget::Site, REPLACE),
        Err(OutputErrorKind::NotSafe {
            found: "a/b.txt".into()
        })
    );
}

#[rstest]
#[case::a_colon("time 10:30.html")]
#[case::a_backslash("back\\slash.html")]
fn a_page_name_any_system_allows_is_ghg_s(#[case] page: &str) {
    let contents = OutputContents::Folder {
        files: set(&["ghg-site.json", page]),
        folders: BTreeSet::new(),
        record: Some(Record::Site(set(&[page]))),
    };
    assert_eq!(
        output_safety(contents, &BuildTarget::Site, REPLACE),
        Ok(Some(Record::Site(set(&[page]))))
    );
}

#[test]
fn only_the_versions_page_s_files_found_are_kept_for_removal() {
    let mut contents = versions_folder(&[], &[], &["assets/ghg.css", "assets"]);
    if let OutputContents::Folder { record, .. } = &mut contents {
        *record = Some(Record::Versions {
            page: set(&["index.html", "assets/ghg.css", "..\\victim.txt"]),
            versions: BTreeMap::from([
                (name("main"), set(&["index.html"])),
                (name("v1.0.0"), set(&["index.html", "sub/old.html"])),
            ]),
        });
    }
    let Ok(Some(Record::Versions { page, .. })) =
        output_safety(contents, &version("main"), REPLACE)
    else {
        panic!("the folder is safe");
    };
    assert_eq!(page, set(&["index.html"]));
}
