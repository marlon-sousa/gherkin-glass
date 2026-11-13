//! Policy: the keys a configuration file can set, and the value of the configuration each one sets.

use std::path::PathBuf;

use crate::entities::{Colour, Config, Theme};

/// A key the configuration knows: `tag_links`, whose value is a list of tables, or a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConfigKey {
    TagLinks,
    Setting(SettingKey),
}

/// A key whose value is one value, by the kind of value it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingKey {
    Text(TextKey),
    VersionName,
    Flag(FlagKey),
    Accent(Theme),
}

/// A key whose value is text that names something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextKey {
    Input,
    Output,
    SourceLink,
    Title,
    FrontPage,
}

/// A key whose value is true or false.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FlagKey {
    AllowUnnamed,
    CollapseRules,
    CollapseScenarios,
    ShowTree,
    InlineBackgrounds,
    ReplaceExistingVersion,
}

impl ConfigKey {
    /// The key written `name`, or `None` for one the configuration does not know.
    pub(super) fn named(name: &str) -> Option<ConfigKey> {
        Some(ConfigKey::Setting(match name {
            "tag_links" => return Some(ConfigKey::TagLinks),
            "input" => SettingKey::Text(TextKey::Input),
            "output" => SettingKey::Text(TextKey::Output),
            "source_link" => SettingKey::Text(TextKey::SourceLink),
            "title" => SettingKey::Text(TextKey::Title),
            "front_page" => SettingKey::Text(TextKey::FrontPage),
            "version_name" => SettingKey::VersionName,
            "allow_unnamed" => SettingKey::Flag(FlagKey::AllowUnnamed),
            "collapse_rules" => SettingKey::Flag(FlagKey::CollapseRules),
            "collapse_scenarios" => SettingKey::Flag(FlagKey::CollapseScenarios),
            "show_tree" => SettingKey::Flag(FlagKey::ShowTree),
            "inline_backgrounds" => SettingKey::Flag(FlagKey::InlineBackgrounds),
            "replace_existing_version" => SettingKey::Flag(FlagKey::ReplaceExistingVersion),
            "accent_light" => SettingKey::Accent(Theme::Light),
            "accent_dark" => SettingKey::Accent(Theme::Dark),
            _ => return None,
        }))
    }
}

impl TextKey {
    /// Sets the value of `config` this key names to `text`.
    pub(super) fn set(self, config: &mut Config, text: &str) {
        match self {
            TextKey::Input => config.input = Some(PathBuf::from(text)),
            TextKey::Output => config.output = Some(PathBuf::from(text)),
            TextKey::SourceLink => config.source_link = Some(text.to_string()),
            TextKey::Title => config.title = Some(text.to_string()),
            TextKey::FrontPage => config.front_page = Some(PathBuf::from(text)),
        }
    }
}

impl FlagKey {
    /// The value of `config` this key sets.
    pub(super) fn of(self, config: &mut Config) -> &mut Option<bool> {
        match self {
            FlagKey::AllowUnnamed => &mut config.allow_unnamed,
            FlagKey::CollapseRules => &mut config.collapse_rules,
            FlagKey::CollapseScenarios => &mut config.collapse_scenarios,
            FlagKey::ShowTree => &mut config.show_tree,
            FlagKey::InlineBackgrounds => &mut config.inline_backgrounds,
            FlagKey::ReplaceExistingVersion => &mut config.replace_existing_version,
        }
    }
}

/// The accent colour of `theme` that `config` sets.
pub(super) fn accent_of(config: &mut Config, theme: Theme) -> &mut Option<Colour> {
    match theme {
        Theme::Light => &mut config.accent_light,
        Theme::Dark => &mut config.accent_dark,
    }
}
