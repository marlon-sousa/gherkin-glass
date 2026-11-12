//! Service: the gherkin parser, walking the input and parsing the feature files it selects.

mod input_walk;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use input_walk::walk_input;
use rayon::iter::Either;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::entities::{ParsedFeature, Unnamed};
use crate::errors::{InputError, InputErrorKind, InputErrors};
use crate::policies::{
    correct_doc_strings, correct_step_keywords, file_language, page_collisions, unnamed_blocks,
};
use crate::ports::{FeatureParser, InputFolders};

/// What the controller asks of the gherkin parser.
pub(crate) trait ParserApi: Send + Sync {
    /// The paths of the files the `.ghg-ignore` rules select under
    /// `input`, sorted, without reading them.
    fn paths(&self, input: &Path) -> Vec<PathBuf>;

    /// The features under `input`, sorted by path, or every error found:
    /// those met while collecting, the files whose page another has, those
    /// of every file parsed, and, when `unnamed` is refused, every block
    /// that needs a name and has none.
    fn read(&self, input: &Path, unnamed: Unnamed) -> Result<Vec<ParsedFeature>, InputErrors>;
}

#[derive(Clone)]
pub(crate) struct InputService {
    folders: Arc<dyn InputFolders>,
    parser: Arc<dyn FeatureParser>,
}

impl InputService {
    pub(crate) fn new(folders: Arc<dyn InputFolders>, parser: Arc<dyn FeatureParser>) -> Self {
        InputService { folders, parser }
    }

    fn parse(
        &self,
        input: &Path,
        path: &Path,
        unnamed: Unnamed,
    ) -> Result<ParsedFeature, Vec<InputError>> {
        let input_error = |line, kind| InputError::new(input, path, line, kind);
        let mut raw = self
            .parser
            .parse(path)
            .map_err(|error| vec![input_error(error.line, error.kind)])?;
        // Only a file changed between the parse and the read of its source fails here.
        let media_types = correct_doc_strings(&mut raw.feature, &raw.source)
            .ok_or_else(|| vec![input_error(None, InputErrorKind::Changed)])?;
        if unnamed == Unnamed::Refused {
            let found: Vec<_> = unnamed_blocks(&raw.feature)
                .into_iter()
                .map(|(line, block)| input_error(Some(line), InputErrorKind::Unnamed { block }))
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

impl ParserApi for InputService {
    fn paths(&self, input: &Path) -> Vec<PathBuf> {
        walk_input(self.folders.as_ref(), input).paths
    }

    fn read(&self, input: &Path, unnamed: Unnamed) -> Result<Vec<ParsedFeature>, InputErrors> {
        let collected = walk_input(self.folders.as_ref(), input);
        let mut errors = collected.errors;
        errors.extend(page_collisions(input, &collected.paths));
        // Collecting into a `Vec` keeps file order, whatever order the threads finish in.
        let (failed, features): (Vec<_>, Vec<_>) = collected
            .paths
            .par_iter()
            .map(|path| self.parse(input, path, unnamed))
            .partition_map(Either::from);
        errors.extend(failed.into_iter().flatten());
        if errors.is_empty() {
            Ok(features)
        } else {
            Err(InputErrors::new(errors))
        }
    }
}

#[cfg(test)]
mod tests;
