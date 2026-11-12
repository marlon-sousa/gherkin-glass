//! Service: the changes that call for a rebuild under `--watch`, read one at a time as from a channel.

use std::iter::once;
use std::path::{Path, PathBuf};

use crate::entities::{Burst, Changes};
use crate::policies::{Read, calls_for_rebuild, remembered};

/// The changes that call for a rebuild, as `ServerApi::watch` hands them back.
pub(super) struct Rebuilds<'a> {
    changes: Changes,
    config: PathBuf,
    input: PathBuf,
    selected: &'a dyn Fn(&Path) -> Vec<PathBuf>,
    /// The files the site served was built from, as `remembered` keeps them.
    read: Vec<PathBuf>,
}

impl<'a> Rebuilds<'a> {
    pub(super) fn new(
        changes: Changes,
        input: &Path,
        config: &Path,
        selected: &'a dyn Fn(&Path) -> Vec<PathBuf>,
    ) -> Self {
        Rebuilds {
            changes,
            config: config.to_path_buf(),
            input: input.to_path_buf(),
            read: selected(input),
            selected,
        }
    }

    fn calls_for_rebuild(&mut self, bursts: &[Burst]) -> bool {
        let after = (self.selected)(&self.input);
        let read = Read {
            config: &self.config,
            before: &self.read,
            after: &after,
        };
        let rebuild = bursts.iter().any(|burst| calls_for_rebuild(burst, &read));
        self.read = remembered(&self.read, after, rebuild);
        rebuild
    }
}

impl Iterator for Rebuilds<'_> {
    type Item = Vec<Burst>;

    fn next(&mut self) -> Option<Vec<Burst>> {
        loop {
            let burst = self.changes.bursts.recv().ok()?;
            // Bursts that came during the last rebuild need only one more.
            let bursts: Vec<_> = once(burst).chain(self.changes.bursts.try_iter()).collect();
            if self.calls_for_rebuild(&bursts) {
                return Some(bursts);
            }
        }
    }
}
