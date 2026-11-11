//! Entity: why ghg serve cannot listen.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenFailure {
    PortTaken,
    /// The reason is the operating system's words.
    Other {
        reason: String,
    },
}
