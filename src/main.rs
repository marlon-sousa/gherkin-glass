//! Composition root: the ghg command, wired to the real file system.

use std::env::args_os;
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;
use std::sync::Arc;

use gherkin_glass::{CommandLine, ConfigFileReader, GherkinParser, InputService, InputWalker};

fn main() -> ExitCode {
    let input = InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser));
    let reply = CommandLine::new(Arc::new(ConfigFileReader), input).run(args_os().skip(1));
    // A closed pipe or a full disk leaves nothing more to say, so the status still stands.
    let _ = stdout().write_all(reply.stdout.as_bytes());
    let _ = stderr().write_all(reply.stderr.as_bytes());
    ExitCode::from(reply.status)
}
