//! Adapter: the expected sites `cargo xtask build` writes, with the version of ghg in every page's footer put back as the placeholder they commit.

use std::fs::{read, read_dir, write};
use std::path::Path;

#[path = "../../tests/support/version_placeholder.rs"]
mod version_placeholder;

use version_placeholder::with_version_placeholder;

/// Puts the placeholder in place of `version` in every page below `folder`.
pub(crate) fn put_back_placeholder(folder: &Path, version: &str) -> Result<(), String> {
    let entries = read_dir(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("{}: {error}", folder.display()))?
            .path();
        if path.is_dir() {
            put_back_placeholder(&path, version)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "html")
        {
            put_back_in_page(&path, version)?;
        }
    }
    Ok(())
}

/// Writes the page at `path` again only when its footer names `version`.
fn put_back_in_page(path: &Path, version: &str) -> Result<(), String> {
    let page = read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let committed = with_version_placeholder(page.clone(), version);
    if committed == page {
        return Ok(());
    }
    write(path, committed).map_err(|error| format!("{}: {error}", path.display()))
}
