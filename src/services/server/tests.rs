use std::path::PathBuf;

use rstest::rstest;

use super::*;
use crate::entities::ServeMode;
use crate::errors::ListenErrorKind;
use crate::test_support::{FakeServer, FakeWatcher, changed};

fn service(server: &Arc<FakeServer>, watcher: &Arc<FakeWatcher>) -> ServerService {
    ServerService::new(server.clone(), watcher.clone())
}

#[rstest]
// Rule 65: the input and the configuration file are watched, or the configuration file alone,
// and every burst waiting is merged into one.
#[case::the_input_and_the_configuration(
    true,
    &[Watched::Folder(PathBuf::from("features")), Watched::File(PathBuf::from("ghg.toml"))]
)]
#[case::the_configuration_alone(false, &[Watched::File(PathBuf::from("ghg.toml"))])]
fn the_bursts_waiting_are_handed_back_merged(#[case] input: bool, #[case] watched: &[Watched]) {
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let config = Path::new("ghg.toml");
    let mut changes = if input {
        server.watch(Path::new("features"), config)
    } else {
        server.watch_config(config)
    }
    .expect("watching");
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
    let mut changes = server
        .watch(Path::new("features"), Path::new("ghg.toml"))
        .expect("watching");
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
    let error = server
        .watch(Path::new("features"), Path::new("ghg.toml"))
        .err();
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
    assert_eq!(served.address.to_string(), "127.0.0.1:8000");
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
