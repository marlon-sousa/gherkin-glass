//! The sites the fixtures build, compared file by file with the sites committed beside them,
//! and the shell and the store a build writes, read as a browser and the script read them.
//!
//! `cargo xtask build` writes the expected sites again after a change.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{create_dir_all, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use site_files::{assert_same_site as assert_same_files, files};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// `ghg build` run in `folder` with `arguments` after it.
fn build(folder: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("the command runs")
}

fn output(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    folder
}

fn json(file: &Path) -> Value {
    serde_json::from_str(&read_to_string(file).expect("the file is read")).expect("JSON")
}

/// The descriptor of the site in `site`.
fn site_json(site: &Path) -> Value {
    json(&site.join("data/site.json"))
}

/// The object of the page `page` of the site in `site`.
fn object_of(site: &Path, page: &str) -> Value {
    let features = site_json(site)["features"].clone();
    let feature = features
        .as_array()
        .expect("a list")
        .iter()
        .find(|feature| feature["page"] == page)
        .expect("the page is listed")
        .clone();
    let name = feature["object"].as_str().expect("an object");
    json(&site.join(format!("data/objects/{name}.json")))
}

/// The front page of the committed site `site` of the fixture `pages`.
fn front_page(site: &str) -> String {
    read_to_string(fixtures().join("pages").join(site).join("index.html")).expect("read")
}

fn assert_same_site(config: &str, expected: &str) {
    assert_same_site_of("pages", config, expected);
}

fn assert_same_site_of(fixture: &str, config: &str, expected: &str) {
    let written = output(&format!("{fixture}-{expected}"));
    let result = build(
        &fixtures().join(fixture),
        &[
            "--config",
            config,
            "--output",
            written.to_str().expect("UTF-8"),
        ],
    );
    // Rule 17: collapsed.toml gives no title.
    let hint = if fixture == "pages" && config == "collapsed.toml" {
        "The site has no title, so its home page is called Feature files. \
         Set title in ghg.toml, or give --title.\n"
    } else {
        ""
    };
    assert_eq!(String::from_utf8_lossy(&result.stderr), hint);
    assert_eq!(result.status.code(), Some(0));
    assert_same_files(&written, &fixtures().join(fixture).join(expected));
}

// Rules 36 to 39, 81 to 83 and 241 to 252: the shell, its front page, the
// store and the assets of every fixture, as configured.

#[test]
fn every_feature_file_is_in_the_store_as_the_expected_site_shows() {
    assert_same_site("ghg.toml", "expected");
}

#[test]
fn the_settings_start_rules_scenarios_and_backgrounds_collapsed() {
    assert_same_site("collapsed.toml", "expected-collapsed");
}

// "Backgrounds", rules 94 to 99: the descriptor's starting state of the
// checkbox, checked or not.

#[test]
fn scenarios_hold_their_backgrounds_expanded_by_default() {
    assert_same_site_of("backgrounds", "ghg.toml", "expected");
}

#[test]
fn scenarios_hold_their_backgrounds_collapsed_when_inline_backgrounds_is_false() {
    assert_same_site_of("backgrounds", "unchecked.toml", "expected-unchecked");
}

// Rule 39: the assets are those built into ghg, in the folder of its version.

#[test]
fn the_assets_are_the_ones_built_into_ghg() {
    let expected = files(&fixtures().join("pages/expected/assets/{version}"));
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    assert_eq!(expected, files(&repository));
}

// Rules 39 and 194: the shell links the icon and the assets of its version,
// by relative addresses, and ends with the footer.

