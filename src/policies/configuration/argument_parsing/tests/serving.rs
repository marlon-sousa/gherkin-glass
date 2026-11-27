//! `ghg serve`, its mode and its options, as the arguments give them.

use super::*;

#[rstest]
#[case::building_first(&["serve"], ServeMode::Build)]
#[case::without_building(&["serve", "--no-build"], ServeMode::Folder)]
#[case::watching(&["serve", "--watch"], ServeMode::Watch)]
fn serve_s_flags_say_its_mode(#[case] arguments: &[&str], #[case] mode: ServeMode) {
    let Ok(Invocation::Serve(options)) = parse(arguments) else {
        panic!("{arguments:?} is not a serve");
    };
    assert_eq!(options.mode, mode);
}

#[test]
fn serve_takes_the_options_of_build_and_its_own() {
    let arguments = ["serve", "--input=specs", "--host", "0.0.0.0", "--port", "0"];
    assert_eq!(
        parse(&arguments),
        Ok(Invocation::Serve(ServeOptions {
            build: paths(None, Some("specs")),
            host: Some("0.0.0.0".into()),
            port: Some(0),
            mode: ServeMode::Build,
        }))
    );
}
