use std::path::Path;
use std::sync::atomic::Ordering;
use std::thread::scope;

use super::*;
use crate::entities::OutputContents;
use crate::errors::OutputErrorKind;
use crate::ports::SiteWriter;

mod fakes;

use fakes::*;

#[test]
fn watching_rebuilds_on_each_burst_and_keeps_the_last_good_site() {
    let input = Arc::new(Input::default());
    let server = Arc::new(Server::default());
    let watcher = Arc::new(Watcher::default());
    // A file left from an earlier build, which no version may serve.
    let recorder = Arc::new(Recorder::default());
    recorder
        .write(Path::new("site"), "stale.html", b"")
        .expect("written");
    let service = PreviewService::new(
        Arc::new(NoFile),
        site(&input),
        output(OutputContents::Missing),
        reading(&input),
        recorder,
        server.clone(),
        watcher.clone(),
    );
    let options = ServeOptions {
        mode: ServeMode::Watch,
        port: Some(0),
        ..ServeOptions::default()
    };
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| service.serve(&options, words(), &events));
        assert_eq!(events.wait_for(1), ["listening at 127.0.0.1:1"]);
        let sender = watcher
            .sender
            .lock()
            .expect("lock")
            .take()
            .expect("watching");
        let (host, port, preview) = server
            .started
            .lock()
            .expect("lock")
            .clone()
            .expect("started");
        assert_eq!((host.as_str(), port), ("127.0.0.1", 0));
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
        let walked = input.listed.load(Ordering::SeqCst);
        sender.send(changed("features/notes.txt")).expect("sent");
        wait_until(|| input.listed.load(Ordering::SeqCst) > walked);
        assert_eq!(preview.current().number, 1);

        sender.send(changed("ghg.toml")).expect("sent");
        assert_eq!(events.wait_for(2)[1], "rebuilt");
        assert_eq!(preview.current().number, 2);

        input.failing.store(true, Ordering::SeqCst);
        sender.send(changed("ghg.toml")).expect("sent");
        assert!(events.wait_for(3)[2].starts_with("not rebuilt: Input("));
        assert_eq!(preview.current().number, 2);

        drop(sender);
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
    assert_eq!(
        *watcher.watched.lock().expect("lock"),
        [
            Watched::Folder(PathBuf::from("features")),
            Watched::File(PathBuf::from("ghg.toml"))
        ]
    );
}

#[test]
fn a_first_build_that_stops_serves_nothing() {
    let input = Arc::new(Input::default());
    input.failing.store(true, Ordering::SeqCst);
    let server = Arc::new(Server::default());
    let service = PreviewService::new(
        Arc::new(NoFile),
        site(&input),
        output(OutputContents::Missing),
        reading(&input),
        Arc::new(Recorder::default()),
        server.clone(),
        Arc::new(Watcher::default()),
    );
    for mode in [ServeMode::Build, ServeMode::Watch] {
        let options = ServeOptions {
            mode,
            ..ServeOptions::default()
        };
        let outcome = service.serve(&options, words(), &Recorded::default());
        assert!(matches!(outcome, Err(ServeError::Input(_))), "{outcome:?}");
    }
    assert!(server.started.lock().expect("lock").is_none());
}

#[test]
fn bursts_that_wait_need_one_rebuild_and_the_defaults_are_127_0_0_1_at_8000() {
    let input = Arc::new(Input::default());
    let server = Arc::new(Server::default());
    let watcher = Arc::new(Watcher {
        waiting: 3,
        ..Watcher::default()
    });
    let service = PreviewService::new(
        Arc::new(NoFile),
        site(&input),
        output(OutputContents::Missing),
        reading(&input),
        Arc::new(Recorder::default()),
        server.clone(),
        watcher.clone(),
    );
    let options = ServeOptions {
        mode: ServeMode::Watch,
        ..ServeOptions::default()
    };
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| service.serve(&options, words(), &events));
        assert_eq!(events.wait_for(2).len(), 2);
        drop(watcher.sender.lock().expect("lock").take());
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
    assert_eq!(
        *events.0.lock().expect("lock"),
        ["listening at 127.0.0.1:8000", "rebuilt"]
    );
    let (host, port, preview) = server
        .started
        .lock()
        .expect("lock")
        .clone()
        .expect("started");
    assert_eq!(
        (host.as_str(), port, preview.current().number),
        ("127.0.0.1", 8000, 2)
    );
}

#[test]
fn a_build_into_a_folder_ghg_did_not_build_serves_nothing_unless_it_is_in_memory() {
    let input = Arc::new(Input::default());
    let server = Arc::new(Server::default());
    let watcher = Arc::new(Watcher::default());
    let service = PreviewService::new(
        Arc::new(NoFile),
        site(&input),
        output(not_a_site()),
        reading(&input),
        Arc::new(Recorder::default()),
        server.clone(),
        watcher.clone(),
    );
    let outcome = service.serve(&ServeOptions::default(), words(), &Recorded::default());
    assert!(
        matches!(
            &outcome,
            Err(ServeError::Output(error)) if error.kind == OutputErrorKind::NotASite
        ),
        "{outcome:?}"
    );
    assert!(server.started.lock().expect("lock").is_none());
    let options = ServeOptions {
        mode: ServeMode::Watch,
        ..ServeOptions::default()
    };
    let events = Recorded::default();
    scope(|scope| {
        let serving = scope.spawn(|| service.serve(&options, words(), &events));
        assert_eq!(events.wait_for(1).len(), 1);
        drop(watcher.sender.lock().expect("lock").take());
        assert_eq!(serving.join().expect("no panic"), Ok(()));
    });
    assert!(server.started.lock().expect("lock").is_some());
}
