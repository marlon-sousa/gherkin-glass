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
use crate::controllers::{SiteApi, SiteController};
use crate::handlers::CommandLine;
use crate::policies::cargo_plugin_arguments;
use crate::ports::{
    ChangeWatcher, FeatureParser, FileReader, FileWriter, InputFolders, OutputFolders,
    PreviewServer, ServeEventWaiter, StandardInput, StopSignals, Terminal,
};
use crate::services::{
    Converter, ConverterApi, GherkinParser, GherkinParserApi, InputCollector, InputCollectorApi,
    ServerApi, ServerService, SettingsApi, SettingsService, SiteWriter, SiteWriterApi,
};

#[container(leaf_build = default)]
struct GhgParts {
    #[singleton(implementation = FileSystemReader)]
    file_system_reader: Arc<dyn FileReader>,
    #[singleton(implementation = SettingsService, deps = [file_system_reader])]
    settings: Arc<dyn SettingsApi>,
    #[singleton(implementation = InputFolderReader)]
    input_folder_reader: Arc<dyn InputFolders>,
    #[singleton(implementation = StandardInputReader)]
    standard_input_reader: Arc<dyn StandardInput>,
    #[singleton(
        implementation = InputCollector,
        deps = [input_folder_reader, file_system_reader, standard_input_reader]
    )]
    input_collector: Arc<dyn InputCollectorApi>,
    #[singleton(implementation = GherkinCrate)]
    gherkin_crate: Arc<dyn FeatureParser>,
    #[singleton(implementation = GherkinParser, deps = [gherkin_crate])]
    gherkin_parser: Arc<dyn GherkinParserApi>,
    #[singleton(implementation = Converter)]
    converter: Arc<dyn ConverterApi>,
    #[singleton(implementation = OutputFolderStore)]
    output_folder_store: Arc<dyn OutputFolders>,
    #[singleton(implementation = FolderFiles)]
    folder_files: Arc<dyn FileWriter>,
    #[singleton(implementation = MemoryFiles)]
    memory_files: Arc<dyn FileWriter>,
    #[singleton(
        implementation = SiteWriter,
        deps = [file_reader = file_system_reader, folders = output_folder_store, folder_files, memory_files],
        build = builder
    )]
    site_writer: Arc<dyn SiteWriterApi>,
    #[singleton(implementation = HttpPreviewServer)]
    http_preview_server: Arc<dyn PreviewServer>,
    #[singleton(implementation = FileWatcher)]
    file_watcher: Arc<dyn ChangeWatcher>,
    #[singleton(implementation = CtrlcSignals)]
    ctrlc_signals: Arc<dyn StopSignals>,
    #[singleton(implementation = CrossbeamEventWaiter)]
    crossbeam_event_waiter: Arc<dyn ServeEventWaiter>,
    #[singleton(
        implementation = ServerService,
        deps = [http_preview_server, file_watcher, ctrlc_signals, crossbeam_event_waiter]
    )]
    server: Arc<dyn ServerApi>,
    #[singleton(
        implementation = SiteController,
        deps = [settings, input_collector, gherkin_parser, converter, site_writer, server]
    )]
    site: Arc<dyn SiteApi>,
    #[singleton(implementation = StandardStreams)]
    standard_streams: Arc<dyn Terminal>,
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
