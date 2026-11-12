//! The controller's tests of ghg serve, with and without --watch.

use std::sync::atomic::Ordering;
use std::thread::scope;

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
// it is; and, under --watch, the watch starts first, on the input and the configuration file,
// with the files the gherkin parser selects.
#[case::the_output_folder_as_it_is(ServeMode::Folder, "Folder", &[], &[], "Folder(\"site\")")]
#[case::the_site_built_into_the_output_folder(
    ServeMode::Build,
    "Folder",
    &[],
    &a_build("Folder"),
    "Folder(\"site\")"
)]
#[case::the_site_built_in_memory(
    ServeMode::Watch,
    "Memory",
    &[
        "server.watch features ghg.toml",
        "gherkin_parser.paths features",
    ],
    &a_build("Memory"),
    "Memory({\"site 1\": []})"
)]
fn serve_builds_as_its_mode_says_then_serves(
    #[case] mode: ServeMode,
    #[case] destination: &str,
    #[case] watching: &[&str],
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
    let expected: Vec<String> = [
        format!("settings.load {destination}"),
        "settings.config_file".into(),
    ]
    .into_iter()
    .chain(watching.iter().map(|call| call.to_string()))
    .chain(build.iter().cloned())
    .chain([format!("server.serve {site}")])
    .collect();
    assert_eq!(world.calls.taken(), expected);
    assert!(events.wait_for(0).is_empty());
}

#[rstest]
// Rules 56, 67 and 68: when the settings or the first build stop, ghg serve says so and serves
// nothing, before a watch that cannot start.
#[case::by_the_settings(
    ServeMode::Folder,
    Failing::Settings,
    false,
    BuildError::Config(config_errors())
)]
#[case::by_the_output_folder_s_check(
    ServeMode::Build,
    Failing::OutputCheck,
    false,
    BuildError::Output(output_error())
)]
#[case::by_the_input(
    ServeMode::Build,
    Failing::Input,
    false,
    BuildError::Input(input_errors())
)]
#[case::by_the_input_in_memory(
    ServeMode::Watch,
    Failing::Input,
    false,
    BuildError::Input(input_errors())
)]
#[case::by_the_input_with_a_watch_that_cannot_start(
    ServeMode::Watch,
    Failing::Input,
    true,
    BuildError::Input(input_errors())
)]
fn what_stops_the_first_build_stops_serve_and_is_said_first(
    #[case] mode: ServeMode,
    #[case] failing: Failing,
    #[case] watch_failing: bool,
    #[case] error: BuildError,
) {
    let world = world();
    world.fail(failing);
    world
        .server
        .watch_failing
        .store(watch_failing, Ordering::SeqCst);
    let options = serve_options(mode);
    let outcome = world.controller().serve(&options, &Recorded::default());
    assert_eq!(outcome, Err(ServeError::Build(error)));
    let calls = world.calls.taken();
    assert!(
        !calls.iter().any(|call| call.starts_with("server.serve")),
        "{calls:?}"
    );
}

#[test]
fn a_watch_that_cannot_start_stops_serve_once_the_first_build_is_done() {
    // Rule 68.
    let world = world();
    world.server.watch_failing.store(true, Ordering::SeqCst);
    let options = serve_options(ServeMode::Watch);
    let outcome = world.controller().serve(&options, &Recorded::default());
    assert!(matches!(outcome, Err(ServeError::Watch(_))), "{outcome:?}");
    let calls = world.calls.taken();
    assert_eq!(
        calls.last().map(String::as_str),
        Some("site_writer.write a.feature")
    );
}

#[test]
fn each_rebuild_loads_the_settings_again_and_serves_only_a_site_that_built() {
    // Rules 65 to 67.
    let world = world();
    let controller = world.controller();
    let options = serve_options(ServeMode::Watch);
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| controller.serve(&options, &events));
        assert_eq!(events.wait_for(1), ["listening at 127.0.0.1:8000"]);
        let preview = world.server.preview();
        assert_eq!(preview.current().number, 1);
        let served = world.calls.taken().len();

        world.server.change();
        assert_eq!(events.wait_for(2)[1], "rebuilt");
        let rebuild: Vec<_> = ["settings.load Memory".to_string()]
            .into_iter()
            .chain(a_build("Memory"))
            .collect();
        assert_eq!(world.calls.taken()[served..], rebuild);
        assert_eq!(preview.current().number, 2);
        assert_eq!(
            preview.current().site,
            ServedSite::Memory([("site 2".to_string(), Vec::new())].into())
        );

        world.fail(Failing::Input);
        world.server.change();
        let error = RebuildError {
            build: BuildError::Input(input_errors()),
        };
        assert_eq!(events.wait_for(3)[2], format!("not rebuilt: {error:?}"));
        world.fail(Failing::Settings);
        world.server.change();
        let error = RebuildError {
            build: BuildError::Config(config_errors()),
        };
        assert_eq!(events.wait_for(4)[3], format!("not rebuilt: {error:?}"));
        assert_eq!(preview.current().number, 2);

        world.server.stop_watching();
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
}
