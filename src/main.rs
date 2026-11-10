//! gherkin-glass turns a folder of Gherkin feature files into a static website.
//!
//! For now the command only prints its name and version. Its command line is
//! agreed in issue #11 and written in docs/BEHAVIOUR.md before it is built.

fn main() {
    println!("ghg {}", env!("CARGO_PKG_VERSION"));
}
