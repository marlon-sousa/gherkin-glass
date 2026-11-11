//! Entity: the name of a version of the suite, always one folder name.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Names the versions page itself uses, compared ignoring capitals.
const RESERVED: [&str; 2] = ["assets", "index.html"];

/// The names Windows keeps for devices, alone or before a dot, compared ignoring capitals.
const DEVICES: [&str; 4] = ["con", "prn", "aux", "nul"];
const NUMBERED_DEVICES: [&str; 2] = ["com", "lpt"];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionName(String);

impl VersionName {
    /// `None` when `name` cannot be a version name, as docs/BEHAVIOUR.md, rule 121, says.
    pub fn new(name: &str) -> Option<Self> {
        let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+');
        let starts_well = name.starts_with(|c: char| c.is_ascii_alphanumeric());
        let reserved = RESERVED
            .iter()
            .any(|taken| name.eq_ignore_ascii_case(taken))
            || is_device(name);
        (starts_well && !name.ends_with('.') && name.chars().all(allowed) && !reserved)
            .then(|| VersionName(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for VersionName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(&self.0)
    }
}

fn is_device(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_lowercase();
    let numbered = |prefix: &str| {
        stem.strip_prefix(prefix)
            .is_some_and(|digit| matches!(digit.as_bytes(), [b'1'..=b'9']))
    };
    DEVICES.contains(&stem.as_str()) || NUMBERED_DEVICES.into_iter().any(numbered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_name_is_one_folder_name() {
        for name in ["main", "v1.2.0", "2.0.0-rc.1+build.5", "release_3", "A"] {
            assert_eq!(
                VersionName::new(name).map(|n| n.to_string()),
                Some(name.into())
            );
        }
        for name in [
            "",
            ".",
            "..",
            ".hidden",
            "-x",
            "_x",
            "+x",
            "v1.",
            "a/b",
            "a\\b",
            "a b",
            "ação",
            "v1:2",
            "assets",
            "ASSETS",
            "index.html",
            "Index.HTML",
            "con",
            "NUL",
            "nul.txt",
            "Aux.1.0",
            "com1",
            "LPT9",
            "prn",
        ] {
            assert_eq!(VersionName::new(name), None, "{name:?}");
        }
        for name in [
            "assets2",
            "console",
            "com0",
            "com10",
            "lpt",
            "nul-1",
            "connect.1",
        ] {
            assert!(VersionName::new(name).is_some(), "{name:?}");
        }
    }
}
