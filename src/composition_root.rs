//! Composition root: the ghg command, wired to the real file system, network
//! and terminal, for both binaries, `ghg` and the cargo plugin `cargo-ghg`.

use std::ffi::OsString;
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;
use std::sync::Arc;

use crate::adapters::{
    ConfigFileReader, FileWatcher, GherkinCrate, HttpPreviewServer, InputFolderReader, MemorySite,
    OutputFolderStore, SiteFiles, StandardStreams,
};
use crate::controllers::SiteController;
use crate::handlers::CommandLine;
use crate::policies::cargo_plugin_arguments;
use crate::services::{InputService, OutputService, ServerService, SettingsService};

/// Runs ghg with `arguments`, those after the command's own name.
pub fn run(arguments: impl IntoIterator<Item = OsString>) -> ExitCode {
    let site = SiteController::new(
        SettingsService::new(Arc::new(ConfigFileReader)),
        InputService::new(Arc::new(InputFolderReader), Arc::new(GherkinCrate)),
        OutputService::new(Arc::new(OutputFolderStore)),
        Arc::new(SiteFiles),
        Arc::new(MemorySite::default()),
        ServerService::new(Arc::new(HttpPreviewServer), Arc::new(FileWatcher)),
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
