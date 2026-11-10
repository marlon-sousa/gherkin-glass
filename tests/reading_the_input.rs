//! docs/BEHAVIOUR.md, "Reading the input", checked against the fixtures.

#[cfg(unix)]
use std::fs::{Permissions, set_permissions};
use std::fs::{copy, create_dir_all, read_dir, read_to_string, remove_dir_all, write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use gherkin::Step;
use gherkin_glass::{
    GherkinParser, InputService, InputWalker, ParsedFeature, ProblemKind, Problems,
};

/// The service as the command will wire it, over the real file system.
fn read_input(input: &Path) -> Result<Vec<ParsedFeature>, Problems> {
    InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser)).read(input)
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read(input: &Path) -> Vec<ParsedFeature> {
    read_input(input).unwrap_or_else(|problems| panic!("the input is read:\n{problems}"))
}

/// The paths read, with `/` between folders.
fn paths(features: &[ParsedFeature]) -> Vec<String> {
    features
        .iter()
        .map(|feature| {
            let parts: Vec<_> = feature
                .path
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect();
            parts.join("/")
        })
        .collect()
}

/// A folder of its own under the target directory, emptied first.
fn scratch(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    create_dir_all(&folder).expect("the scratch folder is made");
    folder
}

fn copy_folder(from: &Path, to: &Path) {
    create_dir_all(to).expect("the folder is made");
    for entry in read_dir(from).expect("the fixture is listed") {
        let entry = entry.expect("the fixture is listed");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("the entry has a type").is_dir() {
            copy_folder(&entry.path(), &target);
        } else {
            copy(entry.path(), target).expect("the file is copied");
        }
    }
}

// Which files are read: rules 1 to 4.

#[test]
fn only_the_files_the_defaults_and_the_ghg_ignore_files_allow_are_read() {
    let features = read(&fixture("input-files"));
    assert_eq!(
        paths(&features),
        [
            // Rule 3: `!.github/` in the input's .ghg-ignore reads a hidden folder again.
            ".github/read-again.feature",
            // Rule 4: the fixture's .gitignore leaves this folder out, to no effect.
            "ignored-by-gitignore/read.feature",
            // Rule 4: the fixture's .ignore leaves this file out, to no effect.
            "read-despite-ignore.feature",
            // Rule 1.
            "read.feature",
            // Rule 3: `!*.story` reads a file of another type.
            "stories/read.story",
            // Rule 3: the nearest .ghg-ignore decides.
            "sub/deeper/read-again.feature",
            "sub/deeper/read.feature",
            "sub/read.feature",
        ],
        // Left out: notes.md and other.txt (rule 1), .hidden.feature and
        // .hidden/ (rule 2), drafts/ with the `!` rule inside it,
        // sub/left-out.feature and stories/left-out.story (rule 3).
    );
}

#[test]
fn the_ignore_files_of_the_folders_above_the_input_have_no_effect() {
    // Rule 4. The input sits in a git repository with a .gitignore, an .ignore
    // and an exclude file above it, each of which would leave everything out.
    let root = scratch("ignore-files-above");
    create_dir_all(root.join(".git/info")).expect("the git folder is made");
    write(root.join(".git/info/exclude"), "*.feature\n").expect("written");
    write(root.join(".gitignore"), "*.feature\n").expect("written");
    write(root.join(".ignore"), "*.feature\n").expect("written");
    let input = root.join("input");
    copy_folder(&fixture("input-files"), &input);

    assert_eq!(paths(&read(&input)), paths(&read(&fixture("input-files"))));
}

