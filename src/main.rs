//! Composition root: the ghg command, which for now prints only its name and version.

fn main() {
    println!("ghg {}", env!("CARGO_PKG_VERSION"));
}
