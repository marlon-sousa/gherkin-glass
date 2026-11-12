//! The controller's tests of ghg serve --watch once it watches: what each change rebuilds, and
//! the watch started again on the input the settings name.

use std::thread::scope;

use super::*;
use crate::entities::InputChanges;

/// The calls ghg serve --watch makes before it waits for the first change.
const UNTIL_WATCHING: usize = 9;

/// Runs ghg serve --watch with the fakes of `world` on a thread of its own and, once it watches,
/// hands `steps` what the listener was told; then ends the watching, and checks that serve ends.
fn while_watching(world: &World, steps: impl FnOnce(&Recorded)) {
    let controller = world.controller();
    let options = serve_options(ServeMode::Watch);
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| controller.serve(&options, &events));
        // Rule 65: ghg serve says where it listens once it watches.
        assert_eq!(events.wait_for(1), ["listening at 127.0.0.1:8000"]);
        assert_eq!(
            world.calls.taken().last().map(String::as_str),
            Some("server.watch features ghg.toml")
        );
        steps(&events);
        world.server.stop_watching();
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
}

/// The calls made since the first `count`, once there are `more` of them.
fn calls_after(world: &World, count: usize, more: usize) -> Vec<String> {
    world.calls.wait_for(count + more)[count..].to_vec()
}

/// What the listener is told of a rebuild that `build` stopped.
fn not_rebuilt(build: BuildError) -> String {
    format!("not rebuilt: {:?}", RebuildError { build })
}

/// What the listener is told of a watch of `path` that cannot start.
fn not_watched(path: &str) -> String {
    format!("not watched: {}", watch_error(Path::new(path)))
}

fn paths(paths: &[&str]) -> InputChanges {
    InputChanges::Paths(paths.iter().map(PathBuf::from).collect())
}

/// `calls`, then those of a build from `input` in memory.
fn then_a_build(calls: &[&str], input: &str) -> Vec<String> {
    calls
        .iter()
        .map(ToString::to_string)
        .chain(a_build("Memory", input))
        .collect()
}

#[test]
fn everything_changed_loads_the_settings_again_and_watches_again_before_it_rebuilds() {
    // Rules 65 to 67: and only a site that built is served.
    let world = world();
    while_watching(&world, |events| {
        let preview = world.server.preview();
        assert_eq!(preview.current().number, 1);

        world.server.change(InputChanges::Everything);
        assert_eq!(events.wait_for(2)[1], "rebuilt");
        assert_eq!(
            calls_after(&world, UNTIL_WATCHING, 7),
            then_a_build(
                &["settings.load Memory", "server.watch features ghg.toml"],
                "features"
            )
        );
        assert_eq!(preview.current().number, 2);
        assert_eq!(
            preview.current().site,
            ServedSite::Memory([("site 2".to_string(), Vec::new())].into())
        );

        world.fail(Failing::Input);
        world.server.change(InputChanges::Everything);
        let error = not_rebuilt(BuildError::Input(input_errors()));
        assert_eq!(events.wait_for(3)[2], error);

        world.fail(Failing::Settings);
        let count = world.calls.taken().len();
        world.server.change(InputChanges::Everything);
        let error = not_rebuilt(BuildError::Config(config_errors()));
        assert_eq!(events.wait_for(4)[3], error);
        // The configuration stops the rebuild, so the watching stays as it was.
        assert_eq!(world.calls.taken()[count..], ["settings.load Memory"]);
        assert_eq!(preview.current().number, 2);
    });
}

