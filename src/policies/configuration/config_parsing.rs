//! Policy: reading a configuration file's text into the values it sets.

use std::borrow::Cow;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::entities::{Colour, Config, VersionName};
use crate::errors::{ConfigError, ConfigErrorKind, ValueType};

mod accent_contrast;
mod config_key;
mod tag_link_tables;

use accent_contrast::check_accent;
use config_key::{ConfigKey, SettingKey, accent_of};
use tag_link_tables::tag_links;

/// The values `source` sets, or every error in it, sorted by line. A file
/// that is not TOML has one error, where the `toml` crate stopped.
pub(crate) fn parse_config(source: &str) -> Result<Config, Vec<ConfigError>> {
    let table = DeTable::parse(source).map_err(|error| {
        vec![ConfigError {
            line: error.span().map(|span| line_of(source, span.start)),
            kind: ConfigErrorKind::Syntax {
                message: error.message().to_string(),
            },
        }]
    })?;
    let mut config = Config::default();
    let mut errors: Vec<_> = table
        .get_ref()
        .iter()
        .flat_map(|(key, value)| take(&mut config, source, key, value))
        .collect();
    if errors.is_empty() {
        Ok(config)
    } else {
        errors.sort();
        Err(errors)
    }
}

/// Takes the value `value` gives `key` into `config`, or says what is wrong with them.
fn take(
    config: &mut Config,
    source: &str,
    key: &Spanned<Cow<'_, str>>,
    value: &Spanned<DeValue<'_>>,
) -> Vec<ConfigError> {
    let name = key.get_ref().as_ref();
    let Some(known) = ConfigKey::named(name) else {
        let kind = ConfigErrorKind::UnknownKey {
            key: name.to_string(),
        };
        return vec![error_at(source, key.span().start, kind)];
    };
    match known {
        ConfigKey::TagLinks => match tag_links(source, value) {
            Ok(links) => {
                config.tag_links = Some(links);
                Vec::new()
            }
            Err(found) => found,
        },
        ConfigKey::Setting(key) => set(config, key, name, value)
            .err()
            .map(|kind| error_at(source, value.span().start, kind))
            .into_iter()
            .collect(),
    }
}

/// Sets the value `key` names from `value`, or says what is wrong with it, naming the key as
/// written, `name`.
fn set(
    config: &mut Config,
    key: SettingKey,
    name: &str,
    value: &Spanned<DeValue<'_>>,
) -> Result<(), ConfigErrorKind> {
    let wrong_type = |expected| ConfigErrorKind::WrongType {
        key: name.to_string(),
        expected,
    };
    let empty = || ConfigErrorKind::Empty {
        key: name.to_string(),
    };
    match key {
        SettingKey::Text(key) => match text(value) {
            // An empty path names nothing, and would report a line with no path.
            Some("") => Err(empty()),
            Some(text) => {
                key.set(config, text);
                Ok(())
            }
            None => Err(wrong_type(ValueType::Text)),
        },
        SettingKey::VersionName => match text(value) {
            Some("") => Err(empty()),
            Some(text) => {
                config.version_name = Some(VersionName::new(text).ok_or_else(|| {
                    ConfigErrorKind::InvalidVersionName {
                        name: text.to_string(),
                    }
                })?);
                Ok(())
            }
            None => Err(wrong_type(ValueType::Text)),
        },
        SettingKey::Flag(key) => {
            *key.of(config) = Some(
                value
                    .get_ref()
                    .as_bool()
                    .ok_or_else(|| wrong_type(ValueType::Boolean))?,
            );
            Ok(())
        }
        SettingKey::Accent(theme) => {
            let colour = text(value)
                .and_then(Colour::from_hex)
                .ok_or_else(|| wrong_type(ValueType::Colour))?;
            if let Some(shortfall) = check_accent(theme, colour) {
                return Err(ConfigErrorKind::TooLittleContrast {
                    key: name.to_string(),
                    shortfall,
                });
            }
            *accent_of(config, theme) = Some(colour);
            Ok(())
        }
    }
}

fn text<'a>(value: &'a Spanned<DeValue<'_>>) -> Option<&'a str> {
    value.get_ref().as_str()
}

/// The error `kind`, on the line of `offset` in `source`.
fn error_at(source: &str, offset: usize, kind: ConfigErrorKind) -> ConfigError {
    ConfigError {
        line: Some(line_of(source, offset)),
        kind,
    }
}

/// Counted from 1; an offset past the end is on the last line.
fn line_of(source: &str, offset: usize) -> usize {
    let before = source.get(..offset).unwrap_or(source);
    before.matches('\n').count() + 1
}

#[cfg(test)]
mod tests;
