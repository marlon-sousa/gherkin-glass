//! Search, checked by running `ghg` and against the fixtures' pages.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/search"
    ))
}

#[test]
fn every_page_has_the_search_box_before_its_main() {
    // Rules 86 and 151.
    let pages = fixture().parent().unwrap().join("pages");
    for page in [
        "expected/billing/refunds.html",
        "expected/emoji.html",
        "expected/stories/read.html",
        "expected/saque em conta poupança.html",
        "expected-collapsed/billing/refunds.html",
    ] {
        let html = fs::read_to_string(pages.join(page)).expect(page);
        let form = html
            .find("<form class=\"ghg-search\" role=\"search\"")
            .expect(page);
        let main = html.find("<main id=\"ghg_main\">").expect(page);
        let breadcrumb = html.find("<nav class=\"ghg-breadcrumb\"").expect(page);
        assert!(breadcrumb < form && form < main, "{page}");
        let search = &html[form..main];
        assert_eq!(html.matches("role=\"search\"").count(), 1, "{page}");
        assert!(
            search.contains("<label>Search features, rules and scenarios <input type=\"search\"")
        );
        // Rules 156 and 214: a combobox over the list box, then the link to every result.
        assert!(search.contains(
            "role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"false\" aria-controls=\"ghg_search_results\""
        ));
        let link = search.find("<a class=\"ghg-search-all\"").expect(page);
        let status = search
            .find("<p class=\"ghg-search-status\" role=\"status\"></p>")
            .expect(page);
        let results = search
            .find("<ol class=\"ghg-search-results\" id=\"ghg_search_results\" role=\"listbox\" aria-label=\"Search results\" hidden></ol>")
            .expect(page);
        assert!(link < status && status < results, "{page}");
        assert!(
            search[link..].starts_with("<a class=\"ghg-search-all\" href=\""),
            "{page}"
        );
        assert!(
            search[link..status].contains("search.html\" hidden></a>"),
            "{page}"
        );
    }
}

#[test]
fn the_page_of_search_results_is_at_the_root_with_its_own_form_and_no_search_box() {
    // Rules 215 to 217.
    let pages = fixture().parent().unwrap().join("pages/expected");
    let html = fs::read_to_string(pages.join("search.html")).unwrap();
    assert!(html.contains("<html lang=\"en\">"));
    assert!(html.contains("<title>Search results, Banking &#38; &#60;more&#62;</title>"));
    assert!(
        html.contains("<li><a href=\"search.html\" aria-current=\"page\">Search results</a></li>")
    );
    assert!(!html.contains("class=\"ghg-search\""));
    assert_eq!(html.matches("role=\"search\"").count(), 1);
    let main = html.find("<main id=\"ghg_main\">").unwrap();
    let heading = html
        .find("<h1 class=\"ghg-search-heading\">Search results</h1>")
        .unwrap();
    let form = html
        .find(
            "<form class=\"ghg-search-page\" role=\"search\" action=\"search.html\" method=\"get\"",
        )
        .unwrap();
    let field = html
        .find("<label>Search features, rules and scenarios <input type=\"search\" name=\"q\"")
        .unwrap();
    let status = html
        .find("</form>\n<p class=\"ghg-search-status\" role=\"status\"></p>\n<div class=\"ghg-search-groups\"></div>\n</main>")
        .unwrap();
    assert!(main < heading && heading < form && form < field && field < status);
    // The page links to nothing but its breadcrumb's pages and its footer's.
    let home = fs::read_to_string(pages.join("index.html")).unwrap();
    assert!(!home.contains("<a href=\"search.html\""));
}

#[test]
fn documents_above_5_mb_are_written_with_a_warning() {
    // Rule 150, after the hint of rule 17.
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("large-search");
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(folder.join("features")).unwrap();
    let step = "a".repeat(5_050_000);
    fs::write(
        folder.join("features/large.feature"),
        format!("Feature: Large\n  Scenario: Long\n    Given {step}\n"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .current_dir(&folder)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "The site has no title, so its home page is called Feature files. Set title in ghg.toml, or give --title.\n\
         The search documents are 5.1 MB, larger than the 5 MB above which search can be slow to start, mostly on phones.\n"
    );
    let written = fs::metadata(folder.join("site/assets/search.json")).unwrap();
    assert!(written.len() > 5_000_000);
}

#[test]
fn documents_of_5_mb_or_less_bring_no_warning() {
    // Rule 150, and rule 17: a build that succeeds prints nothing else.
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("small-search");
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--title", "Search", "--input"])
        .arg(fixture().join("features"))
        .current_dir(&folder)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    let written = fs::read_to_string(folder.join("site/assets/search.json")).unwrap();
    // Rule 149: the file is compact and keeps letters outside ASCII as they are.
    assert!(!written.contains('\n'));
    assert!(!written.contains("\": ") && !written.contains("\", "));
    assert!(written.contains("poupança") && !written.contains("\\u"));
    let expected = fs::read_to_string(fixture().join("search.json")).unwrap();
    let written: Value = serde_json::from_str(&written).unwrap();
    assert_eq!(written, serde_json::from_str::<Value>(&expected).unwrap());
}
