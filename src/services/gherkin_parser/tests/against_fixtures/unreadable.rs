//! A feature file that cannot be read, reported as an error.

use std::fs::copy;

use super::*;
use crate::errors::InputErrorKind;
use crate::test_support::{lock, read_input, unlock};

#[test]
fn a_feature_file_that_cannot_be_opened_is_reported() {
    // Rules 14 and 16.
    let input = scratch("input/locked-feature");
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    if !lock(&input.join("read.feature")) {
        unlock(&input.join("read.feature"), 0o644);
        return;
    }
    let errors = read_input(&input);
    unlock(&input.join("read.feature"), 0o644);

    let errors = errors.expect_err("cannot be read");
    let errors = errors.as_slice();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path, "read.feature");
    assert!(
        matches!(errors[0].kind, InputErrorKind::CannotBeRead { .. }),
        "{}",
        errors[0]
    );
}
