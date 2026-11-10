//! Entity: what is at the place of an output folder, or of a version's folder.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderState {
    Missing,
    /// A folder with nothing in it.
    Empty,
    /// A folder holding the marker asked about.
    Marked,
    /// A version's folder listed under the same name in other capitals, on
    /// a file system that does not tell them apart.
    OtherCapitals,
    /// A file, a link, or a folder with something in it and not the marker.
    Other,
}
