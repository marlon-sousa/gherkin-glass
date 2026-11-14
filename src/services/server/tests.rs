use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Sender, channel};
use std::thread::{sleep, spawn};

use rstest::rstest;

use super::*;
use crate::entities::ServeMode;
use crate::errors::{ListenErrorKind, SignalsError};
use crate::test_support::{FakeServer, FakeSignals, FakeWatcher, changed};

fn service(server: &Arc<FakeServer>, watcher: &Arc<FakeWatcher>) -> ServerService {
    ServerService::new(
        server.clone(),
        watcher.clone(),
        Arc::new(FakeSignals::default()),
    )
}

/// Stop requests not asked yet, with their sender.
fn stops() -> (Sender<()>, StopRequests) {
    let (sender, requests) = channel();
    (sender, StopRequests::new(requests))
}

type Watch = for<'a> fn(&ServerService, &'a StopRequests) -> Result<WatchedChanges<'a>, WatchError>;

fn watch_the_input<'a>(
    server: &ServerService,
    stops: &'a StopRequests,
) -> Result<WatchedChanges<'a>, WatchError> {
    server.watch(Path::new("features"), &[PathBuf::from("ghg.toml")], stops)
}

fn watch_the_input_and_a_template<'a>(
    server: &ServerService,
    stops: &'a StopRequests,
) -> Result<WatchedChanges<'a>, WatchError> {
    server.watch(
        Path::new("features"),
        &[PathBuf::from("ghg.toml"), PathBuf::from("front.md")],
        stops,
    )
}

fn watch_the_configuration<'a>(
    server: &ServerService,
    stops: &'a StopRequests,
) -> Result<WatchedChanges<'a>, WatchError> {
    server.watch_config(Path::new("ghg.toml"), stops)
}

#[rstest]
// Rule 65: the input and the configuration file are watched, or the configuration file alone,
// and every burst waiting is merged into one.
#[case::the_input_and_the_configuration(
    watch_the_input,
    &[Watched::Folder(PathBuf::from("features")), Watched::File(PathBuf::from("ghg.toml"))]
)]
// Rule 191: and the front page's template beside the configuration file.
#[case::the_front_page_s_template_too(
    watch_the_input_and_a_template,
    &[
        Watched::Folder(PathBuf::from("features")),
        Watched::File(PathBuf::from("ghg.toml")),
        Watched::File(PathBuf::from("front.md")),
    ]
)]
#[case::the_configuration_alone(watch_the_configuration, &[Watched::File(PathBuf::from("ghg.toml"))])]
fn the_bursts_waiting_are_handed_back_merged(#[case] watch: Watch, #[case] watched: &[Watched]) {
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let (_asker, stops) = stops();
    let mut changes = watch(&server, &stops).expect("watching");
    assert_eq!(*watcher.watched.lock().expect("lock"), watched);
    let sender = watcher.sender().expect("watching");
    sender.send(changed("features/b.feature")).expect("sent");
    sender.send(changed("features/a.feature")).expect("sent");
    let paths = ["features/a.feature", "features/b.feature"].map(PathBuf::from);
    assert_eq!(changes.next(), Some(InputChanges::Paths(paths.into())));
    sender.send(changed("ghg.toml")).expect("sent");
    assert_eq!(changes.next(), Some(InputChanges::Everything));
    drop(sender);
    assert_eq!(changes.next(), None);
}

#[test]
fn changes_missed_while_starting_are_everything() {
    // Rule 65.
    let watcher = Arc::new(FakeWatcher {
        waiting: 3,
        ..FakeWatcher::default()
    });
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let (_asker, stops) = stops();
    let mut changes = watch_the_input(&server, &stops).expect("watching");
    drop(watcher.sender());
    assert_eq!(changes.next(), Some(InputChanges::Everything));
    assert_eq!(changes.next(), None);
}

#[test]
fn a_watch_that_cannot_start_says_where() {
    // Rule 68.
    let watcher = Arc::new(FakeWatcher {
        failing: true,
        ..FakeWatcher::default()
    });
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let (_asker, stops) = stops();
    let error = watch_the_input(&server, &stops).err();
    assert_eq!(
        error,
        Some(WatchError {
            path: PathBuf::from("features"),
            reason: "too many folders".into(),
        })
    );
}

