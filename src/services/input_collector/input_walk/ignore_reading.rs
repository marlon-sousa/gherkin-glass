//! Service: the rules of a folder's `.ghg-ignore`, read through the input folders port, with the error met reading it or those of its rules that are not valid.

use std::path::Path;
use std::sync::Arc;

use crate::entities::{IGNORE_FILE, IgnoreRules};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::{InvalidRule, parse_rules};
use crate::ports::InputFolders;

/// The rules of the `.ghg-ignore` of `folder`, in the input `input`.
pub(super) fn read_rules(
    input_folders: &dyn InputFolders,
    input: &Path,
    folder: &Path,
) -> IgnoreRules {
    let path = folder.join(IGNORE_FILE);
    let text = match input_folders.read(&path) {
        Ok(text) => text,
        Err(error) => {
            let reason = error.to_string();
            let kind = InputErrorKind::CannotBeRead { reason };
            return IgnoreRules {
                rules: None,
                errors: vec![InputError::new(input, &path, None, kind)],
            };
        }
    };
    let parsed = parse_rules(folder, &path, &text);
    let errors = parsed
        .invalid
        .into_iter()
        .map(|rule| invalid_rule(input, &path, rule))
        .collect();
    IgnoreRules {
        rules: Some(Arc::new(parsed.rules)),
        errors,
    }
}

fn invalid_rule(
    input: &Path,
    path: &Path,
    InvalidRule { line, reason }: InvalidRule,
) -> InputError {
    InputError::new(input, path, line, InputErrorKind::InvalidRule { reason })
}
