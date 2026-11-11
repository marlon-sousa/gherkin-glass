//! Previewing a site, checked by running `ghg serve` and asking it over HTTP.

use std::fs::{copy, create_dir_all, read, read_to_string, remove_dir_all, rename, write};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/serve")
        .join(name)
}

/// `ghg serve` with `arguments` in `folder`, stopped when dropped.
struct Serving {
    child: Child,
    stdout: BufReader<ChildStdout>,
    /// Such as `127.0.0.1:41234`.
    address: String,
}

impl Drop for Serving {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Starts `ghg serve` and waits for the line that says where it listens.
fn serve(folder: &Path, arguments: &[&str]) -> Serving {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("serve")
        .args(arguments)
        .current_dir(folder)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ghg serve starts");
    let mut stdout = BufReader::new(child.stdout.take().expect("piped"));
    let mut line = String::new();
    stdout.read_line(&mut line).expect("a line");
    let address = line
        .strip_prefix("The site is served at http://")
        .and_then(|rest| rest.strip_suffix("/. Press Control+C to stop.\n"))
        .unwrap_or_else(|| panic!("the address, in {line:?}"))
        .to_string();
    Serving {
        child,
        stdout,
        address,
    }
}

/// Runs `ghg serve` to its end, for a run that stops before it serves.
fn serve_to_the_end(folder: &Path, arguments: &[&str]) -> (String, String, Option<i32>) {
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("serve")
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("ghg serve runs");
    let text = |bytes| String::from_utf8(bytes).expect("UTF-8");
    (
        text(output.stdout),
        text(output.stderr),
        output.status.code(),
    )
}

struct Answer {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Answer {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn text(&self) -> String {
        String::from_utf8(self.body.clone()).expect("UTF-8")
    }
}

/// One HTTP/1.1 request, with `target` sent exactly as given.
fn request(address: &str, method: &str, target: &str) -> Answer {
    let mut stream = TcpStream::connect(address).expect("connects");
    write!(
        stream,
        "{method} {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .expect("sent");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read");
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("headers end");
    let head = String::from_utf8(bytes[..split].to_vec()).expect("ASCII headers");
    let mut lines = head.split("\r\n");
    let status = lines.next().expect("status line")[9..12]
        .parse()
        .expect("a status");
    let headers = lines
        .map(|line| {
            let (name, value) = line.split_once(": ").expect("a header");
            (name.to_string(), value.to_string())
        })
        .collect();
    Answer {
        status,
        headers,
        body: bytes[split + 4..].to_vec(),
    }
}

fn get(address: &str, target: &str) -> Answer {
    request(address, "GET", target)
}

fn scratch(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    create_dir_all(&folder).expect("scratch folder");
    folder
}

/// The `built` fixture, copied where a test may change it.
fn built_copy(name: &str) -> PathBuf {
    let folder = scratch(name);
    create_dir_all(folder.join("features")).expect("features");
    copy(fixture("built/ghg.toml"), folder.join("ghg.toml")).expect("copied");
    copy(
        fixture("built/features/withdraw.feature"),
        folder.join("features/withdraw.feature"),
    )
    .expect("copied");
    folder
}

/// Waits until `check` holds, for at most ten seconds.
fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !check() {
        assert!(Instant::now() < deadline, "{what}");
        sleep(Duration::from_millis(50));
    }
}

/// `--no-build` serves the `site` fixture, the configured output folder.
fn serve_site() -> Serving {
    serve(
        &fixture(""),
        &["--no-build", "--output", "site", "--port", "0"],
    )
}

// Serving: rules 56 to 60.

#[test]
fn it_says_where_it_listens_on_127_0_0_1() {
    let serving = serve_site();
    assert!(
        serving.address.starts_with("127.0.0.1:"),
        "{}",
        serving.address
    );
    assert_eq!(get(&serving.address, "/").status, 200);
}

#[test]
fn a_port_in_use_stops_it_and_names_the_port() {
    let taken = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = taken.local_addr().expect("an address").port().to_string();
    let (stdout, stderr, status) = serve_to_the_end(
        &fixture(""),
        &["--no-build", "--output", "site", "--port", &port],
    );
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        format!("Port {port} is already in use. Choose another with --port.\n")
    );
    assert_eq!(status, Some(1));
}

#[test]
fn a_port_another_server_listens_at_on_every_address_is_in_use() {
    let taken = TcpListener::bind("0.0.0.0:0").expect("a port");
    let port = taken.local_addr().expect("an address").port().to_string();
    let (stdout, stderr, status) = serve_to_the_end(
        &fixture(""),
        &["--no-build", "--output", "site", "--port", &port],
    );
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        format!("Port {port} is already in use. Choose another with --port.\n")
    );
    assert_eq!(status, Some(1));
}

