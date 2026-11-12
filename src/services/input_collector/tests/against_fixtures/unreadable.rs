//! Folders and `.ghg-ignore` files that cannot be read, reported as errors.

use super::*;
#[cfg(unix)]
use crate::test_support::{lock, unlock};

#[cfg(unix)]
#[test]
fn a_folder_that_cannot_be_listed_is_reported() {
    // Rule 14.
    let input = scratch("input/locked-folder");
    create_dir_all(input.join("locked")).expect("made");
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    if !lock(&input.join("locked")) {
        unlock(&input.join("locked"), 0o755);
        return;
    }
    let tree = collected(&input);
    unlock(&input.join("locked"), 0o755);

    let errors = tree.input_errors().expect("cannot be read");
    let errors = errors.as_slice();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path, "locked");
    assert!(
        matches!(errors[0].kind, InputErrorKind::CannotBeListed { .. }),
        "{}",
        errors[0]
    );
}

#[cfg(unix)]
#[test]
fn a_ghg_ignore_file_that_cannot_be_opened_is_reported() {
    // Rules 14 and 16.
    let input = scratch("input/locked-ghg-ignore");
    write(input.join(".ghg-ignore"), "drafts/\n").expect("written");
    if !lock(&input.join(".ghg-ignore")) {
        unlock(&input.join(".ghg-ignore"), 0o644);
        return;
    }
    let tree = collected(&input);
    unlock(&input.join(".ghg-ignore"), 0o644);

    let errors = tree.input_errors().expect("cannot be read");
    let errors = errors.as_slice();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        (errors[0].path.as_str(), errors[0].line),
        (".ghg-ignore", None)
    );
    assert!(
        matches!(errors[0].kind, InputErrorKind::CannotBeRead { .. }),
        "{}",
        errors[0]
    );
}

#[test]
fn an_input_folder_that_does_not_exist_is_reported() {
    // Rule 14.
    let input = scratch("input/missing").join("not-there");
    assert_eq!(
        errors(&input),
        [(input.display().to_string(), InputErrorKind::InputMissing)]
    );
}