#[test]
fn the_shell_links_its_assets_and_ends_with_the_footer() {
    let page = front_page("expected");
    for link in [
        "<link rel=\"stylesheet\" href=\"assets/{version}/gherkin-glass.css\">",
        "<link rel=\"icon\" href=\"assets/{version}/gherkin-glass.svg\" type=\"image/svg+xml\">",
        "<script type=\"module\" src=\"assets/{version}/gherkin-glass.js\"></script>",
    ] {
        assert!(page.contains(link), "{link}");
    }
    assert!(page.contains(
        "</main>\n<p id=\"ghg_announcement\" role=\"status\"></p>\n<footer class=\"ghg-footer\">\n<p>Built with \
         <a href=\"https://github.com/marlon-sousa/gherkin-glass\">gherkin-glass</a> {version}</p>\n\
         </footer>\n</div>\n</div>\n"
    ));
}

// Rule 195: the footer and the assets' folder name the crate's own version,
// which the committed sites hold as the placeholder.

#[test]
fn the_footer_names_the_version_of_the_ghg_that_wrote_the_shell() {
    let written = output("footer-version");
    let result = build(
        &fixtures().join("pages"),
        &["--output", written.to_str().expect("UTF-8")],
    );
    assert_eq!(result.status.code(), Some(0));
    let page = read_to_string(written.join("index.html")).expect("the page is written");
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        page.contains(&format!("gherkin-glass</a> {version}</p>\n</footer>")),
        "{page}"
    );
    assert!(page.contains(&format!("href=\"assets/{version}/gherkin-glass.css\"")));
    assert!(!page.contains("{version}"), "{page}");
}

// Rules 32 and 81: the shell's one h1, its lang, and the ids it gives its own
// parts, which contain `_`.

#[test]
fn the_shell_has_one_h1_a_lang_and_only_its_own_ids() {
    let page = front_page("expected");
    assert_eq!(page.matches("<h1").count(), 1);
    assert!(page.contains("<html lang=\"en\""));
    let ids: Vec<&str> = page
        .split(" id=\"")
        .skip(1)
        .map(|id| &id[..id.find('"').expect("closed")])
        .collect();
    assert_eq!(
        ids,
        [
            "ghg_from_disk",
            "ghg_from_disk_title",
            "ghg_from_disk_text",
            "ghg_show_tree",
            "ghg_tree",
            "ghg_search_results",
            "ghg_main",
            "ghg_announcement",
            "ghg_messages",
        ]
    );
}

// Rule 42: each feature's object holds its file's language as a tag.

#[test]
fn the_page_s_lang_is_its_file_s_language_as_a_tag() {
    let site = fixtures().join("pages/expected");
    let lang = |page: &str| object_of(&site, page)["lang"].clone();
    assert_eq!(lang("billing/refunds"), "en");
    assert_eq!(lang("saque em conta poupança"), "pt");
    assert_eq!(lang("emoji"), "und");
}

// Rules 37, 78, 142 and 215.

#[test]
fn two_files_with_the_same_page_stop_the_build_with_the_files_that_cannot_be_read() {
    let written = output("same-page");
    let result = build(
        &fixtures().join("same-page"),
        &["--input", ".", "--output", written.to_str().expect("UTF-8")],
    );
    // Names that differ only in case cannot share a folder in a checkout on
    // every system, so the service's own tests check them.
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        "a.story: has the same page, a, as a.feature.\n\
         broken.feature:1: Unknown keyword: 'not a feature'.\n\
         search.feature: has the same page, search, as the search results.\n\
         steps.feature: has the same page, steps, as the step sentences.\n"
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(!written.exists(), "nothing is written");
}

// Rule 38.

#[test]
fn an_earlier_build_is_replaced_and_nothing_of_it_is_left() {
    let written = output("replaced");
    let arguments = ["--output", written.to_str().expect("UTF-8")];
    assert_eq!(
        build(&fixtures().join("backgrounds"), &arguments)
            .status
            .code(),
        Some(0)
    );
    let result = build(&fixtures().join("pages"), &arguments);
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(files(&written), files(&fixtures().join("pages/expected")));
}

// Rule 38: a marked folder holding anything ghg did not write is refused.

