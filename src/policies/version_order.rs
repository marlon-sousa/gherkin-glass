//! Policy: the order of the versions page, development versions, then releases, then pre-releases.

use std::cmp::Ordering;

use semver::Version;

use crate::entities::{VersionList, VersionName};

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

/// The Semantic Versioning 2.0.0 version a name is, alone or after a `v`.
fn version_number(name: &VersionName) -> Option<Version> {
    let text = name.as_str();
    let number = text.strip_prefix('v').unwrap_or(text);
    Version::parse(number).ok()
}

fn newest_first(mut versions: Vec<(Version, VersionName)>) -> Vec<VersionName> {
    versions.sort_by(|(a, a_name), (b, b_name)| match b.cmp_precedence(a) {
        Ordering::Equal => a_name.cmp(b_name),
        newer_first => newer_first,
    });
    versions.into_iter().map(|(_, name)| name).collect()
}

#[cfg(test)]
mod tests {
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
}
