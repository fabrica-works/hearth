use hearth_protocol_git::{
    limits::Limits,
    listing::{Entry, Mode},
    oid::Oid,
};
use skein_lib::Rng;
#[derive(Debug)]
pub struct Listing {
    pub stdout: Vec<u8>,
    pub entries: Box<[Entry]>,
}
/// Encode independently of the decoder, with git's mode, type and padding.
#[must_use]
pub fn listing(rng: &mut Rng, limits: &Limits) -> Listing {
    let mut stdout = Vec::new();
    let mut entries = Vec::new();
    let count = rng.between(0, u64::from(limits.entries.min(8)));
    let mut room = limits.paths_total;
    for _ in 0..count {
        if room == 0 || limits.path == 0 {
            break;
        }
        let class = rng.below(4);
        let (mode, text, kind) = match class {
            0 => (Mode::File, "100644", "blob"),
            1 => (Mode::Executable, "100755", "blob"),
            2 => (Mode::Link, "120000", "blob"),
            _ => (Mode::Submodule, "160000", "commit"),
        };
        let length = u32::try_from(rng.between(1, u64::from(limits.path.min(room)))).expect("bounded path");
        let path: Vec<u8> = (0..length).map(|_| b'a' + u8::try_from(rng.below(26)).expect("alphabet")).collect();
        room -= length;
        let hex = if rng.chance(500) {
            "ce013625030ba8dba906f756967f9e9ca394464a"
        } else {
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        };
        let size = if mode == Mode::Submodule { 0 } else { rng.below(100_000_000) };
        let padded = if mode == Mode::Submodule { format!("{:>7}", "-") } else { format!("{size:>7}") };
        stdout.extend_from_slice(format!("{text} {kind} {hex} {padded}\t").as_bytes());
        stdout.extend_from_slice(&path);
        stdout.push(0);
        entries.push(Entry {
            path: path.into_boxed_slice(),
            mode,
            oid: Oid::parse(hex.as_bytes()).expect("literal id"),
            size,
        });
    }
    Listing { stdout, entries: entries.into_boxed_slice() }
}

#[derive(Debug)]
pub struct Batch {
    pub stdout: Vec<u8>,
    pub requests: Box<[hearth_protocol_git::batch::Request]>,
    pub blobs: Box<[crate::world::Blob]>,
}
/// Empty bodies, arbitrary binary bytes and missing ids, encoded independently.
#[must_use]
pub fn batch(rng: &mut Rng, limits: &Limits) -> Batch {
    let mut stdout = Vec::new();
    let mut requests = Vec::new();
    let mut blobs = Vec::new();
    for index in 0..rng.between(1, 8) {
        let hex = if rng.chance(500) { format!("{index:040x}") } else { format!("{index:064x}") };
        let oid = Oid::parse(hex.as_bytes()).expect("generated lowercase id");
        let length = rng.between(0, u64::from(limits.chunk) * 3);
        let missing = rng.chance(250);
        requests.push(hearth_protocol_git::batch::Request::Read { oid, size: length, max: length });
        if missing {
            stdout.extend_from_slice(format!("{hex} missing\n").as_bytes());
            blobs.push(crate::world::Blob { oid, bytes: None });
        } else {
            let bytes: Vec<u8> = (0..length).map(|_| u8::try_from(rng.below(256)).expect("byte")).collect();
            stdout.extend_from_slice(format!("{hex} blob {length}\n").as_bytes());
            stdout.extend_from_slice(&bytes);
            stdout.push(b'\n');
            blobs.push(crate::world::Blob { oid, bytes: Some(bytes.into_boxed_slice()) });
        }
    }
    Batch { stdout, requests: requests.into_boxed_slice(), blobs: blobs.into_boxed_slice() }
}

/// Vary pre-hunk headers independently of the literal change under test.
#[must_use]
pub fn hunks(rng: &mut skein_lib::Rng, change: &[u8]) -> Vec<u8> {
    let mut stdout =
        format!("diff --git a/{} b/{}\n--- a/file\n+++ b/file\n", rng.below(1000), rng.below(1000)).into_bytes();
    stdout.extend_from_slice(change);
    stdout
}
