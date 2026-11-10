//! Composition root: the `ghg` binary, which runs ghg with its arguments.

mod composition_root;

use std::env::args_os;
use std::process::ExitCode;

fn main() -> ExitCode {
    composition_root::run(args_os().skip(1))
}
