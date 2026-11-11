//! Entity: what the command line asks for.

use std::path::PathBuf;

use crate::entities::VersionName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Invocation {
    Help,
    Version,
    Build(BuildOptions),
    Serve(ServeOptions),
}

/// The options of `ghg build`, each `None` when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BuildOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) input: Option<PathBuf>,
    pub(crate) version_name: Option<VersionName>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) title: Option<String>,
}

/// The options of `ghg serve`: those of `ghg build`, and how it serves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ServeOptions {
    pub(crate) build: BuildOptions,
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) mode: ServeMode,
}

/// What `ghg serve` serves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ServeMode {
    /// The output folder as it is, under `--no-build`.
    Folder,
    /// The site built into the output folder first.
    #[default]
    Build,
    /// The site built in memory and built again on every change, under `--watch`.
    Watch,
}

impl ServeMode {
    /// Whether the site is built again on every change.
    pub(crate) fn watches(self) -> bool {
        self == ServeMode::Watch
    }
}

/// An option of `ghg build` that sets how a site is built, every one but
/// `--help` and `--version`. The command line reads every one of them through
/// [`BuildOption::ALL`], and the GitHub Action has an input for each, which
/// the tests below check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildOption {
    Config,
    Input,
    Output,
    Title,
    VersionName,
}

impl BuildOption {
    /// Every option: one missing here cannot be given.
    pub(crate) const ALL: [BuildOption; 5] = [
        BuildOption::Config,
        BuildOption::Input,
        BuildOption::Output,
        BuildOption::Title,
        BuildOption::VersionName,
    ];

    /// The option as it is typed, with its dashes.
    pub(crate) fn option(self) -> &'static str {
        match self {
            BuildOption::Config => "--config",
            BuildOption::Input => "--input",
            BuildOption::Output => "--output",
            BuildOption::Title => "--title",
            BuildOption::VersionName => "--version-name",
        }
    }

    /// The option's name, without its dashes, which is the action's input.
    pub(crate) fn name(self) -> &'static str {
        &self.option()[2..]
    }

    pub(crate) fn named(name: &str) -> Option<BuildOption> {
        BuildOption::ALL
            .into_iter()
            .find(|option| option.name() == name)
    }
}

/// A mistake on the command line. Each `String` is what was typed, shown
/// as it was typed, an option with its dashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArgumentProblem {
    MissingCommand,
    UnknownCommand {
        name: String,
    },
    /// `command` is `ghg`, `ghg build` or `ghg serve`.
    UnknownOption {
        command: String,
        option: String,
    },
    MissingValue {
        option: String,
    },
    UnexpectedValue {
        option: String,
    },
    RepeatedOption {
        option: String,
    },
    UnexpectedArgument {
        command: String,
        argument: String,
    },
    NotAPort {
        value: String,
    },
    WatchWithoutBuild,
    InvalidVersionName {
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use yaml_rust2::{Yaml, YamlLoader};

    use super::*;

    const ACTION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/action.yml"));

    /// The inputs of the action that are not options of `ghg build`.
    const OWN_INPUTS: [&str; 2] = ["version", "build"];

    fn action() -> Yaml {
        YamlLoader::load_from_str(ACTION)
            .expect("action.yml is YAML")
            .remove(0)
    }

    fn options() -> BTreeSet<String> {
        BuildOption::ALL
            .iter()
            .map(|option| option.name().to_string())
            .collect()
    }

    // docs/BEHAVIOUR.md, "The GitHub Action", rule 176.
    #[test]
    fn every_option_of_ghg_build_is_an_input_of_the_action_and_nothing_else_is() {
        let action = action();
        let mut inputs: BTreeSet<String> = action["inputs"]
            .as_hash()
            .expect("action.yml has inputs")
            .keys()
            .map(|key| key.as_str().expect("an input's name").to_string())
            .collect();
        for own in OWN_INPUTS {
            assert!(inputs.remove(own), "action.yml has the input {own}");
        }
        assert_eq!(inputs, options());
    }

    // docs/BEHAVIOUR.md, "The GitHub Action", rule 176.
    #[test]
    fn the_build_step_passes_every_option_input_and_nothing_else() {
        let action = action();
        let steps = action["runs"]["steps"]
            .as_vec()
            .expect("action.yml has steps");
        let build = steps
            .iter()
            .find(|step| {
                step["run"]
                    .as_str()
                    .is_some_and(|run| run.contains("build.sh"))
            })
            .expect("a step runs build.sh");
        let passed: BTreeSet<(String, String)> = build["env"]
            .as_hash()
            .expect("the build step has an environment")
            .iter()
            .filter_map(|(key, value)| {
                let key = key.as_str()?;
                Some((key.to_string(), value.as_str()?.to_string()))
            })
            .filter(|(key, _)| key.starts_with("GHG_OPTION_"))
            .collect();
        let named: BTreeSet<String> = build["env"]["GHG_OPTIONS"]
            .as_str()
            .expect("the build step names its options")
            .split_whitespace()
            .map(str::to_string)
            .collect();
        assert_eq!(named, options());
        let expected: BTreeSet<(String, String)> = options()
            .into_iter()
            .map(|name| {
                (
                    format!("GHG_OPTION_{}", name.to_uppercase().replace('-', "_")),
                    format!("${{{{ inputs.{name} }}}}"),
                )
            })
            .collect();
        assert_eq!(passed, expected);
    }

    #[test]
    fn the_help_names_every_option() {
        let help = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/messages/en/gherkin-glass.ftl"
        ));
        for option in BuildOption::ALL {
            let named = format!("  --{} <", option.name());
            assert!(help.contains(&named), "the help names --{}", option.name());
        }
    }

    #[test]
    fn every_option_is_found_by_its_name() {
        for option in BuildOption::ALL {
            assert_eq!(BuildOption::named(option.name()), Some(option));
        }
        assert_eq!(BuildOption::named("help"), None);
    }
}