#[test]
fn listening_on_every_network_prints_this_machine_s_address() {
    let serving = serve(
        &fixture(""),
        &[
            "--no-build",
            "--output",
            "site",
            "--host",
            "0.0.0.0",
            "--port",
            "0",
        ],
    );
    assert!(
        serving.address.starts_with("127.0.0.1:"),
        "{}",
        serving.address
    );
    assert_eq!(get(&serving.address, "/").status, 200);
}

#[test]
fn an_address_it_cannot_listen_on_is_named() {
    // 203.0.113.1 is kept for documentation, so no machine has it.
    let (stdout, stderr, status) = serve_to_the_end(
        &fixture(""),
        &["--no-build", "--host", "203.0.113.1", "--port", "0"],
    );
    assert_eq!(stdout, "");
    assert!(
        stderr.starts_with("ghg serve cannot listen at 203.0.113.1, port 0: "),
        "{stderr}"
    );
    assert_eq!(status, Some(1));
}

#[test]
fn a_port_must_be_a_number_up_to_65535() {
    for port in ["65536", "-1", "eighty", "+80"] {
        let (stdout, stderr, status) = serve_to_the_end(&fixture(""), &["--port", port]);
        assert_eq!(stdout, "");
        assert_eq!(
            stderr,
            format!("The option --port needs a whole number from 0 to 65535, not {port}.\n")
        );
        assert_eq!(status, Some(2));
    }
}

#[test]
fn a_build_that_stops_serves_nothing() {
    let folder = built_copy("serve-build-stops");
    write(folder.join("features/broken.feature"), "").expect("written");
    for arguments in [&["--port", "0"][..], &["--port", "0", "--watch"]] {
        let (stdout, stderr, status) = serve_to_the_end(&folder, arguments);
        assert_eq!(stdout, "");
        assert!(stderr.starts_with("broken.feature:"), "{stderr}");
        assert_eq!(status, Some(1));
    }
}

#[test]
fn it_builds_into_the_output_folder_and_serves_it() {
    let folder = built_copy("serve-build");
    let serving = serve(&folder, &["--port", "0"]);
    let page = get(&serving.address, "/withdraw.html");
    assert_eq!(page.status, 200);
    assert_eq!(
        page.header("Content-Type"),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        page.body,
        read(folder.join("site/withdraw.html")).expect("written")
    );
    assert!(!page.text().contains("__ghg"));
    let script = get(&serving.address, "/assets/gherkin-glass.js");
    assert_eq!(
        script.header("Content-Type"),
        Some("text/javascript; charset=utf-8")
    );
    assert_eq!(get(&serving.address, "/__ghg/build").status, 404);
}

#[test]
fn the_output_folder_is_checked_as_ghg_build_checks_it_unless_the_site_is_in_memory() {
    let folder = built_copy("serve-not-a-site");
    create_dir_all(folder.join("site")).expect("made");
    write(folder.join("site/notes.txt"), "mine").expect("written");
    let (stdout, stderr, status) = serve_to_the_end(&folder, &["--port", "0"]);
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        "site: this is not a site ghg built, so it is left as it is. \
         Build into an empty or new folder.\n"
    );
    assert_eq!(status, Some(1));
    let serving = serve(&folder, &["--port", "0", "--watch"]);
    assert_eq!(get(&serving.address, "/withdraw.html").status, 200);
    drop(serving);
    assert_eq!(
        read_to_string(folder.join("site/notes.txt")).expect("kept"),
        "mine"
    );
}

#[test]
fn a_version_name_changes_nothing_under_watch() {
    let folder = built_copy("serve-watch-versioned");
    let serving = serve(
        &folder,
        &["--port", "0", "--watch", "--version-name", "main"],
    );
    assert_eq!(get(&serving.address, "/withdraw.html").status, 200);
    assert_eq!(get(&serving.address, "/main/withdraw.html").status, 404);
    drop(serving);
    assert!(!folder.join("site").exists());
}

#[test]
fn a_versioned_build_serves_the_output_folder_with_its_versions_page() {
    let folder = built_copy("serve-versioned");
    let serving = serve(&folder, &["--port", "0", "--version-name", "main"]);
    let page = get(&serving.address, "/");
    assert_eq!(page.status, 200);
    assert!(
        page.text().contains("href=\"main/index.html\""),
        "{}",
        page.text()
    );
    assert_eq!(get(&serving.address, "/main/withdraw.html").status, 200);
    assert!(folder.join("site/main/ghg-version.json").is_file());
}

// What it answers: rules 61 to 64.

