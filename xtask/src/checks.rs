//! Entity: the checks `cargo xtask check` runs with cargo, the same as those of `.github/workflows/rust.yml`.

/// Each check's arguments to cargo, in the order they run. A test keeps them the same as CI's.
pub(crate) const CARGO_CHECKS: [&[&str]; 6] = [
    &["fmt", "--all", "--check"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ],
    &["test", "--workspace", "--locked"],
    &[
        "doc",
        "--workspace",
        "--no-deps",
        "--document-private-items",
        "--locked",
    ],
    &["deny", "check"],
    &["publish", "--dry-run", "--locked", "--allow-dirty"],
];

/// The arguments of the second test run, after `+` and the minimum supported Rust version.
pub(crate) const MINIMUM_RUST_TEST: [&str; 4] = ["test", "--locked", "--package", "gherkin-glass"];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs::read_to_string;
    use std::path::Path;

    use super::*;

    /// The cargo commands of the workflow's jobs, each without `cargo `, and the minimum version's
    /// variable written `+MSRV`.
    fn workflow_commands() -> BTreeSet<String> {
        let workflow = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/rust.yml");
        read_to_string(workflow)
            .expect("the workflow is read")
            .lines()
            .filter_map(|line| line.trim().strip_prefix("- run: cargo "))
            .map(|command| command.replace("\"+$MSRV\"", "+MSRV"))
            .collect()
    }

    #[test]
    fn cargo_xtask_check_runs_what_ci_runs() {
        let mut expected: BTreeSet<String> =
            CARGO_CHECKS.iter().map(|check| check.join(" ")).collect();
        expected.insert(format!("+MSRV {}", MINIMUM_RUST_TEST.join(" ")));
        expected.insert("xtask check-script".to_string());
        expected.insert("xtask check-browser".to_string());
        // The action's scripts run on macOS too, in a job of their own, and the tests on
        // Linux already run them.
        expected.insert("test --locked --test action".to_string());
        assert_eq!(workflow_commands(), expected);
    }
}
