//! The project's own site, checked against `project-site/`.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::collections::BTreeSet;
use std::env::{join_paths, split_paths, var_os};
use std::fs::{copy, create_dir_all, read_dir, read_to_string, remove_dir_all, write};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use gherkin::{Feature, GherkinEnv, Scenario, Step};
use rstest::rstest;
use serde_json::Value;

#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use site_files::assert_same_site;

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Builds `input` with `ghg`, which fails if any of its files cannot be read, and gives the
/// name of every feature's page, as the site's descriptor lists them, so of every feature ghg
/// read.
fn builds(input: &Path, name: &str) -> BTreeSet<String> {
    let site = scratch(name).join("site");
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--title", "Check", "--input"])
        .arg(input)
        .arg("--output")
        .arg(&site)
        .output()
        .expect("ghg runs");
    assert!(
        output.status.success(),
        "{}: {}",
        input.display(),
        text(&output.stderr)
    );
    let descriptor = read_to_string(site.join("data/site.json")).expect("the descriptor is read");
    let descriptor: Value = serde_json::from_str(&descriptor).expect("the descriptor is JSON");
    descriptor["features"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|feature| feature["page"].as_str().expect("a page").to_string())
        .collect()
}

/// The feature files under `folder`, as paths from it.
fn feature_files(folder: &Path) -> BTreeSet<PathBuf> {
    let mut found = BTreeSet::new();
    let mut folders = vec![PathBuf::new()];
    while let Some(relative) = folders.pop() {
        for entry in read_dir(folder.join(&relative)).expect("the folder is listed") {
            let entry = entry.expect("an entry of the folder");
            let path = relative.join(entry.file_name());
            if entry.path().is_dir() {
                folders.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "feature")
            {
                found.insert(path);
            }
        }
    }
    found
}

/// The media type written after a doc string's opening delimiter, on the line after its step.
fn media_type<'a>(source: &'a str, step: &Step) -> Option<&'a str> {
    let opener = source.lines().nth(step.position.line)?.trim();
    let rest = opener
        .strip_prefix("\"\"\"")
        .or_else(|| opener.strip_prefix("```"))?
        .trim();
    Some(rest).filter(|rest| !rest.is_empty())
}

/// An empty folder of its own for one test.
fn scratch(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("project-site")
        .join(name);
    let _ = remove_dir_all(&folder);
    create_dir_all(&folder).expect("the scratch folder is made");
    folder
}

/// The names in `project-site/fixtures-left-out`, without comments and blank lines.
fn fixtures_left_out() -> BTreeSet<String> {
    read_to_string(repository().join("project-site/fixtures-left-out"))
        .expect("project-site/fixtures-left-out is read")
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

fn fixture_folders() -> BTreeSet<String> {
    read_dir(repository().join("tests/fixtures"))
        .expect("tests/fixtures is listed")
        .map(|entry| entry.expect("an entry of tests/fixtures"))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect()
}

// What it holds: rules 178 and 179.

#[test]
fn every_fixture_left_out_exists_and_says_why() {
    let text = read_to_string(repository().join("project-site/fixtures-left-out")).unwrap();
    let mut said_why = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('#') {
            said_why = true;
        } else if line.is_empty() {
            said_why = false;
        } else {
            assert!(said_why, "a comment says why {line} is left out");
            assert!(fixture_folders().contains(line), "{line} is a fixture");
            said_why = false;
        }
    }
}

#[test]
fn every_fixture_the_site_shows_reads_with_no_problem() {
    let shown: Vec<_> = fixture_folders()
        .difference(&fixtures_left_out())
        .cloned()
        .collect();
    assert!(!shown.is_empty());
    for name in shown {
        builds(&repository().join("tests/fixtures").join(&name), &name);
    }
}

#[cfg(unix)]
#[test]
fn the_input_holds_the_example_suite_and_the_fixtures_and_reads_with_no_problem() {
    let input = scratch("input").join("input");
    let output = Command::new(repository().join("project-site/assemble-input.sh"))
        .arg(&input)
        .output()
        .expect("the script runs");
    assert!(output.status.success(), "{}", text(&output.stderr));

    let mut expected = builds(&repository().join("project-site/features"), "suite-pages");
    for name in fixture_folders().difference(&fixtures_left_out()) {
        let fixture = repository().join("tests/fixtures").join(name);
        let pages = builds(&fixture, &format!("pages-{name}"));
        expected.extend(pages.iter().map(|page| format!("fixtures/{name}/{page}")));
    }
    assert_eq!(builds(&input, "input-site"), expected);
}