#[test]
fn a_file_is_served_exactly_with_its_type() {
    let serving = serve_site();
    for (target, file, content_type) in [
        ("/index.html", "index.html", "text/html; charset=utf-8"),
        ("/", "index.html", "text/html; charset=utf-8"),
        (
            "/banking/",
            "banking/index.html",
            "text/html; charset=utf-8",
        ),
        (
            "/banking/withdraw.html?x=1",
            "banking/withdraw.html",
            "text/html; charset=utf-8",
        ),
        (
            "/assets/gherkin-glass.js",
            "assets/gherkin-glass.js",
            "text/javascript; charset=utf-8",
        ),
        (
            "/assets/gherkin-glass.css",
            "assets/gherkin-glass.css",
            "text/css; charset=utf-8",
        ),
        ("/assets/tree.json", "assets/tree.json", "application/json"),
        ("/README", "README", "application/octet-stream"),
        (
            "/p%C3%A1gina%20um.html",
            "página um.html",
            "text/html; charset=utf-8",
        ),
    ] {
        let answer = get(&serving.address, target);
        assert_eq!(answer.status, 200, "{target}");
        assert_eq!(
            answer.header("Content-Type"),
            Some(content_type),
            "{target}"
        );
        assert_eq!(answer.header("Cache-Control"), Some("no-cache"), "{target}");
        assert_eq!(
            answer.body,
            read(fixture("site").join(file)).expect("read"),
            "{target}"
        );
    }
}

#[test]
fn head_answers_without_a_body() {
    let serving = serve_site();
    let answer = request(&serving.address, "HEAD", "/index.html");
    assert_eq!(answer.status, 200);
    assert_eq!(
        answer.header("Content-Type"),
        Some("text/html; charset=utf-8")
    );
    assert!(answer.body.is_empty());
}

#[test]
fn a_folder_without_its_slash_is_redirected() {
    let serving = serve_site();
    let answer = get(&serving.address, "/banking?x=1");
    assert_eq!(answer.status, 301);
    assert_eq!(answer.header("Location"), Some("/banking/?x=1"));
    assert_eq!(answer.header("Cache-Control"), Some("no-cache"));
}

#[test]
fn nothing_else_is_served() {
    let serving = serve_site();
    for target in [
        "/missing.html",
        "/assets/",
        "/../built/ghg.toml",
        "/banking/../../built/ghg.toml",
        "/%2e%2e/built/ghg.toml",
        "/%FF",
        "/__ghg/build",
        "/__ghg/reload.js",
    ] {
        let answer = get(&serving.address, target);
        assert_eq!(answer.status, 404, "{target}");
        assert_eq!(answer.text(), "Nothing is at this address in the site.\n");
        assert_eq!(
            answer.header("Content-Type"),
            Some("text/plain; charset=utf-8")
        );
    }
    let answer = request(&serving.address, "POST", "/index.html");
    assert_eq!(answer.status, 405);
    assert_eq!(answer.header("Allow"), Some("GET, HEAD"));
    assert_eq!(
        answer.text(),
        "ghg serve answers only GET and HEAD requests.\n"
    );
}

// Watching and reloading: rules 65 to 70; tests/browser/reload.mjs checks 71 and 72.

#[test]
fn watch_and_no_build_are_a_mistake_together() {
    let (stdout, stderr, status) = serve_to_the_end(&fixture(""), &["--watch", "--no-build"]);
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        "The options --watch and --no-build cannot be used together.\n"
    );
    assert_eq!(status, Some(2));
}

