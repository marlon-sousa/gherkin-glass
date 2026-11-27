//! Policy: what a shell says of itself in its head and what the script finds in it, from what it is the shell of and the settings of every page.

use i18n_embed_fl::fl;

use crate::entities::{BuildTarget, PageOptions, ShellPage};
use crate::policies::{CATALOGUE, site_title};

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
                version = name.as_str()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::name;

    // Rule 241: a version's title names it, and each shell links its descriptor and the assets.
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
