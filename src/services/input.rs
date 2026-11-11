//! Service: reading the input, the collect and parse steps of a build.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::iter::Either;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{ParsedFeature, Problem, ProblemKind, Problems, Unnamed};
use crate::policies::{
    correct_doc_strings, correct_step_keywords, file_language, page_collisions, unnamed_blocks,
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
        problems.extend(page_collisions(input, &collected.paths));
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        let (failed, features): (Vec<_>, Vec<_>) = collected
            .paths
            .par_iter()
            .map(|path| self.parse(input, path, unnamed))
            .partition_map(Either::from);
        problems.extend(failed.into_iter().flatten());
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

#[cfg(test)]
mod tests;