#[test]
fn a_ghg_ignore_file_is_never_read_as_a_feature() {
    // Rule 3, last point: `!.*` reads hidden files again, but not this one.
    let input = scratch("ghg-ignore-not-a-feature");
    write(input.join(".ghg-ignore"), "!.*\n").expect("written");
    copy(
        fixture("input-files/read.feature"),
        input.join(".hidden.feature"),
    )
    .expect("copied");

    assert_eq!(paths(&read(&input)), [".hidden.feature"]);
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_not_followed() {
    // Rule 5.
    use std::os::unix::fs::symlink;

    let root = scratch("symbolic-links");
    let input = root.join("input");
    create_dir_all(&input).expect("the input is made");
    let elsewhere = root.join("elsewhere");
    copy_folder(&fixture("input-files/sub"), &elsewhere);
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    symlink(elsewhere.join("read.feature"), input.join("linked.feature")).expect("linked");
    symlink(&elsewhere, input.join("linked-folder")).expect("linked");

    assert_eq!(paths(&read(&input)), ["read.feature"]);
}

// What is read: rules 6 to 8.

#[test]
fn every_construct_is_read() {
    // Rule 6.
    let features = read(&fixture("constructs"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new("every-construct.feature"))
        .expect("the feature is read")
        .feature;

    assert_eq!(feature.name, "Every construct");
    assert_eq!(feature.tags, ["billing", "slow"]);
    assert_eq!(
        feature.description.as_deref(),
        Some("A description of the feature,\nover two lines.")
    );
    let background = feature.background.as_ref().expect("a background");
    assert_eq!(background.name, "A logged in customer");

    let plain = &feature.scenarios[0];
    assert_eq!(plain.name, "A plain scenario");
    assert_eq!(
        plain.description.as_deref(),
        Some("A description of the scenario.")
    );
    let keywords: Vec<_> = plain.steps.iter().map(|step| step.keyword.trim()).collect();
    assert_eq!(keywords, ["Given", "When", "Then", "And", "But", "*"]);
    let table = plain.steps[0].table.as_ref().expect("a data table");
    assert_eq!(
        table.rows,
        [["item", "price"], ["pen", "1.50"], ["a | b", "2"]]
    );

    let outline = &feature.scenarios[1];
    assert_eq!(outline.keyword, "Scenario Outline");
    assert_eq!(outline.tags, ["outline"]);
    assert_eq!(outline.examples.len(), 2);
    assert_eq!(outline.examples[0].name.as_deref(), Some("Cheap"));
    assert_eq!(outline.examples[0].tags, ["cheap"]);
    assert_eq!(
        outline.examples[1].table.as_ref().expect("a table").rows,
        [["count", "total"], ["9", "90"]]
    );

    let rule = &feature.rules[0];
    assert_eq!(rule.name, "A rule with its own background");
    assert_eq!(rule.description.as_deref(), Some("The rule's description."));
    assert_eq!(
        rule.background
            .as_ref()
            .expect("the rule's background")
            .name,
        "Inside the rule"
    );
    assert_eq!(rule.scenarios[0].tags, ["in-rule"]);
    assert_eq!(
        rule.scenarios[0].steps[0].docstring.as_deref(),
        Some("Some text.")
    );
}

#[test]
fn a_feature_is_read_in_its_own_language() {
    // Rule 7.
    let features = read(&fixture("constructs"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new("em-portugues.feature"))
        .expect("the feature is read")
        .feature;

    assert_eq!(feature.keyword, "Funcionalidade");
    assert_eq!(feature.name, "Uma funcionalidade em português");
    assert_eq!(
        feature.background.as_ref().expect("a background").keyword,
        "Contexto"
    );
    let keywords: Vec<_> = feature.scenarios[0]
        .steps
        .iter()
        .map(|step| step.keyword.trim())
        .collect();
    assert_eq!(keywords, ["Quando", "Então", "E"]);
}

#[test]
fn comments_are_not_kept() {
    // Rule 8: the fixture has comments at the top, inside the feature's
    // description and between two steps.
    let features = read(&fixture("constructs"));
    let feature = &features
        .iter()
        .find(|feature| feature.path == Path::new("every-construct.feature"))
        .expect("the feature is read")
        .feature;
    assert_eq!(
        feature.description.as_deref(),
        Some("A description of the feature,\nover two lines.")
    );
    let steps: Vec<_> = feature.scenarios[0]
        .steps
        .iter()
        .map(|step| step.value.as_str())
        .collect();
    assert_eq!(
        steps,
        [
            "an order with these lines:",
            "the order is placed",
            "a receipt is sent",
            "no error is shown",
            "nothing is charged twice",
            "the order is saved",
        ]
    );
}

// Doc strings: rules 9 to 13.

/// Each doc string of the fixture, with its media type, in the order of the file.
fn doc_strings(input: &Path) -> Vec<(Option<String>, String)> {
    let features = read(input);
    let parsed = &features[0];
    let feature = &parsed.feature;
    let steps: Vec<&Step> = feature
        .background
        .iter()
        .flat_map(|background| &background.steps)
        .chain(
            feature
                .scenarios
                .iter()
                .flat_map(|scenario| &scenario.steps),
        )
        .chain(
            feature
                .rules
                .iter()
                .flat_map(|rule| rule.scenarios.iter().flat_map(|scenario| &scenario.steps)),
        )
        .collect();
    steps
        .into_iter()
        .filter_map(|step| {
            let content = step.docstring.clone()?;
            Some((parsed.media_type(step).map(str::to_string), content))
        })
        .collect()
}

fn expected_doc_strings() -> Vec<(Option<String>, String)> {
    [
        (None, "From the background."),
        // Rules 9, 11 and 12.
        (Some("json"), "{\n  \"quote\": \"\"\"\"\"\n}"),
        // Rules 9 and 12.
        (Some("markdown"), "Code fences look like ``` in Markdown."),
        // Rule 11.
        (
            None,
            "indented as the delimiter\nindented less\n    indented more",
        ),
        // Rule 10: blank lines inside are kept, and a comment before is not content.
        (None, "\nbetween blank lines\n"),
        // Rule 11: a tab is one character of indentation. Rule 10: a line
        // that looks like a comment is content, and a delimiter on it does
        // not close the doc string.
        (
            None,
            "one tab, as indentation\n\ta tab after the indentation\n# A line like a comment, with \"\"\" in it, is content.\nx = 1 # and from a hash on any line, \"\"\" does not close it either",
        ),
        (Some("text"), "Inside the rule."),
    ]
    .into_iter()
    .map(|(media_type, content)| (media_type.map(str::to_string), content.to_string()))
    .collect()
}

#[test]
fn doc_strings_are_read_as_the_author_wrote_them() {
    assert_eq!(doc_strings(&fixture("doc-strings")), expected_doc_strings());
}

#[test]
fn doc_strings_read_the_same_with_carriage_returns() {
    // Rule 13.
    let input = scratch("doc-strings-crlf");
    let source = read_to_string(fixture("doc-strings/doc-strings.feature")).expect("read");
    let crlf = source.replace("\r\n", "\n").replace('\n', "\r\n");
    write(input.join("doc-strings.feature"), crlf).expect("written");

    assert_eq!(doc_strings(&input), expected_doc_strings());
}

// Files that cannot be read: rules 14 to 16.

#[test]
fn every_file_that_cannot_be_read_is_reported_sorted_by_path() {
    // The report as `ghg build` prints it, in the catalogue's words.
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .arg("--input")
        .arg(fixture("unreadable"))
        .output()
        .expect("the command runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).expect("the report is UTF-8"),
        [
            "empty.feature:1: Syntax error.",
            "folder/.ghg-ignore:2: error parsing glob 'a{b': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)",
            "folder/uneven-table.feature:5: Inconsistent cell count",
            "not-utf-8.feature: cannot be read: stream did not contain valid UTF-8",
            "unknown-keyword.feature:4: Unknown keyword: 'Blah'.",
            "unknown-language.feature:2: Requested language 'xx-unknown' is not supported.",
            "",
        ]
        .join("\n")
    );
}

#[test]
fn a_folder_sorts_by_its_name_before_its_contents() {
    // Rule 15: `a/` comes before `a-b.feature`, folder by folder, although `/`
    // sorts after `-` as a character.
    let input = scratch("sorted-by-folder");
    create_dir_all(input.join("a")).expect("made");
    write(input.join("a/x.feature"), "Nothing\n").expect("written");
    write(input.join("a-b.feature"), "Nothing\n").expect("written");

    let problems = read_input(&input).expect_err("cannot be read");
    let paths: Vec<_> = problems
        .as_slice()
        .iter()
        .map(|problem| problem.path.as_str())
        .collect();
    assert_eq!(paths, ["a/x.feature", "a-b.feature"]);
}

/// Makes `path` unreadable, and says whether it worked: it does not for root.
#[cfg(unix)]
fn lock(path: &Path) -> bool {
    set_permissions(path, Permissions::from_mode(0o000)).expect("the mode is set");
    let locked = read_dir(path).is_err() && read_to_string(path).is_err();
    if !locked {
        eprintln!(
            "skipped: {} can still be read, as it can by root",
            path.display()
        );
    }
    locked
}

#[cfg(unix)]
fn unlock(path: &Path, mode: u32) {
    set_permissions(path, Permissions::from_mode(mode)).expect("the mode is set");
}

#[cfg(unix)]
#[test]
fn a_folder_that_cannot_be_listed_is_reported() {
    // Rule 14.
    let input = scratch("locked-folder");
    create_dir_all(input.join("locked")).expect("made");
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    if !lock(&input.join("locked")) {
        unlock(&input.join("locked"), 0o755);
        return;
    }
    let problems = read_input(&input);
    unlock(&input.join("locked"), 0o755);

    let problems = problems.expect_err("cannot be read");
    let problems = problems.as_slice();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].path, "locked");
    assert!(
        matches!(problems[0].kind, ProblemKind::CannotBeListed { .. }),
        "{}",
        problems[0]
    );
}

#[cfg(unix)]
#[test]
fn a_ghg_ignore_file_or_a_feature_file_that_cannot_be_opened_is_reported() {
    // Rules 14 and 16.
    let input = scratch("locked-files");
    write(input.join(".ghg-ignore"), "drafts/\n").expect("written");
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    let locked = lock(&input.join(".ghg-ignore")) && lock(&input.join("read.feature"));
    if !locked {
        unlock(&input.join(".ghg-ignore"), 0o644);
        unlock(&input.join("read.feature"), 0o644);
        return;
    }
    let problems = read_input(&input);
    unlock(&input.join(".ghg-ignore"), 0o644);
    unlock(&input.join("read.feature"), 0o644);

    let problems = problems.expect_err("cannot be read");
    let found: Vec<_> = problems
        .as_slice()
        .iter()
        .map(|problem| {
            let unreadable = matches!(problem.kind, ProblemKind::CannotBeRead { .. });
            (problem.path.as_str(), problem.line, unreadable)
        })
        .collect();
    assert_eq!(
        found,
        [(".ghg-ignore", None, true), ("read.feature", None, true)]
    );
}

#[test]
fn an_input_folder_that_does_not_exist_is_reported() {
    // Rule 14.
    let input = scratch("missing").join("not-there");
    let problems = read_input(&input).expect_err("cannot be read");
    let problems = problems.as_slice();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].path, input.display().to_string());
    assert!(
        matches!(problems[0].kind, ProblemKind::CannotBeListed { .. }),
        "{}",
        problems[0]
    );
}
