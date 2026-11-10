//! docs/BEHAVIOUR.md, "The command line and its configuration", checked by running `ghg`.

use std::fs::{copy, create_dir_all, read_dir, read_to_string, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/configuration")
        .join(name)
}

/// `ghg` with `arguments`, run in `folder`. A folder of the fixtures is run
/// in a copy of them, so that the site a build writes stays out of the
/// repository.
fn ghg(folder: &Path, arguments: &[&str]) -> Output {
    let folder = in_a_copy(folder);
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("the command runs")
}

fn in_a_copy(folder: &Path) -> PathBuf {
    static COPIES: AtomicUsize = AtomicUsize::new(0);
    let fixtures = fixture("");
    let Ok(inside) = folder.strip_prefix(&fixtures) else {
        return folder.to_path_buf();
    };
    let copy = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("configuration")
        .join(COPIES.fetch_add(1, Ordering::SeqCst).to_string());
    let _ = remove_dir_all(&copy);
    copy_folder(&fixtures, &copy);
    copy.join(inside)
}

fn copy_folder(from: &Path, to: &Path) {
    create_dir_all(to).expect("the folder is made");
    for entry in read_dir(from).expect("the fixture is listed") {
        let entry = entry.expect("the fixture is listed");
        let path = entry.path();
        if path.is_dir() {
            copy_folder(&path, &to.join(entry.file_name()));
        } else {
            copy(&path, to.join(entry.file_name())).expect("the file is copied");
        }
    }
}

/// Rule 17: what a build that succeeds prints when the site has no title.
const NO_TITLE: &str = "The site has no title, so its home page is called Feature files. \
                        Set title in ghg.toml, or give --title.\n";

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("the output is UTF-8")
}

/// What `ghg` printed to standard error, checking it printed nothing else
/// and exited with `status`.
fn report(output: Output, status: i32) -> String {
    assert_eq!(text(output.stdout), "");
    assert_eq!(output.status.code(), Some(status));
    text(output.stderr)
}

// Commands: rules 17 to 20.

#[test]
fn build_reads_the_input_and_says_nothing_when_it_can_be_read() {
    let output = ghg(&fixture("default"), &["build", "--title", "Default"]);
    assert_eq!(report(output, 0), "");
}

#[test]
fn build_without_a_title_hints_how_to_give_one() {
    let output = ghg(&fixture("default"), &["build"]);
    assert_eq!(report(output, 0), NO_TITLE);
}

#[test]
fn help_lists_the_commands_and_options() {
    for arguments in [&["--help"][..], &["-h"], &["build", "--help"]] {
        let output = ghg(&fixture("default"), arguments);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(text(output.stderr), "");
        let help = text(output.stdout);
        for expected in [
            "Usage: ghg <command>",
            "build: ",
            "--config <file>: ",
            "--input <folder>: ",
            "--output <folder>: ",
            "--title <text>: ",
            "--version-name <name>: ",
        ] {
            assert!(help.contains(expected), "{expected:?} in:\n{help}");
        }
    }
}

