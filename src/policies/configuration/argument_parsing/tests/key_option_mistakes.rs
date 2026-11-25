//! The mistakes of rule 234, in the values of the options of rules 232 and 233.

use rstest::rstest;

use super::*;
use crate::entities::{Accents, Colour, Theme};
use crate::errors::AccentShortfall;
use crate::policies::palette;

fn missing_value(option: &str) -> ArgumentError {
    ArgumentError::MissingValue {
        option: option.into(),
    }
}

fn not_boolean(option: &str) -> ArgumentError {
    ArgumentError::NotBoolean {
        option: option.into(),
    }
}

fn tag_link_option() -> String {
    "--tag-link".into()
}

fn light_shortfall(accent: Colour) -> ArgumentError {
    ArgumentError::TooLittleContrast {
        option: "--accent-light".into(),
        shortfall: AccentShortfall {
            theme: Theme::Light,
            accent,
            background: palette(Theme::Light).surface,
            hundredths: 400,
        },
    }
}

// Rules 19 and 234.
#[rstest]
#[case::an_empty_source_link(&["build", "--source-link="], missing_value("--source-link"))]
#[case::no_source_link(&["build", "--source-link"], missing_value("--source-link"))]
#[case::yes(&["build", "--collapse-rules", "yes"], not_boolean("--collapse-rules"))]
#[case::true_in_capitals(&["serve", "--show-tree=True"], not_boolean("--show-tree"))]
#[case::an_empty_flag(&["build", "--allow-unnamed="], missing_value("--allow-unnamed"))]
#[case::a_flag_without_its_value(&["build", "--inline-backgrounds"], missing_value("--inline-backgrounds"))]
#[case::a_flag_before_another_option(
    &["build", "--replace-existing-version", "--title", "x"],
    missing_value("--replace-existing-version")
)]
#[case::a_repeated_flag(
    &["build", "--collapse-scenarios", "true", "--collapse-scenarios", "false"],
    ArgumentError::RepeatedOption { option: "--collapse-scenarios".into() }
)]
#[case::a_colour_by_name(&["build", "--accent-light", "red"], ArgumentError::NotColour { option: "--accent-light".into() })]
#[case::a_colour_without_its_hash(&["build", "--accent-dark=8cc4ff"], ArgumentError::NotColour { option: "--accent-dark".into() })]
#[case::an_empty_colour(&["build", "--accent-dark="], missing_value("--accent-dark"))]
#[case::a_faint_accent(&["build", "--accent-light", "#777"], light_shortfall(Colour::new(0x77, 0x77, 0x77)))]
#[case::the_dark_accent_as_the_light_one(
    &["build", "--accent-light", "#8cc4ff"],
    ArgumentError::TooLittleContrast {
        option: "--accent-light".into(),
        shortfall: AccentShortfall {
            theme: Theme::Light,
            accent: Accents::DEFAULT.dark,
            background: palette(Theme::Light).surface,
            hundredths: 163,
        },
    }
)]
#[case::two_values(&["build", "--tag-link", "@a", "b"], missing_value("--tag-link"))]
#[case::a_joined_value(&["build", "--tag-link=@a", "b", "c"], missing_value("--tag-link"))]
#[case::an_empty_value(&["build", "--tag-link", "@a", "", "c"], missing_value("--tag-link"))]
#[case::not_a_regular_expression(
    &["build", "--tag-link", "@(a", "b", "c"],
    ArgumentError::NotRegex { option: tag_link_option(), reason: "unclosed group".into() }
)]
#[case::too_large(
    &["build", "--tag-link", "@(a{1000}){1000}", "b", "c"],
    ArgumentError::RegexTooLarge { option: tag_link_option() }
)]
#[case::a_group_the_address_names(
    &["build", "--tag-link", "@a(.)", "$2", "$3"],
    ArgumentError::UnknownGroup { option: tag_link_option(), group: "2".into() }
)]
#[case::a_group_the_text_names(
    &["build", "--tag-link", "@a(.)", "$1", "${id}"],
    ArgumentError::UnknownGroup { option: tag_link_option(), group: "id".into() }
)]
#[case::an_empty_value_before_a_pattern_s_mistake(&["build", "--tag-link", "@(", "b", ""], missing_value("--tag-link"))]
#[case::the_pattern_s_mistake_before_the_address_s(
    &["build", "--tag-link", "@(", "$1", "c"],
    ArgumentError::NotRegex { option: tag_link_option(), reason: "unclosed group".into() }
)]
fn each_mistake_in_a_key_s_option_is_named(
    #[case] arguments: &[&str],
    #[case] error: ArgumentError,
) {
    assert_eq!(parse(arguments), Err(error));
}

// Rule 234: the key's sentence, with the option's name.
#[rstest]
#[case::flag(
    not_boolean("--collapse-rules"),
    "--collapse-rules must be true or false."
)]
#[case::colour(
    ArgumentError::NotColour { option: "--accent-light".into() },
    "--accent-light must be a colour in hexadecimal, in quotes, such as \"#0b4f9c\"."
)]
#[case::contrast(
    light_shortfall(Colour::new(0x77, 0x77, 0x77)),
    "--accent-light is #777777, which reaches only 4.00:1 against the light theme's background \
     #f2f2f2. It needs at least 7:1."
)]
#[case::regex(
    ArgumentError::NotRegex { option: tag_link_option(), reason: "unclosed group".into() },
    "--tag-link is not a regular expression: unclosed group"
)]
#[case::too_large(
    ArgumentError::RegexTooLarge { option: tag_link_option() },
    "--tag-link is too large a regular expression."
)]
#[case::group(
    ArgumentError::UnknownGroup { option: tag_link_option(), group: "2".into() },
    "--tag-link names the group 2, which the pattern does not have."
)]
fn each_mistake_reads_as_its_key_s_with_the_option_s_name(
    #[case] error: ArgumentError,
    #[case] sentence: &str,
) {
    assert_eq!(error.to_string(), sentence);
}
