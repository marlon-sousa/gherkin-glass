//! Policy: the arguments cargo gives its plugin `cargo-ghg`, made into those ghg takes.

use std::ffi::OsString;

/// The arguments `cargo-ghg` was given after its own name, without the
/// `ghg` that cargo puts first when it runs `cargo ghg`, so that the rest
/// are what `ghg` would be given. Anything else is kept as it is, so that
/// `cargo-ghg` run by itself behaves as `ghg` too.
pub(crate) fn cargo_plugin_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> impl Iterator<Item = OsString> {
    let mut arguments = arguments.into_iter().peekable();
    let _plugin_name = arguments.next_if(|first| first == "ghg");
    arguments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn given(arguments: &[&str]) -> Vec<OsString> {
        cargo_plugin_arguments(arguments.iter().map(OsString::from)).collect()
    }

    fn expected(arguments: &[&str]) -> Vec<OsString> {
        arguments.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_plugin_name_cargo_puts_first_is_dropped() {
        assert_eq!(
            given(&["ghg", "build", "--input", "specs"]),
            expected(&["build", "--input", "specs"])
        );
        assert_eq!(given(&["ghg"]), expected(&[]));
    }

    #[test]
    fn only_the_first_ghg_is_dropped() {
        assert_eq!(given(&["ghg", "ghg"]), expected(&["ghg"]));
        assert_eq!(
            given(&["build", "--input", "ghg"]),
            expected(&["build", "--input", "ghg"])
        );
    }

    #[test]
    fn arguments_without_the_plugin_name_are_kept() {
        assert_eq!(given(&["build"]), expected(&["build"]));
        assert_eq!(given(&["--version"]), expected(&["--version"]));
        assert_eq!(given(&[]), expected(&[]));
    }
}