#[cfg(unix)]
#[test]
fn the_input_is_never_written_over_an_existing_folder() {
    let existing = scratch("existing");
    let output = Command::new(repository().join("project-site/assemble-input.sh"))
        .arg(&existing)
        .output()
        .expect("the script runs");
    assert_eq!(output.status.code(), Some(1));
}

/// Every scenario of a feature, its rules' included.
fn scenarios(feature: &Feature) -> impl Iterator<Item = &Scenario> {
    feature
        .scenarios
        .iter()
        .chain(feature.rules.iter().flat_map(|rule| &rule.scenarios))
}

fn steps(feature: &Feature) -> impl Iterator<Item = &Step> {
    let backgrounds = feature
        .background
        .iter()
        .chain(feature.rules.iter().flat_map(|rule| &rule.background));
    backgrounds
        .flat_map(|background| &background.steps)
        .chain(scenarios(feature).flat_map(|scenario| &scenario.steps))
}

fn non_empty(text: &Option<String>) -> bool {
    text.as_deref().is_some_and(|text| !text.trim().is_empty())
}

// Rule 179: the example suite's site, the same as its committed copy byte
// for byte, so that a change to any page shows in review. `cargo xtask build`
// writes the copy again.

#[test]
fn the_example_suite_s_site_is_its_committed_copy() {
    let written = scratch("example-site").join("site");
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--config"])
        .arg(repository().join("project-site/ghg.toml"))
        .arg("--output")
        .arg(&written)
        .output()
        .expect("ghg runs");
    assert_eq!(text(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    assert_same_site(&written, &repository().join("project-site/expected"));
}

#[test]
fn the_example_suite_shows_every_construct() {
    let suite = repository().join("project-site/features");
    builds(&suite, "suite");
    let mut shown = BTreeSet::new();
    let mut show = |construct: &'static str, here: bool| {
        if here {
            shown.insert(construct);
        }
    };
    for path in feature_files(&suite) {
        let source = read_to_string(suite.join(&path)).unwrap();
        let feature = &Feature::parse(&source, GherkinEnv::default()).expect("parsed");
        show("folders within folders", path.components().count() > 2);
        show(
            "a language other than English",
            !["Feature", "Business Need", "Ability"].contains(&feature.keyword.as_str()),
        );
        show("a feature's tags", !feature.tags.is_empty());
        show("a feature's description", non_empty(&feature.description));
        show("a feature's background", feature.background.is_some());
        show("a rule", !feature.rules.is_empty());
        for rule in &feature.rules {
            show("a rule's tags", !rule.tags.is_empty());
            show("a rule's description", non_empty(&rule.description));
            show("a rule's background", rule.background.is_some());
        }
        for scenario in scenarios(feature) {
            show("a scenario as Example", scenario.keyword == "Example");
            show(
                "an outline as Scenario Template",
                scenario.keyword == "Scenario Template",
            );
            show("a scenario's tags", !scenario.tags.is_empty());
            show("a scenario's description", non_empty(&scenario.description));
            show("several examples blocks", scenario.examples.len() > 1);
            for examples in &scenario.examples {
                show("examples as Scenarios", examples.keyword == "Scenarios");
                show("named examples", non_empty(&examples.name));
                show("unnamed examples", !non_empty(&examples.name));
                show("an examples block's tags", !examples.tags.is_empty());
            }
        }
        for step in steps(feature) {
            if let Some(table) = &step.table {
                show("a data table", true);
                let cells = table.rows.iter().flatten();
                show(
                    "a | in a table's cell",
                    cells.clone().any(|cell| cell.contains('|')),
                );
            }
            if let Some(content) = &step.docstring {
                show(
                    "a doc string with a media type",
                    media_type(&source, step).is_some(),
                );
                show(
                    "a doc string without one",
                    media_type(&source, step).is_none(),
                );
                show(
                    "an escaped delimiter",
                    content.contains(r#"\"\"\""#) || content.contains(r"\`\`\`"),
                );
            }
        }
        let delimiters = source.lines().map(str::trim);
        show(
            "a doc string in backticks",
            delimiters.clone().any(|line| line.starts_with("```")),
        );
        show(
            "a doc string in quotes",
            delimiters.clone().any(|line| line.starts_with("\"\"\"")),
        );
    }
    let every: BTreeSet<_> = [
        "folders within folders",
        "a language other than English",
        "a feature's tags",
        "a feature's description",
        "a feature's background",
        "a rule",
        "a rule's tags",
        "a rule's description",
        "a rule's background",
        "a scenario as Example",
        "an outline as Scenario Template",
        "examples as Scenarios",
        "a scenario's tags",
        "a scenario's description",
        "several examples blocks",
        "named examples",
        "unnamed examples",
        "an examples block's tags",
        "a data table",
        "a | in a table's cell",
        "a doc string with a media type",
        "a doc string without one",
        "an escaped delimiter",
        "a doc string in backticks",
        "a doc string in quotes",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        every.difference(&shown).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "constructs the example suite does not show"
    );
}

// Rule 179: the other English words for a scenario, an outline and an
// examples block are all in one feature, so the suite's other English
// features read with only the usual ones.
#[test]
fn the_other_english_keywords_are_in_a_feature_of_their_own() {
    let suite = repository().join("project-site/features");
    let holding: Vec<_> = feature_files(&suite)
        .into_iter()
        .filter(|path| uses_other_words(&suite.join(path)))
        .collect();
    assert_eq!(holding, [PathBuf::from("other-gherkin-syntax.feature")]);
}

/// Whether the feature file at `path` writes a scenario as `Example`, an
/// outline as `Scenario Template` or an examples block as `Scenarios`.
fn uses_other_words(path: &Path) -> bool {
    let source = read_to_string(path).unwrap();
    let feature = Feature::parse(&source, GherkinEnv::default()).expect("parsed");
    scenarios(&feature).any(is_written_in_other_words)
}

fn is_written_in_other_words(scenario: &Scenario) -> bool {
    ["Example", "Scenario Template"].contains(&scenario.keyword.as_str())
        || scenario
            .examples
            .iter()
            .any(|examples| examples.keyword == "Scenarios")
}

// How a release adds to it: rules 180 and 181.

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `git` with `arguments`, run in `folder`, with no configuration but the
/// identity a commit needs.
fn git(folder: &Path, arguments: &[&str]) -> Output {
    Command::new("git")
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(arguments)
        .current_dir(folder)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .expect("git runs")
}

fn git_ok(folder: &Path, arguments: &[&str]) -> String {
    let output = git(folder, arguments);
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        text(&output.stderr)
    );
    text(&output.stdout)
}

/// `project-site/publish.sh` with `arguments`, run in the checkout `folder`.
fn publish(folder: &Path, arguments: &[&str]) -> Output {
    Command::new(repository().join("project-site/publish.sh"))
        .args(arguments)
        .current_dir(folder)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .expect("the script runs")
}

fn publish_ok(folder: &Path, arguments: &[&str]) {
    let output = publish(folder, arguments);
    assert!(
        output.status.success(),
        "publish.sh {arguments:?}: {}",
        text(&output.stderr)
    );
}

/// A checkout of `remote`, as a release's run starts with.
fn checkout(remote: &Path, folder: &Path) {
    git_ok(
        folder.parent().unwrap(),
        &[
            "clone",
            "--quiet",
            remote.to_str().unwrap(),
            folder.to_str().unwrap(),
        ],
    );
}

/// The files of gh-pages on `remote`, each with its blob.
fn published(remote: &Path) -> Vec<String> {
    git_ok(remote, &["ls-tree", "-r", "gh-pages"])
        .lines()
        .map(|line| line.split_once(' ').unwrap().1.to_string())
        .collect()
}

fn path(folder: &Path) -> &str {
    folder.to_str().unwrap()
}

fn commits(remote: &Path) -> usize {
    git_ok(remote, &["rev-list", "--count", "gh-pages"])
        .trim()
        .parse()
        .unwrap()
}

#[cfg(unix)]
#[test]
fn each_release_adds_only_its_own_folder_and_the_versions_page() {
    let folder = scratch("publish");
    let remote = folder.join("remote.git");
    git_ok(&folder, &["init", "--quiet", "--bare", "remote.git"]);
    let seed = folder.join("seed");
    checkout(&remote, &seed);
    write(seed.join("README.md"), "The repository.\n").unwrap();
    git_ok(&seed, &["add", "README.md"]);
    git_ok(&seed, &["commit", "--quiet", "-m", "start"]);
    git_ok(&seed, &["push", "--quiet", "origin", "HEAD"]);

    // The first release creates the branch, with .nojekyll.
    let first = folder.join("first");
    checkout(&remote, &first);
    let site = folder.join("first-pages");
    publish_ok(&first, &["open", path(&site)]);
    let mut names: Vec<_> = read_dir(&site)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, [".git", ".nojekyll"]);
    create_dir_all(site.join("v0.1.0")).unwrap();
    write(site.join("v0.1.0/index.html"), "0.1.0\n").unwrap();
    write(site.join("index.html"), "v0.1.0\n").unwrap();
    create_dir_all(site.join("assets/0.1.0")).unwrap();
    write(site.join("assets/0.1.0/versions.css"), "a {}\n").unwrap();
    publish_ok(&first, &["publish", path(&site), "v0.1.0"]);
    let after_first = published(&remote);
    let files: Vec<_> = after_first
        .iter()
        .map(|line| line.split('\t').nth(1).unwrap())
        .collect();
    assert_eq!(
        files,
        [
            ".nojekyll",
            "assets/0.1.0/versions.css",
            "index.html",
            "v0.1.0/index.html"
        ]
    );
    assert_eq!(commits(&remote), 1);

    // A later release that changed an earlier version publishes nothing.
    let second = folder.join("second");
    checkout(&remote, &second);
    let site = folder.join("second-pages");
    publish_ok(&second, &["open", path(&site)]);
    assert_eq!(
        read_to_string(site.join("v0.1.0/index.html")).unwrap(),
        "0.1.0\n"
    );
    create_dir_all(site.join("v0.2.0")).unwrap();
    write(site.join("v0.2.0/index.html"), "0.2.0\n").unwrap();
    write(site.join("index.html"), "v0.2.0\nv0.1.0\n").unwrap();
    create_dir_all(site.join("assets/0.2.0")).unwrap();
    write(site.join("assets/0.2.0/versions.css"), "b {}\n").unwrap();
    write(site.join("v0.1.0/index.html"), "changed\n").unwrap();
    let output = publish(&second, &["publish", path(&site), "v0.2.0"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("v0.1.0/index.html"));
    assert_eq!(published(&remote), after_first);

    // Nor does one that changed the folder of assets of an earlier ghg.
    write(site.join("v0.1.0/index.html"), "0.1.0\n").unwrap();
    write(site.join("assets/0.1.0/versions.css"), "changed {}\n").unwrap();
    let output = publish(&second, &["publish", path(&site), "v0.2.0"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("assets/0.1.0/versions.css"));
    assert_eq!(published(&remote), after_first);

    // Without those changes, it adds its folder, its assets and the versions page alone.
    write(site.join("assets/0.1.0/versions.css"), "a {}\n").unwrap();
    publish_ok(&second, &["publish", path(&site), "v0.2.0"]);
    let after_second = published(&remote);
    assert_eq!(commits(&remote), 2);
    for line in after_first
        .iter()
        .filter(|line| line.contains("v0.1.0/") || line.ends_with(".nojekyll"))
    {
        assert!(after_second.contains(line), "{line} stays as it was");
    }
    assert!(
        after_second
            .iter()
            .any(|line| line.ends_with("\tv0.2.0/index.html"))
    );

    // Publishing the same build again adds no commit.
    publish_ok(&second, &["publish", path(&site), "v0.2.0"]);
    assert_eq!(commits(&remote), 2);

    // Publishing a version again replaces its folder, removals included.
    create_dir_all(site.join("v0.2.0/more")).unwrap();
    write(site.join("v0.2.0/more/page.html"), "more\n").unwrap();
    publish_ok(&second, &["publish", path(&site), "v0.2.0"]);
    assert_eq!(commits(&remote), 3);
    remove_dir_all(site.join("v0.2.0/more")).unwrap();
    publish_ok(&second, &["publish", path(&site), "v0.2.0"]);
    assert_eq!(commits(&remote), 4);
    assert_eq!(published(&remote), after_second);

    // A name that is not one folder is refused.
    for version in [
        "",
        ".",
        "..",
        ".hidden",
        "a/b",
        "index.html",
        "assets",
        "ghg-site.json",
    ] {
        let output = publish(&second, &["publish", path(&site), version]);
        assert_eq!(output.status.code(), Some(1), "{version:?}");
    }
}

#[cfg(unix)]
#[test]
fn a_release_onto_a_site_of_gherkin_glass_1_adds_its_versions_again() {
    // Rule 180: a gh-pages branch that holds an output folder of gherkin-glass 1 is emptied
    // but for a new .nojekyll, its versions, the folders its marker names, are named for the
    // release to add again, one commit each, and nothing is pushed until the release's own
    // version is added.
    let folder = scratch("publish-again");
    let remote = folder.join("remote.git");
    git_ok(&folder, &["init", "--quiet", "--bare", "remote.git"]);
    let seed = folder.join("seed");
    checkout(&remote, &seed);
    git_ok(&seed, &["checkout", "--quiet", "--orphan", "gh-pages"]);
    for version in ["v1.0.0", "v1.1.0"] {
        create_dir_all(seed.join(version)).unwrap();
        write(seed.join(format!("{version}/index.html")), "1.x\n").unwrap();
    }
    create_dir_all(seed.join("assets")).unwrap();
    write(seed.join("assets/gherkin-glass.css"), "a {}\n").unwrap();
    create_dir_all(seed.join("notes")).unwrap();
    write(seed.join("notes/kept.txt"), "not a version\n").unwrap();
    write(seed.join("index.html"), "v1.1.0\nv1.0.0\n").unwrap();
    write(
        seed.join("ghg-versions.json"),
        "{\"versions\":{\"v1.0.0\":[],\"v1.1.0\":[]}}\n",
    )
    .unwrap();
    write(seed.join(".nojekyll"), "").unwrap();
    git_ok(&seed, &["add", "--all"]);
    git_ok(&seed, &["commit", "--quiet", "-m", "site: v1.1.0"]);
    git_ok(&seed, &["push", "--quiet", "origin", "gh-pages"]);
    let published_by_1 = published(&remote);

    let release = folder.join("release");
    checkout(&remote, &release);
    let site = folder.join("pages");
    let opened = publish(&release, &["open", path(&site)]);
    assert!(opened.status.success(), "{}", text(&opened.stderr));
    assert_eq!(text(&opened.stdout), "v1.0.0\nv1.1.0\n");
    let mut names: Vec<_> = read_dir(&site)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, [".git", ".nojekyll"]);

    // Each version added again is committed, and nothing is pushed yet.
    for version in ["v1.0.0", "v1.1.0"] {
        create_dir_all(site.join(version)).unwrap();
        write(site.join(format!("{version}/index.html")), "2\n").unwrap();
        write(site.join("index.html"), format!("{version}\n")).unwrap();
        write(site.join("ghg-site.json"), "{\"format\":1}\n").unwrap();
        publish_ok(&release, &["commit", path(&site), version]);
    }
    assert_eq!(published(&remote), published_by_1);
    assert_eq!(commits(&remote), 1);

    // The release's own version pushes them all, each a commit of its own.
    create_dir_all(site.join("v2.0.0")).unwrap();
    write(site.join("v2.0.0/index.html"), "2\n").unwrap();
    write(site.join("index.html"), "v2.0.0\n").unwrap();
    publish_ok(&release, &["publish", path(&site), "v2.0.0"]);
    assert_eq!(commits(&remote), 4);
    let files: Vec<_> = published(&remote)
        .iter()
        .map(|line| line.split('\t').nth(1).unwrap().to_string())
        .collect();
    assert_eq!(
        files,
        [
            ".nojekyll",
            "ghg-site.json",
            "index.html",
            "v1.0.0/index.html",
            "v1.1.0/index.html",
            "v2.0.0/index.html"
        ]
    );
    let subjects = git_ok(&remote, &["log", "--format=%s", "gh-pages"]);
    assert_eq!(
        subjects,
        "site: v2.0.0\nsite: v1.1.0\nsite: v1.0.0\nsite: v1.1.0\n"
    );

    // A site of gherkin-glass 2 is kept as it is, and names no version to add again.
    let next = folder.join("next");
    checkout(&remote, &next);
    let site = folder.join("next-pages");
    let opened = publish(&next, &["open", path(&site)]);
    assert!(opened.status.success(), "{}", text(&opened.stderr));
    assert_eq!(text(&opened.stdout), "");
    assert_eq!(
        read_to_string(site.join("v2.0.0/index.html")).unwrap(),
        "2\n"
    );
}

/// `project-site/publish.sh rebuild <pages> <front page>`, run in the checkout `folder` with
/// ghg on the path and `versions` on standard input.
fn rebuild(folder: &Path, pages: &Path, front_page: &Path, versions: &str) -> Output {
    let ghg = Path::new(env!("CARGO_BIN_EXE_ghg")).parent().unwrap();
    let mut search = vec![ghg.to_path_buf()];
    search.extend(split_paths(&var_os("PATH").unwrap()));
    let mut child = Command::new(repository().join("project-site/publish.sh"))
        .args(["rebuild", path(pages), path(front_page)])
        .current_dir(folder)
        .env("PATH", join_paths(search).unwrap())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the script runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(versions.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
#[test]
fn a_version_of_gherkin_glass_1_is_built_again_from_its_own_tag() {
    // Rule 180: each version named is built with ghg from the example suite and the
    // fixtures of its own tag, gathered by that tag's assemble-input.sh, and committed on
    // its own; a version with no tag stops the rebuild, naming it, and commits nothing more.
    let folder = scratch("rebuild");
    let remote = folder.join("remote.git");
    git_ok(&folder, &["init", "--quiet", "--bare", "remote.git"]);
    let seed = folder.join("seed");
    checkout(&remote, &seed);
    create_dir_all(seed.join("project-site/features")).unwrap();
    create_dir_all(seed.join("tests/fixtures/only")).unwrap();
    copy(
        repository().join("project-site/assemble-input.sh"),
        seed.join("project-site/assemble-input.sh"),
    )
    .unwrap();
    write(seed.join("project-site/fixtures-left-out"), "").unwrap();
    write(
        seed.join("project-site/features/lending.feature"),
        "Feature: Lending\n  Scenario: Borrowing\n    Given a book\n",
    )
    .unwrap();
    write(
        seed.join("tests/fixtures/only/kept.feature"),
        "Feature: Kept\n  Scenario: Kept\n    Given a fixture\n",
    )
    .unwrap();
    git_ok(&seed, &["add", "--all"]);
    git_ok(&seed, &["commit", "--quiet", "-m", "v1.0.0"]);
    git_ok(&seed, &["tag", "v1.0.0"]);
    git_ok(&seed, &["push", "--quiet", "--tags", "origin", "HEAD"]);

    let release = folder.join("release");
    checkout(&remote, &release);
    let site = folder.join("pages");
    publish_ok(&release, &["open", path(&site)]);
    let front_page = folder.join("front-page.md");
    write(&front_page, "# Examples\n\n<!-- ghg:contents -->\n").unwrap();

    let built = rebuild(&release, &site, &front_page, "v1.0.0\n");
    assert!(built.status.success(), "{}", text(&built.stderr));
    let descriptor: Value =
        serde_json::from_str(&read_to_string(site.join("data/versions/v1.0.0.json")).unwrap())
            .unwrap();
    let pages: Vec<_> = descriptor["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|feature| feature["page"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(pages, ["fixtures/only/kept", "lending"]);
    assert_eq!(
        git_ok(&site, &["log", "--format=%s"]),
        "site: v1.0.0\n",
        "committed on its own"
    );
    assert_eq!(
        git_ok(&remote, &["branch", "--list", "gh-pages"]),
        "",
        "nothing pushed"
    );

    let missing = rebuild(&release, &site, &front_page, "v0.9.0\nv1.0.0\n");
    assert_eq!(missing.status.code(), Some(1));
    assert!(text(&missing.stderr).starts_with("v0.9.0: "));
    assert_eq!(git_ok(&site, &["log", "--format=%s"]), "site: v1.0.0\n");
}

/// `ghg version add` of the site's input as `version`, into `site`, a
/// checkout of gh-pages, keeping `.nojekyll` and `.git` as the release does.
fn build_site(input: &Path, site: &Path, version: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["version", "add", version])
        .args(["--title", "gherkin-glass examples", "--input"])
        .arg(input)
        .arg("--output")
        .arg(site)
        .args(["--keep-in-output", ".nojekyll", "--keep-in-output", ".git"])
        .current_dir(input)
        .output()
        .expect("ghg runs");
    assert!(output.status.success(), "{}", text(&output.stderr));
}

#[cfg(unix)]
#[test]
fn a_release_publishes_what_ghg_builds_and_keeps_the_earlier_ones() {
    let folder = scratch("release");
    let remote = folder.join("remote.git");
    git_ok(&folder, &["init", "--quiet", "--bare", "remote.git"]);
    let input = folder.join("input");
    let assembled = Command::new(repository().join("project-site/assemble-input.sh"))
        .arg(&input)
        .output()
        .expect("the script runs");
    assert!(assembled.status.success(), "{}", text(&assembled.stderr));

    let mut before = Vec::new();
    for version in ["v0.1.0", "v0.2.0"] {
        let checked_out = folder.join(format!("checkout-{version}"));
        checkout(&remote, &checked_out);
        let site = folder.join(format!("pages-{version}"));
        publish_ok(&checked_out, &["open", path(&site)]);
        build_site(&input, &site, version);
        publish_ok(&checked_out, &["publish", path(&site), version]);
        let after = published(&remote);
        for line in before
            .iter()
            .filter(|line: &&String| line.contains("\tv0.1.0/"))
        {
            assert!(after.contains(line), "{line} stays as it was");
        }
        before = after;
    }
    assert_eq!(commits(&remote), 2);
    for page in [
        "index.html",
        "ghg-site.json",
        "v0.1.0/index.html",
        "v0.2.0/index.html",
        "data/versions/v0.2.0.json",
    ] {
        assert!(
            before
                .iter()
                .any(|line| line.ends_with(&format!("\t{page}"))),
            "{page} is published"
        );
    }
}

#[cfg(unix)]
#[test]
fn every_release_s_site_opens_on_the_readme_of_its_tag() {
    // Rules 192 and 193: the front page made from the README holds the marker, after the
    // part only the site shows, and without the part only GitHub shows, and builds as the
    // versions page's template; the release makes it and passes it to the build.
    let scratch = scratch("readme-front-page");
    let front_page = scratch.join("front-page.md");
    let made = front_page_of(&repository().join("README.md"), &front_page);
    assert_eq!(text(&made.stderr), "");
    assert!(made.status.success());
    let site = scratch.join("site");
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["version", "add", "v0.1.0"])
        .args(["--title", "gherkin-glass examples", "--input"])
        .arg(repository().join("project-site/features"))
        .arg("--front-page")
        .arg(&front_page)
        .arg("--output")
        .arg(&site)
        .output()
        .expect("ghg runs");
    assert_eq!(text(&output.stderr), "");
    assert!(output.status.success());
    let page = read_to_string(site.join("index.html")).expect("the versions page is read");
    assert!(page.contains("<title>gherkin-glass</title>"), "{page}");
    assert!(
        page.contains(
            "<a href=\"https://github.com/marlon-sousa/gherkin-glass\">its repository on GitHub</a>"
        ),
        "{page}"
    );
    assert!(page.contains("<h2>Status</h2>\n<p>"), "{page}");
    assert!(
        page.contains("<h2>The example suite</h2>\n<p>Each version of gherkin-glass builds"),
        "{page}"
    );
    assert!(page.contains("<h3 id=\"releases\">Releases</h3>"), "{page}");
    assert!(!page.contains("Contributing"), "{page}");
    let workflow = read_to_string(repository().join(".github/workflows/site.yml"))
        .expect("the site's workflow is read");
    assert!(
        workflow.contains(
            "\n        run: project-site/front-page.sh README.md \"$RUNNER_TEMP/front-page.md\"\n"
        ),
        "{workflow}"
    );
    assert!(
        workflow.contains(
            "project-site/publish.sh build \"$RUNNER_TEMP/pages\" \"$VERSION\" \"$RUNNER_TEMP/input\" \"$RUNNER_TEMP/front-page.md\"\n"
        ),
        "{workflow}"
    );
    let script = read_to_string(repository().join("project-site/publish.sh"))
        .expect("the publishing script is read");
    assert!(
        script.contains("\n    --front-page \"$front_page\" \\\n"),
        "{script}"
    );
}

/// Rule 193: each part marked for one place is kept or left out with its marks, and every
/// other line is kept as it is.
#[cfg(unix)]
#[test]
fn the_front_page_keeps_the_site_s_parts_and_leaves_out_github_s() {
    let scratch = scratch("front-page-marks");
    let readme = scratch.join("README.md");
    write(
        &readme,
        "# Title\n\n<!-- site-only\nOn the site.\n-->\n\
         <!-- github-only -->\nOn GitHub.\n<!-- end github-only -->\n\
         <!-- ghg:contents -->\nEverywhere.\n",
    )
    .unwrap();
    let front_page = scratch.join("front-page.md");
    let output = front_page_of(&readme, &front_page);
    assert_eq!(text(&output.stderr), "");
    assert!(output.status.success());
    assert_eq!(
        read_to_string(&front_page).unwrap(),
        "# Title\n\nOn the site.\n<!-- ghg:contents -->\nEverywhere.\n"
    );
}

/// Rule 193: a mark out of place stops the script, naming its line, and nothing is written.
#[cfg(unix)]
#[rstest]
#[case::nested(
    "nested",
    "<!-- site-only\n<!-- github-only -->\n",
    "2: a GitHub-only part opens inside another part."
)]
#[case::nested_site(
    "nested-site",
    "<!-- github-only -->\n<!-- site-only\n",
    "2: a site-only part opens inside another part."
)]
#[case::never_closed(
    "never-closed",
    "# Title\n<!-- site-only\nText.\n",
    "2: this part is never closed."
)]
#[case::closing_alone("closing-alone", "# Title\n-->\n", "2: --> closes no site-only part.")]
#[case::closing_the_other(
    "closing-the-other",
    "<!-- github-only -->\n-->\n",
    "2: --> closes no site-only part."
)]
#[case::end_alone(
    "end-alone",
    "<!-- end github-only -->\n",
    "1: <!-- end github-only --> closes no GitHub-only part."
)]
fn a_mark_out_of_place_stops_the_front_page(
    #[case] name: &str,
    #[case] readme_text: &str,
    #[case] message: &str,
) {
    let scratch = scratch(&format!("front-page-{name}"));
    let readme = scratch.join("README.md");
    write(&readme, readme_text).unwrap();
    let front_page = scratch.join("front-page.md");
    let output = front_page_of(&readme, &front_page);
    assert!(!output.status.success());
    assert_eq!(
        text(&output.stderr),
        format!("{}:{message}\n", readme.display())
    );
    assert!(!front_page.exists());
}

