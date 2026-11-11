//! Port: collecting the paths of the feature files under the input folder.

use std::path::Path;

use crate::entities::CollectedInput;

pub trait InputCollector: Send + Sync {
    fn collect(&self, input: &Path) -> CollectedInput;
}
