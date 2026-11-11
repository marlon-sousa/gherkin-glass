//! The pages, checked by building the fixtures and comparing every file written with the site
//! committed beside them.
//!
//! `cargo run -- build --config tests/fixtures/pages/ghg.toml`, and again with
//! `collapsed.toml`, writes the expected sites again after a change, and so
//! do `tests/fixtures/backgrounds/ghg.toml` and `linked.toml` for
//! "Backgrounds".

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::collections::BTreeMap;
use std::fs::{create_dir_all, read, read_dir, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str::from_utf8;

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

/// Every file below `folder`, by its `/`-joined path in it.
fn files(folder: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut folders = vec![(folder.to_path_buf(), String::new())];
    while let Some((path, prefix)) = folders.pop() {
        for entry in read_dir(&path).expect("the folder is listed") {
            let entry = entry.expect("the folder is listed");
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                folders.push((entry.path(), format!("{name}/")));
            } else {
                found.insert(name, read(entry.path()).expect("the file is read"));
            }
        }
    }
    found
}

fn output(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    folder
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
    let expected = files(&fixtures().join(fixture).join(expected));
    let written = files(&written);
    assert_eq!(
        written.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the files written"
    );
    for (name, contents) in &expected {
        assert!(
            written[name] == *contents,
            "{name} differs from the expected file:\n{}",
            String::from_utf8_lossy(&written[name])
        );
    }
}

// Rules 36, 38 to 51, 77 to 85 and 129 to 132: every page of the fixture,
// its folder pages, and the assets beside them.

#[test]
fn every_feature_file_has_its_page_as_the_expected_site_shows() {
    assert_same_site("ghg.toml", "expected");
}

#[test]
fn the_settings_start_rules_scenarios_and_backgrounds_collapsed() {
    assert_same_site("collapsed.toml", "expected-collapsed");
}

// "Backgrounds", rules 94 to 99: each scenario with its backgrounds' steps, or
// links to them, and the checkbox on every page with a background, disabled
// where no scenario starts with one's steps.

#[test]
fn scenarios_show_their_backgrounds_inline_by_default() {
    assert_same_site_of("backgrounds", "ghg.toml", "expected");
}

#[test]
fn scenarios_link_to_their_backgrounds_when_inline_backgrounds_is_false() {
    assert_same_site_of("backgrounds", "linked.toml", "expected-linked");
}

#[test]
fn every_page_with_a_background_has_the_checkbox() {
    let site = fixtures().join("backgrounds/expected");
    let has = |page: &str| {
        let html = String::from_utf8(read(site.join(page)).expect("read")).expect("UTF-8");
        html.contains("class=\"ghg-inline-backgrounds\"")
    };
    assert!(has("both.html"));
    assert!(has("regra.html"));
    assert!(has("no-steps.html"));
    assert!(has("no-scenarios.html"));
    assert!(!has("none.html"));
}

#[test]
fn the_assets_are_the_ones_built_into_ghg() {
    let expected = fixtures().join("pages/expected/assets");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut built = files(&expected);
    // Rule 39: the folder tree's data is the site's own.
    assert!(built.remove("tree.json").is_some());
    // Rule 147: so are the search documents.
    assert!(built.remove("search.json").is_some());
    assert_eq!(built, files(&repository));
}

#[test]
fn every_page_has_one_h1_and_a_lang_and_its_content_in_its_html() {
    for (name, page) in files(&fixtures().join("pages/expected")) {
        if !name.ends_with(".html") {
            continue;
        }
        let page = String::from_utf8(page).expect("UTF-8");
        assert_eq!(page.matches("<h1").count(), 1, "{name}");
        assert!(page.contains("<html lang=\""), "{name}");
        // Rule 32: the ids the page gives its own parts contain `_`, and
        // are exactly these; every other id is a block's, without one.
        let own = ["ghg_show_tree", "ghg_tree", "ghg_main"];
        let ids: Vec<&str> = page
            .split(" id=\"")
            .skip(1)
            .map(|id| &id[..id.find('"').expect("closed")])
            .collect();
        let background_ids: Vec<&&str> = ids
            .iter()
            .filter(|id| id.contains("_background_"))
            .collect();
        // "Backgrounds", rule 96: each named after its scenario's anchor.
        for id in &background_ids {
            let (anchor, place) = id.split_once("_background_").expect("found");
            assert!(ids.contains(&anchor), "{name}: {id}");
            assert!(place.parse::<usize>().is_ok(), "{name}: {id}");
        }
        // Rule 129: an outline's examples as scenarios, after its anchor.
        for id in ids.iter().filter(|id| id.ends_with("_scenarios")) {
            let anchor = id.strip_suffix("_scenarios").expect("found");
            assert!(ids.contains(&anchor), "{name}: {id}");
        }
        assert_eq!(
            ids.iter()
                .filter(|id| {
                    id.contains('_') && !id.contains("_background_") && !id.ends_with("_scenarios")
                })
                .collect::<Vec<_>>(),
            own.iter().collect::<Vec<_>>(),
            "{name}"
        );
    }
}

