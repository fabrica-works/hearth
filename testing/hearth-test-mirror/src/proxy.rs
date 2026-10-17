use crate::Mirror;
use std::{
    fs::File,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

/// The mirror on a loopback port, served by one thread (testing.md, 6.2).
#[derive(Debug)]
pub struct Proxy {
    port: u16,
    log: Receiver<Served>,
}
/// One answer, logged before close: request target, status, and body length.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Served {
    pub target: Box<[u8]>,
    pub status: u16,
    pub length: u64,
}
/// The GET and HEAD methods, which are answered with a file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Method {
    Get,
    Head,
}
/// The longest request head, including its blank line.
pub const HEAD: usize = 8_192;
/// The only origin served by this proxy.
pub const ORIGIN: &[u8] = b"http://snapshot.debian.org/";
impl Proxy {
    pub fn serve(mirror: &Mirror) -> io::Result<Self> {
        Self::with_timeout(mirror, Duration::from_secs(10))
    }
    pub(crate) fn with_timeout(mirror: &Mirror, timeout: Duration) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        let mirror = mirror.clone();
        let (send, log) = mpsc::channel();
        thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut stream) = connection else { break };
                answer(&mut stream, &mirror, &send, timeout);
            }
        });
        Ok(Self { port, log })
    }
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }
    /// The proxy URL, as apt's `Acquire::http::Proxy` takes it.
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }
    /// Answers since the previous call, in order.
    #[must_use]
    pub fn served(&self) -> Vec<Served> {
        self.log.try_iter().collect()
    }
}
/// A GET or HEAD target below the fixed origin, without a query or escapes.
#[must_use]
pub fn target(head: &[u8]) -> Option<(Method, PathBuf)> {
    let line = head.split(|byte| *byte == b'\r' || *byte == b'\n').next()?;
    let mut words = line.split(|byte| *byte == b' ');
    let method = match words.next()? {
        b"GET" => Method::Get,
        b"HEAD" => Method::Head,
        _ => return None,
    };
    let url = words.next()?;
    if !matches!(words.next()?, b"HTTP/1.0" | b"HTTP/1.1") || words.next().is_some() {
        return None;
    }
    let path = url.strip_prefix(ORIGIN)?;
    if !path.starts_with(b"archive/") || path.iter().any(|b| matches!(b, b'?' | b'%' | b'#')) {
        return None;
    }
    Some((method, PathBuf::from(std::str::from_utf8(path).ok()?)))
}
fn read_head(stream: &mut TcpStream, head: &mut Vec<u8>, timeout: Duration) -> io::Result<bool> {
    stream.set_read_timeout(Some(timeout))?;
    let mut buffer = [0; HEAD];
    while head.len() < HEAD {
        let read = stream.read(&mut buffer[..HEAD - head.len()])?;
        if read == 0 {
            return Ok(false);
        }
        head.extend_from_slice(&buffer[..read]);
        if let Some(end) = head.windows(4).position(|window| window == b"\r\n\r\n") {
            head.truncate(end + 4);
            return Ok(true);
        }
    }
    Ok(false)
}
fn answer(stream: &mut TcpStream, mirror: &Mirror, log: &Sender<Served>, timeout: Duration) {
    let mut head = Vec::new();
    let complete = read_head(stream, &mut head, timeout).unwrap_or(false);
    let requested = head.split(|byte| *byte == b' ').nth(1).unwrap_or(b"").into();
    let found = complete.then(|| target(&head)).flatten().and_then(|(method, path)| {
        let file = File::open(mirror.file(&path)?).ok()?;
        let length = file.metadata().ok()?.len();
        Some((method, file, length))
    });
    let (status, length) = if let Some((method, mut file, length)) = found {
        let sent = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n");
        if stream.write_all(sent.as_bytes()).is_ok() && method == Method::Get {
            let _result = io::copy(&mut file, stream);
        }
        (200, length)
    } else {
        let _result = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        (404, 0)
    };
    let _result = log.send(Served { target: requested, status, length });
}
