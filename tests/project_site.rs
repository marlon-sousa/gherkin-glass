//! The project's own site, checked against `project-site/`.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::collections::BTreeSet;
use std::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use gherkin::{Feature, GherkinEnv, Scenario, Step};
use serde_json::Value;

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Builds `input` with `ghg`, which fails if any of its files cannot be read, and gives the
/// address of every feature's page, as the site's search documents list them, so of every
/// feature ghg read.
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
    let documents = read_to_string(site.join("assets/search.json")).expect("search.json is read");
    let documents: Vec<Value> = serde_json::from_str(&documents).expect("search.json is JSON");
    documents
        .iter()
        .filter(|document| document["kind"] == "feature")
        .map(|document| document["page"].as_str().expect("a page").to_string())
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
            show("a scenario's tags", !scenario.tags.is_empty());
            show("a scenario's description", non_empty(&scenario.description));
            show("several examples blocks", scenario.examples.len() > 1);
            for examples in &scenario.examples {
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
    let pages = folder.join("first-pages");
    publish_ok(&first, &["open", pages.to_str().unwrap()]);
    let mut names: Vec<_> = read_dir(&pages)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, [".git", ".nojekyll"]);
    create_dir_all(pages.join("v0.1.0")).unwrap();
    write(pages.join("v0.1.0/index.html"), "0.1.0\n").unwrap();
    write(pages.join("index.html"), "v0.1.0\n").unwrap();
    create_dir_all(pages.join("assets")).unwrap();
    write(pages.join("assets/versions.css"), "a {}\n").unwrap();
    publish_ok(&first, &["publish", pages.to_str().unwrap(), "v0.1.0"]);
    let after_first = published(&remote);
    let files: Vec<_> = after_first
        .iter()
        .map(|line| line.split('\t').nth(1).unwrap())
        .collect();
    assert_eq!(
        files,
        [
            ".nojekyll",
            "assets/versions.css",
            "index.html",
            "v0.1.0/index.html"
        ]
    );
    assert_eq!(commits(&remote), 1);

    // A later release that changed an earlier version publishes nothing.
    let second = folder.join("second");
    checkout(&remote, &second);
    let pages = folder.join("second-pages");
    publish_ok(&second, &["open", pages.to_str().unwrap()]);
    create_dir_all(pages.join("v0.2.0")).unwrap();
    write(pages.join("v0.2.0/index.html"), "0.2.0\n").unwrap();
    write(pages.join("index.html"), "v0.2.0\nv0.1.0\n").unwrap();
    write(pages.join("assets/versions.css"), "b {}\n").unwrap();
    write(pages.join("v0.1.0/index.html"), "changed\n").unwrap();
    let output = publish(&second, &["publish", pages.to_str().unwrap(), "v0.2.0"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("v0.1.0/index.html"));
    assert_eq!(published(&remote), after_first);

    // Without that change, it adds its folder and the versions page alone.
    git_ok(&pages, &["reset", "--quiet"]);
    git_ok(&pages, &["checkout", "--", "v0.1.0"]);
    publish_ok(&second, &["publish", pages.to_str().unwrap(), "v0.2.0"]);
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
    publish_ok(&second, &["publish", pages.to_str().unwrap(), "v0.2.0"]);
    assert_eq!(commits(&remote), 2);

    // Publishing a version again replaces its folder, removals included.
    create_dir_all(pages.join("v0.2.0/more")).unwrap();
    write(pages.join("v0.2.0/more/page.html"), "more\n").unwrap();
    publish_ok(&second, &["publish", pages.to_str().unwrap(), "v0.2.0"]);
    assert_eq!(commits(&remote), 3);
    remove_dir_all(pages.join("v0.2.0/more")).unwrap();
    publish_ok(&second, &["publish", pages.to_str().unwrap(), "v0.2.0"]);
    assert_eq!(commits(&remote), 4);
    assert_eq!(published(&remote), after_second);

    // A name that is not one folder is refused.
    for version in ["", ".", "..", ".hidden", "a/b", "index.html", "assets"] {
        let output = publish(&second, &["publish", pages.to_str().unwrap(), version]);
        assert_eq!(output.status.code(), Some(1), "{version:?}");
    }
}

/// `ghg build` of the site's input, versioned as `version`, into `pages`.
fn build_site(input: &Path, pages: &Path, version: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--title", "gherkin-glass examples", "--input"])
        .arg(input)
        .arg("--output")
        .arg(pages)
        .args(["--version-name", version])
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
        let pages = folder.join(format!("pages-{version}"));
        publish_ok(&checked_out, &["open", pages.to_str().unwrap()]);
        build_site(&input, &pages, version);
        publish_ok(&checked_out, &["publish", pages.to_str().unwrap(), version]);
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
        "v0.1.0/index.html",
        "v0.2.0/fixtures/index.html",
    ] {
        assert!(
            before
                .iter()
                .any(|line| line.ends_with(&format!("\t{page}"))),
            "{page} is published"
        );
    }
}
