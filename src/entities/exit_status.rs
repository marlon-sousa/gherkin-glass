//! Entity: the status a run of ghg exits with.

/// What became of a run, as the status it exits with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ExitStatus {
    /// It did what was asked.
    Done = 0,
    /// The configuration, the input or the output folder stopped it.
    Stopped = 1,
    /// The command line was mistaken.
    Mistaken = 2,
}
