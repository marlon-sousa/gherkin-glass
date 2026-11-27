//! Composition root: `cargo xtask build`, `cargo xtask check` and `cargo xtask comment-ratio`, the commands contributors and CI run.

mod checks;
mod comment_ratio;
mod expected_sites;
mod expected_stores;

use std::env::consts::EXE_SUFFIX;
use std::env::{args, var_os};
use std::fs::{read, read_to_string};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use checks::{CARGO_CHECKS, MINIMUM_RUST_TEST};
use comment_ratio::Lines;
use expected_sites::{put_back_placeholder, replace_folder};
use expected_stores::{fixture_stores, history_store, versions_site};

const USAGE: &str = "\
Usage: cargo xtask <command>

Commands:
  build: builds the pages' script from script/ into assets/gherkin-glass.js, minified, with the licences of the packages it holds in assets/THIRD-PARTY-LICENSES.txt, writes the fixtures' and the example suite's expected sites again, the expected stores of the fixtures that commit no site and of the history of versions, and the crate's README, README.crate.md, from README.md, then builds the release binary.
  check: runs every check a pull request runs: formatting, lints, the tests on both Rust versions, the documentation, the dependencies with cargo deny, the package, the script check and the browser check.
  check-script: type-checks and tests the script, and checks that assets/gherkin-glass.js is what it compiles to and assets/THIRD-PARTY-LICENSES.txt the licences of what it holds.
  check-browser: builds ghg, installs the browser, and checks every fixture's site and the example suite's, alone and with the README's front page, in it: accessibility, the keyboard, the width of a narrow window, pages opened from disk, the folder tree on a narrow window, the dialog of a keyword's note, markup in feature files shown as text by the folder tree and search, the reload of ghg serve --watch, and the pages a shell draws as it is navigated.
  comment-ratio [path ...]: counts the comment lines and the code lines of the given Rust and TypeScript files, or of every one git tracks, with the total last.";

/// The browser checks, run with Node from the repository's root.
const BROWSER_CHECKS: [&str; 7] = [
    "tests/browser/accessibility.mjs",
    "tests/browser/from_disk.mjs",
    "tests/browser/hostile.mjs",
    "tests/browser/narrow.mjs",
    "tests/browser/navigation.mjs",
    "tests/browser/note_dialog.mjs",
    "tests/browser/reload.mjs",
];

/// The compiled script, committed so that building the crate needs no Node.
const SCRIPT: &str = "assets/gherkin-glass.js";

/// The licences of the packages the compiled script holds, committed beside it.
const LICENSES: &str = "assets/THIRD-PARTY-LICENSES.txt";

