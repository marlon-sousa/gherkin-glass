//! Policy: the versions page, from the names of the versions the output folder holds once the build is written.

use i18n_embed::LanguageLoader;
use i18n_embed_fl::fl;

use super::order_versions;
use crate::entities::{VersionEntry, VersionName, VersionsPage, VersionsWords};
use crate::policies::CATALOGUE;

/// The versions page listing `names`, every version the output folder holds.
pub(crate) fn build_versions_page(names: impl IntoIterator<Item = VersionName>) -> VersionsPage {
    let list = order_versions(names);
    VersionsPage {
        // The page holds interface text only, in the catalogue's language.
        language: CATALOGUE.current_language().to_string(),
        title: fl!(CATALOGUE, "versions-title"),
        words: VersionsWords {
            development: fl!(CATALOGUE, "versions-development"),
            releases: fl!(CATALOGUE, "versions-releases"),
            pre_releases: fl!(CATALOGUE, "versions-pre-releases"),
        },
        development: list
            .development
            .into_iter()
            .map(development_entry)
            .collect(),
        releases: list.releases.into_iter().map(numbered_entry).collect(),
        pre_releases: list.pre_releases.into_iter().map(numbered_entry).collect(),
    }
}

/// A development version's entry, whose words say it is one.
fn development_entry(name: VersionName) -> VersionEntry {
    VersionEntry {
        words: fl!(
            CATALOGUE,
            "versions-development-version",
            name = name.as_str()
        ),
        name,
    }
}

/// A release's or a pre-release's entry, whose words are its name.
fn numbered_entry(name: VersionName) -> VersionEntry {
    VersionEntry {
        words: name.to_string(),
        name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::name;

    #[test]
    fn each_version_is_listed_once_by_its_kind_with_its_words() {
        let page = build_versions_page([name("v2.0.0-rc.1"), name("main"), name("v1.0.0")]);
        let listed = |entries: &[VersionEntry]| -> Vec<(String, String)> {
            entries
                .iter()
                .map(|entry| (entry.name.to_string(), entry.words.clone()))
                .collect()
        };
        assert_eq!(
            listed(&page.development),
            [(
                "main".to_string(),
                "main, a development version".to_string()
            )]
        );
        assert_eq!(
            listed(&page.releases),
            [("v1.0.0".to_string(), "v1.0.0".to_string())]
        );
        assert_eq!(
            listed(&page.pre_releases),
            [("v2.0.0-rc.1".to_string(), "v2.0.0-rc.1".to_string())]
        );
        assert_eq!(page.language, "en");
    }
}
