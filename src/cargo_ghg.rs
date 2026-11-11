//! Binary: `cargo-ghg`, the cargo plugin that cargo runs as `cargo ghg`, which hands the
//! process's arguments to the library's entry point for the plugin.

use std::env::args_os;
use std::process::ExitCode;

use gherkin_glass::run_cargo_plugin;

fn main() -> ExitCode {
    run_cargo_plugin(args_os().skip(1))
}
