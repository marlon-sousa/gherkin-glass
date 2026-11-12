//! Adapter: the file writer port in memory, keeping the site `ghg serve --watch` serves.

use std::collections::BTreeMap;
use std::io::Result as IoResult;
use std::mem::take;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use crate::entities::ServedSite;
use crate::ports::FileWriter;

/// `output` is left aside: nothing is written to disk.
#[derive(Debug, Default)]
pub(crate) struct MemoryFiles {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl FileWriter for MemoryFiles {
    fn write(&self, _output: &Path, file: &str, contents: &[u8]) -> IoResult<()> {
        self.files
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(file.to_string(), contents.to_vec());
        Ok(())
    }

    fn site(&self, _output: &Path) -> ServedSite {
        ServedSite::Memory(take(
            &mut *self.files.lock().unwrap_or_else(PoisonError::into_inner),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taking_the_site_empties_it() {
        let files = MemoryFiles::default();
        let output = Path::new("site");
        files.write(output, "a/b.html", b"b").expect("written");
        files.write(output, "a/b.html", b"c").expect("written");
        assert_eq!(
            files.site(output),
            ServedSite::Memory(BTreeMap::from([("a/b.html".to_string(), b"c".to_vec())]))
        );
        assert_eq!(files.site(output), ServedSite::Memory(BTreeMap::new()));
    }
}
