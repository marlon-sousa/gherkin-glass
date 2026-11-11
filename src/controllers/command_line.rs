//! Controller: the `ghg` command, from its arguments to what the person at the terminal reads.

mod wording;

use std::ffi::OsString;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use i18n_embed_fl::fl;

use crate::entities::{
    BuildFailure, BuildOptions, ExitStatus, Invocation, ServeOptions, ServerWords,
};
use crate::policies::{CATALOGUE, parse_arguments};
use crate::ports::{ConfigReader, Terminal};
use crate::services::{
    OutputService, PreviewEvents, PreviewService, RebuildFailure, SettingsService, SiteService,
};

pub(crate) struct CommandLine {
    settings: SettingsService,
    site: SiteService,
    output: OutputService,
    preview: PreviewService,
    terminal: Arc<dyn Terminal>,
}

/// What a run prints when it ends, and the status it exits with. A command
/// that keeps running, `ghg serve`, says what happens meanwhile through the
/// terminal it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reply {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) status: ExitStatus,
}

impl CommandLine {
    pub(crate) fn new(
        config_reader: Arc<dyn ConfigReader>,
        site: SiteService,
        output: OutputService,
        preview: PreviewService,
        terminal: Arc<dyn Terminal>,
    ) -> Self {
        CommandLine {
            settings: SettingsService::new(config_reader),
            site,
            output,
            preview,
            terminal,
        }
    }

    /// `arguments` come after the command's own name.
    pub(crate) fn run(&self, arguments: impl IntoIterator<Item = OsString>) -> Reply {
        match parse_arguments(arguments) {
            Err(problem) => {
                Reply::failure(ExitStatus::Mistaken, wording::argument_problem(&problem))
            }
            Ok(Invocation::Help) => Reply::success(fl!(CATALOGUE, "help")),
            Ok(Invocation::Version) => Reply::success(fl!(
                CATALOGUE,
                "version",
                version = env!("CARGO_PKG_VERSION")
            )),
            Ok(Invocation::Build(options)) => self.build(&options),
            Ok(Invocation::Serve(options)) => self.serve(&options),
        }
    }

    fn serve(&self, options: &ServeOptions) -> Reply {
        let words = ServerWords {
            not_found: fl!(CATALOGUE, "serve-not-found"),
            wrong_method: fl!(CATALOGUE, "serve-wrong-method"),
        };
        let events = Events(self.terminal.as_ref());
        match self.preview.serve(options, words, &events) {
            Ok(()) => Reply::success(String::new()),
            Err(failure) => Reply::failure(ExitStatus::Stopped, wording::serve_failure(&failure)),
        }
    }

    fn build(&self, options: &BuildOptions) -> Reply {
        let settings = match self.settings.load(options) {
            Ok(settings) => settings,
            Err(problems) => {
                return Reply::failure(ExitStatus::Stopped, wording::config_problems(&problems));
            }
        };
        match self.output.build(&self.site, &settings) {
            Ok(built) => {
                let mut stderr = wording::warnings(&built);
                if settings.pages.title.is_none() {
                    stderr.insert_str(0, &lines(fl!(CATALOGUE, "no-title")));
                }
                Reply {
                    stderr: lines(stderr),
                    ..Reply::success(String::new())
                }
            }
            Err(BuildFailure::Problems(problems)) => {
                Reply::failure(ExitStatus::Stopped, wording::problems(&problems))
            }
            Err(BuildFailure::Output(problem)) => {
                Reply::failure(ExitStatus::Stopped, wording::output_problem(&problem))
            }
        }
    }
}

impl Reply {
    fn success(stdout: String) -> Self {
        Reply {
            stdout: lines(stdout),
            stderr: String::new(),
            status: ExitStatus::Done,
        }
    }

    fn failure(status: ExitStatus, stderr: String) -> Self {
        Reply {
            stdout: String::new(),
            stderr: lines(stderr),
            status,
        }
    }
}

/// ghg serve's events, worded for the terminal.
struct Events<'a>(&'a dyn Terminal);

impl PreviewEvents for Events<'_> {
    fn listening(&self, mut address: SocketAddr) {
        // A browser cannot open the address that means every network.
        if address.ip().is_unspecified() {
            address.set_ip(match address {
                SocketAddr::V4(_) => IpAddr::V4(Ipv4Addr::LOCALHOST),
                SocketAddr::V6(_) => IpAddr::V6(Ipv6Addr::LOCALHOST),
            });
        }
        let address = format!("http://{address}/");
        self.0
            .say(&lines(fl!(CATALOGUE, "serve-listening", address = address)));
    }

    fn rebuilt(&self) {
        self.0.say(&lines(fl!(CATALOGUE, "serve-rebuilt")));
    }

    fn not_rebuilt(&self, failure: &RebuildFailure) {
        self.0.warn(&wording::rebuild_failure(failure));
    }
}

