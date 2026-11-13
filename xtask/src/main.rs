//! Composition root: `cargo xtask build`, `cargo xtask check` and `cargo xtask comment-ratio`, the commands contributors and CI run.

mod checks;
mod comment_ratio;

use std::env::consts::EXE_SUFFIX;
use std::env::{args, var_os};
use std::fs::{read, read_to_string};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use checks::{CARGO_CHECKS, MINIMUM_RUST_TEST};
use comment_ratio::Lines;

const USAGE: &str = "\
Usage: cargo xtask <command>

Commands:
  build: builds the pages' script from script/ into assets/gherkin-glass.js, writes the fixtures' and the example suite's expected sites again, then builds the release binary.
  check: runs every check a pull request runs: formatting, lints, the tests on both Rust versions, the documentation, the dependencies with cargo deny, the package, the script check and the browser check.
  check-script: type-checks and tests the script, and checks that assets/gherkin-glass.js is what it compiles to.
  check-browser: builds ghg, installs the browser, and checks every fixture's site and the example suite's in it: accessibility, the keyboard, a page opened from disk, and the reload of ghg serve --watch.
  comment-ratio [path ...]: counts the comment lines and the code lines of the given Rust and TypeScript files, or of every one git tracks, with the total last.";

/// The browser checks, run with Node from the repository's root.
const BROWSER_CHECKS: [&str; 3] = [
    "tests/browser/accessibility.mjs",
    "tests/browser/from_disk.mjs",
    "tests/browser/reload.mjs",
];

/// The compiled script, committed so that building the crate needs no Node.
const SCRIPT: &str = "assets/gherkin-glass.js";

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
    // The expected sites of the fixtures hold the assets as built.
    for config in [
        "pages/ghg.toml",
        "pages/collapsed.toml",
        "backgrounds/ghg.toml",
        "backgrounds/linked.toml",
        "tags/ghg.toml",
        "highlighting/ghg.toml",
    ] {
        let config = format!("tests/fixtures/{config}");
        cargo(&[
            "run",
            "--quiet",
            "--locked",
            "--package",
            "gherkin-glass",
            "--",
            "build",
            "--config",
            &config,
        ])?;
    }
    // So does the example suite's, which the project's site shows.
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
    let read_file =
        |path: &Path| read(path).map_err(|error| format!("{}: {error}", path.display()));
    if read_file(&compiled)? != read_file(&root().join(SCRIPT))? {
        return Err(format!(
            "{SCRIPT} is not what script/ compiles to. Run cargo xtask build, and commit {SCRIPT}."
        ));
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

fn npm(arguments: &[&str]) -> Result<(), String> {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    run(Command::new(npm)
        .args(arguments)
        .current_dir(root().join("script")))
}

fn cargo(arguments: &[&str]) -> Result<(), String> {
    run(&mut cargo_command(arguments))
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
    let manifest = root().join("Cargo.toml");
    let text =
        read_to_string(&manifest).map_err(|error| format!("{}: {error}", manifest.display()))?;
    text.lines()
        .find_map(|line| line.strip_prefix("rust-version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::to_string)
        .ok_or_else(|| format!("{}: no rust-version", manifest.display()))
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