#[test]
fn paths_rebuild_only_when_the_input_collector_says_they_call_for_it() {
    // Rule 65: judged against the last build's input tree before the settings are read again,
    // and rebuilt without watching again.
    let world = world();
    while_watching(&world, |events| {
        let collector = &world.input_collector;
        collector.nothing_read_changes.store(true, Ordering::SeqCst);
        world.server.change(paths(&["features/notes.txt"]));
        assert_eq!(
            calls_after(&world, UNTIL_WATCHING, 1),
            ["input_collector.calls_for_rebuild features features/notes.txt"]
        );

        collector
            .nothing_read_changes
            .store(false, Ordering::SeqCst);
        world.server.change(paths(&["features/a.feature"]));
        assert_eq!(
            events.wait_for(2),
            ["listening at 127.0.0.1:8000", "rebuilt"]
        );
        assert_eq!(
            calls_after(&world, UNTIL_WATCHING + 1, 7),
            then_a_build(
                &[
                    "input_collector.calls_for_rebuild features features/a.feature",
                    "settings.load Memory",
                ],
                "features"
            )
        );
    });
}

#[test]
fn the_input_the_settings_name_is_watched_built_and_judged_from_then_on() {
    // Rule 65: whether paths changed and the settings read again differ, or everything
    // changed, and even when the input stops the rebuild.
    let world = world();
    let configure = |input: &str| {
        *world.settings.configured_input.lock().expect("lock") = Some(input.into());
    };
    while_watching(&world, |events| {
        configure("elsewhere");
        world.server.change(paths(&["features/a.feature"]));
        assert_eq!(events.wait_for(2)[1], "rebuilt");
        assert_eq!(
            calls_after(&world, UNTIL_WATCHING, 8),
            then_a_build(
                &[
                    "input_collector.calls_for_rebuild features features/a.feature",
                    "settings.load Memory",
                    "server.watch elsewhere ghg.toml",
                ],
                "elsewhere"
            )
        );

        configure("other");
        world.fail(Failing::Input);
        world.server.change(InputChanges::Everything);
        let error = not_rebuilt(BuildError::Input(input_errors()));
        assert_eq!(events.wait_for(3)[2], error);
        let count = world.calls.taken().len();
        world.server.change(paths(&["other/a.feature"]));
        assert_eq!(events.wait_for(4)[3], error);
        assert_eq!(
            world.calls.taken()[count..count + 2],
            [
                "input_collector.calls_for_rebuild other other/a.feature",
                "settings.load Memory",
            ]
        );
    });
}

#[test]
fn a_watch_that_cannot_start_again_keeps_the_last_site_and_the_configuration_watched() {
    // Rule 68.
    let world = world();
    while_watching(&world, |events| {
        let preview = world.server.preview();
        world.server.watch_failing.store(true, Ordering::SeqCst);
        *world.settings.configured_input.lock().expect("lock") = Some("missing".into());
        world.server.change(InputChanges::Everything);
        assert_eq!(events.wait_for(2)[1], not_watched("missing"));
        assert_eq!(
            calls_after(&world, UNTIL_WATCHING, 3),
            [
                "settings.load Memory",
                "server.watch missing ghg.toml",
                "server.watch_config ghg.toml",
            ]
        );
        assert_eq!(preview.current().number, 1);

        // The configuration file, watched alone, tells of its next change, which tries again.
        world.server.watch_failing.store(false, Ordering::SeqCst);
        *world.settings.configured_input.lock().expect("lock") = Some("features".into());
        world.server.change(InputChanges::Everything);
        assert_eq!(events.wait_for(3)[2], "rebuilt");
        assert_eq!(preview.current().number, 2);
    });
}

#[test]
fn when_not_even_the_configuration_file_can_be_watched_the_last_watch_is_kept() {
    // Rule 68.
    let world = world();
    while_watching(&world, |events| {
        let server = &world.server;
        server.watch_failing.store(true, Ordering::SeqCst);
        server.config_watch_failing.store(true, Ordering::SeqCst);
        server.change(InputChanges::Everything);
        assert_eq!(
            events.wait_for(3)[1..],
            [not_watched("features"), not_watched("ghg.toml")]
        );
        server.watch_failing.store(false, Ordering::SeqCst);
        server.change(InputChanges::Everything);
        assert_eq!(events.wait_for(4)[3], "rebuilt");
    });
}
