//! Policy: the kind of version a name makes, the order of the versions page, development versions, then releases, then pre-releases, and every version number grouped by its precedence, newest first, which the store's list of versions holds.

use std::cmp::Ordering;

use semver::Version;

use crate::entities::{VersionKind, VersionList, VersionName};

/// Sorts `names` into development versions, by name, and releases and pre-releases,
/// newest first by Semantic Versioning's precedence, then by name. Names compare by
/// their bytes, which for the characters a version name may hold is by their
/// character codes.
pub(crate) fn order_versions(names: impl IntoIterator<Item = VersionName>) -> VersionList {
    let mut list = VersionList::default();
    let mut releases = Vec::new();
    let mut pre_releases = Vec::new();
    for name in names {
        match version_number(&name) {
            None => list.development.push(name),
            Some(number) if number.pre.is_empty() => releases.push((number, name)),
            Some(number) => pre_releases.push((number, name)),
        }
    }
    list.development.sort();
    list.releases = newest_first(releases);
    list.pre_releases = newest_first(pre_releases);
    list
}

/// What `name` makes a version, as the versions page groups it.
pub(crate) fn version_kind(name: &VersionName) -> VersionKind {
    match version_number(name) {
        None => VersionKind::Development,
        Some(number) if number.pre.is_empty() => VersionKind::Release,
        Some(_) => VersionKind::PreRelease,
    }
}

/// The Semantic Versioning 2.0.0 version a name is, alone or after a `v`.
fn version_number(name: &VersionName) -> Option<Version> {
    let text = name.as_str();
    let number = text.strip_prefix('v').unwrap_or(text);
    Version::parse(number).ok()
}

/// The version numbers among `names`, releases and pre-releases together, newest first by
/// Semantic Versioning's precedence, those of the same precedence in one group, by name.
pub(crate) fn precedence_groups(
    names: impl IntoIterator<Item = VersionName>,
) -> Vec<Vec<VersionName>> {
    let mut numbered: Vec<(Version, VersionName)> =
        names.into_iter().filter_map(numbered).collect();
    numbered.sort_by(newer_then_by_name);
    numbered.chunk_by(same_precedence).map(names_of).collect()
}

fn numbered(name: VersionName) -> Option<(Version, VersionName)> {
    Some((version_number(&name)?, name))
}

fn same_precedence((a, _): &(Version, VersionName), (b, _): &(Version, VersionName)) -> bool {
    a.cmp_precedence(b) == Ordering::Equal
}

fn names_of(group: &[(Version, VersionName)]) -> Vec<VersionName> {
    group.iter().map(|(_, name)| name.clone()).collect()
}

fn newer_then_by_name(
    (a, a_name): &(Version, VersionName),
    (b, b_name): &(Version, VersionName),
) -> Ordering {
    b.cmp_precedence(a).then_with(|| a_name.cmp(b_name))
}

fn newest_first(mut versions: Vec<(Version, VersionName)>) -> Vec<VersionName> {
    versions.sort_by(newer_then_by_name);
    versions.into_iter().map(|(_, name)| name).collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn order(names: &[&str]) -> (Vec<String>, Vec<String>, Vec<String>) {
        let names = names
            .iter()
            .map(|name| VersionName::new(name).expect("a version name"));
        let list = order_versions(names);
        let text = |names: Vec<VersionName>| names.iter().map(|n| n.to_string()).collect();
        (
            text(list.development),
            text(list.releases),
            text(list.pre_releases),
        )
    }

    #[test]
    fn development_versions_then_releases_then_pre_releases() {
        let (development, releases, pre_releases) = order(&[
            "v1.2.0",
            "main",
            "2.0.0-rc.1",
            "v1.10.0",
            "Next",
            "v2.0.0-beta.2",
            "1.9.3",
            "2.0.0-beta.11",
        ]);
        assert_eq!(development, ["Next", "main"]);
        assert_eq!(releases, ["v1.10.0", "1.9.3", "v1.2.0"]);
        assert_eq!(
            pre_releases,
            ["2.0.0-rc.1", "2.0.0-beta.11", "v2.0.0-beta.2"]
        );
    }

    #[test]
    fn only_a_full_semantic_version_is_a_version_number() {
        let (development, releases, _) = order(&[
            "1.2",
            "V1.2.0",
            "v01.2.0",
            "1.2.0.1",
            "vv1.0.0",
            "1.0.0",
            "18446744073709551616.0.0",
        ]);
        assert_eq!(
            development,
            [
                "1.2",
                "1.2.0.1",
                "18446744073709551616.0.0",
                "V1.2.0",
                "v01.2.0",
                "vv1.0.0"
            ]
        );
        assert_eq!(releases, ["1.0.0"]);
    }

    #[test]
    fn names_of_the_same_precedence_are_sorted_by_name() {
        let (_, releases, _) = order(&["v1.0.0", "1.0.0+b", "1.0.0+a", "1.0.0", "0.9.0"]);
        assert_eq!(releases, ["1.0.0", "1.0.0+a", "1.0.0+b", "v1.0.0", "0.9.0"]);
    }

    // Rule 268: a version's kind is told from its name alone, as the versions page groups it.
    #[rstest]
    #[case::a_name_that_is_not_a_version_number("main", VersionKind::Development)]
    #[case::a_version_number_after_a_v("v1.0.0", VersionKind::Release)]
    #[case::a_version_number_with_build_metadata("1.0.0+b", VersionKind::Release)]
    #[case::a_version_number_with_a_pre_release_part("v2.0.0-rc.1", VersionKind::PreRelease)]
    fn a_version_s_kind_is_told_from_its_name(#[case] name: &str, #[case] kind: VersionKind) {
        assert_eq!(
            version_kind(&VersionName::new(name).expect("a version name")),
            kind
        );
    }

    // Rule 253: `precedence` holds every version number, newest first, those of the same
    // precedence together.
    #[test]
    fn version_numbers_are_grouped_by_precedence_newest_first() {
        let names = ["main", "v1.2.0", "2.0.0-rc.1", "1.2.0", "2.0.0", "1.2.0+b"]
            .iter()
            .map(|name| VersionName::new(name).expect("a version name"));
        let groups: Vec<Vec<String>> = precedence_groups(names)
            .iter()
            .map(|group| group.iter().map(ToString::to_string).collect())
            .collect();
        assert_eq!(
            groups,
            [
                vec!["2.0.0"],
                vec!["2.0.0-rc.1"],
                vec!["1.2.0", "1.2.0+b", "v1.2.0"]
            ]
        );
    }
}
