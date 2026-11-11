//! Installing through cargo, checked by running `cargo ghg` through cargo itself, and
//! `cargo-ghg` by itself, beside `ghg`.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::env::{join_paths, split_paths, var_os};
use std::ffi::OsStr;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/configuration")
        .join(name)
}

/// `command`, run in `folder`. A folder of the fixtures is run in a copy of
/// them of its own, so that the site a build writes stays out of the
/// repository, and each run starts from the fixtures as they are.
fn run(command: &mut Command, folder: &Path) -> Output {
    command
        .current_dir(in_a_copy(folder))
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
        .join("cargo-plugin")
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

fn ghg(folder: &Path, arguments: &[&str]) -> Output {
    run(
        Command::new(env!("CARGO_BIN_EXE_ghg")).args(arguments),
        folder,
    )
}

/// The cargo running these tests, run in `folder` with `arguments`, so that
/// it finds the `cargo-ghg` just built first on the `PATH`. Cargo looks in its
/// home's `bin` folder too, so the home is an empty folder, where no
/// `cargo-ghg` installed before can be found instead.
fn cargo(folder: &Path, arguments: &[&OsStr]) -> Output {
    let plugin = Path::new(env!("CARGO_BIN_EXE_cargo-ghg"));
    let mut path = vec![plugin.parent().expect("a folder").to_path_buf()];
    path.extend(split_paths(&var_os("PATH").unwrap_or_default()));
    let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cargo-home");
    create_dir_all(&home).expect("the home is made");
    run(
        Command::new(env!("CARGO"))
            .args(arguments)
            .env("PATH", join_paths(path).expect("a PATH"))
            .env("CARGO_HOME", home),
        folder,
    )
}

/// `cargo ghg` with `arguments`, run in `folder`.
fn cargo_ghg(folder: &Path, arguments: &[&str]) -> Output {
    let arguments: Vec<&OsStr> = once("ghg")
        .chain(arguments.iter().copied())
        .map(OsStr::new)
        .collect();
    cargo(folder, &arguments)
}

fn cargo_ghg_by_itself(folder: &Path, arguments: &[&str]) -> Output {
    run(
        Command::new(env!("CARGO_BIN_EXE_cargo-ghg")).args(arguments),
        folder,
    )
}

fn same(left: &Output, right: &Output, what: &str) {
    assert_eq!(left.status.code(), right.status.code(), "status of {what}");
    assert_eq!(left.stdout, right.stdout, "standard output of {what}");
    assert_eq!(left.stderr, right.stderr, "standard error of {what}");
}

/// Arguments that try every outcome: success, help, version, each kind of
/// mistake, and a configuration or input that stops the build.
const CASES: &[(&str, &[&str])] = &[
    ("default", &[]),
    ("default", &["build"]),
    ("default", &["--version"]),
    ("default", &["-V"]),
    ("default", &["--help"]),
    ("default", &["build", "--help"]),
    ("default", &["buil"]),
    ("default", &["build", "--input"]),
    ("default", &["build", "--input", "missing"]),
    ("default", &["build", "--config", "missing.toml"]),
    ("configured", &["build"]),
    ("configured", &["build", "--input", "features"]),
    ("configured", &["build", "--config", "ci/ghg.toml"]),
    ("mistaken", &["build"]),
];

// Rule 37.

#[test]
fn cargo_ghg_behaves_exactly_as_ghg() {
    for (folder, arguments) in CASES {
        let folder = fixture(folder);
        same(
            &cargo_ghg(&folder, arguments),
            &ghg(&folder, arguments),
            &format!("{arguments:?} in {}", folder.display()),
        );
    }
}

#[test]
fn cargo_ghg_prints_the_version_of_ghg() {
    let output = cargo_ghg(&fixture("default"), &["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8"),
        format!("ghg {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn cargo_help_ghg_prints_the_help() {
    let folder = fixture("default");
    same(
        &cargo(&folder, &["help".as_ref(), "ghg".as_ref()]),
        &ghg(&folder, &["--help"]),
        "cargo help ghg",
    );
}

// Rule 38.

#[test]
fn cargo_ghg_by_itself_behaves_as_ghg_with_or_without_its_name_first() {
    for (folder, arguments) in CASES {
        let folder = fixture(folder);
        let expected = ghg(&folder, arguments);
        let what = format!("{arguments:?} in {}", folder.display());
        same(&cargo_ghg_by_itself(&folder, arguments), &expected, &what);
        let named: Vec<&str> = once("ghg").chain(arguments.iter().copied()).collect();
        same(&cargo_ghg_by_itself(&folder, &named), &expected, &what);
    }
}

#[test]
fn only_one_ghg_is_dropped() {
    let folder = fixture("default");
    same(
        &cargo_ghg_by_itself(&folder, &["ghg", "ghg"]),
        &ghg(&folder, &["ghg"]),
        "ghg ghg",
    );
    same(
        &cargo_ghg(&folder, &["ghg"]),
        &ghg(&folder, &["ghg"]),
        "cargo ghg ghg",
    );
}

// Rule 36.

/// The files of the repository that are not built into the binaries, so
/// that the published crate leaves them out. `script/` is the source of
/// `assets/gherkin-glass.js`, which is committed, `xtask/` a crate of its
/// own, and `action.yml` with `action/` the GitHub Action. A file git tracks
/// anywhere else must be published: a new folder fails the test until it is
/// placed here or in Cargo.toml's `include`.
const NOT_PUBLISHED: &[&str] = &[
    ".cargo/",
    ".claude/",
    ".github/",
    "action/",
    "docs/",
    "project-site/",
    "script/",
    "tests/",
    "xtask/",
    ".gitattributes",
    ".gitignore",
    "action.yml",
    "AGENTS.md",
    "CHANGELOG.md",
    "CLAUDE.md",
    "CONTRIBUTING.md",
    "cliff.toml",
    "clippy.toml",
    "deny.toml",
    "rust-toolchain.toml",
];

#[test]
fn the_published_crate_holds_what_both_binaries_are_built_from() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lines = |output: Output| -> Vec<String> {
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let text = String::from_utf8(output.stdout).expect("UTF-8");
        text.lines().map(str::to_owned).collect()
    };
    let published = lines(run(
        Command::new(env!("CARGO")).args(["package", "--list", "--offline", "--allow-dirty"]),
        root,
    ));
    let tracked = run(Command::new("git").args(["ls-files", "-z"]), root);
    assert_eq!(tracked.status.code(), Some(0), "{tracked:?}");
    let tracked: Vec<String> = String::from_utf8(tracked.stdout)
        .expect("UTF-8")
        .split_terminator('\0')
        .map(str::to_owned)
        .collect();
    for file in [
        "src/main.rs",
        "src/cargo_ghg.rs",
        "messages/en/gherkin-glass.ftl",
    ] {
        assert!(
            tracked.iter().any(|tracked| tracked == file),
            "{file} is tracked"
        );
    }
    for file in &tracked {
        let left_out = NOT_PUBLISHED.iter().any(|left_out| {
            file == left_out || (left_out.ends_with('/') && file.starts_with(left_out))
        });
        assert_eq!(
            published.contains(file),
            !left_out,
            "{file} is published only if it is not left out on purpose"
        );
    }
}

// Rule 37, with an argument that is not UTF-8, which cargo passes on as it is.

#[cfg(unix)]
#[test]
fn an_argument_that_is_not_utf_8_reaches_ghg_as_it_is() {
    use std::os::unix::ffi::OsStrExt;

    let folder = fixture("default");
    let argument = OsStr::from_bytes(b"\xff");
    let through_cargo = cargo(
        &folder,
        &[
            "ghg".as_ref(),
            "build".as_ref(),
            "--input".as_ref(),
            argument,
        ],
    );
    let direct = run(
        Command::new(env!("CARGO_BIN_EXE_ghg"))
            .args(["build", "--input"])
            .arg(argument),
        &folder,
    );
    assert_eq!(direct.status.code(), Some(1), "{direct:?}");
    same(&through_cargo, &direct, "a path that is not UTF-8");
}
