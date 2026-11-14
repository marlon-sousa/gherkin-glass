//! Adapter: the preview server port over HTTP, with tiny_http.

mod answering;
mod content_type;
mod reload_injection;
mod request_route;

use std::fs::read;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::entities::{Listening, Preview, ServedSite, ServerWords, Version};
use crate::errors::ListenErrorKind;
use crate::ports::PreviewServer;
use answering::Answering;
use content_type::{content_type, is_html};
use reload_injection::add_reload_script;
use request_route::{Route, route};

/// The reload script, added to every page under `--watch`.
const RELOAD_SCRIPT: &str = include_str!("preview_http/reload.js");

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct HttpPreviewServer;

impl PreviewServer for HttpPreviewServer {
    fn start(
        &self,
        host: &str,
        port: u16,
        preview: Arc<Preview>,
        words: ServerWords,
    ) -> Result<Listening, ListenErrorKind> {
        let bind = || {
            TcpListener::bind((host, port)).map_err(|error| match error.kind() {
                ErrorKind::AddrInUse => ListenErrorKind::PortTaken,
                _ => ListenErrorKind::Other {
                    reason: error.to_string(),
                },
            })
        };
        let mut listener = bind()?;
        // Only once binding shows the address is this machine's, and with
        // the port free again, so that the probe cannot reach ghg itself.
        if port != 0 {
            let bound = listener.local_addr().ok();
            drop(listener);
            if bound.is_some_and(someone_listens) {
                return Err(ListenErrorKind::PortTaken);
            }
            listener = bind()?;
        }
        let address = listener
            .local_addr()
            .map_err(|error| ListenErrorKind::Other {
                reason: error.to_string(),
            })?;
        let server = Arc::new(Server::from_listener(listener, None).map_err(|error| {
            ListenErrorKind::Other {
                reason: error.to_string(),
            }
        })?);
        Ok(Listening {
            address,
            answering: Box::new(Answering::start(server, preview, Arc::new(words))),
        })
    }
}

/// Whether another server already answers at `address`, one of this
/// machine's. Binding alone does not tell: on macOS and the BSDs, the
/// `SO_REUSEADDR` the standard library sets lets a socket bind `127.0.0.1`
/// at a port another listens at on `0.0.0.0`, and Windows allows the same.
fn someone_listens(mut address: SocketAddr) -> bool {
    // Windows refuses a connection to the address that means every network.
    if address.ip().is_unspecified() {
        address.set_ip(match address {
            SocketAddr::V4(_) => IpAddr::V4(Ipv4Addr::LOCALHOST),
            SocketAddr::V6(_) => IpAddr::V6(Ipv6Addr::LOCALHOST),
        });
    }
    TcpStream::connect_timeout(&address, Duration::from_millis(250)).is_ok()
}

/// The HTTP status of an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
enum Status {
    Ok = 200,
    MovedPermanently = 301,
    NotFound = 404,
    MethodNotAllowed = 405,
}

/// What a request is answered with, before it is sent.
struct Answer {
    status: Status,
    content_type: String,
    body: Vec<u8>,
    location: Option<String>,
}

impl Answer {
    fn ok(content_type: String, body: Vec<u8>) -> Self {
        Answer {
            status: Status::Ok,
            content_type,
            body,
            location: None,
        }
    }

    fn text(status: Status, sentence: &str) -> Self {
        Answer {
            status,
            content_type: "text/plain; charset=utf-8".into(),
            body: format!("{sentence}\n").into_bytes(),
            location: None,
        }
    }

    fn redirect(location: String) -> Self {
        Answer {
            status: Status::MovedPermanently,
            content_type: "text/plain; charset=utf-8".into(),
            body: Vec::new(),
            location: Some(location),
        }
    }
}

fn answer(request: Request, preview: &Preview, words: &ServerWords) {
    let answer = match request.method() {
        Method::Get | Method::Head => choose(request.url(), preview, words),
        _ => Answer::text(Status::MethodNotAllowed, &words.wrong_method),
    };
    let mut response = Response::from_data(answer.body)
        .with_status_code(StatusCode(answer.status as u16))
        .with_header(header("Content-Type", &answer.content_type))
        .with_header(header("Cache-Control", "no-cache"));
    if answer.status == Status::MethodNotAllowed {
        response = response.with_header(header("Allow", "GET, HEAD"));
    }
    if let Some(location) = answer.location {
        response = response.with_header(header("Location", &location));
    }
    // A browser that went away needs no answer.
    let _ = request.respond(response);
}

