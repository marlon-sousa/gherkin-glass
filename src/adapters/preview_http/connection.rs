//! Adapter: one connection to ghg serve, its request read with httparse and answered once.

use std::io::{self, BufRead, BufReader, Read, Write, copy, sink};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

use http::{Method, StatusCode};

use super::{Answer, answer};
use crate::entities::{Preview, ServerWords};

/// The most bytes a request's line and headers may take.
const HEAD_LIMIT: u64 = 16 * 1024;
/// The most headers a request may have.
const HEADERS_LIMIT: usize = 64;
/// How long a connection may send nothing, before its request, in it or after its answer,
/// before it is closed.
pub(super) const PATIENCE: Duration = Duration::from_secs(10);

/// Answers the one request `stream` sends, then closes it.
pub(super) fn answer_connection(stream: TcpStream, preview: &Preview, words: &ServerWords) {
    // A connection that sends no request, or what is not one, is closed unanswered.
    let Some((method, target)) = read_request(&stream) else {
        return;
    };
    let answer = answer(&method, &target, preview, words);
    // A browser that went away needs no answer.
    if write_answer(&stream, &answer, &method).is_ok() && stream.shutdown(Shutdown::Write).is_ok() {
        // What the browser still sends, such as a body, is read and left aside until it
        // closes the connection or waits too long, since closing it first could lose the
        // answer.
        let _ = copy(&mut &stream, &mut sink());
    }
}

/// The method and target of the request `stream` sends.
fn read_request(stream: &TcpStream) -> Option<(Method, String)> {
    stream.set_read_timeout(Some(PATIENCE)).ok()?;
    let mut reader = BufReader::new(stream.take(HEAD_LIMIT));
    let mut head = Vec::new();
    loop {
        if reader.read_until(b'\n', &mut head).ok()? == 0 {
            return None;
        }
        let mut headers = [httparse::EMPTY_HEADER; HEADERS_LIMIT];
        let mut request = httparse::Request::new(&mut headers);
        if request.parse(&head).ok()?.is_complete() {
            let method = Method::from_bytes(request.method?.as_bytes()).ok()?;
            return Some((method, request.path?.to_owned()));
        }
    }
}

/// Writes `answer` to a request of `method`, without its body for `HEAD`.
fn write_answer(mut stream: &TcpStream, answer: &Answer, method: &Method) -> io::Result<()> {
    let mut head = Vec::new();
    write!(head, "HTTP/1.1 {}\r\n", answer.status)?;
    write!(head, "Content-Type: {}\r\n", answer.content_type)?;
    write!(head, "Content-Length: {}\r\n", answer.body.len())?;
    write!(head, "Cache-Control: no-cache\r\nConnection: close\r\n")?;
    if answer.status == StatusCode::METHOD_NOT_ALLOWED {
        write!(head, "Allow: GET, HEAD\r\n")?;
    }
    if let Some(location) = &answer.location {
        write!(head, "Location: {location}\r\n")?;
    }
    head.extend_from_slice(b"\r\n");
    if *method != Method::HEAD {
        head.extend_from_slice(&answer.body);
    }
    stream.write_all(&head)
}
