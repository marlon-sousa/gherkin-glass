//! Entity: the site ghg serve serves, the version of it being served, where it is served, and the words of its answers.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, PoisonError, RwLock};

use crate::entities::ServeMode;

/// A site to serve: a folder on disk, or one built in memory, its files by
/// their `/`-joined address below the site's root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServedSite {
    Folder(PathBuf),
    Memory(BTreeMap<String, Vec<u8>>),
}

/// One version of the site, numbered from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Version {
    pub(crate) number: u64,
    pub(crate) site: ServedSite,
}

/// What ghg serve serves, shared by every request: the version being
/// served, which a rebuild replaces whole, and how it serves.
#[derive(Debug)]
pub(crate) struct Preview {
    pub(crate) mode: ServeMode,
    current: RwLock<Arc<Version>>,
}

impl Preview {
    pub(crate) fn new(mode: ServeMode, site: ServedSite) -> Self {
        Preview {
            mode,
            current: RwLock::new(Arc::new(Version { number: 1, site })),
        }
    }

    /// A request keeps the version it started with, whatever replaces it meanwhile.
    pub(crate) fn current(&self) -> Arc<Version> {
        // A version is replaced whole, so a panic elsewhere never leaves one half made.
        self.current
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Serves `site` from now on, as the next version; its number.
    pub(crate) fn replace(&self, site: ServedSite) -> u64 {
        let mut current = self.current.write().unwrap_or_else(PoisonError::into_inner);
        let number = current.number + 1;
        *current = Arc::new(Version { number, site });
        number
    }
}

/// A site being served: the address it is served at, and what is served,
/// which a rebuild replaces.
#[derive(Debug)]
pub(crate) struct Served {
    pub(crate) address: SocketAddr,
    pub(crate) preview: Arc<Preview>,
}

/// The sentences ghg serve answers with when it has no file to serve,
/// from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServerWords {
    pub(crate) not_found: String,
    pub(crate) wrong_method: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_replaced_version_takes_the_next_number_and_an_old_one_stays_whole() {
        let preview = Preview::new(ServeMode::Watch, ServedSite::Memory(BTreeMap::new()));
        let first = preview.current();
        let site = ServedSite::Memory(BTreeMap::from([("a.html".into(), b"a".to_vec())]));
        assert_eq!(preview.replace(site.clone()), 2);
        assert_eq!(first.number, 1);
        assert_eq!(first.site, ServedSite::Memory(BTreeMap::new()));
        assert_eq!(*preview.current(), Version { number: 2, site });
    }
}
