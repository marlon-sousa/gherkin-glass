//! The options that give a key's setting on the command line, checked by building the fixtures
//! with a file that sets the key one way, or not at all, and the option that sets it the way the
//! expected site was built.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rstest::rstest;

#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use site_files::{assert_same_site, files};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A folder for a site a test writes, empty.
fn output(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("key-options")
        .join(name);
    let _ = remove_dir_all(&folder);
    folder
}

/// `ghg build` run in the fixture `name`, writing into `written`, with `arguments` after it.
fn build(name: &str, written: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .arg("--output")
        .arg(written)
        .current_dir(fixture(name))
        .output()
        .expect("the command runs")
}

fn assert_built(result: &Output) {
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
}

// Rule 232: collapsed.toml collapses what ghg.toml expands, and has no title or source link.
#[test]
fn the_options_turn_off_what_the_file_turns_on_and_give_the_source_link() {
    let written = output("pages");
    let result = build(
        "pages",
        &written,
        &[
            "--config",
            "collapsed.toml",
            "--collapse-rules",
            "false",
            "--collapse-scenarios=false",
            "--show-tree",
            "true",
            "--title",
            "Banking & <more>",
            "--source-link",
            "https://example.com/repo/blob/main/features/{path}#L{line}",
        ],
    );
    assert_built(&result);
    assert_same_site(&written, &fixture("pages/expected"));
}

// Rule 232.
#[test]
fn the_backgrounds_option_wins_over_the_file_s_false() {
    let written = output("backgrounds");
    let arguments = ["--config", "unchecked.toml", "--inline-backgrounds", "true"];
    assert_built(&build("backgrounds", &written, &arguments));
    assert_same_site(&written, &fixture("backgrounds/expected"));
}

// Rule 232.
#[test]
fn the_accent_options_colour_the_pages_as_the_keys_do() {
    let written = output("highlighting");
    let arguments = [
        "--config",
        "plain.toml",
        "--accent-light",
        "#5A1E8C",
        "--accent-dark=#fc8",
    ];
    assert_built(&build("highlighting", &written, &arguments));
    assert_same_site(&written, &fixture("highlighting/expected"));
}

// Rule 233: the links of the fixture's ghg.toml, in its order.
#[test]
fn the_tag_link_options_link_the_tags_as_the_tables_do() {
    let written = output("tags");
    let arguments = [
        "--config",
        "plain.toml",
        "--tag-link",
        "@issue-(\\d+)",
        "https://github.com/owner/repo/issues/$1",
        "Issue $1 on GitHub",
        "--tag-link",
        "@issue-(?<id>.+)",
        "https://example.com/other/${id}",
        "Other $id",
        "--tag-link",
        "@later(.*)",
        "https://example.com/later/$1",
        "$1",
        "--tag-link",
        "@ticket:(?<id>.+)",
        "https://example.com/tickets/${id}?view=$$full",
        "Ticket ${id}",
    ];
    assert_built(&build("tags", &written, &arguments));
    assert_same_site(&written, &fixture("tags/expected"));
}

// Rule 233: the links given replace the file's, so that a link that matches no tag leaves
// every tag without a link to another site.
#[test]
fn the_tag_link_options_replace_the_file_s_links() {
    let written = output("tags-replaced");
    let arguments = ["--tag-link", "@nothing", "https://example.com/", "Nothing"];
    assert_built(&build("tags", &written, &arguments));
    let tagged = files(&written)
        .into_iter()
        .filter(|(name, _)| name.ends_with(".html"))
        .find(|(_, page)| String::from_utf8_lossy(page).contains("https://github.com/owner"));
    assert_eq!(tagged.map(|(name, _)| name), None);
}

/// Rule 17: what a build that succeeds prints when the site has no title.
const NO_TITLE: &str = "The site has no title, so its home page is called Feature files. \
                        Set title in ghg.toml, or give --title.\n";

// Rules 35 and 232.
#[test]
fn the_unnamed_option_allows_what_the_file_refuses() {
    let written = output("unnamed");
    let result = build(
        "configuration/unnamed",
        &written,
        &["--allow-unnamed", "true"],
    );
    assert_eq!(String::from_utf8_lossy(&result.stderr), NO_TITLE);
    assert_eq!(result.status.code(), Some(0));
}

// Rules 35 and 232.
#[test]
fn the_unnamed_option_refuses_what_the_file_allows() {
    let written = output("unnamed-refused");
    let arguments = ["--config", "allowed.toml", "--allow-unnamed=false"];
    let result = build("configuration/unnamed-allowed", &written, &arguments);
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("has no name, and allow_unnamed is false."),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(!written.exists(), "nothing is written");
}

// Rules 38 and 232.
#[test]
fn the_replace_option_wins_over_the_file_both_ways() {
    let written = output("replaced");
    assert_built(&build("backgrounds", &written, &[]));
    let arguments = [
        "--config",
        "keep.toml",
        "--replace-existing-version",
        "true",
    ];
    let result = build("pages", &written, &arguments);
    assert_eq!(String::from_utf8_lossy(&result.stderr), NO_TITLE);
    assert_eq!(result.status.code(), Some(0));
    let before = files(&written);
    let result = build("pages", &written, &["--replace-existing-version", "false"]);
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "{}: a site is already built here, and replace_existing_version is false.\n",
            written.display()
        )
    );
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(files(&written), before);
}

// Rules 19 and 234: each such mistake stops the run before anything is read, with status 2.
#[rstest]
#[case::not_boolean(&["--collapse-rules", "yes"], "--collapse-rules must be true or false.\n")]
#[case::faint_accent(
    &["--accent-light", "#777777"],
    "--accent-light is #777777, which reaches only 4.00:1 against the light theme's \
     background #f2f2f2. It needs at least 7:1.\n"
)]
#[case::unknown_group(
    &["--tag-link", "@a(.)", "$2", "Issue"],
    "--tag-link names the group 2, which the pattern does not have.\n"
)]
fn a_value_the_key_would_refuse_is_a_mistake_on_the_command_line(
    #[case] arguments: &[&str],
    #[case] message: &str,
) {
    let option = arguments.first().expect("an option");
    let written = output(&format!("mistaken{option}"));
    let result = build("configuration/default", &written, arguments);
    assert_eq!(String::from_utf8_lossy(&result.stderr), message);
    assert_eq!(result.status.code(), Some(2));
    assert!(!written.exists(), "nothing is written");
}