/// Text ending in a line break, unless it is empty.
fn lines(mut text: String) -> String {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::{Error as IoError, ErrorKind, Result as IoResult};
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::entities::{
        Changes, CollectedInput, FolderState, ListenFailure, Marker, ParseFailure, Preview,
        ProblemKind, RawFeature, VersionName, WatchFailure, Watched,
    };
    use crate::ports::{
        ChangeWatcher, FeatureParser, InputCollector, OutputFolders, PreviewServer, SiteRecorder,
        SiteWriter,
    };
    use crate::services::{InputService, ServeFailure};

    /// Keeps nothing; the parser fails every file before anything is written.
    struct NoRecorder;

    impl SiteWriter for NoRecorder {
        fn write(&self, _output: &Path, _file: &str, _contents: &[u8]) -> IoResult<()> {
            Ok(())
        }
    }

    impl SiteRecorder for NoRecorder {
        fn take(&self) -> BTreeMap<String, Vec<u8>> {
            BTreeMap::new()
        }
    }

    /// An output folder with no version in it, whose folders are never touched.
    struct NoVersions;

    impl OutputFolders for NoVersions {
        fn state(&self, _folder: &Path, _marker: &Marker) -> IoResult<FolderState> {
            Ok(FolderState::Missing)
        }

        fn clear(&self, _folder: &Path) -> IoResult<()> {
            Ok(())
        }

        fn remove(&self, _path: &Path) -> IoResult<()> {
            Ok(())
        }

        fn mark(&self, _folder: &Path, _marker: &Marker) -> IoResult<()> {
            Ok(())
        }

        fn marked(&self, _output: &Path) -> IoResult<Vec<VersionName>> {
            Ok(Vec::new())
        }
    }

    /// Files held in memory; any other path is not found.
    struct FakeFiles(BTreeMap<PathBuf, String>);

    impl ConfigReader for FakeFiles {
        fn read(&self, path: &Path) -> IoResult<String> {
            self.0
                .get(path)
                .cloned()
                .ok_or_else(|| IoError::new(ErrorKind::NotFound, "not found"))
        }
    }

    /// Finds one file in the input folder, `a.feature`, whatever the folder,
    /// so the folder read shows in the report.
    struct FakeCollector;

    impl InputCollector for FakeCollector {
        fn collect(&self, input: &Path) -> CollectedInput {
            CollectedInput {
                paths: vec![input.join("a.feature")],
                problems: Vec::new(),
            }
        }
    }

    /// Fails every file, with the input folder in its message.
    struct FailingParser;

    impl FeatureParser for FailingParser {
        fn parse(&self, path: &Path) -> Result<RawFeature, ParseFailure> {
            Err(ParseFailure {
                line: Some(1),
                kind: ProblemKind::Parser {
                    message: format!("read in {}", path.parent().unwrap_or(path).display()),
                },
            })
        }
    }

    fn run(files: &[(&str, &str)], arguments: &[&str]) -> Reply {
        let files = files
            .iter()
            .map(|(path, text)| (PathBuf::from(path), text.to_string()))
            .collect();
        let files: Arc<FakeFiles> = Arc::new(FakeFiles(files));
        let input = InputService::new(Arc::new(FakeCollector), Arc::new(FailingParser));
        let site = || SiteService::new(input.clone(), Arc::new(NoRecorder));
        let output = OutputService::new(Arc::new(NoVersions));
        let preview = PreviewService::new(
            files.clone(),
            site(),
            output.clone(),
            input.clone(),
            Arc::new(NoRecorder),
            Arc::new(PortTaken),
            Arc::new(NoWatching),
        );
        CommandLine::new(files, site(), output, preview, Arc::new(Silent))
            .run(arguments.iter().map(OsString::from))
    }

    /// Every port is taken.
    struct PortTaken;

    impl PreviewServer for PortTaken {
        fn start(
            &self,
            _: &str,
            _: u16,
            _: Arc<Preview>,
            _: ServerWords,
        ) -> Result<SocketAddr, ListenFailure> {
            Err(ListenFailure::PortTaken)
        }
    }

    struct NoWatching;

    impl ChangeWatcher for NoWatching {
        fn watch(&self, watched: &[Watched]) -> Result<Changes, WatchFailure> {
            Err(WatchFailure {
                path: match &watched[0] {
                    Watched::Folder(path) | Watched::File(path) => path.clone(),
                },
                reason: "no watching here".into(),
            })
        }
    }

    struct Silent;

    impl Terminal for Silent {
        fn say(&self, _: &str) {}
        fn warn(&self, _: &str) {}
    }

    #[test]
    fn serve_reads_the_configuration_and_builds_as_build_does() {
        let file = [("ghg.toml", "input = \"specs\"\n")];
        assert_eq!(
            run(&file, &["serve"]).stderr,
            "a.feature:1: read in specs\n"
        );
        let reply = run(&[("ghg.toml", "inputs = 1\n")], &["serve", "--no-build"]);
        assert_eq!(
            reply.stderr,
            "ghg.toml:1: ghg has no setting named inputs.\n"
        );
        assert_eq!(reply.status, ExitStatus::Stopped);
    }

    #[test]
    fn serve_names_the_port_it_cannot_listen_at() {
        let reply = run(&[], &["serve", "--no-build", "--port", "8123"]);
        assert_eq!(
            reply.stderr,
            "Port 8123 is already in use. Choose another with --port.\n"
        );
        assert_eq!(reply.stdout, "");
        assert_eq!(reply.status, ExitStatus::Stopped);
    }

    #[test]
    fn serve_names_what_it_cannot_watch_and_where_it_cannot_listen() {
        let failure = ServeFailure::Watch(WatchFailure {
            path: PathBuf::from("features"),
            reason: "too many folders".into(),
        });
        assert_eq!(
            wording::serve_failure(&failure),
            "ghg serve cannot watch features for changes: too many folders"
        );
        let failure = ServeFailure::Listen {
            host: "203.0.113.1".into(),
            port: 8000,
            failure: ListenFailure::Other {
                reason: "no such address".into(),
            },
        };
        assert_eq!(
            wording::serve_failure(&failure),
            "ghg serve cannot listen at 203.0.113.1, port 8000: no such address"
        );
    }

    #[derive(Default)]
    struct Said(std::sync::Mutex<String>);

    impl Terminal for Said {
        fn say(&self, text: &str) {
            self.0.lock().expect("lock").push_str(text);
        }
        fn warn(&self, _: &str) {}
    }

    #[test]
    fn the_address_that_means_every_network_is_printed_as_this_machine_s() {
        for (listening, printed) in [
            ("0.0.0.0:8000", "http://127.0.0.1:8000/"),
            ("[::]:8000", "http://[::1]:8000/"),
            ("192.0.2.1:80", "http://192.0.2.1:80/"),
        ] {
            let said = Said::default();
            Events(&said).listening(listening.parse().expect("an address"));
            assert_eq!(
                *said.0.lock().expect("lock"),
                format!("The site is served at {printed}. Press Control+C to stop.\n")
            );
        }
    }

    #[test]
    fn serve_mistakes_are_named() {
        let reply = run(&[], &["serve", "--port", "80a"]);
        assert_eq!(
            reply.stderr,
            "The option --port needs a whole number from 0 to 65535, not 80a.\n"
        );
        assert_eq!(reply.status, ExitStatus::Mistaken);
        let reply = run(&[], &["serve", "--watch", "--no-build"]);
        assert_eq!(
            reply.stderr,
            "The options --watch and --no-build cannot be used together.\n"
        );
        assert_eq!(reply.status, ExitStatus::Mistaken);
        let reply = run(&[], &["build", "--watch"]);
        assert_eq!(
            reply.stderr,
            "ghg build has no option --watch. ghg --help lists the options.\n"
        );
    }

    #[test]
    fn without_a_file_the_input_is_the_default_folder() {
        let reply = run(&[], &["build"]);
        assert_eq!(reply.stderr, "a.feature:1: read in features\n");
        assert_eq!(reply.status, ExitStatus::Stopped);
    }

    #[test]
    fn the_file_s_input_is_read_and_the_option_overrides_it() {
        let file = [("ghg.toml", "input = \"specs\"\n")];
        assert_eq!(
            run(&file, &["build"]).stderr,
            "a.feature:1: read in specs\n"
        );
        assert_eq!(
            run(&file, &["build", "--input", "other"]).stderr,
            "a.feature:1: read in other\n"
        );
    }

    #[test]
    fn a_file_asked_for_must_exist() {
        let reply = run(&[], &["build", "--config", "ci.toml"]);
        assert_eq!(reply.stderr, "ci.toml: cannot be read: not found\n");
        assert_eq!(reply.status, ExitStatus::Stopped);
    }

    #[test]
    fn a_mistaken_file_stops_before_the_input_is_read() {
        let reply = run(&[("ghg.toml", "inputs = \"x\"\ninput = 1\n")], &["build"]);
        assert_eq!(
            reply.stderr,
            "ghg.toml:1: ghg has no setting named inputs.\n\
             ghg.toml:2: input must be text, in quotes.\n"
        );
        assert_eq!(reply.status, ExitStatus::Stopped);
    }

    #[test]
    fn a_mistaken_command_line_exits_with_2() {
        let reply = run(&[], &["build", "--bogus"]);
        assert_eq!(
            reply.stderr,
            "ghg build has no option --bogus. ghg --help lists the options.\n"
        );
        assert_eq!(reply.status, ExitStatus::Mistaken);
        assert_eq!(reply.stdout, "");
    }
}
