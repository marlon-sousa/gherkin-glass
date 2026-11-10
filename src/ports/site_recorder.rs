//! Port: a site writer that keeps what it is given, for a site served from memory.

use std::collections::BTreeMap;

use crate::ports::SiteWriter;

pub trait SiteRecorder: SiteWriter {
    /// Every file written since the last call, by its `/`-joined address,
    /// which starts the next site empty.
    fn take(&self) -> BTreeMap<String, Vec<u8>>;
}
