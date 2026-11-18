// Facade: gherkin-glass turns a folder of Gherkin feature files into a static website.
//
// The library exists so that the two binaries, `ghg` and the cargo plugin `cargo-ghg`, share
// their code, and offers nothing else: gherkin-glass is used through the `ghg` command.
#![doc = include_str!("../README.md")]

mod adapters;
mod composition_root;
mod controllers;
mod entities;
mod errors;
mod handlers;
mod policies;
mod ports;
mod services;
#[cfg(test)]
mod test_support;

pub use composition_root::{run, run_cargo_plugin};
