//! Search, checked in the shells of the fixtures' sites, and the warning a
//! build of large data prints, by running `ghg`.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn fixtures() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))
}

/// The part of `html` from `start` on.
fn from<'a>(html: &'a str, start: &str) -> &'a str {
    &html[html.find(start).expect(start)..]
}

// Rules 86, 151, 156 and 214: the shell's search box, between its
// breadcrumb and its main: a combobox over the list box, then the link to
// every result, the status and the list.
#[test]
fn the_shell_has_the_search_box_before_its_main() {
    for site in ["pages/expected", "pages/expected-collapsed"] {
        let html = fs::read_to_string(fixtures().join(site).join("index.html")).expect(site);
        let form = html
            .find("<form class=\"ghg-search\" role=\"search\"")
            .expect(site);
        let main = html.find("<main id=\"ghg_main\"").expect(site);
        let breadcrumb = html.find("<nav class=\"ghg-breadcrumb\"").expect(site);
        assert!(breadcrumb < form && form < main, "{site}");
        assert_eq!(html.matches("role=\"search\"").count(), 1, "{site}");
        let search = &html[form..main];
        let field = from(
            search,
            "<label>Search features, rules and scenarios <input type=\"search\" class=\"ghg-search-field\" \
             role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"false\" aria-controls=\"ghg_search_results\"",
        );
        let link = from(
            field,
            "<a class=\"ghg-search-all\" href=\"#search\" hidden></a>\n",
        );
        let status = from(
            link,
            "<p class=\"ghg-search-status\" role=\"status\"></p>\n",
        );
        from(
            status,
            "<ol class=\"ghg-search-results\" id=\"ghg_search_results\" role=\"listbox\" \
             aria-label=\"Search results\" hidden></ol>\n</form>",
        );
    }
}

/// `ghg build` run in `folder` with `arguments` after it.
fn build(folder: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("the command runs")
}

// Rule 150, after the hint of rule 17.
#[test]
fn data_above_5_mb_is_written_with_a_warning() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("large-data");
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(folder.join("features")).expect("made");
    // Its object holds the step three times: its text, its sentence and its one part.
    let step = "a".repeat(1_690_000);
    fs::write(
        folder.join("features/large.feature"),
        format!("Feature: Large\n  Scenario: Long\n    Given {step}\n"),
    )
    .expect("written");
    let output = build(&folder, &[]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "The site has no title, so its home page is called Feature files. Set title in ghg.toml, or give --title.\n\
         The site's data is 5.1 MB, larger than the 5 MB above which its pages and search can be slow to start, mostly on phones.\n"
    );
    let objects: u64 = fs::read_dir(folder.join("site/data/objects"))
        .expect("listed")
        .map(|entry| entry.expect("listed").metadata().expect("read").len())
        .sum();
    assert!(objects > 5_000_000);
}

// Rule 150, and rule 17: a build that succeeds prints nothing else.
#[test]
fn data_of_5_mb_or_less_brings_no_warning() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("small-data");
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("made");
    let input = fixtures().join("search/features");
    let output = build(
        &folder,
        &[
            "--title",
            "Search",
            "--input",
            input.to_str().expect("UTF-8"),
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
