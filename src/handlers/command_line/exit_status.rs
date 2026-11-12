//! Handler: the status a run of ghg exits with, a closed set private to the command line.

/// What became of a run, as the status it exits with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ExitStatus {
    /// It did what was asked.
    Done = 0,
    /// Something stopped it before it was done, such as the configuration,
    /// the input, the output folder, a port in use or a watch that cannot start.
    Stopped = 1,
    /// The command line was mistaken.
    Mistaken = 2,
}
