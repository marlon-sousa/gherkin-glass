//! Composition root: the gherkin-glass command, which for now prints only its name and version.

fn main() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
