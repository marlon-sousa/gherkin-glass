use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use rstest::rstest;

use super::*;
use crate::entities::{Burst, ServeMode};
use crate::errors::ListenErrorKind;
use crate::test_support::{FakeServer, FakeWatcher, changed};

fn service(server: &Arc<FakeServer>, watcher: &Arc<FakeWatcher>) -> ServerService {
    ServerService::new(server.clone(), watcher.clone())
}

#[rstest]
// Rule 65.
#[case::the_configuration("ghg.toml", true)]
#[case::a_file_the_rules_select("features/a.feature", true)]
#[case::a_file_they_leave_out("features/notes.txt", false)]
fn only_a_change_a_build_reads_calls_for_a_rebuild(#[case] path: &str, #[case] rebuilds: bool) {
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let selected = |_: &Path| vec![PathBuf::from("features/a.feature")];
    let mut found = server
        .watch(Path::new("features"), Path::new("ghg.toml"), &selected)
        .expect("watching");
    let sender = watcher.sender().expect("watching");
    sender.send(changed(path)).expect("sent");
    drop(sender);
    let expected = rebuilds.then(|| vec![changed(path)]);
    assert_eq!(found.next(), expected);
    assert_eq!(found.next(), None);
}

#[test]
fn the_input_and_the_configuration_are_watched_and_walked_before_each_burst() {
    let watcher = Arc::new(FakeWatcher::default());
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let walks = AtomicUsize::new(0);
    let selected = |_: &Path| {
        walks.fetch_add(1, Ordering::SeqCst);
        Vec::new()
    };
    let rebuilds = server
        .watch(Path::new("features"), Path::new("ghg.toml"), &selected)
        .expect("watching");
    let sender = watcher.sender().expect("watching");
    sender.send(changed("features/notes.txt")).expect("sent");
    drop(sender);
    assert_eq!(rebuilds.count(), 0);
    assert_eq!(
        *watcher.watched.lock().expect("lock"),
        [
            Watched::Folder(PathBuf::from("features")),
            Watched::File(PathBuf::from("ghg.toml"))
        ]
    );
    // Once when watching starts, and once for the burst.
    assert_eq!(walks.load(Ordering::SeqCst), 2);
}

#[test]
fn bursts_already_waiting_call_for_one_rebuild() {
    // Rule 65: an editor's burst, or several, is one rebuild.
    let watcher = Arc::new(FakeWatcher {
        waiting: 3,
        ..FakeWatcher::default()
    });
    let server = service(&Arc::new(FakeServer::default()), &watcher);
    let selected = |_: &Path| Vec::new();
    let mut rebuilds = server
        .watch(Path::new("features"), Path::new("ghg.toml"), &selected)
        .expect("watching");
    drop(watcher.sender());
    assert_eq!(rebuilds.next(), Some(vec![Burst::Missed; 3]));
    assert_eq!(rebuilds.next(), None);
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
