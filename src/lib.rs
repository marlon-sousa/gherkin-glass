//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! This library is not a stable API. It exists so that the two binaries, `ghg` and the cargo
//! plugin `cargo-ghg`, share their code: use gherkin-glass through the `ghg` command.

#![warn(unreachable_pub)]

mod adapters;
mod composition_root;
mod controllers;
mod entities;
#[cfg(test)]
mod fixtures;
mod policies;
mod ports;
mod services;

pub use adapters::{
    ConfigFileReader, FileWatcher, GherkinParser, HttpPreviewServer, InputWalker, MemorySite,
    OutputFolderStore, SiteFiles, StandardStreams,
};
pub use controllers::{CommandLine, Reply};
pub use entities::{
    Accents, Anchors, BuildFailure, BuildFolder, BuiltSite, Burst, Changes, CollectedInput, Colour,
    FolderState, HighlightedStep, ListenFailure, Marker, OutputProblem, OutputProblemKind,
    PageOptions, ParseFailure, ParsedFeature, Preview, Problem, ProblemKind, Problems, RawFeature,
    SearchDocument, SearchKind, Sentence, SentenceGroup, SentenceUse, ServedSite, ServerWords,
    Settings, StepKind, StepPart, StepSentences, TagLink, Unnamed, UnnamedBlock, Version,
    VersionList, VersionName, WatchFailure, Watched,
};
pub use policies::{
    DEFAULT_ACCENTS, cargo_plugin_arguments, find_step_sentences, highlight_step, make_anchors,
    search_documents, search_json,
};
pub use ports::{
    ChangeWatcher, ConfigReader, FeatureParser, InputCollector, OutputFolders, PreviewServer,
    SiteRecorder, SiteWriter, Terminal,
};
pub use services::{InputService, OutputService, PreviewService, SiteService};
pub use composition_root::{run, run_cargo_plugin};
