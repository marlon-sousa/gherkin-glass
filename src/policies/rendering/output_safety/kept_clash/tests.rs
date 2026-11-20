use rstest::rstest;

use super::*;
use crate::test_support::kept_in_output as kept;

fn set(paths: &[&str]) -> BTreeSet<FilePath> {
    paths.iter().map(|path| path.to_string()).collect()
}

fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(VersionName::new(text).expect("a version name"))
}

fn clash(found: &str, pattern: &str) -> Result<(), OutputErrorKind> {
    Err(OutputErrorKind::KeptClash {
        found: found.into(),
        pattern: pattern.into(),
    })
}

/// A site without versions of `index.html` and `billing/a.html`.
fn site() -> Record {
    Record::Site(set(&["index.html", "billing/a.html"]))
}

const CHECKING: Moment<'static> = Moment::Checking { replace: true };

// Rule 228: what the marker lists, and the marker, are checked before the input is read.
#[rstest]
#[case::a_host_s_files(&[".nojekyll", "CNAME", ".git"], Ok(()))]
#[case::a_file_ghg_wrote(&["index.html"], clash("index.html", "index.html"))]
#[case::the_marker(&["*.json"], clash("ghg-site.json", "*.json"))]
#[case::a_folder_ghg_wrote(&["billing/"], clash("billing", "billing/"))]
#[case::the_last_pattern_decides(&["*.html", "!*.html"], Ok(()))]
#[case::whatever_the_case(&["Index.HTML"], clash("index.html", "Index.HTML"))]
#[case::the_first_folder_by_folder(&["a.html", "index.html"], clash("billing/a.html", "a.html"))]
fn a_pattern_that_keeps_what_ghg_wrote_clashes(
    #[case] patterns: &[&str],
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    let found = BTreeMap::new();
    let judged = kept_clash(
        &kept(patterns),
        &found,
        Some(&site()),
        &BuildTarget::Site,
        CHECKING,
    );
    assert_eq!(judged, outcome);
}

// Rule 228: the site is checked right before it is written.
#[test]
fn a_pattern_that_keeps_a_page_of_the_site_clashes_once_the_site_is_known() {
    let site = set(&["index.html", "drafts/a.html", "assets/gherkin-glass.css"]);
    let patterns = kept(&["drafts/"]);
    let found = BTreeMap::new();
    let checking = kept_clash(&patterns, &found, None, &BuildTarget::Site, CHECKING);
    assert_eq!(checking, Ok(()));
    let writing = Moment::Writing { site: &site };
    let judged = kept_clash(&patterns, &found, None, &BuildTarget::Site, writing);
    assert_eq!(judged, clash("drafts", "drafts/"));
}

// Rule 228: the version built, and every version the marker lists.
#[rstest]
#[case::the_version_built(&["v1.0"], version("v1.0"), clash("v1.0", "v1.0"))]
#[case::another_version_listed(&["main"], version("v1.0"), clash("main", "main"))]
#[case::the_versions_page(&["/index.html"], version("v1.0"), clash("index.html", "/index.html"))]
#[case::the_previews(&["previews"], version("v1.0"), Ok(()))]
fn a_pattern_that_keeps_a_version_clashes(
    #[case] patterns: &[&str],
    #[case] target: BuildTarget,
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    let main = VersionName::new("main").expect("a version name");
    let record = Record::Versions {
        page: set(&["index.html"]),
        versions: BTreeMap::from([(main, set(&["index.html"]))]),
    };
    let found = BTreeMap::new();
    let judged = kept_clash(&kept(patterns), &found, Some(&record), &target, CHECKING);
    assert_eq!(judged, outcome);
}

// Rule 228: something kept in a folder ghg writes.
#[rstest]
#[case::in_a_folder_ghg_wrote(
    "billing/custom.css",
    EntryKind::File,
    clash("billing/custom.css", "custom.css")
)]
#[case::at_the_root("custom.css", EntryKind::File, Ok(()))]
#[case::in_a_folder_of_its_own("docs/custom.css", EntryKind::File, Ok(()))]
#[case::in_a_folder_ghg_wrote_in_other_capitals(
    "Billing/custom.css",
    EntryKind::File,
    clash("Billing/custom.css", "custom.css")
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
        Some(&site()),
        &BuildTarget::Site,
        CHECKING,
    );
    assert_eq!(judged, outcome);
}