/// `project-site/front-page.sh`, writing the front page made from `readme` into `front_page`.
fn front_page_of(readme: &Path, front_page: &Path) -> Output {
    Command::new(repository().join("project-site/front-page.sh"))
        .arg(readme)
        .arg(front_page)
        .output()
        .expect("the script runs")
}

/// Rule 223: the crate's README is what `project-site/crate-readme.sh` writes from the README.
#[cfg(unix)]
#[test]
fn the_crate_s_readme_is_written_from_the_readme() {
    let scratch = scratch("crate-readme");
    let crate_readme = scratch.join("README.crate.md");
    let output = Command::new(repository().join("project-site/crate-readme.sh"))
        .arg(repository().join("README.md"))
        .arg(&crate_readme)
        .output()
        .expect("the script runs");
    assert_eq!(text(&output.stderr), "");
    assert!(output.status.success());
    let written = read_to_string(&crate_readme).expect("the crate's README is written");
    let committed = read_to_string(repository().join("README.crate.md"))
        .expect("the committed crate's README is read");
    assert_eq!(
        committed, written,
        "README.crate.md is not what README.md gives: run project-site/crate-readme.sh README.md README.crate.md"
    );
    assert!(!written.contains("## Contributing"), "{written}");
    assert!(
        written.contains(
            "to show what its pages look like.\n\nSee them on [the project's site](https://marlon-sousa.github.io/gherkin-glass/).\n"
        ),
        "{written}"
    );
}
