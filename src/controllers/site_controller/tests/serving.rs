//! The controller's tests of ghg serve, with and without --watch, up to the first change.

mod rebuilding;

use std::path::Path;
use std::sync::atomic::Ordering;

use rstest::rstest;

use super::*;

fn serve_options(mode: ServeMode) -> ServeOptions {
    ServeOptions {
        mode,
        ..ServeOptions::default()
    }
}

#[rstest]
// Rules 56 and 66: ghg serve loads the settings as ghg build does, with the destination its
// mode gives, builds as its mode says, then serves the site it built, or the output folder as
// it is.
#[case::the_output_folder_as_it_is(ServeMode::Folder, "Folder", &[], "Folder(\"site\")")]
#[case::the_site_built_into_the_output_folder(
    ServeMode::Build,
    "Folder",
    &a_build("Folder", "features"),
    "Folder(\"site\")"
)]
#[case::the_site_built_in_memory(
    ServeMode::Watch,
    "Memory",
    &a_build("Memory", "features"),
    "Memory({\"site 1\": []})"
)]
fn serve_builds_as_its_mode_says_then_serves(
    #[case] mode: ServeMode,
    #[case] destination: &str,
    #[case] build: &[String],
    #[case] site: &str,
) {
    let world = world();
    // The fake server stops serve once it is asked to serve.
    world.server.listen_failing.store(true, Ordering::SeqCst);
    let options = serve_options(mode);
    let events = Recorded::default();
    assert_eq!(
        world.controller().serve(&options, &events),
        Err(listen_error())
    );
    let expected: Vec<String> = [format!("settings.load {destination}")]
        .into_iter()
        .chain(build.iter().cloned())
        .chain([format!("server.serve {site}")])
        .collect();
    assert_eq!(world.calls.taken(), expected);
    assert!(events.wait_for(0).is_empty());
}

#[rstest]
// Rules 56 and 67: when the settings or the first build stop, ghg serve says so and serves
// nothing, nor watches.
#[case::by_the_settings(
    ServeMode::Folder,
    Failing::Settings,
    BuildError::Config(config_errors())
)]
#[case::by_the_output_folder_s_check(
    ServeMode::Build,
    Failing::OutputCheck,
    BuildError::Output(output_error())
)]
#[case::by_the_input(ServeMode::Build, Failing::Input, BuildError::Input(input_errors()))]
#[case::by_the_input_in_memory(ServeMode::Watch, Failing::Input, BuildError::Input(input_errors()))]
fn what_stops_the_first_build_stops_serve(
    #[case] mode: ServeMode,
    #[case] failing: Failing,
    #[case] error: BuildError,
) {
    let world = world();
    world.fail(failing);
    let options = serve_options(mode);
    let outcome = world.controller().serve(&options, &Recorded::default());
    assert_eq!(outcome, Err(ServeError::Build(error)));
    let calls = world.calls.taken();
    assert!(
        !calls.iter().any(|call| call.starts_with("server.")),
        "{calls:?}"
    );
}

#[test]
fn watching_starts_once_the_site_is_served_and_one_that_cannot_start_stops_serve() {
    // Rule 68: a watch that cannot start as ghg serve starts stops it, after the first build,
    // before ghg serve says where it listens.
    let world = world();
    world.server.watch_failing.store(true, Ordering::SeqCst);
    let options = serve_options(ServeMode::Watch);
    let events = Recorded::default();
    let outcome = world.controller().serve(&options, &events);
    let error = watch_error(Path::new("features"));
    assert_eq!(outcome, Err(ServeError::Watch(error)));
    let calls = world.calls.taken();
    assert_eq!(
        calls[calls.len() - 3..],
        [
            "server.serve Memory({\"site 1\": []})",
            "settings.config_file",
            "server.watch features ghg.toml",
        ]
    );
    assert!(events.wait_for(0).is_empty());
}
