//! Which files the input collector allows, checked against the fixtures.

use super::*;

// Which files are read: rules 1 to 4.

#[test]
fn only_the_files_the_defaults_and_the_ghg_ignore_files_allow_are_read() {
    assert_eq!(
        allowed(&fixture("input-files")),
        [
            // Rule 3: `!.github/` in the input's .ghg-ignore reads a hidden folder again.
            ".github/read-again.feature",
            // Rule 4: the fixture's .gitignore leaves this folder out, to no effect.
            "ignored-by-gitignore/read.feature",
            // Rule 4: the fixture's .ignore leaves this file out, to no effect.
            "read-despite-ignore.feature",
            // Rule 1.
            "read.feature",
            // Rule 3: `!*.story` reads a file of another type.
            "stories/read.story",
            // Rule 3: the nearest .ghg-ignore decides.
            "sub/deeper/read-again.feature",
            "sub/deeper/read.feature",
            "sub/read.feature",
        ],
        // Left out: notes.md and other.txt (rule 1), .hidden.feature and
        // .hidden/ (rule 2), drafts/ with the `!` rule inside it,
        // sub/left-out.feature and stories/left-out.story (rule 3).
    );
}

#[test]
fn the_ignore_files_of_the_folders_above_the_input_have_no_effect() {
    // Rule 4. The input sits in a git repository with a .gitignore, an .ignore
    // and an exclude file above it, each of which would leave everything out.
    let root = scratch("input/ignore-files-above");
    create_dir_all(root.join(".git/info")).expect("the git folder is made");
    write(root.join(".git/info/exclude"), "*.feature\n").expect("written");
    write(root.join(".gitignore"), "*.feature\n").expect("written");
    write(root.join(".ignore"), "*.feature\n").expect("written");
    let input = root.join("input");
    copy_folder(&fixture("input-files"), &input);

    assert_eq!(allowed(&input), allowed(&fixture("input-files")));
}

#[test]
fn a_ghg_ignore_file_is_never_read_as_a_feature() {
    // Rule 3, last point: `!.*` reads hidden files again, but not this one.
    let input = scratch("input/ghg-ignore-not-a-feature");
    write(input.join(".ghg-ignore"), "!.*\n").expect("written");
    copy(
        fixture("input-files/read.feature"),
        input.join(".hidden.feature"),
    )
    .expect("copied");

    assert_eq!(allowed(&input), [".hidden.feature"]);
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_not_followed() {
    // Rule 5.
    use std::os::unix::fs::symlink;

    let root = scratch("input/symbolic-links");
    let input = root.join("input");
    create_dir_all(&input).expect("the input is made");
    let elsewhere = root.join("elsewhere");
    copy_folder(&fixture("input-files/sub"), &elsewhere);
    copy(
        fixture("input-files/read.feature"),
        input.join("read.feature"),
    )
    .expect("copied");
    symlink(elsewhere.join("read.feature"), input.join("linked.feature")).expect("linked");
    symlink(&elsewhere, input.join("linked-folder")).expect("linked");

    assert_eq!(allowed(&input), ["read.feature"]);
}

#[test]
fn a_folder_sorts_by_its_name_before_its_contents() {
    // Rule 15: `a/` comes before `a-b.feature`, folder by folder, although `/`
    // sorts after `-` as a character.
    let input = scratch("input/sorted-by-folder");
    create_dir_all(input.join("a")).expect("made");
    write(input.join("a/x.feature"), "Nothing\n").expect("written");
    write(input.join("a-b.feature"), "Nothing\n").expect("written");

    assert_eq!(allowed(&input), ["a/x.feature", "a-b.feature"]);
}
