//! Adapter: the site recorder port, in memory, for the versions `ghg serve --watch` serves.

use std::collections::BTreeMap;
use std::io::Result as IoResult;
use std::mem::take;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use crate::ports::{SiteRecorder, SiteWriter};

#[derive(Debug, Default)]
pub(crate) struct MemorySite {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl SiteRecorder for MemorySite {
    fn take(&self) -> BTreeMap<String, Vec<u8>> {
        take(&mut *self.files.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

impl SiteWriter for MemorySite {
    /// `output` is left aside: nothing is written to disk.
    fn write(&self, _output: &Path, file: &str, contents: &[u8]) -> IoResult<()> {
        self.files
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(file.to_string(), contents.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taking_the_files_empties_the_site() {
        let site = MemorySite::default();
        site.write(Path::new("site"), "a/b.html", b"b")
            .expect("written");
        site.write(Path::new("site"), "a/b.html", b"c")
            .expect("written");
        assert_eq!(
            site.take(),
            BTreeMap::from([("a/b.html".to_string(), b"c".to_vec())])
        );
        assert!(site.take().is_empty());
    }
}
