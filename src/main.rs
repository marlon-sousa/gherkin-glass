//! Composition root: the ghg command, wired to the real file system, network and terminal.

use std::env::args_os;
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;
use std::sync::Arc;

use gherkin_glass::{
    CommandLine, ConfigFileReader, FileWatcher, GherkinParser, HttpPreviewServer, InputService,
    InputWalker, MemorySite, PreviewService, SiteFiles, SiteService, StandardStreams,
};

fn main() -> ExitCode {
    let config_reader = Arc::new(ConfigFileReader);
    let input = InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser));
    let memory = Arc::new(MemorySite::default());
    let preview = PreviewService::new(
        config_reader.clone(),
        SiteService::new(input.clone(), Arc::new(SiteFiles)),
        SiteService::new(input.clone(), memory.clone()),
        memory,
        Arc::new(HttpPreviewServer),
        Arc::new(FileWatcher),
    );
    let site = SiteService::new(input, Arc::new(SiteFiles));
    let reply = CommandLine::new(config_reader, site, preview, Arc::new(StandardStreams))
        .run(args_os().skip(1));
    // A closed pipe or a full disk leaves nothing more to say, so the status still stands.
    let _ = stdout().write_all(reply.stdout.as_bytes());
    let _ = stderr().write_all(reply.stderr.as_bytes());
    ExitCode::from(reply.status)
}
