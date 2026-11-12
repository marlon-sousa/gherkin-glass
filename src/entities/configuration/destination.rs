//! Entity: where a build writes its site, the output folder or memory.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Destination {
    Folder,
    /// For `ghg serve --watch`.
    Memory,
}