fn main() -> ExitCode {
    let command = args().nth(1);
    let result = match command.as_deref() {
        Some("build") => build(),
        Some("check") => check(),
        Some("check-script") => check_script(),
        Some("check-browser") => check_browser(),
        Some("comment-ratio") => comment_ratio(args().skip(2).collect()),
        Some("--help" | "-h") => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn build() -> Result<(), String> {
    prepare_script()?;
    bundle(&root().join(SCRIPT))?;
    licenses(&root().join(LICENSES))?;
    // The expected sites of the fixtures hold the assets as built. Each is built in its
    // fixture's folder, from its configuration file and the list `--only` names, if any, into
    // the folder the file names, emptied first.
    for (fixture, config, output, only) in [
        ("pages", "ghg.toml", "expected", None),
        ("pages", "collapsed.toml", "expected-collapsed", None),
        ("backgrounds", "ghg.toml", "expected", None),
        ("backgrounds", "unchecked.toml", "expected-unchecked", None),
        ("tags", "ghg.toml", "expected", None),
        ("highlighting", "ghg.toml", "expected", None),
        ("front-page", "ghg.toml", "expected", None),
        ("only", "ghg.toml", "expected", Some("changed.txt")),
    ] {
        let folder = root().join("tests/fixtures").join(fixture);
        replace_folder(&folder.join(output), None)?;
        let mut arguments = vec![
            "run",
            "--quiet",
            "--locked",
            "--package",
            "gherkin-glass",
            "--",
            "build",
            "--config",
            config,
        ];
        arguments.extend(only.into_iter().flat_map(only_arguments));
        cargo_in(&folder, &arguments)?;
    }
    // The preview of `only`'s listed files, added beside its site.
    let only = root().join("tests/fixtures/only");
    replace_folder(&only.join("expected-preview"), Some(&only.join("expected")))?;
    cargo_in(
        &only,
        &[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "gherkin-glass",
            "--",
            "preview",
            "add",
            "pr-131",
            "--only",
            "changed.txt",
            "--output",
            "expected-preview",
        ],
    )?;
    // So does the example suite's, which the project's site shows.
    replace_folder(&root().join("project-site/expected"), None)?;
    cargo(&[
        "run",
        "--quiet",
        "--locked",
        "--package",
        "gherkin-glass",
        "--",
        "build",
        "--config",
        "project-site/ghg.toml",
    ])?;
    fixture_stores()?;
    versions_site()?;
    history_store()?;
    let version = manifest_value("version")?;
    put_back_placeholder(&root().join("tests/fixtures"), &version)?;
    put_back_placeholder(&root().join("project-site/expected"), &version)?;
    // The README crates.io and docs.rs show, through bash, which Windows has with Git Bash.
    run(Command::new("bash")
        .args([
            "project-site/crate-readme.sh",
            "README.md",
            "README.crate.md",
        ])
        .current_dir(root()))?;
    cargo(&[
        "build",
        "--release",
        "--locked",
        "--package",
        "gherkin-glass",
    ])
}

fn check() -> Result<(), String> {
    CARGO_CHECKS
        .iter()
        .try_for_each(|arguments| run(cargo_command(arguments).envs(rustdoc_flags(arguments))))?;
    let msrv = minimum_rust()?;
    run(Command::new("rustup").args(["toolchain", "install", &msrv, "--profile", "minimal"]))?;
    let version = format!("+{msrv}");
    cargo(&[[version.as_str()].as_slice(), &MINIMUM_RUST_TEST].concat())?;
    check_script()?;
    browser_checks()
}

/// The doc check's flags, which make a broken link in a doc comment fail, as on CI. The other
/// checks get none, so that a doctest is not held to them.
fn rustdoc_flags(arguments: &[&str]) -> Option<(&'static str, &'static str)> {
    (arguments.first() == Some(&"doc")).then_some(("RUSTDOCFLAGS", "-D warnings"))
}

/// Prints the comment lines and the code lines of each of `paths`, or of every Rust and
/// TypeScript file git tracks outside `node_modules`, then their total.
fn comment_ratio(paths: Vec<String>) -> Result<(), String> {
    let paths = if paths.is_empty() {
        tracked_sources()?
    } else {
        paths
    };
    let counts = paths
        .iter()
        .map(|path| count(path))
        .collect::<Result<Vec<_>, _>>()?;
    println!("{}", Lines::total(&counts).line("TOTAL"));
    Ok(())
}

/// Counts the lines of the source at `path`, from the repository's root, and prints them.
fn count(path: &str) -> Result<Lines, String> {
    let source = read_to_string(root().join(path)).map_err(|error| format!("{path}: {error}"))?;
    let lines = Lines::of(&source);
    println!("{}", lines.line(path));
    Ok(lines)
}

/// The Rust and TypeScript files git tracks, outside `node_modules`.
fn tracked_sources() -> Result<Vec<String>, String> {
    let listed = Command::new("git")
        .arg("ls-files")
        .current_dir(root())
        .output()
        .map_err(|error| format!("git ls-files could not start: {error}"))?;
    if !listed.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&listed.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter(|path| is_source(path))
        .map(str::to_string)
        .collect())
}

/// Whether `path` is Rust or TypeScript of the repository's own, not a Node package's.
fn is_source(path: &str) -> bool {
    let rust_or_typescript = path.ends_with(".rs") || path.ends_with(".ts");
    rust_or_typescript && !path.contains("node_modules")
}

fn check_script() -> Result<(), String> {
    prepare_script()?;
    let compiled = target().join("xtask/gherkin-glass.js");
    bundle(&compiled)?;
    let written = target().join("xtask/THIRD-PARTY-LICENSES.txt");
    licenses(&written)?;
    let read_file =
        |path: &Path| read(path).map_err(|error| format!("{}: {error}", path.display()));
    for (made, committed) in [(compiled, SCRIPT), (written, LICENSES)] {
        if read_file(&made)? != read_file(&root().join(committed))? {
            return Err(format!(
                "{committed} is not what script/ makes. Run cargo xtask build, and commit {committed}."
            ));
        }
    }
    Ok(())
}

fn check_browser() -> Result<(), String> {
    npm(&["ci", "--no-audit", "--no-fund"])?;
    browser_checks()
}

/// The browser check, once the Node packages are installed.
fn browser_checks() -> Result<(), String> {
    // Headless Chromium alone, with the system's own libraries: on GitHub's
    // ubuntu-latest runner, measured on 2026-11-10, --with-deps added only fonts,
    // and failed whenever apt could not reach Ubuntu's archive.
    npm(&[
        "exec",
        "--",
        "playwright",
        "install",
        "--only-shell",
        "chromium",
    ])?;
    cargo(&["build", "--locked", "--package", "gherkin-glass"])?;
    let ghg = target().join(format!("debug/ghg{EXE_SUFFIX}"));
    for check in BROWSER_CHECKS {
        run(Command::new("node")
            .arg(check)
            .env("GHG", &ghg)
            .current_dir(root()))?;
    }
    Ok(())
}

/// Installs the Node packages from the lockfile, then type-checks and tests the script.
fn prepare_script() -> Result<(), String> {
    npm(&["ci", "--no-audit", "--no-fund"])?;
    npm(&["run", "typecheck"])?;
    npm(&["test"])
}

fn bundle(to: &Path) -> Result<(), String> {
    let outfile = format!("--outfile={}", to.display());
    npm(&["run", "--silent", "bundle", "--", &outfile])
}

/// Writes the licences of the packages the compiled script holds into `to`.
fn licenses(to: &Path) -> Result<(), String> {
    let file = to.display().to_string();
    npm(&["run", "--silent", "licenses", "--", &file])
}

fn npm(arguments: &[&str]) -> Result<(), String> {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    run(Command::new(npm)
        .args(arguments)
        .current_dir(root().join("script")))
}

fn cargo(arguments: &[&str]) -> Result<(), String> {
    run(&mut cargo_command(arguments))
}

/// The arguments that give `ghg build` the list `list` with `--only`.
fn only_arguments(list: &str) -> [&str; 2] {
    ["--only", list]
}

/// Cargo with `arguments`, run from `folder`.
fn cargo_in(folder: &Path, arguments: &[&str]) -> Result<(), String> {
    run(cargo_command(arguments).current_dir(folder))
}

/// Cargo with `arguments`, run from the repository's root.
fn cargo_command(arguments: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command.args(arguments).current_dir(root());
    command
}

fn run(command: &mut Command) -> Result<(), String> {
    let shown = format!("{command:?}");
    let status = command
        .status()
        .map_err(|error| format!("{shown} could not start: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{shown} failed: {status}"))
    }
}

/// `rust-version` of the crate's `Cargo.toml`.
fn minimum_rust() -> Result<String, String> {
    manifest_value("rust-version")
}

/// The first value of `key` in the crate's `Cargo.toml`, its package's.
fn manifest_value(key: &str) -> Result<String, String> {
    let manifest = root().join("Cargo.toml");
    let text =
        read_to_string(&manifest).map_err(|error| format!("{}: {error}", manifest.display()))?;
    let start = format!("{key} = \"");
    text.lines()
        .find_map(|line| line.strip_prefix(start.as_str()))
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::to_string)
        .ok_or_else(|| format!("{}: no {key}", manifest.display()))
}

/// Cargo's target folder, `CARGO_TARGET_DIR` when it is set, relative to the repository's root.
fn target() -> PathBuf {
    var_os("CARGO_TARGET_DIR").map_or_else(|| root().join("target"), |folder| root().join(folder))
}

/// The repository's root, the folder above this crate's.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}
