//! Policy: how a build is run, the command line over the configuration file over the defaults.

use std::path::{Path, PathBuf};

use crate::entities::{
    Accents, BuildOptions, Config, Destination, PageOptions, SearchIndexing, Settings, SiteScope,
    Unnamed,
};

/// The input folder when neither the command line nor the file names one,
/// relative to the configuration file's folder.
pub(crate) const DEFAULT_INPUT: &str = "features";

/// The output folder when neither the command line nor the file names one,
/// relative to the configuration file's folder.
pub(crate) const DEFAULT_OUTPUT: &str = "site";

/// `folder` is the configuration file's folder, the current folder when the
/// file is `ghg.toml` there or there is none; paths from the command line are
/// kept as given, relative to the current folder.
pub(crate) fn resolve_settings(
    options: &BuildOptions,
    folder: &Path,
    config: &Config,
    destination: Destination,
) -> Settings {
    let path = |option: &Option<PathBuf>, set: &Option<PathBuf>, default: &str| match (option, set)
    {
        (Some(given), _) => given.clone(),
        (None, Some(set)) => folder.join(set),
        (None, None) => folder.join(default),
    };
    let input = path(&options.input, &config.input, DEFAULT_INPUT);
    let output = path(&options.output, &config.output, DEFAULT_OUTPUT);
    let unnamed = match config.allow_unnamed {
        Some(false) => Unnamed::Refused,
        Some(true) | None => Unnamed::Allowed,
    };
    let scope = match options.only {
        Some(_) => SiteScope::OnlyListed,
        None => SiteScope::EveryFeature,
    };
    let version = match options.indexing {
        SearchIndexing::Allowed => options
            .version_name
            .clone()
            .or_else(|| config.version_name.clone()),
        SearchIndexing::Refused => None,
    };
    Settings {
        input,
        output,
        destination,
        unnamed,
        version,
        replace: config.replace_existing_version.unwrap_or(true),
        front_page: options
            .front_page
            .clone()
            .or_else(|| config.front_page.as_ref().map(|set| folder.join(set))),
        only: options.only.clone(),
        keep_in_output: options
            .keep_in_output
            .clone()
            .or_else(|| config.keep_in_output.clone())
            .unwrap_or_default(),
        pages: PageOptions {
            title: options.title.clone().or_else(|| config.title.clone()),
            source_link: config.source_link.clone(),
            collapse_rules: config.collapse_rules.unwrap_or(false),
            collapse_scenarios: config.collapse_scenarios.unwrap_or(false),
            show_tree: config.show_tree.unwrap_or(true),
            inline_backgrounds: config.inline_backgrounds.unwrap_or(true),
            tag_links: config.tag_links.clone().unwrap_or_default(),
            accents: Accents {
                light: config.accent_light.unwrap_or(Accents::DEFAULT.light),
                dark: config.accent_dark.unwrap_or(Accents::DEFAULT.dark),
            },
            scope,
            indexing: options.indexing,
        },
    }
}

/// The folder a configuration file's relative paths start from: an empty
/// path, the current folder, for `ghg.toml`.
pub(crate) fn config_folder(config: &Path) -> PathBuf {
    config.parent().map(Path::to_path_buf).unwrap_or_default()
}

#[cfg(test)]
mod tests;
