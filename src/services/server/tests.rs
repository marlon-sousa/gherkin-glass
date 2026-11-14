use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

use rstest::rstest;

use super::*;
use crate::entities::ServeMode;
use crate::errors::{ListenErrorKind, SignalsError};
use crate::test_support::{FakeServer, FakeSignals, FakeWaiter, FakeWatcher, changed};

/// The service's fakes, which a test can look at and script.
#[derive(Default)]
struct Fakes {
    server: Arc<FakeServer>,
    watcher: Arc<FakeWatcher>,
    signals: Arc<FakeSignals>,
    waiter: Arc<FakeWaiter>,
}

impl Fakes {
    fn service(&self) -> ServerService {
        ServerService::new(
            self.server.clone(),
            self.watcher.clone(),
            self.signals.clone(),
            self.waiter.clone(),
        )
    }

    /// Has each wait for a change or a request to stop that finds no burst told end as `script` says.
    fn script(&self, script: impl IntoIterator<Item = ChangeOrStop>) {
        lock(&self.waiter.script).extend(script);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

type Watch = fn(&ServerService) -> Result<WatchedChanges, WatchError>;

fn watch_the_input(server: &ServerService) -> Result<WatchedChanges, WatchError> {
    server.watch(Path::new("features"), &[PathBuf::from("ghg.toml")])
}

fn watch_the_input_and_a_template(server: &ServerService) -> Result<WatchedChanges, WatchError> {
    server.watch(
        Path::new("features"),
        &[PathBuf::from("ghg.toml"), PathBuf::from("front.md")],
    )
}

fn watch_the_configuration(server: &ServerService) -> Result<WatchedChanges, WatchError> {
    server.watch_config(Path::new("ghg.toml"))
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
    let fakes = Fakes::default();
    let server = fakes.service();
    let mut changes = watch(&server).expect("watching");
    assert_eq!(*lock(&fakes.watcher.watched), watched);
    fakes.watcher.tell(changed("features/b.feature"));
    fakes.watcher.tell(changed("features/a.feature"));
    let paths = ["features/a.feature", "features/b.feature"].map(PathBuf::from);
    assert_eq!(changes.next(), Some(InputChanges::Paths(paths.into())));
    fakes.watcher.tell(changed("ghg.toml"));
    assert_eq!(changes.next(), Some(InputChanges::Everything));
    assert_eq!(changes.next(), None);
}

#[test]
fn changes_missed_while_starting_are_everything() {
    // Rule 65.
    let fakes = Fakes {
        watcher: Arc::new(FakeWatcher {
            waiting: 3,
            ..FakeWatcher::default()
        }),
        ..Fakes::default()
    };
    let mut changes = watch_the_input(&fakes.service()).expect("watching");
    assert_eq!(changes.next(), Some(InputChanges::Everything));
    assert_eq!(changes.next(), None);
}

#[test]
fn a_watch_that_cannot_start_says_where() {
    // Rule 68.
    let fakes = Fakes {
        watcher: Arc::new(FakeWatcher {
            failing: true,
            ..FakeWatcher::default()
        }),
        ..Fakes::default()
    };
    let error = watch_the_input(&fakes.service()).err();
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
    let fakes = Fakes::default();
    let service = fakes.service();
    let site = ServedSite::Folder(PathBuf::from("site"));
    let served = service
        .serve(site.clone(), &ServeOptions::default())
        .expect("served");
    assert_eq!(served.listening.address.to_string(), "127.0.0.1:8000");
    let (host, port, preview) = fakes.server.started().expect("started");
    assert_eq!((host.as_str(), port), ("127.0.0.1", 8000));
    assert_eq!(preview.current().site, site);
    assert_eq!(preview.mode, ServeMode::Build);
}

#[test]
fn a_port_in_use_is_named_with_its_host() {
    // Rule 59.
    let fakes = Fakes {
        server: Arc::new(FakeServer {
            taken: true,
            ..FakeServer::default()
        }),
        ..Fakes::default()
    };
    let service = fakes.service();
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

#[rstest]
// Rule 199: under --watch, a request to stop ends the wait for the next change, as the end of
// the watching does.
#[case::asked_to_stop(ChangeOrStop::Stop)]
#[case::the_watching_ended(ChangeOrStop::WatchingEnded)]
fn the_changes_end_once_ghg_serve_is_asked_to_stop_or_the_watching_ends(#[case] end: ChangeOrStop) {
    let fakes = Fakes::default();
    fakes.script([
        ChangeOrStop::Changed(vec![changed("features/a.feature")]),
        end,
    ]);
    let mut changes = watch_the_input(&fakes.service()).expect("watching");
    let paths = [PathBuf::from("features/a.feature")];
    assert_eq!(changes.next(), Some(InputChanges::Paths(paths.into())));
    assert_eq!(changes.next(), None);
}

#[rstest]
// Rules 199 and 201: the requests to stop are listened for, or what stops listening is said.
#[case::listened_for(false, None)]
#[case::refused(true, Some(ServeError::Signals(SignalsError { reason: "not permitted".into() })))]
fn the_requests_to_stop_are_listened_for_or_their_refusal_said(
    #[case] failing: bool,
    #[case] error: Option<ServeError>,
) {
    let fakes = Fakes::default();
    fakes.signals.failing.store(failing, Ordering::SeqCst);
    let server = fakes.service();
    assert_eq!(server.listen_for_stops().err(), error);
    if error.is_none() {
        assert!(!server.asked_to_stop());
        fakes.signals.ask_to_stop();
        assert!(server.asked_to_stop());
        server.wait_for_stop();
    }
}

#[rstest]
// Rule 202: once asked to stop, another request ends ghg at once, and can cut short the wait for
// the requests being answered, since the site served is dropped only then; without a request,
// the watch having ended alone, nothing is ended at once.
#[case::once_asked(true, Some(0))]
#[case::not_asked(false, None)]
fn stopping_has_another_request_end_ghg_before_it_waits(
    #[case] asked: bool,
    #[case] ending: Option<usize>,
) {
    let no_longer_served = Arc::new(AtomicUsize::new(0));
    let fakes = Fakes {
        server: Arc::new(FakeServer {
            no_longer_served: no_longer_served.clone(),
            ..FakeServer::default()
        }),
        signals: Arc::new(FakeSignals {
            no_longer_served: no_longer_served.clone(),
            ..FakeSignals::default()
        }),
        ..Fakes::default()
    };
    let server = fakes.service();
    server.listen_for_stops().expect("listening");
    let served = server
        .serve(
            ServedSite::Folder(PathBuf::from("site")),
            &ServeOptions::default(),
        )
        .expect("served");
    if asked {
        fakes.signals.ask_to_stop();
    }
    server.stop(served);
    assert_eq!(*lock(&fakes.signals.ending), ending);
    assert_eq!(no_longer_served.load(Ordering::SeqCst), 1);
}
