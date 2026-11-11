//! Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! This library is not a stable API. It exists so that the two binaries, `ghg` and the cargo
//! plugin `cargo-ghg`, share their code: use gherkin-glass through the `ghg` command.

mod adapters;
mod composition_root;
mod controllers;
mod entities;
mod errors;
#[cfg(test)]
mod fixtures;
mod policies;
mod ports;
mod services;

pub use composition_root::{run, run_cargo_plugin};