#[test]
fn the_page_s_lang_is_its_file_s_language_as_a_tag() {
    let site = fixtures().join("pages/expected");
    let lang = |page: &str| {
        let html = String::from_utf8(read(site.join(page)).expect("read")).expect("UTF-8");
        html.split("<html lang=\"").nth(1).expect("a lang")[..]
            .split('"')
            .next()
            .expect("closed")
            .to_string()
    };
    assert_eq!(lang("billing/refunds.html"), "en");
    assert_eq!(lang("saque em conta poupança.html"), "pt");
    assert_eq!(lang("emoji.html"), "und");
}

// Rules 37, 78 and 142.

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
        "a.story: has the same page, a.html, as a.feature.\n\
         broken.feature:1: Unknown keyword: 'not a feature'.\n\
         index.feature: has the same page, index.html, as the home page.\n\
         steps.feature: has the same page, steps.html, as the step sentences.\n\
         sub/Index.story: has the same page, sub/Index.html, as the page of the folder sub.\n"
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

// Rule 122: a marked folder holding anything ghg did not write is refused.

#[test]
fn a_file_ghg_did_not_write_stops_the_build() {
    let written = output("stray-file");
    let arguments = ["--output", written.to_str().expect("UTF-8")];
    assert_eq!(
        build(&fixtures().join("pages"), &arguments).status.code(),
        Some(0)
    );
    let stray = written.join("billing/old.html");
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
    expected.insert("billing/old.html".into(), b"old".to_vec());
    assert_eq!(files(&written), expected);
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
            "{given}: the output folder holds versions of a site, so build with a version name, or into another folder.\n"
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
    write(written.join("ghg-site.json"), "{\"files\": []}\n").expect("written");
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

// Rules 39, 79 and 85: every link within the site is relative, and reaches a
// page, and an anchor on it when it names one.

#[test]
fn every_link_in_the_site_reaches_a_page_and_its_anchor() {
    for site in ["expected", "expected-collapsed"] {
        let site = fixtures().join("pages").join(site);
        let pages = files(&site);
        for (name, page) in &pages {
            if !name.ends_with(".html") {
                continue;
            }
            let page = String::from_utf8(page.clone()).expect("UTF-8");
            for address in page.split(" href=\"").skip(1) {
                let address = &address[..address.find('"').expect("closed")];
                if address.starts_with("https://") {
                    continue;
                }
                assert!(!address.starts_with('/'), "{name}: {address}");
                let (file, anchor) = address.split_once('#').unwrap_or((address, ""));
                let target = if file.is_empty() {
                    name.clone()
                } else {
                    resolve(name, &percent_decoded(file))
                };
                let found = pages.get(&target);
                assert!(found.is_some(), "{name}: {address} reaches nothing");
                if !anchor.is_empty() {
                    let found = String::from_utf8_lossy(found.expect("found"));
                    assert!(
                        found.contains(&format!(" id=\"{anchor}\"")),
                        "{name}: {address} reaches no anchor"
                    );
                }
            }
        }
    }
}

/// `address`, relative to the page `from`, as a path from the site's root.
fn resolve(from: &str, address: &str) -> String {
    let mut parts: Vec<&str> = from.split('/').collect();
    parts.pop();
    for part in address.split('/') {
        match part {
            ".." => {
                parts.pop().expect("inside the site");
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn percent_decoded(address: &str) -> String {
    let bytes = address.as_bytes();
    let mut decoded = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = from_utf8(&bytes[index + 1..index + 3]).expect("ASCII");
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
    let home = String::from_utf8(read(written.join("index.html")).expect("read")).expect("UTF-8");
    assert!(home.contains("<h1>Given</h1>"), "{home}");
    let page = String::from_utf8(read(written.join("billing/refunds.html")).expect("read"))
        .expect("UTF-8");
    assert!(page.contains("<li><a href=\"../index.html\">Given</a></li>"));
}

// Rules 81 and 82.

#[test]
fn the_home_page_has_the_site_s_title_and_totals() {
    let home = |site: &str| {
        String::from_utf8(
            read(fixtures().join("pages").join(site).join("index.html")).expect("read"),
        )
        .expect("UTF-8")
    };
    let titled = home("expected");
    assert!(titled.contains("<title>Banking &#38; &#60;more&#62;</title>"));
    assert!(titled.contains("<h1>Banking &#38; &#60;more&#62;</h1>"));
    assert!(titled.contains(
        "<ul class=\"ghg-totals\">\n<li>5 folders</li>\n<li>7 features</li>\n<li>3 rules</li>\n<li>11 scenarios</li>\n</ul>"
    ));
    let untitled = home("expected-collapsed");
    assert!(untitled.contains("<title>Feature files</title>"));
    assert!(untitled.contains("<h1>Feature files</h1>"));
}

// Rules 86 to 88 and 114: the skip link, the checkbox, the Theme control and
// the tree's landmark come before the breadcrumb and the content, the tree
// shown as configured.

#[test]
fn every_page_starts_with_the_skip_link_then_the_tree_then_the_breadcrumb() {
    for (site, shown) in [("expected", true), ("expected-collapsed", false)] {
        for (name, page) in files(&fixtures().join("pages").join(site)) {
            if !name.ends_with(".html") {
                continue;
            }
            let page = String::from_utf8(page).expect("UTF-8");
            let body = page.split_once("<body>\n").expect("a body").1;
            let checked = if shown { " checked" } else { "" };
            let hidden = if shown { "" } else { " hidden" };
            let start = format!(
                "<a class=\"ghg-skip\" href=\"#ghg_main\">Skip to content</a>\n\
                 <div class=\"ghg-options\">\n\
                 <input type=\"checkbox\" id=\"ghg_show_tree\" class=\"ghg-show-tree\" aria-controls=\"ghg_tree\"{checked}> \
                 <label for=\"ghg_show_tree\">Show the folder tree</label>\n\
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
            assert!(body.starts_with(&start), "{site}/{name}:\n{body}");
            assert_eq!(page.matches("<main id=\"ghg_main\">").count(), 1, "{name}");
        }
    }
}

// Rule 89: the tree's data, whose every address reaches a page of the site,
// and the anchor it names.

#[test]
fn every_item_of_the_tree_reaches_a_page_and_its_anchor() {
    for site in ["expected", "expected-collapsed"] {
        let pages = files(&fixtures().join("pages").join(site));
        let tree = String::from_utf8(pages["assets/tree.json"].clone()).expect("UTF-8");
        let mut reached = BTreeMap::new();
        for address in tree.split("\"address\":\"").skip(1) {
            let address = &address[..address.find('"').expect("closed")];
            let (file, anchor) = address.split_once('#').unwrap_or((address, ""));
            let found = pages.get(&percent_decoded(file));
            assert!(found.is_some(), "{site}: {address} reaches nothing");
            if !anchor.is_empty() {
                let found = String::from_utf8_lossy(found.expect("found"));
                assert!(
                    found.contains(&format!(" id=\"{anchor}\"")),
                    "{site}: {address} reaches no anchor"
                );
            }
            reached.insert(percent_decoded(file), ());
        }
        // Every page of the site has its item, but the tag pages and the page
        // of step sentences, which the home page links.
        let html: Vec<_> = pages
            .keys()
            .filter(|name| {
                name.ends_with(".html") && !name.starts_with("tags/") && *name != "steps.html"
            })
            .collect();
        assert_eq!(reached.keys().collect::<Vec<_>>(), html, "{site}");
        assert!(
            tree.starts_with("[{\"text\":\""),
            "{site}: the home page first"
        );
        assert!(tree.ends_with("]\n"));
    }
}
