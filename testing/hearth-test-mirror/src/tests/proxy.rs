use super::fixtures;
use crate::{Method, Proxy, Served, target};
use skein_scratch::Scratch;
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    time::Duration,
};

const URL: &str = "http://snapshot.debian.org/archive/debian/20261016T091523Z/dists/forky/InRelease";
const FILE: &str = "archive/debian/20261016T091523Z/dists/forky/InRelease";
const FOUND: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nsigned\n";
const HEAD_ONLY: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\n";
const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
const XZ: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nxz\n";
const TARGETS: &[(&str, Method, &str)] = &[("GET", Method::Get, FILE), ("HEAD", Method::Head, FILE)];
#[test]
fn a_snapshot_request_names_its_path_below_the_origin() {
    for &(verb, method, path) in TARGETS {
        let head = format!("{verb} {URL} HTTP/1.1\r\n\r\n");
        assert_eq!(target(head.as_bytes()), Some((method, PathBuf::from(path))), "{verb}");
    }
}
const REFUSED: &[&str] = &[
    "GET http://elsewhere.test/archive/file HTTP/1.1\r\n\r\n",
    "GET /archive/file HTTP/1.1\r\n\r\n",
    "POST http://snapshot.debian.org/archive/file HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/archive/file?x HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/archive/%61 HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/archive/file#x HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org/checked/file HTTP/1.1\r\n\r\n",
    "GET http://snapshot.debian.org:80/archive/file HTTP/1.1\r\n\r\n",
];
#[test]
fn another_host_origin_form_or_another_method_is_not_found() {
    for head in REFUSED {
        assert_eq!(target(head.as_bytes()), None, "{head}");
    }
}
#[test]
fn the_proxy_serves_a_laid_mirror_over_loopback() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).expect("a loopback port");
    assert_eq!(fixtures::get(&proxy, URL), FOUND);
}
#[test]
fn a_by_hash_request_is_answered_the_file_it_names() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).unwrap();
    let url = format!(
        "http://snapshot.debian.org/{}/main/binary-amd64/by-hash/SHA256/{}",
        fixtures::SNAPSHOT,
        fixtures::XZ_SHA256
    );
    assert_eq!(fixtures::get(&proxy, &url), XZ);
}
#[test]
fn a_head_request_is_answered_the_head_alone() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).unwrap();
    let head = format!("HEAD {URL} HTTP/1.1\r\nRange: bytes=0-1\r\nIf-None-Match: anything\r\n\r\n");
    assert_eq!(fixtures::request(&proxy, &head), HEAD_ONLY);
}
const LOGGED: &[(&str, u16, u64)] = &[(URL, 200, 7), ("/missing", 404, 0)];
#[test]
fn each_answer_is_logged_before_its_connection_closes() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).unwrap();
    for &(url, status, length) in LOGGED {
        fixtures::get(&proxy, url);
        assert_eq!(proxy.served(), [Served { target: url.as_bytes().into(), status, length }], "{url}");
    }
    assert_eq!(proxy.served(), []);
}
#[test]
fn a_stalled_head_is_answered_404() {
    let scratch = Scratch::new("test-mirror-proxy");
    let mirror = fixtures::laid(scratch.path());
    let proxy = Proxy::with_timeout(&mirror, Duration::from_millis(50)).unwrap();
    let mut stream = TcpStream::connect(("127.0.0.1", proxy.port())).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    stream.write_all(format!("GET {URL} HTTP/1.1\r\n").as_bytes()).unwrap();
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).unwrap();
    assert_eq!(answer, NOT_FOUND);
}
const REFUSED_FILES: &[&str] = &[
    "http://snapshot.debian.org/checked/main/binary-amd64/Packages",
    "http://snapshot.debian.org/archive/../checked/main/binary-amd64/Packages",
    "http://snapshot.debian.org/archive/missing",
];
#[test]
fn an_unavailable_or_unserved_file_is_answered_404() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).unwrap();
    for url in REFUSED_FILES {
        assert_eq!(fixtures::get(&proxy, url), NOT_FOUND, "{url}");
    }
}
#[test]
fn a_head_past_the_limit_is_answered_404() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).unwrap();
    let mut head = format!("GET {URL} HTTP/1.1\r\nX: ");
    head.extend(std::iter::repeat_n('a', crate::HEAD - head.len()));
    assert_eq!(fixtures::request(&proxy, &head), NOT_FOUND);
}
