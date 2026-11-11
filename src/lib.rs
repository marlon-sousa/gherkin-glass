//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! docs/BEHAVIOUR.md says what it promises. The build runs in four steps, as
//! issue #3 sets out; [`InputService`] runs the first two, collecting and
//! parsing the input; [`make_anchors`] makes a feature's anchors, part of
//! the third; [`SiteService`] runs the whole build, writing each feature's
//! page; and [`CommandLine`] is the `ghg` command, configured by `ghg.toml`.

#![warn(unreachable_pub)]

mod adapters;
mod controllers;
mod entities;
mod policies;
mod ports;
mod services;

pub use adapters::{ConfigFileReader, GherkinParser, InputWalker, SiteFiles};
pub use controllers::{CommandLine, Reply};
pub use entities::{
    Anchors, CollectedInput, PageOptions, ParseFailure, ParsedFeature, Problem, ProblemKind,
    Problems, RawFeature, Settings, Unnamed, UnnamedBlock,
};
pub use policies::make_anchors;
pub use ports::{ConfigReader, FeatureParser, InputCollector, SiteWriter};
pub use services::{InputService, SiteService};
