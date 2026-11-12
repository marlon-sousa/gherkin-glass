//! Entity: the revision of a file of the input tree, which the tree gives each time it takes in what parsing the file gave, never giving one twice.

/// Revisions are given in order, from the first, the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct Revision(u64);

impl Revision {
    /// The revision given after this one. Even a revision given every nanosecond would take
    /// centuries to reach the last.
    pub(crate) fn next(self) -> Revision {
        Revision(self.0.saturating_add(1))
    }
}
