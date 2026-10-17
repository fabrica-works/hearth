use crate::{CACHE, Mirror, Pin};
use std::{fs, path::Path};

pub const PIN: &[u8] = b"# a pin\narchive 20261016T091523Z\nsuite forky\narchitecture amd64\ncomponents main\n\
digest ad093de033f89b590459503f94d25ea4a36fd044f7ce6d674f43e0d8412161f9\ntarget test-min apt\ntarget test-boot apt efibootmgr git\n";
pub const XZ_SHA256: &str = "b44f7d6b5283a44ee5f2bd98f84087a04810092122d75e8fbf8ad85f8f2981f1";
pub const SNAPSHOT: &str = "archive/debian/20261016T091523Z/dists/forky";
pub fn laid(dir: &Path) -> Mirror {
    let pin = Pin::read(PIN).unwrap();
    let root = dir.join(CACHE).join(&*pin.digest);
    let suite = root.join(SNAPSHOT);
    let index = suite.join("main/binary-amd64");
    fs::create_dir_all(index.join("by-hash/SHA256")).unwrap();
    fs::write(suite.join("InRelease"), b"signed\n").unwrap();
    fs::write(index.join("Packages.xz"), b"xz\n").unwrap();
    fs::hard_link(index.join("Packages.xz"), index.join("by-hash/SHA256").join(XZ_SHA256)).unwrap();
    let checked = root.join("checked/main/binary-amd64");
    fs::create_dir_all(&checked).unwrap();
    fs::write(checked.join("Packages"), b"index\n").unwrap();
    Mirror::find_in(dir, &pin).unwrap()
}

/// One complete GET, read until the proxy closes its connection.
pub fn get(proxy: &crate::Proxy, url: &str) -> Vec<u8> {
    request(proxy, &format!("GET {url} HTTP/1.1\r\nHost: snapshot.debian.org\r\n\r\n"))
}
pub fn request(proxy: &crate::Proxy, head: &str) -> Vec<u8> {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        time::Duration,
    };
    let mut stream = TcpStream::connect(("127.0.0.1", proxy.port())).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    stream.write_all(head.as_bytes()).unwrap();
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).unwrap();
    answer
}
