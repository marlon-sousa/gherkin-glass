//! Binary: `ghg`, which hands the process's arguments to the library's entry point.

use std::env::args_os;
use std::process::ExitCode;

use gherkin_glass::run;

fn main() -> ExitCode {
    run(args_os().skip(1))
}
