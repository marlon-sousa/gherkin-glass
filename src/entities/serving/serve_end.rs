//! Entity: how a ghg serve that nothing stopped ended, once it was asked to stop, or with nothing to build and so nothing served.

use crate::entities::NothingToBuild;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServeEnd {
    /// It served until it was asked to stop, or until it was asked while its first build ran.
    NoLongerServed,
    /// Its first build, into the output folder, read no feature file, so nothing was served.
    NothingToBuild(NothingToBuild),
}