fn choose(url: &str, preview: &Preview, words: &ServerWords) -> Answer {
    let not_found = || Answer::text(Status::NotFound, &words.not_found);
    match route(url, preview.mode) {
        Route::NotFound => not_found(),
        Route::BuildNumber => Answer::ok(
            "text/plain; charset=utf-8".into(),
            preview.current().number.to_string().into_bytes(),
        ),
        Route::ReloadScript => Answer::ok(
            "text/javascript; charset=utf-8".into(),
            RELOAD_SCRIPT.as_bytes().to_vec(),
        ),
        Route::File {
            address,
            folder_redirect,
        } => {
            let version = preview.current();
            match find(&version, &address) {
                Found::File(body) => {
                    let content_type = content_type(&address);
                    let body = if preview.mode.watches() && is_html(&content_type) {
                        add_reload_script(&body, version.number)
                    } else {
                        body
                    };
                    Answer::ok(content_type, body)
                }
                Found::Folder => match folder_redirect {
                    Some(location) => Answer::redirect(location),
                    None => not_found(),
                },
                Found::Nothing => not_found(),
            }
        }
    }
}

enum Found {
    File(Vec<u8>),
    Folder,
    Nothing,
}

fn find(version: &Version, address: &str) -> Found {
    match &version.site {
        ServedSite::Memory(files) => {
            if let Some(body) = files.get(address) {
                return Found::File(body.to_vec());
            }
            let folder = format!("{address}/");
            match files.range(folder.clone()..).next() {
                Some((file, _)) if file.starts_with(&folder) => Found::Folder,
                _ => Found::Nothing,
            }
        }
        ServedSite::Folder(root) => {
            let path = address
                .split('/')
                .fold(root.clone(), |path, part| path.join(part));
            if path.is_dir() {
                Found::Folder
            } else {
                read(&path).map_or(Found::Nothing, Found::File)
            }
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "every header the server sends is ASCII without line breaks"
)]
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("a header's name and value are ASCII without line breaks")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::thread::sleep;

    use super::*;
    use crate::entities::ServeMode;

    fn words() -> ServerWords {
        ServerWords {
            not_found: "not found".into(),
            wrong_method: "wrong method".into(),
        }
    }

    fn memory(mode: ServeMode) -> Preview {
        let files = BTreeMap::from([
            ("a/index.html".to_string(), Arc::from(*b"<p>a</p></body>")),
            ("a/s.js".to_string(), Arc::from(*b"</body>")),
        ]);
        let preview = Preview::new(mode, ServedSite::Memory(BTreeMap::new()));
        preview.replace(ServedSite::Memory(files));
        preview
    }

    #[test]
    fn under_watch_a_page_has_the_script_of_its_version_and_nothing_else_does() {
        let preview = memory(ServeMode::Watch);
        let page = choose("/a/", &preview, &words());
        assert_eq!(page.status, Status::Ok);
        assert_eq!(
            page.body,
            br#"<p>a</p><script type="module" src="/__ghg/reload.js?build=2"></script></body>"#
        );
        assert_eq!(choose("/a/s.js", &preview, &words()).body, b"</body>");
        assert_eq!(choose("/__ghg/build", &preview, &words()).body, b"2");
        let script = choose("/__ghg/reload.js?build=2", &preview, &words());
        assert_eq!(script.content_type, "text/javascript; charset=utf-8");
        assert_eq!(script.body, RELOAD_SCRIPT.as_bytes());
    }

    #[test]
    fn without_watch_a_page_is_as_built() {
        let preview = memory(ServeMode::Build);
        assert_eq!(choose("/a/", &preview, &words()).body, b"<p>a</p></body>");
        assert_eq!(
            choose("/__ghg/build", &preview, &words()).status,
            Status::NotFound
        );
    }

    #[test]
    fn a_folder_in_memory_is_redirected_and_a_missing_file_is_not_found() {
        let preview = memory(ServeMode::Watch);
        let folder = choose("/a?x", &preview, &words());
        assert_eq!(folder.status, Status::MovedPermanently);
        assert_eq!(folder.location.as_deref(), Some("/a/?x"));
        let missing = choose("/a/b.html", &preview, &words());
        assert_eq!(
            (missing.status, missing.body),
            (Status::NotFound, b"not found\n".to_vec())
        );
        assert_eq!(choose("/b/", &preview, &words()).status, Status::NotFound);
        assert_eq!(choose("/a/s", &preview, &words()).status, Status::NotFound);
    }

    #[test]
    fn once_what_it_returns_is_dropped_the_server_is_closed() {
        let preview = Arc::new(memory(ServeMode::Build));
        let listening = HttpPreviewServer
            .start("127.0.0.1", 0, preview, words())
            .expect("listening");
        let address = listening.address;
        let mut stream = TcpStream::connect(address).expect("connects");
        stream
            .write_all(b"GET /a/ HTTP/1.1\r\nConnection: close\r\n\r\n")
            .expect("sent");
        let mut answer = String::new();
        stream.read_to_string(&mut answer).expect("answered");
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        drop(listening);
        // tiny_http 0.12.0 closes its listening socket on a thread of its own, soon after.
        let closed = (0..100).any(|_| {
            sleep(Duration::from_millis(20));
            TcpStream::connect(address).is_err()
        });
        assert!(closed);
    }
}