#[test]
fn a_file_ghg_did_not_write_stops_the_build() {
    let written = output("stray-file");
    let arguments = ["--output", written.to_str().expect("UTF-8")];
    assert_eq!(
        build(&fixtures().join("pages"), &arguments).status.code(),
        Some(0)
    );
    let stray = written.join("old.html");
    write(&stray, "old").expect("written");
    let result = build(&fixtures().join("pages"), &arguments);
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "{}: ghg did not write this, so the output folder is left as it is.\n",
            stray.to_str().expect("UTF-8")
        )
    );
    assert_eq!(result.status.code(), Some(1));
    let mut expected = files(&fixtures().join("pages/expected"));
    expected.insert("old.html".into(), b"old".to_vec());
    assert_eq!(files(&written), expected);
}

#[cfg(unix)]
#[test]
fn a_site_whose_pages_have_any_name_the_system_allows_is_built_again() {
    let input = output("odd-names-input");
    create_dir_all(&input).expect("made");
    for name in ["time 10:30.feature", "back\\slash.feature"] {
        write(input.join(name), "Feature: Odd\n").expect("written");
    }
    let written = output("odd-names");
    let arguments = [
        "--input",
        input.to_str().expect("UTF-8"),
        "--output",
        written.to_str().expect("UTF-8"),
        "--title",
        "Odd",
    ];
    for _ in 0..2 {
        let result = build(&fixtures().join("pages"), &arguments);
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
        assert_eq!(result.status.code(), Some(0));
    }
    let pages: Vec<String> = site_json(&written)["features"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|feature| feature["page"].as_str().expect("a page").to_string())
        .collect();
    assert_eq!(pages, ["back\\slash", "time 10:30"]);
}

#[test]
fn an_earlier_build_is_kept_when_replace_existing_version_is_false() {
    let written = output("kept-build");
    let given = written.to_str().expect("UTF-8");
    assert_eq!(
        build(&fixtures().join("pages"), &["--output", given])
            .status
            .code(),
        Some(0)
    );
    let result = build(
        &fixtures().join("pages"),
        &["--output", given, "--config", "keep.toml"],
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!("{given}: a site is already built here, and replace_existing_version is false.\n")
    );
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(files(&written), files(&fixtures().join("pages/expected")));
}

#[test]
fn an_empty_output_folder_is_used() {
    let written = output("empty-output");
    create_dir_all(&written).expect("made");
    let result = build(
        &fixtures().join("pages"),
        &["--output", written.to_str().expect("UTF-8")],
    );
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(files(&written), files(&fixtures().join("pages/expected")));
}

#[cfg(unix)]
#[test]
fn a_link_in_the_output_folder_s_place_is_never_followed() {
    use std::os::unix::fs::symlink;
    let target = output("link-target");
    create_dir_all(&target).expect("made");
    let written = output("link-output");
    symlink(&target, &written).expect("linked");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("pages"), &["--output", given]);
    assert_eq!(result.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .starts_with(&format!("{given}: this is not a site ghg built")),
    );
    assert!(files(&target).is_empty());
}

#[test]
fn an_output_folder_with_other_files_is_left_as_it_is() {
    let written = output("other-files");
    create_dir_all(&written).expect("made");
    write(written.join("kept.txt"), "kept").expect("written");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("pages"), &["--output", given]);
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "{given}: this is not a site ghg built, so it is left as it is. Build into an empty or new folder.\n"
        )
    );
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(files(&written).keys().collect::<Vec<_>>(), ["kept.txt"]);
}

#[test]
fn an_output_folder_holding_versions_is_left_as_it_is() {
    let written = output("holding-versions");
    create_dir_all(&written).expect("made");
    write(
        written.join("ghg-versions.json"),
        "{\"page\": [], \"versions\": {}}\n",
    )
    .expect("written");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("pages"), &["--output", given]);
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "{given}: a site of gherkin-glass 1 is built here, which gherkin-glass 2 cannot add to, so it is left as it is. Build into an empty or new folder.\n"
        )
    );
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn a_file_in_the_output_folder_s_place_is_left_as_it_is() {
    let written = output("not-a-folder");
    write(&written, "a file").expect("written");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("pages"), &["--output", given]);
    let report = String::from_utf8_lossy(&result.stderr).into_owned();
    assert_eq!(result.status.code(), Some(1));
    assert!(
        report.starts_with(&format!("{given}: this is not a site ghg built")),
        "{report}"
    );
}

