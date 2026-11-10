//! docs/BEHAVIOUR.md, "The GitHub Action", checked by running the action's scripts
//! against releases served from memory.
//!
//! The scripts run in bash, with curl, tar, unzip and sha256sum or shasum,
//! and the tests make the Windows archive with zip, which every Linux and
//! macOS machine has or can install; the tests run there only. On macOS they
//! run the system's own bash, /bin/bash, the oldest the action meets.
#![cfg(unix)]

use std::collections::HashMap;
use std::fs::{self, create_dir_all, remove_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

const REPOSITORY: &str = "marlon-sousa/gherkin-glass";
const LINUX: &str = "x86_64-unknown-linux-musl";

fn bash() -> Command {
    Command::new(if cfg!(target_os = "macos") {
        "/bin/bash"
    } else {
        "bash"
    })
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// A folder of its own for each test, emptied first.
fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "action-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = remove_dir_all(&folder);
    create_dir_all(&folder).expect("the scratch folder is made");
    folder
}

fn run(command: &mut Command) -> Output {
    command.output().expect("the command runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// GitHub, as far as the action asks it anything: its API's list of releases
/// and the release downloads, served from memory over HTTP.
struct FakeGitHub {
    address: String,
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    requests: Arc<Mutex<Vec<String>>>,
}

impl FakeGitHub {
    fn start() -> FakeGitHub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a port is free");
        let address = format!("http://{}", listener.local_addr().expect("an address"));
        let files: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::default();
        let requests: Arc<Mutex<Vec<String>>> = Arc::default();
        let (served, asked) = (files.clone(), requests.clone());
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut request = String::new();
                let mut reader = BufReader::new(&stream);
                if reader.read_line(&mut request).is_err() {
                    continue;
                }
                loop {
                    let mut header = String::new();
                    match reader.read_line(&mut header) {
                        Ok(0) | Err(_) => break,
                        Ok(_) if header == "\r\n" => break,
                        Ok(_) => {}
                    }
                }
                let target = request.split(' ').nth(1).unwrap_or("").to_string();
                asked.lock().unwrap().push(target.clone());
                let body = served.lock().unwrap().get(&target).cloned();
                // A page of releases past the last is empty, as GitHub's is.
                let body = body.or_else(|| target.contains("/releases?").then(|| b"[]".to_vec()));
                let head = match &body {
                    Some(body) => format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    ),
                    None => {
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_string()
                    }
                };
                let _ = stream.write_all(head.as_bytes());
                if let Some(body) = body {
                    let _ = stream.write_all(&body);
                }
            }
        });
        FakeGitHub {
            address,
            files,
            requests,
        }
    }

    fn serve(&self, target: String, body: Vec<u8>) {
        self.files.lock().unwrap().insert(target, body);
    }

    /// Lists `tags` as releases, on pages of `per_page`, as GitHub's API
    /// does: a tag ending in ` draft` is a draft release of the tag before it.
    fn list_releases(&self, tags: &[&str], per_page: usize) {
        for (index, page) in tags.chunks(per_page).enumerate() {
            let listing: Vec<String> = page
                .iter()
                .map(|tag| {
                    let (tag, draft) = match tag.strip_suffix(" draft") {
                        Some(tag) => (tag, true),
                        None => (*tag, false),
                    };
                    format!(
                        "{{\"tag_name\":\"{tag}\",\"name\":\"{tag}\",\"draft\":{draft},\
                         \"body\":\"Not \\\"tag_name\\\": \\\"v1.99.0\\\"\",\
                         \"assets\":[{{\"name\":\"SHA256SUMS\"}}]}}"
                    )
                })
                .collect();
            self.serve(
                format!(
                    "/repos/{REPOSITORY}/releases?per_page=100&page={}",
                    index + 1
                ),
                format!("[{}]", listing.join(",")).into_bytes(),
            );
        }
    }

    /// Publishes an archive of a `ghg` that prints `ghg <version>`, with
    /// SHA256SUMS. `checksums` replaces the right SHA256SUMS when given.
    fn release(&self, version: &str, target: &str, checksums: Option<&str>) {
        let work = scratch("release");
        let windows = target.contains("windows");
        let binary = if windows { "ghg.exe" } else { "ghg" };
        let program = work.join(binary);
        fs::write(&program, format!("#!/bin/sh\necho ghg {version}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(work.join("LICENSE-MIT"), "MIT").unwrap();
        let archive = format!(
            "ghg-v{version}-{target}.{}",
            if windows { "zip" } else { "tar.gz" }
        );
        let packed = if windows {
            run(Command::new("zip")
                .arg("-q")
                .arg(&archive)
                .args([binary, "LICENSE-MIT"])
                .current_dir(&work))
        } else {
            run(Command::new("tar")
                .args(["-czf", &archive, binary, "LICENSE-MIT"])
                .current_dir(&work))
        };
        assert!(packed.status.success(), "{}", text(&packed.stderr));
        let bytes = fs::read(work.join(&archive)).unwrap();
        let digest = run(Command::new("sh")
            .arg("-c")
            .arg(format!(
                "(sha256sum {archive} 2>/dev/null || shasum -a 256 {archive}) | cut -d ' ' -f 1"
            ))
            .current_dir(&work));
        let digest = text(&digest.stdout).trim().to_string();
        let sums = checksums.map(str::to_string).unwrap_or_else(|| {
            format!(
                "{}  ghg-v{version}-other.tar.gz\n{digest}  {archive}\n",
                "0".repeat(64)
            )
        });
        let download = format!("/{REPOSITORY}/releases/download/v{version}");
        self.serve(format!("{download}/{archive}"), bytes);
        self.serve(format!("{download}/SHA256SUMS"), sums.into_bytes());
    }

    fn downloads(&self) -> Vec<String> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|target| target.contains("/releases/download/"))
            .cloned()
            .collect()
    }
}

