//! Service: reading the input, the first two steps of the build that issue #3 sets out.
//!
//! First the files to read are collected, then they are parsed in parallel.
//! A problem found while collecting does not stop the files already found
//! from being parsed, so that one run reports everything.

mod collect;
mod parse;

use std::path::Path;
use std::sync::Arc;

use crate::entities::{ParsedFeature, Problems};
use crate::ports::{FeatureParser, InputFolder};

pub struct InputService {
    folder: Arc<dyn InputFolder>,
    parser: Arc<dyn FeatureParser>,
}

impl InputService {
    pub fn new(folder: Arc<dyn InputFolder>, parser: Arc<dyn FeatureParser>) -> Self {
        InputService { folder, parser }
    }

    /// The features under `input`, sorted by path, or every problem found.
    pub fn read(&self, input: &Path) -> Result<Vec<ParsedFeature>, Problems> {
        let (files, mut problems) = collect::collect(self.folder.as_ref(), input);
        let features = parse::parse(
            self.folder.as_ref(),
            self.parser.as_ref(),
            input,
            &files,
            &mut problems,
        );
        if problems.is_empty() {
            Ok(features)
        } else {
            Err(Problems::new(problems))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::{Error, ErrorKind, Result as IoResult};
    use std::path::PathBuf;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{EntryKind, FolderEntry, ParseFailure};

    /// An input held in memory, whose listed paths can also fail.
    #[derive(Default)]
    struct FakeFolder {
        entries: BTreeMap<PathBuf, Vec<FolderEntry>>,
        texts: BTreeMap<PathBuf, String>,
    }

    impl FakeFolder {
        fn folder(mut self, path: &str, files: &[&str], folders: &[&str]) -> Self {
            let path = PathBuf::from(path);
            let entry = |name: &&str, kind| FolderEntry {
                path: path.join(name),
                kind,
            };
            let mut listing: Vec<_> = files
                .iter()
                .map(|name| entry(name, EntryKind::File))
                .collect();
            listing.extend(folders.iter().map(|name| entry(name, EntryKind::Folder)));
            self.entries.insert(path, listing);
            self
        }

        fn text(mut self, path: &str, text: &str) -> Self {
            self.texts.insert(PathBuf::from(path), text.to_string());
            self
        }
    }

    impl InputFolder for FakeFolder {
        fn entries(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
            self.entries
                .get(folder)
                .cloned()
                .ok_or_else(|| Error::new(ErrorKind::PermissionDenied, "permission denied"))
        }

        fn read_text(&self, path: &Path) -> IoResult<String> {
            self.texts
                .get(path)
                .cloned()
                .ok_or_else(|| Error::new(ErrorKind::PermissionDenied, "permission denied"))
        }
    }

    /// Parses the fake folder's text, or fails as a file that cannot be opened.
    struct FakeParser(Arc<FakeFolder>);

    impl FeatureParser for FakeParser {
        fn parse(&self, path: &Path) -> Result<Feature, ParseFailure> {
            let text = self.0.read_text(path).map_err(|error| ParseFailure {
                line: None,
                message: format!("cannot be read: {error}"),
            })?;
            Feature::parse(text, GherkinEnv::default()).map_err(|_| ParseFailure {
                line: None,
                message: "Syntax error.".to_string(),
            })
        }
    }

    fn service(folder: FakeFolder) -> InputService {
        let folder = Arc::new(folder);
        InputService::new(folder.clone(), Arc::new(FakeParser(folder)))
    }

    #[test]
    fn a_folder_that_cannot_be_listed_is_reported_and_the_rest_is_read() {
        // docs/BEHAVIOUR.md, rule 14.
        let folder = FakeFolder::default()
            .folder("in", &["a.feature"], &["locked"])
            .text("in/a.feature", "Feature: A\n");
        let problems = service(folder)
            .read(Path::new("in"))
            .expect_err("a problem");
        assert_eq!(
            problems.to_string(),
            "locked: cannot be listed: permission denied\n"
        );
    }

    #[test]
    fn a_feature_file_that_cannot_be_opened_is_reported() {
        // docs/BEHAVIOUR.md, rules 14 and 16.
        let folder = FakeFolder::default().folder("in", &["a.feature"], &[]);
        let problems = service(folder)
            .read(Path::new("in"))
            .expect_err("a problem");
        assert_eq!(
            problems.to_string(),
            "a.feature: cannot be read: permission denied\n"
        );
    }

    #[test]
    fn a_ghg_ignore_file_that_cannot_be_read_is_reported() {
        // docs/BEHAVIOUR.md, rule 14.
        let folder = FakeFolder::default()
            .folder("in", &[".ghg-ignore", "a.feature"], &[])
            .text("in/a.feature", "Feature: A\n");
        let problems = service(folder)
            .read(Path::new("in"))
            .expect_err("a problem");
        assert_eq!(
            problems.to_string(),
            ".ghg-ignore: cannot be read: permission denied\n"
        );
    }
}
