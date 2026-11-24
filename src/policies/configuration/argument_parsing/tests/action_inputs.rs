//! The action's inputs and the help, checked against the options of `ghg build`, the action's
//! inputs against those that set how a site is built.

use std::collections::BTreeSet;

use yaml_rust2::{Yaml, YamlLoader};

use super::{building_options, flags_of, long_options, repeatable_of};

const ACTION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/action.yml"));

/// The inputs of the action that are not options of `ghg build`.
const OWN_INPUTS: [&str; 2] = ["version", "build"];

fn action() -> Yaml {
    YamlLoader::load_from_str(ACTION)
        .expect("action.yml is YAML")
        .remove(0)
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
    assert_eq!(inputs, building_options());
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
    assert_eq!(named, building_options());
    let expected: BTreeSet<(String, String)> = building_options()
        .into_iter()
        .map(|name| {
            (
                format!("GHG_OPTION_{}", name.to_uppercase().replace('-', "_")),
                format!("${{{{ inputs.{name} }}}}"),
            )
        })
        .collect();
    assert_eq!(passed, expected);
    let flags: BTreeSet<String> = build["env"]["GHG_FLAGS"]
        .as_str()
        .expect("the build step names its flags")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    assert_eq!(flags, flags_of("build"));
    let lists: BTreeSet<String> = build["env"]["GHG_LISTS"]
        .as_str()
        .expect("the build step names its lists")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    assert_eq!(lists, repeatable_of("build"));
}

#[test]
fn the_help_names_every_option() {
    let help = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/messages/en/gherkin-glass.ftl"
    ));
    for option in long_options("build") {
        let with_a_value = format!("  --{option} <");
        let without = format!("  --{option}:");
        assert!(
            help.contains(&with_a_value) || help.contains(&without),
            "the help names --{option}"
        );
    }
}