#[test]
fn watching_rebuilds_on_a_change_and_keeps_the_last_site_that_built() {
    let folder = built_copy("serve-watch");
    let mut serving = serve(&folder, &["--watch", "--port", "0"]);
    let address = serving.address.clone();
    let build = || get(&address, "/__ghg/build").text();
    assert_eq!(build(), "1");
    let page = get(&address, "/withdraw.html").text();
    assert!(
        page.ends_with(
            "<script type=\"module\" src=\"/__ghg/reload.js?build=1\"></script></body>\n</html>\n"
        ),
        "{page}"
    );
    assert_eq!(page.matches("__ghg").count(), 1);
    let script = get(&address, "/__ghg/reload.js?build=1");
    assert_eq!(script.status, 200);
    assert_eq!(
        script.header("Content-Type"),
        Some("text/javascript; charset=utf-8")
    );
    assert!(script.text().contains("/__ghg/build"));

    let feature = folder.join("features/withdraw.feature");
    let source = read_to_string(&feature).expect("read");
    write(&feature, source.replace("20", "30")).expect("written");
    eventually("a rebuild after a feature changed", || build() == "2");
    let page = get(&address, "/withdraw.html").text();
    assert!(
        page.contains("withdraws <span class=\"ghg-number\">30</span>"),
        "{page}"
    );
    assert!(page.contains("/__ghg/reload.js?build=2"), "{page}");
    let mut line = String::new();
    serving.stdout.read_line(&mut line).expect("a line");
    assert_eq!(line, "The site was rebuilt.\n");

    write(
        folder.join("ghg.toml"),
        "input = \"features\"\nallow_unnamed = true\n",
    )
    .expect("written");
    eventually("a rebuild after ghg.toml changed", || build() == "3");
    line.clear();
    serving.stdout.read_line(&mut line).expect("a line");
    assert_eq!(line, "The site was rebuilt.\n");

    write(folder.join("features/.ghg-ignore"), "withdraw.feature\n").expect("written");
    eventually("a rebuild after a .ghg-ignore file changed", || {
        build() == "4"
    });
    assert_eq!(get(&address, "/withdraw.html").status, 404);

    write(folder.join("ghg.toml"), "inputs = \"features\"\n").expect("written");
    let mut stderr = BufReader::new(serving.child.stderr.take().expect("piped"));
    let mut report = String::new();
    stderr.read_line(&mut report).expect("a line");
    assert_eq!(report, "ghg.toml:1: ghg has no setting named inputs.\n");
    report.clear();
    stderr.read_line(&mut report).expect("a line");
    assert_eq!(
        report,
        "The site was not rebuilt, so the last site that built is still served.\n"
    );
    assert_eq!(build(), "4");
    write(folder.join("ghg.toml"), "input = \"features\"\n").expect("written");
    eventually("a rebuild after ghg.toml was mended", || build() == "5");

    write(folder.join("features/broken.feature"), "").expect("written");
    report.clear();
    stderr.read_line(&mut report).expect("a line");
    assert!(report.starts_with("broken.feature:"), "{report}");
    report.clear();
    stderr.read_line(&mut report).expect("a line");
    assert_eq!(
        report,
        "The site was not rebuilt, so the last site that built is still served.\n"
    );
    assert_eq!(build(), "5");
    assert_eq!(get(&address, "/assets/gherkin-glass.css").status, 200);
}

#[test]
fn watching_rebuilds_only_for_changes_a_build_reads() {
    let folder = built_copy("serve-watch-reads");
    let serving = serve(&folder, &["--watch", "--port", "0"]);
    let build = || get(&serving.address, "/__ghg/build").text();
    assert_eq!(build(), "1");
    write(folder.join("features/notes.txt"), "notes").expect("written");
    write(folder.join("features/.withdraw.feature.swp"), "").expect("written");
    write(folder.join("ghg.toml~"), "").expect("written");
    // Longer than a burst takes to end and be looked at.
    sleep(Duration::from_millis(1000));
    assert_eq!(build(), "1");

    let outside = folder.join("outside/more");
    create_dir_all(&outside).expect("folder");
    write(outside.join("more.feature"), "Feature: More\n").expect("written");
    rename(&outside, folder.join("features/more")).expect("moved");
    eventually(
        "a rebuild after a folder with a feature was moved in",
        || build() == "2",
    );
    assert_eq!(get(&serving.address, "/more/more.html").status, 200);
}

#[test]
fn watching_never_writes_the_output_folder() {
    let folder = built_copy("serve-watch-writes-nothing");
    let serving = serve(&folder, &["--watch", "--port", "0"]);
    assert_eq!(get(&serving.address, "/__ghg/build").text(), "1");
    assert_eq!(get(&serving.address, "/withdraw.html").status, 200);
    assert!(!folder.join("site").exists());
}

#[test]
fn a_configuration_file_made_while_watching_starts_a_rebuild() {
    let folder = built_copy("serve-watch-new-config");
    std::fs::remove_file(folder.join("ghg.toml")).expect("removed");
    let serving = serve(&folder, &["--watch", "--port", "0"]);
    let build = || get(&serving.address, "/__ghg/build").text();
    assert_eq!(build(), "1");
    write(folder.join("ghg.toml"), "allow_unnamed = true\n").expect("written");
    eventually("a rebuild after ghg.toml was made", || build() == "2");
}

#[test]
fn a_configuration_file_inside_the_input_keeps_every_subfolder_watched() {
    let folder = built_copy("serve-watch-config-inside");
    create_dir_all(folder.join("features/sub")).expect("folder");
    write(folder.join("features/ghg.toml"), "input = \".\"\n").expect("written");
    let serving = serve(
        &folder,
        &["--watch", "--port", "0", "--config", "features/ghg.toml"],
    );
    let build = || get(&serving.address, "/__ghg/build").text();
    assert_eq!(build(), "1");
    let feature = folder.join("features/sub/more.feature");
    write(&feature, "Feature: More\n").expect("written");
    eventually("a rebuild after a feature in a subfolder changed", || {
        build() == "2"
    });
}