/// A runner for the action: its folders and files, and the action's own
/// folder, whose Cargo.toml says the action's version.
struct Runner {
    folder: PathBuf,
    os: &'static str,
    arch: &'static str,
}

impl Runner {
    fn new(action_version: &str) -> Runner {
        let folder = scratch("runner");
        for name in ["action", "cache", "temp"] {
            create_dir_all(folder.join(name)).unwrap();
        }
        fs::write(
            folder.join("action/Cargo.toml"),
            format!(
                "[package]\nname = \"gherkin-glass\"\nversion = \"{action_version}\"\n\n\
                 [dependencies]\ngherkin = {{ version = \"0.16.0\" }}\n"
            ),
        )
        .unwrap();
        fs::write(folder.join("path"), "").unwrap();
        fs::write(folder.join("output"), "").unwrap();
        Runner {
            folder,
            os: "Linux",
            arch: "X64",
        }
    }

    fn on(mut self, os: &'static str, arch: &'static str) -> Runner {
        self.os = os;
        self.arch = arch;
        self
    }

    /// Runs install.sh with the inputs `version` and `build`.
    fn install(&self, github: &FakeGitHub, version: &str, build: &str) -> Output {
        run(bash()
            .arg(root().join("action/install.sh"))
            .args([&github.address, &github.address])
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("GHG_VERSION", version)
            .env("GHG_BUILD", build)
            .env("GHG_TOKEN", "a-token")
            .env("GHG_ACTION_PATH", self.folder.join("action"))
            .env("RUNNER_OS", self.os)
            .env("RUNNER_ARCH", self.arch)
            .env("RUNNER_TOOL_CACHE", self.folder.join("cache"))
            .env("RUNNER_TEMP", self.folder.join("temp"))
            .env("GITHUB_PATH", self.folder.join("path"))
            .env("GITHUB_OUTPUT", self.folder.join("output")))
    }

    /// The ghg the last install named in its output, and what it prints.
    fn installed(&self) -> (PathBuf, String) {
        let output = fs::read_to_string(self.folder.join("output")).unwrap();
        let bin = output
            .lines()
            .filter_map(|line| line.strip_prefix("bin="))
            .next_back()
            .expect("the install names ghg");
        let printed = run(&mut Command::new(bin));
        (PathBuf::from(bin), text(&printed.stdout).trim().to_string())
    }

