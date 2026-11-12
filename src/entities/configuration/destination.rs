//! Entity: where a build writes its site, the output folder or memory.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Destination {
    /// The output folder, once it is checked as safe to write.
    #[default]
    Folder,
    /// Memory, for `ghg serve --watch`: the output folder is neither checked
    /// nor written, and a version name changes nothing.
    Memory,
}
