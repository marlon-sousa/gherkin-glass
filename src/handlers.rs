//! Handler: the ways into gherkin-glass, each turning what came in into a call to the controller.

mod command_line;

pub(crate) use command_line::CommandLine;
