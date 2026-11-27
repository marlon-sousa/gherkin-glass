//! `ghg version remove` and `ghg preview remove`, as the arguments give them.

use super::*;

// Rules 257 and 258.
#[rstest]
#[case::a_version(
    &["version", "remove", "main", "--output", "site", "--message-format=json"],
    BuildTarget::Version(name("main")),
    MessageFormat::Json
)]
#[case::a_preview(
    &["preview", "remove", "--output=site", "pr-1"],
    BuildTarget::Preview(name("pr-1")),
    MessageFormat::Human
)]
fn remove_takes_a_name_and_the_output_folder(
    #[case] arguments: &[&str],
    #[case] target: BuildTarget,
    #[case] message_format: MessageFormat,
) {
    assert_eq!(
        parse(arguments),
        Ok(Invocation::Remove {
            options: RemoveOptions {
                output: Some(PathBuf::from("site")),
                ..RemoveOptions::default()
            },
            target,
            message_format,
        })
    );
}

#[test]
fn remove_takes_the_configuration_and_what_to_keep() {
    let arguments = [
        "version",
        "remove",
        "v1",
        "--config",
        "ghg.toml",
        "--keep-in-output",
        "CNAME",
    ];
    let Ok(Invocation::Remove { options, .. }) = parse(&arguments) else {
        panic!("a removal");
    };
    assert_eq!(options.config, Some(PathBuf::from("ghg.toml")));
    assert_eq!(
        options.keep_in_output,
        Some(KeptInOutput::new(vec![
            KeptPattern::new("CNAME").expect("valid")
        ]))
    );
}
