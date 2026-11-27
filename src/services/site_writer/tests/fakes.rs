//! The site writer's fakes: an output folder held in memory, which it looks into and writes, and a front page's template.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Result as IoResult};
use std::path::PathBuf;
use std::sync::Mutex;

use super::*;
use crate::entities::{OutputContents, PageOptions, ParsedFeature, Unnamed};
use crate::policies::build_store;
use crate::test_support::{name, parsed_at};

/// An output folder held in memory, missing until a file is written into it. A path in `fails`
/// cannot be written or removed, and the folder cannot be looked into while `unreadable`. Once `changes` is
/// set, the next look after the first finds that file there too, as though written meanwhile.
#[derive(Default)]
pub(super) struct MemoryOutput {
    pub(super) files: Mutex<BTreeMap<String, Arc<[u8]>>>,
    pub(super) fails: Option<&'static str>,
    pub(super) unreadable: bool,
    pub(super) changes: Option<&'static str>,
    pub(super) looks: Mutex<usize>,
}

impl MemoryOutput {
    /// The paths of the files it holds.
    pub(super) fn paths(&self) -> Vec<String> {
        self.files.lock().expect("lock").keys().cloned().collect()
    }

    /// The text of the file at `path`, if it holds one.
    pub(super) fn text(&self, path: &str) -> Option<String> {
        let files = self.files.lock().expect("lock");
        files
            .get(path)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    }
}

impl OutputFolders for MemoryOutput {
    fn contents(&self, _: &Path, _: &KeptInOutput) -> IoResult<OutputContents> {
        if self.unreadable {
            return Err(IoError::new(ErrorKind::PermissionDenied, "denied"));
        }
        let mut looks = self.looks.lock().expect("lock");
        *looks += 1;
        let mut files: BTreeSet<String> = self.paths().into_iter().collect();
        if *looks > 1 {
            files.extend(self.changes.map(str::to_string));
        }
        if files.is_empty() {
            return Ok(OutputContents::Missing);
        }
        let folders = files
            .iter()
            .flat_map(|file| {
                file.match_indices('/')
                    .map(|(end, _)| file[..end].to_string())
            })
            .collect();
        Ok(OutputContents::Folder {
            marker: self.text("ghg-site.json"),
            files,
            folders,
            kept: BTreeMap::new(),
        })
    }

    fn read(&self, _: &Path, file: &str) -> IoResult<Option<String>> {
        Ok(self.text(file))
    }

    fn remove(&self, _: &Path, path: &str) -> IoResult<()> {
        if self.fails == Some(path) {
            return Err(IoError::other("disk full"));
        }
        let folder = format!("{path}/");
        self.files
            .lock()
            .expect("lock")
            .retain(|file, _| file != path && !file.starts_with(&folder));
        Ok(())
    }
}

impl FileWriter for MemoryOutput {
    fn write(&self, _: &Path, file: &str, contents: &Arc<[u8]>) -> IoResult<()> {
        if self.fails == Some(file) {
            return Err(IoError::other("disk full"));
        }
        self.files
            .lock()
            .expect("lock")
            .insert(file.to_string(), Arc::clone(contents));
        Ok(())
    }

    fn site(&self, output: &Path) -> ServedSite {
        ServedSite::Folder(output.to_path_buf())
    }
}

/// A front page's template, `text`, whatever its path; none can be read when it is `None`.
#[derive(Default)]
pub(super) struct FakeTemplate {
    pub(super) text: Option<&'static str>,
}

impl FileReader for FakeTemplate {
    fn read(&self, _: &Path) -> IoResult<String> {
        self.text
            .map(str::to_string)
            .ok_or_else(|| IoError::from(ErrorKind::NotFound))
    }
}

/// A site writer into `output`, with `memory` for a site in memory, reading `template`.
pub(super) fn site_writer(
    output: &Arc<MemoryOutput>,
    memory: Arc<dyn FileWriter>,
    template: FakeTemplate,
) -> SiteWriter {
    SiteWriter::builder()
        .file_reader(Arc::new(template))
        .folders(output.clone())
        .folder_files(output.clone())
        .memory_files(memory)
        .build()
}

/// The settings of a build into `site` of `target`.
pub(super) fn settings(destination: Destination, target: BuildTarget) -> Settings {
    Settings {
        input: PathBuf::from("in"),
        output: PathBuf::from("site"),
        destination,
        unnamed: Unnamed::Allowed,
        pages: PageOptions::default(),
        target,
        replace: true,
        front_page: None,
        only: None,
        keep_in_output: KeptInOutput::default(),
    }
}

pub(super) fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(name(text))
}

pub(super) fn preview(text: &str) -> BuildTarget {
    BuildTarget::Preview(name(text))
}

/// The store of one feature, `a`, whose step is `step`.
pub(super) fn store(step: &str) -> SiteStore {
    let parsed: ParsedFeature = parsed_at(
        "a.feature",
        &format!("Feature: A\n  Scenario: S\n    Given {step}\n"),
    );
    build_store(&[&parsed], &PageOptions::default())
}

/// Checks the output of `site_writer` for `target`, then writes into it the store of `step`.
pub(super) fn add(
    site_writer: &SiteWriter,
    target: BuildTarget,
    step: &str,
) -> Result<(), BuildError> {
    let settings = settings(Destination::Folder, target);
    site_writer.check(&settings)?;
    site_writer.write(&settings, &store(step)).map(drop)
}

/// Removes `target` from the output of `site_writer`.
pub(super) fn remove(site_writer: &SiteWriter, target: BuildTarget) -> Result<bool, BuildError> {
    let settings = settings(Destination::Folder, target);
    site_writer.remove(&settings)
}