#[cfg(unix)]
#[test]
fn an_output_folder_that_cannot_be_written_stops_the_build() {
    use std::fs::{Permissions, set_permissions};
    use std::os::unix::fs::PermissionsExt;
    // An earlier build in a folder that cannot be written into cannot be
    // emptied. Root writes anywhere, so there the build succeeds, and the
    // service's own tests check a file that cannot be written.
    let written = output("cannot-be-written");
    create_dir_all(&written).expect("made");
    write(
        written.join("ghg-site.json"),
        format!(
            "{{\"format\": 1, \"ghg\": \"{}\", \"files\": [\"index.html\"]}}\n",
            env!("CARGO_PKG_VERSION")
        ),
    )
    .expect("written");
    write(written.join("index.html"), "an earlier page").expect("written");
    set_permissions(&written, Permissions::from_mode(0o500)).expect("set");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("pages"), &["--output", given]);
    set_permissions(&written, Permissions::from_mode(0o700)).expect("set");
    if result.status.code() == Some(0) {
        return;
    }
    let report = String::from_utf8_lossy(&result.stderr).into_owned();
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(report.lines().count(), 1, "{report}");
    assert!(
        report.starts_with(&format!("{given}: cannot be written: ")),
        "{report}"
    );
}

// Rules 79 and 85: every link of the front page is an address within the
// site, and names a page the descriptor lists, and an anchor its object has.

#[test]
fn every_link_of_the_front_page_reaches_a_page_and_its_anchor() {
    let site = fixtures().join("pages/expected");
    let descriptor = site_json(&site);
    let features = descriptor["features"].as_array().expect("a list");
    let pages: Vec<&str> = features
        .iter()
        .map(|feature| feature["page"].as_str().expect("a page"))
        .collect();
    let page = front_page("expected");
    for address in page.split(" href=\"").skip(1) {
        let address = &address[..address.find('"').expect("closed")];
        if address.starts_with("https://") || address.starts_with("assets/") {
            continue;
        }
        let named = address.strip_prefix('#').expect("a hash address");
        let named = percent_decoded(named);
        // Rule 236: an address of two `#` names a part of the shell itself.
        if named.starts_with('#') {
            continue;
        }
        let (target, anchor) = named.split_once('#').unwrap_or((&named, ""));
        let reached = target.is_empty()
            || ["steps", "search", "tags/"].contains(&target)
            || pages.contains(&target)
            || (target.ends_with('/') && pages.iter().any(|page| page.starts_with(target)));
        assert!(reached, "{address} reaches nothing");
        // Rule 45: the block of the scenarios outside every rule.
        if !anchor.is_empty() && anchor != "outside_rules" {
            let object = object_of(&site, target).to_string();
            assert!(
                object.contains(&format!("\"anchor\":\"{anchor}\"")),
                "{address} reaches no anchor"
            );
        }
    }
}

fn percent_decoded(address: &str) -> String {
    let bytes = address.as_bytes();
    let mut decoded = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).expect("ASCII");
            decoded.push(u8::from_str_radix(hex, 16).expect("hexadecimal"));
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).expect("UTF-8")
}

// Rules 22 and 81: --title wins over the file's title.

#[test]
fn the_title_option_wins_over_the_configuration_s() {
    let written = output("titled");
    let result = build(
        &fixtures().join("pages"),
        &[
            "--title",
            "Given",
            "--output",
            written.to_str().expect("UTF-8"),
        ],
    );
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    let home = read_to_string(written.join("index.html")).expect("read");
    assert!(home.contains("<title>Given</title>"), "{home}");
    assert!(home.contains("<h1>Given</h1>"), "{home}");
    assert_eq!(site_json(&written)["title"], "Given");
}