#[test]
fn version_prints_the_name_and_the_version() {
    for arguments in [&["--version"][..], &["-V"]] {
        let output = ghg(&fixture("default"), arguments);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            text(output.stdout),
            format!("ghg {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn each_mistake_on_the_command_line_is_one_sentence_and_status_2() {
    let cases: [(&[&str], &str); 8] = [
        (
            &[],
            "ghg needs a command, such as ghg build. ghg --help lists the commands.",
        ),
        (
            &["biuld"],
            "ghg has no command named biuld. ghg --help lists the commands.",
        ),
        (
            &["build", "--inptu", "x"],
            "ghg build has no option --inptu. ghg --help lists the options.",
        ),
        (&["build", "--input"], "The option --input needs a value."),
        (
            &["build", "--config="],
            "The option --config needs a value.",
        ),
        (&["--help=yes"], "The option --help takes no value."),
        (
            &["build", "--input", "a", "--input", "b"],
            "The option --input is given more than once.",
        ),
        (
            &["build", "features"],
            "ghg build takes no argument features.",
        ),
    ];
    for (arguments, sentence) in cases {
        let output = ghg(&fixture("default"), arguments);
        assert_eq!(report(output, 2), format!("{sentence}\n"), "{arguments:?}");
    }
}

// The configuration file: rules 21 to 26, with the output of rule 38.

#[test]
fn the_file_s_input_is_read_relative_to_its_folder() {
    assert_eq!(report(ghg(&fixture("configured"), &["build"]), 0), NO_TITLE);
    // ci/ghg.toml names ../features, the broken folder beside ci.
    let output = ghg(
        &fixture("configured"),
        &["build", "--config", "ci/ghg.toml"],
    );
    assert_eq!(
        report(output, 1),
        "broken.feature:4: Unknown keyword: 'Blah'.\n"
    );
}

#[test]
fn the_default_input_is_beside_the_file_too() {
    // nested/ghg.toml sets no input, so nested/features is read, not the
    // broken features folder of the current one.
    let output = ghg(
        &fixture("configured"),
        &["build", "--config", "nested/ghg.toml"],
    );
    assert_eq!(report(output, 0), NO_TITLE);
}

#[test]
fn a_ghg_toml_that_cannot_be_read_stops_the_run() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ghg-toml-is-a-folder");
    let _ = remove_dir_all(&folder);
    create_dir_all(folder.join("ghg.toml")).expect("the folder is made");
    let report = report(ghg(&folder, &["build"]), 1);
    assert!(report.starts_with("ghg.toml: cannot be read: "), "{report}");
    assert_eq!(report.lines().count(), 1);
}

#[test]
fn the_site_is_written_into_the_output_folder_the_option_or_the_file_names() {
    // configured/ghg.toml sets no output, so its site is beside it, in site.
    let folder = in_a_copy(&fixture("configured"));
    let run = |arguments: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ghg"))
            .args(arguments)
            .current_dir(&folder)
            .output()
            .expect("the command runs")
    };
    assert_eq!(report(run(&["build"]), 0), NO_TITLE);
    assert!(folder.join("site/read.html").is_file());
    assert!(folder.join("site/assets/gherkin-glass.css").is_file());
    assert_eq!(
        report(run(&["build", "--output", "elsewhere"]), 0),
        NO_TITLE
    );
    assert!(folder.join("elsewhere/read.html").is_file());
    // ci/ghg.toml's output, out, is relative to its own folder; its input
    // is broken, so --input names the good one.
    let output = run(&["build", "--config", "ci/ghg.toml", "--input", "specs"]);
    assert_eq!(report(output, 0), NO_TITLE);
    let page = read_to_string(folder.join("ci/out/read.html")).expect("the page is written");
    // Rule 47: ci/ghg.toml's source_link.
    assert!(
        page.contains(r#"<a href="https://example.com/blob/main/specs/read.feature#L1">"#),
        "{page}"
    );
}

#[test]
fn the_option_overrides_the_file() {
    let output = ghg(&fixture("configured"), &["build", "--input", "features"]);
    assert_eq!(
        report(output, 1),
        "broken.feature:4: Unknown keyword: 'Blah'.\n"
    );
}

#[test]
fn a_file_asked_for_that_cannot_be_read_stops_the_run() {
    let output = ghg(&fixture("default"), &["build", "--config", "missing.toml"]);
    let report = report(output, 1);
    assert!(
        report.starts_with("missing.toml: cannot be read: "),
        "{report}"
    );
    assert_eq!(report.lines().count(), 1);
}

#[test]
fn the_file_is_looked_for_in_the_current_folder_only() {
    // tests/fixtures/configuration/configured/specs has no ghg.toml, and the
    // one in the folder above is not read, so the default input is missing.
    // Rule 16 words the missing input folder.
    let output = ghg(&fixture("configured").join("specs"), &["build"]);
    assert_eq!(
        report(output, 1),
        "features: the input folder does not exist. Create it, or name another with input in ghg.toml or with --input.\n"
    );
}

#[test]
fn every_unknown_key_and_wrong_type_is_reported_with_its_line() {
    let output = ghg(&fixture("mistaken"), &["build"]);
    assert_eq!(
        report(output, 1),
        "ghg.toml:3: ghg has no setting named inputs.\n\
         ghg.toml:4: input must be text, in quotes.\n\
         ghg.toml:6: ghg has no setting named site.\n"
    );
}

#[test]
fn an_empty_path_is_reported() {
    let output = ghg(&fixture("empty-input"), &["build"]);
    assert_eq!(report(output, 1), "ghg.toml:2: input must not be empty.\n");
}

#[test]
fn a_file_that_is_not_toml_is_reported_where_it_stops() {
    // The message is the toml crate's.
    let output = ghg(&fixture("not-toml"), &["build"]);
    assert_eq!(
        report(output, 1),
        "ghg.toml:2: string values must be quoted, expected literal string\n"
    );
}

#[test]
fn every_problem_with_the_keys_of_pages_is_reported() {
    let output = ghg(&fixture("mistaken-pages"), &["build"]);
    assert_eq!(
        report(output, 1),
        "ghg.toml:2: output must not be empty.\n\
         ghg.toml:3: source_link must be text, in quotes.\n\
         ghg.toml:4: collapse_rules must be true or false.\n\
         ghg.toml:5: collapse_scenarios must be true or false.\n\
         ghg.toml:6: show_tree must be true or false.\n\
         ghg.toml:7: inline_backgrounds must be true or false.\n"
    );
}

#[test]
fn the_file_given_is_shown_as_it_was_given() {
    let output = ghg(&fixture("."), &["build", "--config", "mistaken/ghg.toml"]);
    let report = report(output, 1);
    assert!(report.starts_with("mistaken/ghg.toml:3: "), "{report}");
}

#[test]
fn allow_unnamed_must_be_true_or_false() {
    let output = ghg(&fixture("not-boolean"), &["build"]);
    assert_eq!(
        report(output, 1),
        "ghg.toml:1: allow_unnamed must be true or false.\n"
    );
}

// Anchors: rule 35.

#[test]
fn unnamed_blocks_stop_the_build_when_allow_unnamed_is_false() {
    let output = ghg(&fixture("unnamed"), &["build"]);
    assert_eq!(
        report(output, 1),
        "unnamed.feature:1: The feature has no name, and allow_unnamed is false.\n\
         unnamed.feature:6: The scenario has no name, and allow_unnamed is false.\n\
         unnamed.feature:9: The scenario outline has no name, and allow_unnamed is false.\n\
         unnamed.feature:16: The rule has no name, and allow_unnamed is false.\n"
    );
}

#[test]
fn unnamed_blocks_are_allowed_by_default_and_when_allow_unnamed_is_true() {
    let folder = fixture("unnamed-allowed");
    assert_eq!(report(ghg(&folder, &["build"]), 0), NO_TITLE);
    let output = ghg(&folder, &["build", "--config", "allowed.toml"]);
    assert_eq!(report(output, 0), NO_TITLE);
}

// Highlighting and themes: rules 118 and 119.

#[test]
fn accents_that_reach_7_to_1_build() {
    let output = ghg(&fixture("accents"), &["build"]);
    assert_eq!(report(output, 0), "");
}

#[test]
fn an_accent_must_be_a_colour_in_hexadecimal() {
    let output = ghg(&fixture("not-colour"), &["build"]);
    assert_eq!(
        report(output, 1),
        "ghg.toml:1: accent_light must be a colour in hexadecimal, in quotes, such as \"#0b4f9c\".\n\
         ghg.toml:2: accent_dark must be a colour in hexadecimal, in quotes, such as \"#0b4f9c\".\n"
    );
}

#[test]
fn an_accent_short_of_7_to_1_names_the_colour_the_theme_and_the_ratio() {
    let output = ghg(&fixture("faint-accents"), &["build"]);
    assert_eq!(
        report(output, 1),
        "ghg.toml:2: accent_light is #8cc4ff, which reaches only 1.63:1 against the light \
         theme's background #f2f2f2. It needs at least 7:1.\n\
         ghg.toml:4: accent_dark is #0b4f9c, which reaches only 1.97:1 against the dark \
         theme's background #222222. It needs at least 7:1.\n"
    );
}