    fn path_entries(&self) -> Vec<String> {
        fs::read_to_string(self.folder.join("path"))
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

/// Checks the install succeeded, and returns what the installed ghg prints.
fn installs(runner: &Runner, output: Output) -> String {
    assert!(
        output.status.success(),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    let (bin, printed) = runner.installed();
    assert_eq!(
        runner.path_entries().last().map(PathBuf::from),
        bin.parent().map(Path::to_path_buf)
    );
    printed
}

/// Checks the install failed with `message`, and installed nothing.
fn fails(runner: &Runner, output: Output, message: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), format!("::error::{message}\n"));
    assert!(runner.path_entries().is_empty());
    assert_eq!(
        fs::read_to_string(runner.folder.join("output")).unwrap(),
        ""
    );
}

// Installing: rules 129 to 133.

#[test]
fn installs_the_newest_release_of_the_actions_major_version_by_default() {
    let github = FakeGitHub::start();
    github.list_releases(
        &[
            "v10.0.0",
            "v2.0.0",
            "v1.12.0 draft",
            "v1.11.0-rc.1",
            "v1.9.3",
            "v1",
            "v1.10.0",
            "v1.2.0",
            "v0.9.0",
        ],
        3,
    );
    for version in [
        "10.0.0",
        "1.12.0",
        "1.10.0",
        "1.9.3",
        "2.0.0",
        "1.11.0-rc.1",
    ] {
        github.release(version, LINUX, None);
    }
    let runner = Runner::new("1.2.0");
    assert_eq!(
        installs(&runner, runner.install(&github, "", "false")),
        "ghg 1.10.0"
    );
}

#[test]
fn before_1_0_the_newest_release_of_the_actions_minor_version() {
    let github = FakeGitHub::start();
    github.list_releases(&["v0.10.0", "v0.2.0", "v0.1.4", "v0.1.0", "v1.0.0"], 100);
    github.release("0.1.4", LINUX, None);
    let runner = Runner::new("0.1.0");
    assert_eq!(
        installs(&runner, runner.install(&github, "", "false")),
        "ghg 0.1.4"
    );
}

#[test]
fn version_installs_the_release_it_names() {
    let github = FakeGitHub::start();
    github.list_releases(&["v1.3.0", "v1.2.0"], 100);
    github.release("1.2.0", LINUX, None);
    github.release("2.0.0-rc.1", LINUX, None);
    for version in ["1.2.0", "v1.2.0"] {
        let runner = Runner::new("1.3.0");
        assert_eq!(
            installs(&runner, runner.install(&github, version, "false")),
            "ghg 1.2.0"
        );
    }
    let runner = Runner::new("1.3.0");
    assert_eq!(
        installs(&runner, runner.install(&github, "2.0.0-rc.1", "false")),
        "ghg 2.0.0-rc.1"
    );
}

#[test]
fn a_version_that_names_no_release_fails() {
    let github = FakeGitHub::start();
    github.list_releases(&["v2.0.0", "v1.0.0-rc.1"], 100);
    let runner = Runner::new("1.0.0");
    fails(
        &runner,
        runner.install(&github, "", "false"),
        "ghg has no release compatible with this action's version.",
    );
    let runner = Runner::new("1.0.0");
    fails(
        &runner,
        runner.install(&github, "latest", "false"),
        "version must be a version of ghg, such as 1.2.0.",
    );
    for version in ["01.2.0", "1.2", "1.2.0+build", "1.2.0-..", "1.2.0 "] {
        let runner = Runner::new("1.0.0");
        fails(
            &runner,
            runner.install(&github, version, "false"),
            "version must be a version of ghg, such as 1.2.0.",
        );
    }
    let runner = Runner::new("1.0.0");
    fails(
        &runner,
        runner.install(&github, "1.4.0", "false"),
        "ghg-v1.4.0-x86_64-unknown-linux-musl.tar.gz cannot be downloaded from the release v1.4.0 of ghg.",
    );
}

#[test]
fn the_archive_is_the_runners_system_and_processor() {
    let github = FakeGitHub::start();
    let platforms = [
        ("Linux", "X64", LINUX),
        ("Linux", "ARM64", "aarch64-unknown-linux-musl"),
        ("macOS", "X64", "x86_64-apple-darwin"),
        ("macOS", "ARM64", "aarch64-apple-darwin"),
        ("Windows", "X64", "x86_64-pc-windows-msvc"),
    ];
    for (os, arch, target) in platforms {
        github.release("1.0.0", target, None);
        let runner = Runner::new("1.0.0").on(os, arch);
        assert_eq!(
            installs(&runner, runner.install(&github, "1.0.0", "false")),
            "ghg 1.0.0"
        );
        let (bin, _) = runner.installed();
        let name = if os == "Windows" { "ghg.exe" } else { "ghg" };
        assert_eq!(bin.file_name().unwrap(), name);
        assert!(
            github
                .downloads()
                .iter()
                .any(|download| download.contains(target)),
            "{target}"
        );
    }
    for (os, arch) in [("Linux", "ARM"), ("Windows", "ARM64"), ("Linux", "X86")] {
        let runner = Runner::new("1.0.0").on(os, arch);
        fails(
            &runner,
            runner.install(&github, "1.0.0", "false"),
            &format!("ghg has no build for {os} on {arch}."),
        );
    }
}

#[test]
fn an_archive_that_does_not_match_its_checksum_is_not_installed() {
    let github = FakeGitHub::start();
    github.release(
        "1.0.0",
        LINUX,
        Some(&format!("{}  ghg-v1.0.0-{LINUX}.tar.gz\n", "0".repeat(64))),
    );
    let runner = Runner::new("1.0.0");
    let archive = format!("ghg-v1.0.0-{LINUX}.tar.gz");
    fails(
        &runner,
        runner.install(&github, "1.0.0", "false"),
        &format!("{archive} does not match its checksum in SHA256SUMS."),
    );
    assert!(!runner.folder.join("cache/ghg/1.0.0").join(LINUX).exists());

    github.release("1.0.0", LINUX, Some("\n"));
    let runner = Runner::new("1.0.0");
    fails(
        &runner,
        runner.install(&github, "1.0.0", "false"),
        &format!("{archive} is not in the release's SHA256SUMS."),
    );
}

#[test]
fn an_installed_version_is_kept_in_the_tool_cache_and_not_downloaded_again() {
    let github = FakeGitHub::start();
    github.release("1.0.0", LINUX, None);
    let runner = Runner::new("1.0.0");
    installs(&runner, runner.install(&github, "1.0.0", "false"));
    assert_eq!(github.downloads().len(), 2);
    assert_eq!(
        installs(&runner, runner.install(&github, "1.0.0", "false")),
        "ghg 1.0.0"
    );
    assert_eq!(github.downloads().len(), 2);
    assert_eq!(runner.path_entries().len(), 2);
}

// Building: rules 134 and 135.

#[test]
fn build_is_true_or_false() {
    let github = FakeGitHub::start();
    for build in ["yes", "TRUE"] {
        let runner = Runner::new("1.0.0");
        fails(
            &runner,
            runner.install(&github, "1.0.0", build),
            "build must be true or false.",
        );
    }
    assert!(github.requests.lock().unwrap().is_empty());
}

/// A copy of `from` at `to`, folders and files.
fn copy(from: &Path, to: &Path) {
    create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Runs build.sh with the real ghg in a copy of the action's fixture, with
/// `options` as the environment action.yml gives it, and returns the copy.
fn build(options: &[(&str, &str)]) -> (Output, PathBuf) {
    let workspace = scratch("workspace");
    copy(&root().join("tests/fixtures/action"), &workspace);
    let mut command = bash();
    command
        .arg(root().join("action/build.sh"))
        .current_dir(&workspace)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GHG_BIN", env!("CARGO_BIN_EXE_ghg"))
        .env("GHG_OPTIONS", "config input output title version-name");
    for (name, value) in options {
        command.env(name, value);
    }
    (run(&mut command), workspace)
}

fn built(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "");
}

#[test]
fn build_runs_ghg_build_with_the_configuration_when_no_option_is_given() {
    for options in [
        &[][..],
        &[
            ("GHG_OPTION_CONFIG", ""),
            ("GHG_OPTION_INPUT", ""),
            ("GHG_OPTION_OUTPUT", ""),
            ("GHG_OPTION_TITLE", ""),
            ("GHG_OPTION_VERSION_NAME", ""),
        ],
        // Only the options GHG_OPTIONS names are passed.
        &[("GHG_OPTION_PORT", "9")],
    ] {
        let (output, workspace) = build(options);
        built(&output);
        assert!(workspace.join("site/read.html").is_file());
    }
}

#[test]
fn each_option_input_is_passed_to_ghg_build() {
    let (output, _) = build(&[("GHG_OPTION_INPUT", "features"), ("GHG_OPTION_CONFIG", "")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).starts_with("broken.feature:1: "),
        "{}",
        text(&output.stderr)
    );

    let (output, _) = build(&[("GHG_OPTION_CONFIG", "missing.toml")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).starts_with("missing.toml: cannot be read: "),
        "{}",
        text(&output.stderr)
    );

    let (output, workspace) = build(&[
        ("GHG_OPTION_OUTPUT", "public"),
        ("GHG_OPTION_TITLE", "The action's suite"),
    ]);
    built(&output);
    assert!(!workspace.join("site").exists());
    let home = fs::read_to_string(workspace.join("public/index.html")).unwrap();
    assert!(
        home.contains("<title>The action&#39;s suite</title>"),
        "{home}"
    );

    let (output, workspace) = build(&[("GHG_OPTION_VERSION_NAME", "v1.0.0")]);
    built(&output);
    assert!(workspace.join("site/v1.0.0/read.html").is_file());
}
