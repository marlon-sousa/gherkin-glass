mod fakes;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::scope;

use fakes::*;
use rstest::rstest;

use super::*;
use crate::entities::{OutputContents, Watched};
use crate::errors::OutputErrorKind;
use crate::fixtures::{FakeWatcher, changed};
use crate::ports::FileWriter;

fn build_options(config: Option<&str>, input: Option<&str>) -> BuildOptions {
    BuildOptions {
        config: config.map(PathBuf::from),
        input: input.map(PathBuf::from),
        ..BuildOptions::default()
    }
}

#[rstest]
// Rules 20 and 21.
#[case::the_default_folder_without_a_file(&[], None, None, "a.feature:1: read in features\n")]
#[case::the_file_s_input(&[("ghg.toml", "input = \"specs\"\n")], None, None, "a.feature:1: read in specs\n")]
#[case::the_option_over_the_file_s(
    &[("ghg.toml", "input = \"specs\"\n")],
    None,
    Some("other"),
    "a.feature:1: read in other\n"
)]
#[case::a_file_asked_for_that_does_not_exist(&[], Some("ci.toml"), None, "ci.toml: cannot be read: not found\n")]
#[case::a_mistaken_file_before_the_input_is_read(
    &[("ghg.toml", "inputs = \"x\"\ninput = 1\n")],
    None,
    None,
    "ghg.toml:1: ghg has no setting named inputs.\nghg.toml:2: input must be text, in quotes.\n"
)]
fn a_build_reads_the_input_the_configuration_and_the_options_name(
    #[case] files: &[(&str, &str)],
    #[case] config: Option<&str>,
    #[case] input: Option<&str>,
    #[case] report: &str,
) {
    let world = World::default();
    world.input.failing.store(true, Ordering::SeqCst);
    let controller = world.controller(files, OutputContents::Missing);
    let error = controller
        .build(&build_options(config, input))
        .expect_err("stops");
    assert_eq!(error.to_string(), report);
}

#[test]
fn serve_reads_the_configuration_as_build_does() {
    // Rule 56.
    let world = World::default();
    world.input.failing.store(true, Ordering::SeqCst);
    let files = [("ghg.toml", "input = \"specs\"\n")];
    let error = world
        .controller(&files, OutputContents::Missing)
        .serve(&ServeOptions::default(), &Recorded::default())
        .expect_err("stops");
    assert_eq!(error.to_string(), "a.feature:1: read in specs\n");
    let options = ServeOptions {
        mode: ServeMode::Folder,
        ..ServeOptions::default()
    };
    let error = world
        .controller(&[("ghg.toml", "inputs = 1\n")], OutputContents::Missing)
        .serve(&options, &Recorded::default())
        .expect_err("stops");
    assert_eq!(
        error.to_string(),
        "ghg.toml:1: ghg has no setting named inputs.\n"
    );
    assert!(world.server.started().is_none());
}

#[test]
fn watching_rebuilds_on_each_burst_and_keeps_the_last_good_site() {
    // Rules 65 to 67.
    let world = World::default();
    // A file left from an earlier build, which no version may serve.
    world
        .memory
        .write(Path::new("site"), "stale.html", b"")
        .expect("written");
    let controller = world.controller(&[], OutputContents::Missing);
    let options = ServeOptions {
        mode: ServeMode::Watch,
        port: Some(0),
        ..ServeOptions::default()
    };
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| controller.serve(&options, &events));
        assert_eq!(events.wait_for(1), ["listening at 127.0.0.1:1"]);
        let sender = world.watcher.sender().expect("watching");
        let (_, _, preview) = world.server.started().expect("started");
        assert!(preview.mode.watches());
        assert_eq!(preview.current().number, 1);
        let ServedSite::Memory(files) = &preview.current().site else {
            panic!("a site in memory");
        };
        assert!(
            files.contains_key("assets/gherkin-glass.js"),
            "{:?}",
            files.keys()
        );
        assert!(!files.contains_key("stale.html"));

        // A change no build reads is looked at, and rebuilds nothing.
        let listed = world.input.listed.load(Ordering::SeqCst);
        sender.send(changed("features/notes.txt")).expect("sent");
        wait_until(|| world.input.listed.load(Ordering::SeqCst) > listed);
        assert_eq!(preview.current().number, 1);

        sender.send(changed("ghg.toml")).expect("sent");
        assert_eq!(events.wait_for(2)[1], "rebuilt");
        assert_eq!(preview.current().number, 2);

        world.input.failing.store(true, Ordering::SeqCst);
        sender.send(changed("ghg.toml")).expect("sent");
        assert!(events.wait_for(3)[2].contains("build: Input("));
        assert_eq!(preview.current().number, 2);

        drop(sender);
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
    assert_eq!(
        *world.watcher.watched.lock().expect("lock"),
        [
            Watched::Folder(PathBuf::from("features")),
            Watched::File(PathBuf::from("ghg.toml"))
        ]
    );
}

