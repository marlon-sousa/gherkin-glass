//! The controller's tests of ghg serve when it reads no feature file: into the output folder it
//! builds and serves nothing, and under --watch it serves the empty site and says so.

use std::thread::scope;

use super::*;

/// A build of the input `features`, without `--only`, that read no feature file.
fn nothing() -> NothingToBuild {
    NothingToBuild::InputHoldsNone {
        input: PathBuf::from("features"),
    }
}

// Rule 230.
#[test]
fn serve_into_the_output_folder_builds_and_serves_nothing() {
    let world = world();
    world.input_collector.no_file.store(true, Ordering::SeqCst);
    let events = Recorded::default();
    assert_eq!(
        world
            .controller()
            .serve(&serve_options(ServeMode::Build), &events),
        Ok(ServeEnd::NothingToBuild(nothing()))
    );
    assert_eq!(
        world.calls.taken(),
        [
            "server.listen_for_stops",
            "settings.load Folder",
            "site_writer.check Folder",
            "input_collector.collect features",
        ]
    );
    assert!(events.wait_for(0).is_empty());
}

// Rule 230.
#[test]
fn serve_under_watch_serves_the_empty_site_and_says_so() {
    let world = world();
    world.input_collector.no_file.store(true, Ordering::SeqCst);
    let controller = world.controller();
    let options = serve_options(ServeMode::Watch);
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| controller.serve(&options, &events));
        assert_eq!(
            events.wait_for(2),
            [
                "listening at 127.0.0.1:8000".to_string(),
                format!("empty: {:?}", nothing()),
            ]
        );
        world.server.stop_watching();
        assert_eq!(
            serving.join().expect("no panic"),
            Ok(ServeEnd::NoLongerServed)
        );
    });
}
