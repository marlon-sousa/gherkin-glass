//! Policy: what a shell says of itself in its head and what the script finds in it, from what it is the shell of and the settings of every page.

use i18n_embed_fl::fl;

use crate::entities::{BuildTarget, PageOptions, ShellPage, VersionKind, VersionName};
use crate::policies::{CATALOGUE, site_title, version_kind};

/// The shell of `target`, linking the assets of the ghg `ghg`.
pub(crate) fn build_shell_page(
    target: &BuildTarget,
    options: &PageOptions,
    ghg: &str,
) -> ShellPage {
    let site = site_title(options);
    let (title, descriptor, assets) = match target {
        BuildTarget::Site => (site, "site.json".to_string(), format!("assets/{ghg}")),
        BuildTarget::Version(name) => (
            fl!(
                CATALOGUE,
                "version-shell-title",
                site = site,
                version = version_words(name)
            ),
            format!("versions/{name}.json"),
            format!("../assets/{ghg}"),
        ),
        BuildTarget::Preview(name) => (
            site,
            format!("previews/{name}.json"),
            format!("../../assets/{ghg}"),
        ),
    };
    ShellPage {
        description: fl!(CATALOGUE, "shell-description", title = title.as_str()),
        title,
        descriptor,
        assets,
        holds_front_page: !matches!(target, BuildTarget::Version(_)),
        unindexed: matches!(target, BuildTarget::Preview(_)),
    }
}

/// `name`, with what it is when it is a development version or a pre-release.
fn version_words(name: &VersionName) -> String {
    match version_kind(name) {
        VersionKind::Development => fl!(
            CATALOGUE,
            "versions-development-version",
            name = name.as_str()
        ),
        VersionKind::Release => name.to_string(),
        VersionKind::PreRelease => fl!(
            CATALOGUE,
            "versions-pre-release-version",
            name = name.as_str()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::name;

    // Rules 241 and 268: a version's title names it, and each shell links its descriptor and the assets.
    #[test]
    fn each_shell_names_its_descriptor_and_the_assets_by_its_depth() {
        let options = PageOptions {
            title: Some("Itinera conformance".to_string()),
            ..PageOptions::default()
        };
        let site = build_shell_page(&BuildTarget::Site, &options, "2.0.0");
        assert_eq!(
            (
                site.title.as_str(),
                site.descriptor.as_str(),
                site.assets.as_str()
            ),
            ("Itinera conformance", "site.json", "assets/2.0.0")
        );
        assert_eq!(
            site.description,
            "The feature files of Itinera conformance, shown as pages."
        );
        let version = build_shell_page(&BuildTarget::Version(name("v0.1.1")), &options, "2.0.0");
        assert_eq!(version.title, "Itinera conformance, v0.1.1");
        // Rule 268: a development version and a pre-release say what they are.
        let main = build_shell_page(&BuildTarget::Version(name("main")), &options, "2.0.0");
        assert_eq!(
            main.title,
            "Itinera conformance, main, a development version"
        );
        let candidate = build_shell_page(
            &BuildTarget::Version(name("v2.0.0-rc.1")),
            &options,
            "2.0.0",
        );
        assert_eq!(
            candidate.title,
            "Itinera conformance, v2.0.0-rc.1, a pre-release"
        );
        assert_eq!(
            (version.descriptor.as_str(), version.assets.as_str()),
            ("versions/v0.1.1.json", "../assets/2.0.0")
        );
        assert!(!version.holds_front_page && !version.unindexed);
        let preview = build_shell_page(&BuildTarget::Preview(name("pr-1")), &options, "2.0.0");
        assert_eq!(
            (preview.descriptor.as_str(), preview.assets.as_str()),
            ("previews/pr-1.json", "../../assets/2.0.0")
        );
        assert!(preview.holds_front_page && preview.unindexed);
    }
}
