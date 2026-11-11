//! Folders and files that cannot be read, reported as problems.

use super::*;

/// Makes `path` unreadable, and says whether it worked: it does not for root.
#[cfg(unix)]
fn lock(path: &Path) -> bool {
    set_permissions(path, Permissions::from_mode(0o000)).expect("the mode is set");
    let locked = read_dir(path).is_err() && read_to_string(path).is_err();
    if !locked {
        eprintln!(
            "skipped: {} can still be read, as it can by root",
            path.display()
        );
    }
    locked
}

#[cfg(unix)]
fn unlock(path: &Path, mode: u32) {
    set_permissions(path, Permissions::from_mode(mode)).expect("the mode is set");
}

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
    let problems = read_input(&input);
    unlock(&input.join("locked"), 0o755);

    let problems = problems.expect_err("cannot be read");
    let problems = problems.as_slice();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].path, "locked");
    assert!(
        matches!(problems[0].kind, ProblemKind::CannotBeListed { .. }),
        "{}",
        problems[0]
    );
}

#[cfg(unix)]
#[test]
fn a_ghg_ignore_file_or_a_feature_file_that_cannot_be_opened_is_reported() {
    // Rules 14 and 16.
    let input = scratch("input/locked-files");
    write(input.join(".ghg-ignore"), "drafts/\n").expect("written");
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    let locked = lock(&input.join(".ghg-ignore")) && lock(&input.join("read.feature"));
    if !locked {
        unlock(&input.join(".ghg-ignore"), 0o644);
        unlock(&input.join("read.feature"), 0o644);
        return;
    }
    let problems = read_input(&input);
    unlock(&input.join(".ghg-ignore"), 0o644);
    unlock(&input.join("read.feature"), 0o644);

    let problems = problems.expect_err("cannot be read");
    let found: Vec<_> = problems
        .as_slice()
        .iter()
        .map(|problem| {
            let unreadable = matches!(problem.kind, ProblemKind::CannotBeRead { .. });
            (problem.path.as_str(), problem.line, unreadable)
        })
        .collect();
    assert_eq!(
        found,
        [(".ghg-ignore", None, true), ("read.feature", None, true)]
    );
}

#[test]
fn an_input_folder_that_does_not_exist_is_reported() {
    // Rule 14.
    let input = scratch("input/missing").join("not-there");
    let problems = read_input(&input).expect_err("cannot be read");
    let problems = problems.as_slice();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].path, input.display().to_string());
    assert!(
        matches!(problems[0].kind, ProblemKind::InputMissing),
        "{}",
        problems[0]
    );
}
