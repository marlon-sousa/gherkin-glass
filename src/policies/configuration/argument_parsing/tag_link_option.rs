//! Policy: the links of `--tag-link`, each made of its three values and checked as a table of `tag_links` is.

use crate::entities::TagLink;
use crate::errors::{ArgumentError, PatternErrorKind};
use crate::policies::configuration::{unknown_group, whole_tag_pattern};

const OPTION: &str = "--tag-link";

/// The link of one `--tag-link`, from its pattern, its address and its text, or its first
/// mistake, in that order.
pub(super) fn tag_link(values: &[String]) -> Result<TagLink, ArgumentError> {
    let [source, address, text] = values else {
        return Err(missing_value());
    };
    let pattern = whole_tag_pattern(source).map_err(pattern_error)?;
    let group = [address, text]
        .into_iter()
        .find_map(|replacement| unknown_group(replacement, &pattern));
    if let Some(group) = group {
        return Err(ArgumentError::UnknownGroup {
            option: OPTION.to_string(),
            group,
        });
    }
    Ok(TagLink {
        source: source.clone(),
        pattern,
        address: address.clone(),
        text: text.clone(),
    })
}

/// clap gives `--tag-link` its three values or stops, so this is never met.
fn missing_value() -> ArgumentError {
    ArgumentError::MissingValue {
        option: OPTION.to_string(),
    }
}

fn pattern_error(reason: PatternErrorKind) -> ArgumentError {
    let option = OPTION.to_string();
    match reason {
        PatternErrorKind::TooLarge => ArgumentError::RegexTooLarge { option },
        PatternErrorKind::Syntax(reason) => ArgumentError::NotRegex { option, reason },
    }
}