#[test]
fn a_site_is_served_at_127_0_0_1_port_8000_unless_told_otherwise() {
    // Rule 58.
    let server = Arc::new(FakeServer::default());
    let service = service(&server, &Arc::new(FakeWatcher::default()));
    let site = ServedSite::Folder(PathBuf::from("site"));
    let served = service
        .serve(site.clone(), &ServeOptions::default())
        .expect("served");
    assert_eq!(served.listening.address.to_string(), "127.0.0.1:8000");
    let (host, port, preview) = server.started().expect("started");
    assert_eq!((host.as_str(), port), ("127.0.0.1", 8000));
    assert_eq!(preview.current().site, site);
    assert_eq!(preview.mode, ServeMode::Build);
}

#[test]
fn a_port_in_use_is_named_with_its_host() {
    // Rule 59.
    let server = Arc::new(FakeServer {
        taken: true,
        ..FakeServer::default()
    });
    let service = service(&server, &Arc::new(FakeWatcher::default()));
    let options = ServeOptions {
        host: Some("0.0.0.0".into()),
        port: Some(8123),
        ..ServeOptions::default()
    };
    let error = service
        .serve(ServedSite::Folder(PathBuf::from("site")), &options)
        .err();
    assert_eq!(
        error,
        Some(ServeError::Listen {
            host: "0.0.0.0".into(),
            port: 8123,
            kind: ListenErrorKind::PortTaken
        })
    );
}

#[test]
fn the_changes_end_once_ghg_serve_is_asked_to_stop_while_it_waits() {
    // Rule 196: under --watch, a request to stop ends the wait for the next change.
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let (asker, stops) = stops();
    let mut changes = watch_the_input(&server, &stops).expect("watching");
    let _sender = watcher.sender();
    let asking = spawn(move || {
        sleep(STOP_LOOKED_FOR_EVERY * 2);
        asker.send(()).expect("sent");
    });
    assert_eq!(changes.next(), None);
    asking.join().expect("no panic");
}

#[test]
fn a_change_waiting_once_asked_to_stop_is_left() {
    // Rule 197: a request to stop that came during a rebuild ends the watch, not another rebuild.
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let (asker, stops) = stops();
    let mut changes = watch_the_input(&server, &stops).expect("watching");
    let sender = watcher.sender().expect("watching");
    sender.send(changed("features/a.feature")).expect("sent");
    asker.send(()).expect("sent");
    assert_eq!(changes.next(), None);
}

#[rstest]
// Rules 196 and 198: the requests to stop are listened for, or what stops listening is said.
#[case::listened_for(false, None)]
#[case::refused(true, Some(ServeError::Signals(SignalsError { reason: "not permitted".into() })))]
fn the_requests_to_stop_are_listened_for_or_their_refusal_said(
    #[case] failing: bool,
    #[case] error: Option<ServeError>,
) {
    let signals = Arc::new(FakeSignals::default());
    signals.failing.store(failing, Ordering::SeqCst);
    let server = ServerService::new(
        Arc::new(FakeServer::default()),
        Arc::new(FakeWatcher::default()),
        signals.clone(),
    );
    let requests = server.stop_requests();
    assert_eq!(requests.as_ref().err(), error.as_ref());
    if let Ok(requests) = requests {
        assert!(!requests.asked());
        signals.ask_to_stop();
        assert!(requests.asked());
    }
}

#[test]
fn a_change_that_comes_with_a_request_to_stop_is_left() {
    // Rule 196: a change that comes during the same wait as a request to stop starts no rebuild.
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let (asker, stops) = stops();
    let mut changes = watch_the_input(&server, &stops).expect("watching");
    let sender = watcher.sender().expect("watching");
    let asking = spawn(move || {
        sleep(STOP_LOOKED_FOR_EVERY / 4);
        asker.send(()).expect("sent");
        sender.send(changed("features/a.feature")).expect("sent");
        sender
    });
    assert_eq!(changes.next(), None);
    drop(asking.join().expect("no panic"));
}
