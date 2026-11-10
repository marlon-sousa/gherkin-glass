//! Composition root: `cargo xtask build` and `cargo xtask check`, the commands contributors and CI run.

use std::env::args;
use std::fs::{read, read_to_string};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <command>

Commands:
  build: builds the pages' script from script/ into assets/gherkin-glass.js, then the release binary.
  check: runs every check a pull request runs: formatting, lints, the tests on both Rust versions, and the script check.
  check-script: type-checks and tests the script, and checks that assets/gherkin-glass.js is what it compiles to.";

/// The compiled script, committed so that building the crate needs no Node.
const SCRIPT: &str = "assets/gherkin-glass.js";

fn main() -> ExitCode {
    let command = args().nth(1);
    let result = match command.as_deref() {
        Some("build") => build(),
        Some("check") => check(),
        Some("check-script") => check_script(),
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
    cargo(&[
        "build",
        "--release",
        "--locked",
        "--package",
        "gherkin-glass",
    ])
}

fn check() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo(&["test", "--workspace", "--locked"])?;
    let msrv = minimum_rust()?;
    run(Command::new("rustup").args(["toolchain", "install", &msrv, "--profile", "minimal"]))?;
    cargo(&[
        &format!("+{msrv}"),
        "test",
        "--locked",
        "--package",
        "gherkin-glass",
    ])?;
    check_script()
}

fn check_script() -> Result<(), String> {
    prepare_script()?;
    let compiled = root().join("target/xtask/gherkin-glass.js");
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
    run(Command::new("cargo").args(arguments).current_dir(root()))
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

/// The repository's root, the folder above this crate's.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}
