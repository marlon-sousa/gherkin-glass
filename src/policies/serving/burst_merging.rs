//! Policy: the bursts of changes waiting under `--watch`, merged into one.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::entities::{Burst, InputChanges};

/// Everything, when a burst may have missed changes or names one of `settings_files`, the
/// configuration file and the front page's template, and otherwise every path the bursts named.
pub(crate) fn merged_bursts(bursts: &[Burst], settings_files: &[PathBuf]) -> InputChanges {
    let named: Option<Vec<&[PathBuf]>> = bursts.iter().map(changed_paths).collect();
    let Some(named) = named else {
        return InputChanges::Everything;
    };
    let paths: BTreeSet<PathBuf> = named.concat().into_iter().collect();
    if settings_files.iter().any(|file| paths.contains(file)) {
        return InputChanges::Everything;
    }
    InputChanges::Paths(paths)
}

/// The paths `burst` changed, or nothing when changes may have been missed.
fn changed_paths(burst: &Burst) -> Option<&[PathBuf]> {
    match burst {
        Burst::Changed(paths) => Some(paths),
        Burst::Missed => None,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_support::changed;

    fn paths(paths: &[&str]) -> InputChanges {
        InputChanges::Paths(paths.iter().map(PathBuf::from).collect())
    }

    #[rstest]
    // Rule 65: an editor's burst of changes, and those made during a rebuild, are one rebuild,
    // and a change to the configuration file, or one the watching may have missed, reads
    // everything again.
    #[case::one_burst(vec![changed("features/a.feature")], paths(&["features/a.feature"]))]
    #[case::bursts_waiting_joined(
        vec![changed("features/b.feature"), changed("features/a.feature"), changed("features/b.feature")],
        paths(&["features/a.feature", "features/b.feature"])
    )]
    #[case::the_configuration_file(
        vec![changed("features/a.feature"), changed("ghg.toml")],
        InputChanges::Everything
    )]
    // Rule 191: a change to the front page's template reads everything again too.
    #[case::the_front_page_s_template(
        vec![changed("features/a.feature"), changed("docs/front.md")],
        InputChanges::Everything
    )]
    #[case::changes_missed(
        vec![changed("features/a.feature"), Burst::Missed],
        InputChanges::Everything
    )]
    #[case::a_file_named_like_the_configuration(
        vec![changed("features/ghg.toml"), changed("ghg.toml~")],
        paths(&["features/ghg.toml", "ghg.toml~"])
    )]
    fn the_bursts_waiting_are_merged_into_one(
        #[case] bursts: Vec<Burst>,
        #[case] merged: InputChanges,
    ) {
        let settings_files = [PathBuf::from("ghg.toml"), PathBuf::from("docs/front.md")];
        assert_eq!(merged_bursts(&bursts, &settings_files), merged);
    }
}
