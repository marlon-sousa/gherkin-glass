use std::path::Path;
use std::sync::Mutex;

use rstest::rstest;

use super::*;
use crate::errors::{
    BuildError, InputError, InputErrorKind, InputErrors, ListenErrorKind, ServeError,
};

/// Answers every build and serve as it was made to, and counts how often it was asked.
struct FakeSite {
    build: Result<BuiltSite, BuildError>,
    serve: Result<(), ServeError>,
    asked: Mutex<usize>,
}

impl Default for FakeSite {
    fn default() -> Self {
        FakeSite {
            build: Ok(BuiltSite::default()),
            serve: Ok(()),
            asked: Mutex::new(0),
        }
    }
}

impl SiteApi for FakeSite {
    fn build(&self, _: &BuildOptions) -> Result<BuiltSite, BuildError> {
        *self.asked.lock().expect("lock") += 1;
        self.build.clone()
    }

    fn serve(&self, _: &ServeOptions, _: &dyn SiteListener) -> Result<(), ServeError> {
        *self.asked.lock().expect("lock") += 1;
        self.serve.clone()
    }
}

/// Keeps what is said to standard output.
#[derive(Default)]
struct Said {
    out: Mutex<String>,
}

impl Terminal for Said {
    fn say(&self, text: &str) {
        self.out.lock().expect("lock").push_str(text);
    }

    fn warn(&self, _: &str) {}
}

fn run(site: FakeSite, arguments: &[&str]) -> (Reply, usize) {
    let site = Arc::new(site);
    let reply = CommandLine::new(site.clone(), Arc::new(Said::default()))
        .run(arguments.iter().map(OsString::from));
    let asked = *site.asked.lock().expect("lock");
    (reply, asked)
}

#[rstest]
// Rule 19.
#[case::an_unknown_option(
    &["build", "--bogus"],
    "ghg build has no option --bogus. ghg --help lists the options.\n"
)]
#[case::an_option_of_serve_only(
    &["build", "--watch"],
    "ghg build has no option --watch. ghg --help lists the options.\n"
)]
// Rule 58.
#[case::a_port_that_is_not_a_number(
    &["serve", "--port", "80a"],
    "The option --port needs a whole number from 0 to 65535, not 80a.\n"
)]
// Rule 68.
#[case::watch_and_no_build_together(
    &["serve", "--watch", "--no-build"],
    "The options --watch and --no-build cannot be used together.\n"
)]
fn a_mistaken_command_line_is_named_and_exits_with_2(
    #[case] arguments: &[&str],
    #[case] said: &str,
) {
    let (reply, asked) = run(FakeSite::default(), arguments);
    assert_eq!(reply.stderr, said);
    assert_eq!(reply.stdout, "");
    assert_eq!(reply.status, ExitStatus::Mistaken);
    assert_eq!(asked, 0);
}

#[test]
fn a_build_that_stops_prints_its_report_and_exits_with_1() {
    // Rule 20.
    let error = InputError::new(
        Path::new("in"),
        Path::new("in/a.feature"),
        Some(1),
        InputErrorKind::Syntax,
    );
    let site = FakeSite {
        build: Err(BuildError::Input(InputErrors::new(vec![error]))),
        ..FakeSite::default()
    };
    let (reply, asked) = run(site, &["build"]);
    assert_eq!(reply.stderr, "a.feature:1: Syntax error.\n");
    assert_eq!(reply.stdout, "");
    assert_eq!(reply.status, ExitStatus::Stopped);
    assert_eq!(asked, 1);
}

#[test]
fn a_site_with_no_title_is_hinted_at_before_large_search_documents() {
    // Rule 17.
    let site = FakeSite {
        build: Ok(BuiltSite {
            large_search_documents: Some(5_200_001),
            untitled: true,
        }),
        ..FakeSite::default()
    };
    let (reply, _) = run(site, &["build"]);
    assert_eq!(
        reply.stderr,
        "The site has no title, so its home page is called Feature files. Set title in ghg.toml, or give --title.\n\
         The search documents are 5.3 MB, larger than the 5 MB above which search can be slow to start, mostly on phones.\n"
    );
    assert_eq!(reply.status, ExitStatus::Done);
}

#[test]
fn serve_names_the_port_it_cannot_listen_at() {
    // Rule 59.
    let site = FakeSite {
        serve: Err(ServeError::Listen {
            host: "127.0.0.1".into(),
            port: 8123,
            kind: ListenErrorKind::PortTaken,
        }),
        ..FakeSite::default()
    };
    let (reply, _) = run(site, &["serve", "--port", "8123"]);
    assert_eq!(
        reply.stderr,
        "Port 8123 is already in use. Choose another with --port.\n"
    );
    assert_eq!(reply.stdout, "");
    assert_eq!(reply.status, ExitStatus::Stopped);
}

#[rstest]
// Rule 60.
#[case::every_ipv4_network("0.0.0.0:8000", "http://127.0.0.1:8000/")]
#[case::every_ipv6_network("[::]:8000", "http://[::1]:8000/")]
#[case::one_address("192.0.2.1:80", "http://192.0.2.1:80/")]
fn the_address_that_means_every_network_is_printed_as_this_machine_s(
    #[case] listening: &str,
    #[case] printed: &str,
) {
    let said = Said::default();
    TerminalListener { terminal: &said }.listening(listening.parse().expect("an address"));
    assert_eq!(
        *said.out.lock().expect("lock"),
        format!("The site is served at {printed}. Press Control+C to stop.\n")
    );
}
