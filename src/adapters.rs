//! Adapter: the ports' implementations that touch the world.

mod gherkin_parser;
mod input_walker;

pub use gherkin_parser::GherkinParser;
pub use input_walker::InputWalker;
