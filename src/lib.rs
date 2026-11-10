//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! docs/BEHAVIOUR.md says what it promises. The build runs in four steps, as
//! issue #3 sets out; [`InputService`] runs the first two, collecting and
//! parsing the input, and
//! [`CommandLine`] is the `ghg` command, configured by `ghg.toml`.

#![warn(unreachable_pub)]

mod adapters;
mod controllers;
mod entities;
mod policies;
mod ports;
mod services;

pub use adapters::{ConfigFileReader, GherkinParser, InputWalker};
pub use controllers::{CommandLine, Reply};
pub use entities::{
    CollectedInput, ParseFailure, ParsedFeature, Problem, ProblemKind, Problems, RawFeature,
};
pub use ports::{ConfigReader, FeatureParser, InputCollector};
pub use services::InputService;
