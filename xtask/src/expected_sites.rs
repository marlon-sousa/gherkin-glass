//! The expected sites `cargo xtask build` writes, with the version of ghg in every page's footer put back as the placeholder they commit, as docs/BEHAVIOUR.md, rule 195, describes.

use std::fs::{read_dir, read_to_string, write};
use std::path::Path;

#[path = "../../tests/support/version_placeholder.rs"]
mod version_placeholder;

use version_placeholder::with_version_placeholder;

/// Puts the placeholder in place of `version` in every page below `folder`.
pub(crate) fn put_back_placeholder(folder: &Path, version: &str) -> Result<(), String> {
    let entries = read_dir(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    entries.into_iter().try_for_each(|entry| {
        let path = entry
            .map_err(|error| format!("{}: {error}", folder.display()))?
            .path();
        if path.is_dir() {
            return put_back_placeholder(&path, version);
        }
        if path.extension().is_none_or(|extension| extension != "html") {
            return Ok(());
        }
        let page = read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let committed = with_version_placeholder(&page, version);
        if committed == page {
            return Ok(());
        }
        write(&path, committed).map_err(|error| format!("{}: {error}", path.display()))
    })
}
