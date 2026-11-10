//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! docs/BEHAVIOUR.md says what it promises. The build runs in four steps, as
//! issue #3 sets out; [`InputService`] runs the first two, collecting and
//! parsing the input; [`make_anchors`] makes a feature's anchors,
//! [`find_step_sentences`] finds the suite's step sentences and
//! [`search_documents`] what search reads, all part of the third, as [`highlight_step`] splits a step into the parts its highlighting
//! tells apart; [`SiteService`] runs the whole build, writing each feature's
//! page and the documents of search; [`OutputService`] writes a build into its output folder, with or
//! without versions; [`PreviewService`] previews a site, built and served; and
//! [`CommandLine`] is the `ghg` command, configured by `ghg.toml`.

#![warn(unreachable_pub)]

mod adapters;
mod controllers;
mod entities;
mod policies;
mod ports;
mod services;

pub use adapters::{
    ConfigFileReader, FileWatcher, GherkinParser, HttpPreviewServer, InputWalker, MemorySite,
    OutputFolderStore, SiteFiles, StandardStreams,
};
pub use controllers::{CommandLine, Reply};
pub use entities::{
    Accents, Anchors, BuildFailure, BuildFolder, Burst, Changes, CollectedInput, Colour,
    FolderState, HighlightedStep, ListenFailure, Marker, OutputProblem, OutputProblemKind,
    PageOptions, ParseFailure, ParsedFeature, Preview, Problem, ProblemKind, Problems, RawFeature,
    Sentence, SentenceGroup, SentenceUse, ServedSite, ServerWords, Settings, StepKind, StepPart,
    StepSentences, TagLink, Unnamed, UnnamedBlock, Version, VersionList, VersionName, WatchFailure,
    Watched,
    Accents, Anchors, BuildFailure, BuildFolder, BuiltSite, Changes, CollectedInput, Colour,
    FolderState, HighlightedStep, ListenFailure, Marker, OutputProblem, OutputProblemKind,
    PageOptions, ParseFailure, ParsedFeature, Preview, Problem, ProblemKind, Problems, RawFeature,
    SearchDocument, SearchKind, Sentence, SentenceGroup, SentenceUse, ServedSite, ServerWords,
    Settings, StepKind, StepPart, StepSentences, TagLink, Unnamed, UnnamedBlock, Version,
    VersionList, VersionName, WatchFailure, Watched,
};
pub use policies::{
    DEFAULT_ACCENTS, find_step_sentences, highlight_step, make_anchors, search_documents,
    search_json,
};
pub use ports::{
    ChangeWatcher, ConfigReader, FeatureParser, InputCollector, OutputFolders, PreviewServer,
    SiteRecorder, SiteWriter, Terminal,
};
pub use services::{InputService, OutputService, PreviewService, SiteService};
