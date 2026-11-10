//! Controller: the `ghg` command, from its arguments to what the person at the terminal reads.

mod wording;

use std::ffi::OsString;
use std::sync::Arc;

use i18n_embed_fl::fl;

use crate::entities::{BuildOptions, Invocation};
use crate::policies::{CATALOGUE, parse_arguments};
use crate::ports::ConfigReader;
use crate::services::{SettingsService, SiteService};

pub struct CommandLine {
    settings: SettingsService,
    site: SiteService,
}

/// What a run prints, and the status it exits with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub stdout: String,
    pub stderr: String,
    /// 0 when it did what was asked, 1 when the configuration or the input
    /// stopped it, 2 when the command line was mistaken.
    pub status: u8,
}

impl CommandLine {
    pub fn new(config_reader: Arc<dyn ConfigReader>, site: SiteService) -> Self {
        CommandLine {
            settings: SettingsService::new(config_reader),
            site,
        }
    }

    /// `arguments` come after the command's own name.
    pub fn run(&self, arguments: impl IntoIterator<Item = OsString>) -> Reply {
        match parse_arguments(arguments) {
            Err(problem) => Reply::failure(2, wording::argument_problem(&problem)),
            Ok(Invocation::Help) => Reply::success(fl!(CATALOGUE, "help")),
            Ok(Invocation::Version) => Reply::success(fl!(
                CATALOGUE,
                "version",
                version = env!("CARGO_PKG_VERSION")
            )),
            Ok(Invocation::Build(options)) => self.build(&options),
        }
    }

    fn build(&self, options: &BuildOptions) -> Reply {
        let settings = match self.settings.load(options) {
            Ok(settings) => settings,
            Err(problems) => return Reply::failure(1, wording::config_problems(&problems)),
        };
        match self.site.build(&settings, &settings.output) {
            Ok(_) => Reply::success(String::new()),
            Err(problems) => Reply::failure(1, wording::problems(&problems)),
        }
    }
}

impl Reply {
    fn success(stdout: String) -> Self {
        Reply {
            stdout: lines(stdout),
            stderr: String::new(),
            status: 0,
        }
    }

    fn failure(status: u8, stderr: String) -> Self {
        Reply {
            stdout: String::new(),
            stderr: lines(stderr),
            status,
        }
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
    use crate::entities::{CollectedInput, ParseFailure, ProblemKind, RawFeature};
    use crate::ports::{FeatureParser, InputCollector, SiteWriter};
    use crate::services::InputService;

    /// Writes nothing; the parser fails every file before anything is written.
    struct NoWriter;

    impl SiteWriter for NoWriter {
        fn write(&self, _output: &Path, _file: &str, _contents: &[u8]) -> IoResult<()> {
            Ok(())
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
        let input = InputService::new(Arc::new(FakeCollector), Arc::new(FailingParser));
        let site = SiteService::new(input, Arc::new(NoWriter));
        CommandLine::new(Arc::new(FakeFiles(files)), site).run(arguments.iter().map(OsString::from))
    }

    #[test]
    fn without_a_file_the_input_is_the_default_folder() {
        let reply = run(&[], &["build"]);
        assert_eq!(reply.stderr, "a.feature:1: read in features\n");
        assert_eq!(reply.status, 1);
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
        assert_eq!(reply.status, 1);
    }

    #[test]
    fn a_mistaken_file_stops_before_the_input_is_read() {
        let reply = run(&[("ghg.toml", "inputs = \"x\"\ninput = 1\n")], &["build"]);
        assert_eq!(
            reply.stderr,
            "ghg.toml:1: ghg has no setting named inputs.\n\
             ghg.toml:2: input must be text, in quotes.\n"
        );
        assert_eq!(reply.status, 1);
    }

    #[test]
    fn a_mistaken_command_line_exits_with_2() {
        let reply = run(&[], &["build", "--bogus"]);
        assert_eq!(
            reply.stderr,
            "ghg build has no option --bogus. ghg --help lists the options.\n"
        );
        assert_eq!(reply.status, 2);
        assert_eq!(reply.stdout, "");
    }
}
