//! Composition root: the `cargo-ghg` binary, the cargo plugin that cargo runs
//! as `cargo ghg`, which runs ghg as the `ghg` binary does.

mod composition_root;

use std::env::args_os;
use std::process::ExitCode;

use gherkin_glass::cargo_plugin_arguments;

fn main() -> ExitCode {
    composition_root::run(cargo_plugin_arguments(args_os().skip(1)))
}