// Rules 81 and 82.

#[test]
fn the_home_page_has_the_site_s_title_and_totals() {
    let home = front_page;
    let titled = home("expected");
    assert!(titled.contains("<title>Banking &#38; &#60;more&#62;</title>"));
    assert!(titled.contains("<h1>Banking &amp; &lt;more&gt;</h1>"));
    assert!(titled.contains(
        "<ul class=\"ghg-totals\">\n<li>5 folders</li>\n<li>8 features</li>\n<li>4 rules</li>\n<li>18 scenarios</li>\n</ul>"
    ));
    let untitled = home("expected-collapsed");
    assert!(untitled.contains("<title>Feature files</title>"));
    assert!(untitled.contains("<h1>Feature files</h1>"));
}

// Rules 86 to 88, 114 and 182: the dialog for a page opened from disk, closed,
// then the skip link, the checkbox and the button, the Theme control and the
// tree's landmark come before the breadcrumb and the content, the tree shown
// as configured.

#[test]
fn the_shell_starts_with_the_dialog_skip_link_tree_controls_tree_and_breadcrumb() {
    for (site, shown) in [("expected", true), ("expected-collapsed", false)] {
        let page = front_page(site);
        let body = page.split_once("<body>\n").expect("a body").1;
        let checked = if shown { " checked" } else { "" };
        let hidden = if shown { "" } else { " hidden" };
        let start = "<dialog id=\"ghg_from_disk\" class=\"ghg-from-disk\" tabindex=\"-1\" \
             aria-labelledby=\"ghg_from_disk_title\" aria-describedby=\"ghg_from_disk_text\">\n\
             <p id=\"ghg_from_disk_title\" class=\"ghg-from-disk-title\">Opened from disk</p>\n\
             <p id=\"ghg_from_disk_text\">This site is meant to be served, \
             so its pages cannot be shown when opened from disk, only its front page. \
             Run ghg serve in the folder of your ghg.toml, or serve the site&#39;s folder with any web server, \
             then open the address it gives.</p>\n\
             <form method=\"dialog\"><button>I agree</button></form>\n\
             </dialog>\n";
        assert!(body.starts_with(start), "{site}:\n{body}");
        let controls = format!(
            "<a class=\"ghg-skip\" href=\"##ghg_main\">Skip to content</a>\n\
             <div class=\"ghg-options\">\n\
             <span class=\"ghg-option ghg-tree-choice\">\
             <input type=\"checkbox\" id=\"ghg_show_tree\" class=\"ghg-show-tree\" aria-controls=\"ghg_tree\"{checked}> \
             <label for=\"ghg_show_tree\">Show the folder tree</label></span>\n\
             <button type=\"button\" class=\"ghg-tree-button\" aria-expanded=\"false\" aria-controls=\"ghg_tree\">Folder tree</button>\n\
             <fieldset class=\"ghg-theme\">\n\
             <legend>Theme</legend>\n\
             <label><input type=\"radio\" name=\"ghg-theme\" value=\"system\" checked> System</label>\n\
             <label><input type=\"radio\" name=\"ghg-theme\" value=\"light\"> Light</label>\n\
             <label><input type=\"radio\" name=\"ghg-theme\" value=\"dark\"> Dark</label>\n\
             </fieldset>\n\
             </div>\n\
             <div class=\"ghg-layout\">\n\
             <nav id=\"ghg_tree\" class=\"ghg-tree\" aria-label=\"Folder tree\"{hidden}></nav>\n\
             <div class=\"ghg-page\">\n\
             <nav class=\"ghg-breadcrumb\""
        );
        let after_script = body
            .split_once("</script>\n")
            .expect("the dialog's script")
            .1;
        assert!(
            after_script.starts_with(&controls),
            "{site}:\n{after_script}"
        );
        assert_eq!(page.matches("<main id=\"ghg_main\"").count(), 1, "{site}");
    }
}
