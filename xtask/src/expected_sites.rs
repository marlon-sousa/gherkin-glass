//! Adapter: the expected sites `cargo xtask build` writes, with the version of ghg in every page's footer and every file of the store put back as the placeholder they commit, and the files of the store indented.

use std::fs::{copy, create_dir_all, read, read_dir, remove_dir_all, write};
use std::io::ErrorKind;
use std::path::Path;

#[path = "../../tests/support/store_json.rs"]
mod store_json;
#[path = "../../tests/support/version_placeholder.rs"]
mod version_placeholder;

use store_json::indented;
use version_placeholder::with_version_placeholder;

/// Puts the placeholder in place of `version` in every page below `folder`, and in every file
/// of a store, which it indents.
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
        } else if is_in_store(&path) {
            put_back_in_store_file(&path, version)?;
        }
    }
    Ok(())
}

/// Whether `path` is a file of JSON in a folder `data` or below one.
fn is_in_store(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "json")
        && path.components().any(|part| part.as_os_str() == "data")
}

/// Writes the file of the store at `path` again, with the placeholder in place of `version`,
/// indented.
fn put_back_in_store_file(path: &Path, version: &str) -> Result<(), String> {
    let file = read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let text = String::from_utf8(with_version_placeholder(file, version))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    write(path, indented(&text)).map_err(|error| format!("{}: {error}", path.display()))
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

/// Removes `folder`, with all in it, then copies `from` in its place, when it is given.
pub(crate) fn replace_folder(folder: &Path, from: Option<&Path>) -> Result<(), String> {
    match remove_dir_all(folder) {
        Err(error) if error.kind() != ErrorKind::NotFound => {
            return Err(format!("{}: {error}", folder.display()));
        }
        _ => {}
    }
    from.map_or(Ok(()), |from| copy_folder(from, folder))
}

fn copy_folder(from: &Path, to: &Path) -> Result<(), String> {
    create_dir_all(to).map_err(|error| format!("{}: {error}", to.display()))?;
    let entries = read_dir(from).map_err(|error| format!("{}: {error}", from.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("{}: {error}", from.display()))?
            .path();
        let target = to.join(path.file_name().unwrap_or_default());
        if path.is_dir() {
            copy_folder(&path, &target)?;
        } else {
            copy(&path, &target).map_err(|error| format!("{}: {error}", path.display()))?;
        }
    }
    Ok(())
}
