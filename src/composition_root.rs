//! Composition root: the ghg command, wired to the real file system, network,
//! signals, standard input and terminal, for both binaries, `ghg` and the cargo plugin `cargo-ghg`.

use std::ffi::OsString;
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;
use std::sync::Arc;

use snapfit::container;

use crate::adapters::{
    CrossbeamEventWaiter, CtrlcSignals, FileSystemReader, FileWatcher, FolderFiles, GherkinCrate,
    HttpPreviewServer, InputFolderReader, MemoryFiles, OutputFolderStore, StandardInputReader,
    StandardStreams,
};
use crate::controllers::SiteController;
use crate::handlers::CommandLine;
use crate::policies::cargo_plugin_arguments;
use crate::services::{
    Converter, GherkinParser, InputCollector, ServerService, SettingsService, SiteWriter,
};

#[container(leaf_build = default)]
struct GhgParts {
    #[singleton]
    file_system_reader: Arc<FileSystemReader>,
    #[singleton(deps = [file_system_reader])]
    settings: Arc<SettingsService>,
    #[singleton]
    input_folder_reader: Arc<InputFolderReader>,
    #[singleton]
    standard_input_reader: Arc<StandardInputReader>,
    #[singleton(deps = [input_folder_reader, file_system_reader, standard_input_reader])]
    input_collector: Arc<InputCollector>,
    #[singleton]
    gherkin_crate: Arc<GherkinCrate>,
    #[singleton(deps = [gherkin_crate])]
    gherkin_parser: Arc<GherkinParser>,
    #[singleton]
    converter: Arc<Converter>,
    #[singleton]
    output_folder_store: Arc<OutputFolderStore>,
    #[singleton]
    folder_files: Arc<FolderFiles>,
    #[singleton]
    memory_files: Arc<MemoryFiles>,
    #[singleton(deps = [file_system_reader, output_folder_store, folder_files, memory_files])]
    site_writer: Arc<SiteWriter>,
    #[singleton]
    http_preview_server: Arc<HttpPreviewServer>,
    #[singleton]
    file_watcher: Arc<FileWatcher>,
    #[singleton]
    ctrlc_signals: Arc<CtrlcSignals>,
    #[singleton]
    crossbeam_event_waiter: Arc<CrossbeamEventWaiter>,
    #[singleton(deps = [http_preview_server, file_watcher, ctrlc_signals, crossbeam_event_waiter])]
    server: Arc<ServerService>,
    #[singleton(deps = [settings, input_collector, gherkin_parser, converter, site_writer, server])]
    site: Arc<SiteController>,
    #[singleton]
    standard_streams: Arc<StandardStreams>,
    #[singleton(deps = [site, standard_streams])]
    command_line: Arc<CommandLine>,
}

/// Runs ghg with `arguments`, those after the command's own name.
pub fn run(arguments: impl IntoIterator<Item = OsString>) -> ExitCode {
    let reply = GhgParts::new().get_command_line().run(arguments);
    // A closed pipe or a full disk leaves nothing more to say, so the status still stands.
    let _ = stdout().write_all(reply.stdout.as_bytes());
    let _ = stderr().write_all(reply.stderr.as_bytes());
    reply.exit_code()
}

/// Runs ghg as cargo runs the plugin, `cargo ghg`, with `arguments`, those after the plugin's own name.
pub fn run_cargo_plugin(arguments: impl IntoIterator<Item = OsString>) -> ExitCode {
    run(cargo_plugin_arguments(arguments))
}
