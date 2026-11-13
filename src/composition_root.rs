//! Composition root: the ghg command, wired to the real file system, network
//! and terminal, for both binaries, `ghg` and the cargo plugin `cargo-ghg`.

use std::ffi::OsString;
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;
use std::sync::Arc;

use crate::adapters::{
    FileSystemReader, FileWatcher, FolderFiles, GherkinCrate, HttpPreviewServer, InputFolderReader,
    MemoryFiles, OutputFolderStore, StandardStreams,
};
use crate::controllers::SiteController;
use crate::handlers::CommandLine;
use crate::policies::cargo_plugin_arguments;
use crate::services::{
    Converter, GherkinParser, InputCollector, ServerService, SettingsService, SiteWriter,
};

/// Runs ghg with `arguments`, those after the command's own name.
pub fn run(arguments: impl IntoIterator<Item = OsString>) -> ExitCode {
    let site = SiteController::new(
        Arc::new(SettingsService::new(Arc::new(FileSystemReader))),
        Arc::new(InputCollector::new(Arc::new(InputFolderReader))),
        Arc::new(GherkinParser::new(Arc::new(GherkinCrate))),
        Arc::new(Converter),
        Arc::new(SiteWriter::new(
            Arc::new(FileSystemReader),
            Arc::new(OutputFolderStore),
            Arc::new(FolderFiles),
            Arc::new(MemoryFiles::default()),
        )),
        Arc::new(ServerService::new(
            Arc::new(HttpPreviewServer),
            Arc::new(FileWatcher),
        )),
    );
    let reply = CommandLine::new(Arc::new(site), Arc::new(StandardStreams)).run(arguments);
    // A closed pipe or a full disk leaves nothing more to say, so the status still stands.
    let _ = stdout().write_all(reply.stdout.as_bytes());
    let _ = stderr().write_all(reply.stderr.as_bytes());
    reply.exit_code()
}

/// Runs ghg as cargo runs the plugin, `cargo ghg`, with `arguments`, those after the plugin's own name.
pub fn run_cargo_plugin(arguments: impl IntoIterator<Item = OsString>) -> ExitCode {
    run(cargo_plugin_arguments(arguments))
}
