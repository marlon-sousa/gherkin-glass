use rstest::rstest;

use super::*;
use crate::test_support::{kept_in_output as kept, name, set};

fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(name(text))
}

fn clash(found: &str, pattern: &str) -> Result<(), OutputErrorKind> {
    Err(OutputErrorKind::KeptClash {
        found: found.into(),
        pattern: pattern.into(),
    })
}

/// What the marker of a site without versions lists.
fn site() -> BTreeSet<FilePath> {
    set(&["index.html", "data/site.json", "data/objects/a.json"])
}

const CHECKING: Moment<'static> = Moment::Checking { replace: true };

// Rule 228: what the marker lists, and the marker, are checked before the input is read.
#[rstest]
#[case::a_host_s_files(&[".nojekyll", "CNAME", ".git"], Ok(()))]
#[case::a_file_ghg_wrote(&["index.html"], clash("index.html", "index.html"))]
#[case::the_marker(&["ghg-*.json"], clash("ghg-site.json", "ghg-*.json"))]
#[case::a_folder_ghg_wrote(&["data/"], clash("data", "data/"))]
#[case::the_last_pattern_decides(&["*.html", "!*.html"], Ok(()))]
#[case::whatever_the_case(&["Index.HTML"], clash("index.html", "Index.HTML"))]
#[case::the_first_folder_by_folder(&["a.json", "index.html"], clash("data/objects/a.json", "a.json"))]
fn a_pattern_that_keeps_what_ghg_wrote_clashes(
    #[case] patterns: &[&str],
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    let found = BTreeMap::new();
    let judged = kept_clash(
        &kept(patterns),
        &found,
        &site(),
        &BuildTarget::Site,
        CHECKING,
    );
    assert_eq!(judged, outcome);
}

// Rule 228: the site is checked right before it is written.
#[test]
fn a_pattern_that_keeps_a_page_of_the_site_clashes_once_the_site_is_known() {
    let site = set(&["index.html", "assets/2.0.0/gherkin-glass.css"]);
    let patterns = kept(&["assets/"]);
    let found = BTreeMap::new();
    let none = BTreeSet::new();
    let checking = kept_clash(&patterns, &found, &none, &BuildTarget::Site, CHECKING);
    assert_eq!(checking, Ok(()));
    let writing = Moment::Writing { site: &site };
    let judged = kept_clash(&patterns, &found, &none, &BuildTarget::Site, writing);
    assert_eq!(judged, clash("assets", "assets/"));
}

// Rule 228: the folder of the version built, every version the marker lists, and, once a
// version is written, the versions page and the files made from the descriptors.
#[rstest]
#[case::the_version_built(&["v1.0"], CHECKING, clash("v1.0", "v1.0"))]
#[case::another_version_listed(&["main"], CHECKING, clash("main", "main"))]
#[case::the_versions_page_listed(&["/index.html"], CHECKING, clash("index.html", "/index.html"))]
#[case::the_previews(&["previews"], CHECKING, Ok(()))]
#[case::the_index_of_pages(&["pages.json"], CHECKING, Ok(()))]
#[case::the_index_of_pages_written(&["pages.json"], Moment::Writing { site: &BTreeSet::new() }, clash("data/pages.json", "pages.json"))]
fn a_pattern_that_keeps_a_version_clashes(
    #[case] patterns: &[&str],
    #[case] moment: Moment<'_>,
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    let listed = set(&["index.html", "main/index.html", "data/versions/main.json"]);
    let found = BTreeMap::new();
    let judged = kept_clash(&kept(patterns), &found, &listed, &version("v1.0"), moment);
    assert_eq!(judged, outcome);
}

// Rule 228: something kept in a folder ghg writes.
#[rstest]
#[case::in_a_folder_ghg_wrote(
    "data/custom.css",
    EntryKind::File,
    clash("data/custom.css", "custom.css")
)]
#[case::at_the_root("custom.css", EntryKind::File, Ok(()))]
#[case::in_a_folder_of_its_own("docs/custom.css", EntryKind::File, Ok(()))]
#[case::in_a_folder_ghg_wrote_in_other_capitals(
    "Data/custom.css",
    EntryKind::File,
    clash("Data/custom.css", "custom.css")
)]
fn something_kept_in_a_folder_ghg_writes_clashes(
    #[case] path: &str,
    #[case] kind: EntryKind,
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    let found = BTreeMap::from([(path.to_string(), kind)]);
    let judged = kept_clash(
        &kept(&["custom.css"]),
        &found,
        &site(),
        &BuildTarget::Site,
        CHECKING,
    );
    assert_eq!(judged, outcome);
}

// Rules 227 and 228: what is found kept as its patterns are written clashes, even when a pattern
// ignoring the case of its letters gives it back.
#[test]
fn something_kept_as_written_clashes_whatever_a_pattern_in_other_capitals_gives_back() {
    let found = BTreeMap::from([("data/NOTES.TXT".to_string(), EntryKind::File)]);
    let judged = kept_clash(
        &kept(&["*.TXT", "!notes.txt"]),
        &found,
        &site(),
        &BuildTarget::Site,
        CHECKING,
    );
    assert_eq!(judged, clash("data/NOTES.TXT", "*.TXT"));
}
