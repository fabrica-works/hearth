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
