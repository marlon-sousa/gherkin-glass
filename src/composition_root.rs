//! Composition root: the ghg command, wired to the real file system, network
//! and terminal, for both binaries, `ghg` and the cargo plugin `cargo-ghg`.

use std::ffi::OsString;
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;
use std::sync::Arc;

use gherkin_glass::{
    CommandLine, ConfigFileReader, FileWatcher, GherkinParser, HttpPreviewServer, InputService,
    InputWalker, MemorySite, OutputFolderStore, OutputService, PreviewService, SiteFiles,
    SiteService, StandardStreams,
};

/// Runs ghg with `arguments`, those after the command's own name.
pub(crate) fn run(arguments: impl IntoIterator<Item = OsString>) -> ExitCode {
    let config_reader = Arc::new(ConfigFileReader);
    let input = InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser));
    let output = OutputService::new(Arc::new(OutputFolderStore));
    let preview = PreviewService::new(
        config_reader.clone(),
        SiteService::new(input.clone(), Arc::new(SiteFiles)),
        output.clone(),
        input.clone(),
        Arc::new(MemorySite::default()),
        Arc::new(HttpPreviewServer),
        Arc::new(FileWatcher),
    );
    let site = SiteService::new(input, Arc::new(SiteFiles));
    let reply = CommandLine::new(
        config_reader,
        site,
        output,
        preview,
        Arc::new(StandardStreams),
    )
    .run(arguments);
    // A closed pipe or a full disk leaves nothing more to say, so the status still stands.
    let _ = stdout().write_all(reply.stdout.as_bytes());
    let _ = stderr().write_all(reply.stderr.as_bytes());
    ExitCode::from(reply.status)
}
