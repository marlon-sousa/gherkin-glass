//! Entity: what parsing a file the input tree handed out gave, which only such a file can be given back with.

use super::FileToParse;
use crate::entities::ParsedFeature;
use crate::errors::InputError;

#[derive(Debug)]
pub(crate) struct ParsedFile {
    pub(crate) file: FileToParse,
    pub(crate) result: Result<ParsedFeature, Vec<InputError>>,
}
