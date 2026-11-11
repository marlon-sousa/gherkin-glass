//! Policy: whether a build may write into a folder, from what is at its place, what it is to hold, and when it is looked at.

use crate::entities::{FolderState, Marker};
use crate::errors::OutputErrorKind;

/// When a folder is looked at: when it is checked, before the input is read, or again just
/// before it is emptied, once the input has been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moment {
    /// `replace` says whether a build already there may be replaced.
    Checking {
        replace: bool,
    },
    Emptying,
}

/// Nothing when the folder, in `state`, may hold what `marker` marks, or what stops it.
pub(crate) fn folder_verdict(
    state: FolderState,
    marker: &Marker,
    moment: Moment,
) -> Result<(), OutputErrorKind> {
    match (state, marker, moment) {
        (FolderState::Missing | FolderState::Empty, _, _)
        | (FolderState::Marked, _, Moment::Checking { replace: true } | Moment::Emptying) => Ok(()),
        (FolderState::Marked, _, Moment::Checking { replace: false }) => {
            Err(OutputErrorKind::AlreadyBuilt)
        }
        (FolderState::Other, Marker::Site, _)
        | (FolderState::OtherCapitals, Marker::Site, Moment::Checking { .. }) => {
            Err(OutputErrorKind::NotASite)
        }
        (FolderState::Other, Marker::Version(_), _) => Err(OutputErrorKind::NotAVersion),
        (FolderState::OtherCapitals, _, _) => Err(OutputErrorKind::OtherCapitals),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::entities::VersionName;

    fn version() -> Marker {
        Marker::Version(VersionName::new("v1").expect("a version name"))
    }

    const KEEP: Moment = Moment::Checking { replace: false };
    const REPLACE: Moment = Moment::Checking { replace: true };
    const EMPTYING: Moment = Moment::Emptying;

    #[rstest]
    fn a_missing_or_empty_folder_may_always_be_written(
        #[values(FolderState::Missing, FolderState::Empty)] state: FolderState,
        #[values(Marker::Site, version())] marker: Marker,
        #[values(KEEP, REPLACE, EMPTYING)] moment: Moment,
    ) {
        assert_eq!(folder_verdict(state, &marker, moment), Ok(()));
    }

    #[rstest]
    #[case::kept(KEEP, Err(OutputErrorKind::AlreadyBuilt))]
    #[case::replaced(REPLACE, Ok(()))]
    #[case::emptied(EMPTYING, Ok(()))]
    fn a_folder_ghg_built_may_be_written_unless_it_is_kept(
        #[values(Marker::Site, version())] marker: Marker,
        #[case] moment: Moment,
        #[case] verdict: Result<(), OutputErrorKind>,
    ) {
        assert_eq!(
            folder_verdict(FolderState::Marked, &marker, moment),
            verdict
        );
    }

    #[rstest]
    #[case::a_site_s(Marker::Site, OutputErrorKind::NotASite)]
    #[case::a_version_s(version(), OutputErrorKind::NotAVersion)]
    fn a_folder_holding_something_else_is_not_what_it_is_to_hold(
        #[case] marker: Marker,
        #[case] error: OutputErrorKind,
        #[values(KEEP, REPLACE, EMPTYING)] moment: Moment,
    ) {
        assert_eq!(
            folder_verdict(FolderState::Other, &marker, moment),
            Err(error)
        );
    }

    #[rstest]
    #[case::a_site_s_when_checked(Marker::Site, KEEP, OutputErrorKind::NotASite)]
    #[case::a_site_s_when_checked_to_replace(Marker::Site, REPLACE, OutputErrorKind::NotASite)]
    #[case::a_site_s_when_emptied(Marker::Site, EMPTYING, OutputErrorKind::OtherCapitals)]
    #[case::a_version_s_when_checked(version(), KEEP, OutputErrorKind::OtherCapitals)]
    #[case::a_version_s_when_checked_to_replace(version(), REPLACE, OutputErrorKind::OtherCapitals)]
    #[case::a_version_s_when_emptied(version(), EMPTYING, OutputErrorKind::OtherCapitals)]
    fn a_folder_holding_other_capitals_is_reported_as_such_unless_a_site_s_is_checked(
        #[case] marker: Marker,
        #[case] moment: Moment,
        #[case] error: OutputErrorKind,
    ) {
        assert_eq!(
            folder_verdict(FolderState::OtherCapitals, &marker, moment),
            Err(error)
        );
    }
}
