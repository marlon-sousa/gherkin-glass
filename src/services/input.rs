//! Service: reading the input, the collect and parse steps of a build.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{ParsedFeature, Problem, ProblemKind, Problems, TAGS_FOLDER, Unnamed};
use crate::policies::{
    FOLDER_PAGE, SENTENCES_PAGE, correct_doc_strings, correct_step_keywords, file_language,
    page_address, unnamed_blocks,
};
use crate::ports::{FeatureParser, InputCollector};

#[derive(Clone)]
pub(crate) struct InputService {
    collector: Arc<dyn InputCollector>,
    parser: Arc<dyn FeatureParser>,
}

impl InputService {
    pub(crate) fn new(collector: Arc<dyn InputCollector>, parser: Arc<dyn FeatureParser>) -> Self {
        InputService { collector, parser }
    }

    /// The paths of the files the `.ghg-ignore` rules select under
    /// `input`, sorted, without reading them.
    pub(crate) fn paths(&self, input: &Path) -> Vec<PathBuf> {
        self.collector.collect(input).paths
    }

    /// The features under `input`, sorted by path, or every problem found:
    /// those met while collecting, the files whose page another has, those
    /// of every file parsed, and, when `unnamed` is refused, every block
    /// that needs a name and has none.
    pub(crate) fn read(
        &self,
        input: &Path,
        unnamed: Unnamed,
    ) -> Result<Vec<ParsedFeature>, Problems> {
        let collected = self.collector.collect(input);
        let mut problems = collected.problems;
        problems.extend(same_pages(input, &collected.paths));
        problems.extend(pages_in_tags_folder(input, &collected.paths));
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        let results: Vec<_> = collected
            .paths
            .par_iter()
            .map(|path| self.parse(input, path, unnamed))
            .collect();
        let mut features = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Ok(feature) => features.push(feature),
                Err(found) => problems.extend(found),
            }
        }
        if problems.is_empty() {
            Ok(features)
        } else {
            Err(Problems::new(problems))
        }
    }

    fn parse(
        &self,
        input: &Path,
        path: &Path,
        unnamed: Unnamed,
    ) -> Result<ParsedFeature, Vec<Problem>> {
        let problem = |line, kind| Problem::new(input, path, line, kind);
        let mut raw = self
            .parser
            .parse(path)
            .map_err(|failure| vec![problem(failure.line, failure.kind)])?;
        // Only a file changed between the parse and the read of its source fails here.
        let media_types = correct_doc_strings(&mut raw.feature, &raw.source)
            .ok_or_else(|| vec![problem(None, ProblemKind::Changed)])?;
        if unnamed == Unnamed::Refused {
            let found: Vec<_> = unnamed_blocks(&raw.feature)
                .into_iter()
                .map(|(line, block)| problem(Some(line), ProblemKind::Unnamed { block }))
                .collect();
            if !found.is_empty() {
                return Err(found);
            }
        }
        let relative = path.strip_prefix(input).unwrap_or(path).to_path_buf();
        correct_step_keywords(&mut raw.feature, &raw.source);
        let language = file_language(&raw.source);
        Ok(ParsedFeature::new(
            relative,
            raw.feature,
            language,
            media_types,
        ))
    }
}

/// Every file whose page, compared without regard to case, since some file
/// systems do not, is a folder's page or the page of step sentences, or one
/// an earlier file in path order already has; and every folder whose page an
/// earlier folder has.
fn same_pages(input: &Path, paths: &[PathBuf]) -> Vec<Problem> {
    let mut first: BTreeMap<String, &Path> = BTreeMap::new();
    let mut folders: BTreeMap<String, String> = BTreeMap::new();
    let mut problems = Vec::new();
    for path in paths {
        let relative = path.strip_prefix(input).unwrap_or(path);
        let page = page_address(relative).to_string();
        let parts: Vec<&str> = page.split('/').collect();
        for depth in 1..parts.len() {
            let folder = parts.get(..depth).unwrap_or_default().join("/");
            match folders.get(&folder.to_lowercase()) {
                Some(earlier) if *earlier != folder => {
                    let problem = Problem::new(
                        input,
                        &input.join(&folder),
                        None,
                        ProblemKind::SameFolderPage {
                            page: format!("{folder}/{FOLDER_PAGE}"),
                            first: earlier.clone(),
                        },
                    );
                    if !problems.contains(&problem) {
                        problems.push(problem);
                    }
                }
                Some(_) => {}
                None => {
                    folders.insert(folder.to_lowercase(), folder);
                }
            }
        }
        let (folder, file) = match page.rsplit_once('/') {
            Some((folder, file)) => (Some(folder.to_string()), file),
            None => (None, page.as_str()),
        };
        if page.to_lowercase() == SENTENCES_PAGE {
            problems.push(Problem::new(
                input,
                path,
                None,
                ProblemKind::ReservedPage {
                    page: SENTENCES_PAGE.to_string(),
                },
            ));
            continue;
        }
        if file.to_lowercase() == FOLDER_PAGE {
            problems.push(Problem::new(
                input,
                path,
                None,
                ProblemKind::SameAsFolderPage {
                    page: page.clone(),
                    folder,
                },
            ));
            continue;
        }
        match first.get(&page.to_lowercase()) {
            Some(earlier) => problems.push(Problem::new(
                input,
                path,
                None,
                ProblemKind::SamePage {
                    page,
                    first: Problem::shown_path(input, earlier),
                },
            )),
            None => {
                first.insert(page.to_lowercase(), path);
            }
        }
    }
    problems
}

/// Every file whose page would be in the folder of the tag pages, its name
/// compared without regard to case, as `same_pages` compares.
fn pages_in_tags_folder(input: &Path, paths: &[PathBuf]) -> Vec<Problem> {
    paths
        .iter()
        .filter_map(|path| {
            let page = page_address(path.strip_prefix(input).unwrap_or(path)).to_string();
            let (folder, _) = page.split_once('/')?;
            (folder.to_lowercase() == TAGS_FOLDER)
                .then(|| Problem::new(input, path, None, ProblemKind::InTagsFolder { page }))
        })
        .collect()
}

#[cfg(test)]
mod tests;