#[rstest]
// Rule 67: when the first build stops, ghg serve stops.
#[case::into_the_output_folder(ServeMode::Build, false)]
#[case::in_memory(ServeMode::Watch, false)]
#[case::in_memory_with_a_watch_that_cannot_start(ServeMode::Watch, true)]
fn a_first_build_that_stops_serves_nothing_and_is_said_first(
    #[case] mode: ServeMode,
    #[case] watch_fails: bool,
) {
    let world = World {
        watcher: Arc::new(FakeWatcher {
            failing: watch_fails,
            ..FakeWatcher::default()
        }),
        ..World::default()
    };
    world.input.failing.store(true, Ordering::SeqCst);
    let options = ServeOptions {
        mode,
        ..ServeOptions::default()
    };
    let outcome = world
        .controller(&[], OutputContents::Missing)
        .serve(&options, &Recorded::default());
    assert!(
        matches!(outcome, Err(ServeError::Build(BuildError::Input(_)))),
        "{outcome:?}"
    );
    assert!(world.server.started().is_none());
    // The watch started before the first build, so that no change made during it is missed.
    let watched = !world.watcher.watched.lock().expect("lock").is_empty();
    assert_eq!(watched, mode.watches());
}

#[test]
fn a_watch_that_cannot_start_stops_serve_once_the_first_build_is_done() {
    // Rule 68.
    let world = World {
        watcher: Arc::new(FakeWatcher {
            failing: true,
            ..FakeWatcher::default()
        }),
        ..World::default()
    };
    let options = ServeOptions {
        mode: ServeMode::Watch,
        ..ServeOptions::default()
    };
    let outcome = world
        .controller(&[], OutputContents::Missing)
        .serve(&options, &Recorded::default());
    assert!(matches!(outcome, Err(ServeError::Watch(_))), "{outcome:?}");
    assert!(world.server.started().is_none());
}

#[test]
fn bursts_that_wait_need_one_rebuild() {
    // Rule 65.
    let world = World {
        watcher: Arc::new(FakeWatcher {
            waiting: 3,
            ..FakeWatcher::default()
        }),
        ..World::default()
    };
    let controller = world.controller(&[], OutputContents::Missing);
    let options = ServeOptions {
        mode: ServeMode::Watch,
        ..ServeOptions::default()
    };
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| controller.serve(&options, &events));
        assert_eq!(events.wait_for(2).len(), 2);
        drop(world.watcher.sender());
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
    assert_eq!(
        events.wait_for(2),
        ["listening at 127.0.0.1:8000", "rebuilt"]
    );
    let (_, _, preview) = world.server.started().expect("started");
    assert_eq!(preview.current().number, 2);
}

#[test]
fn a_build_into_a_folder_ghg_did_not_build_serves_nothing_unless_it_is_in_memory() {
    // Rules 38 and 66.
    let world = World::default();
    let controller = world.controller(&[], not_a_site());
    let outcome = controller.serve(&ServeOptions::default(), &Recorded::default());
    assert!(
        matches!(
            &outcome,
            Err(ServeError::Build(BuildError::Output(error))) if error.kind == OutputErrorKind::NotASite
        ),
        "{outcome:?}"
    );
    assert!(world.server.started().is_none());
    let options = ServeOptions {
        mode: ServeMode::Watch,
        ..ServeOptions::default()
    };
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| controller.serve(&options, &events));
        assert_eq!(events.wait_for(1).len(), 1);
        drop(world.watcher.sender());
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
    assert!(world.server.started().is_some());
}
