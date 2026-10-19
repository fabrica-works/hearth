use hearth_protocol_accounts::{File, Ids, Line, Password, limits::Limits};
use skein_lib::Rng;
use std::collections::BTreeSet;
/// A valid file, with the entries it must read to.
#[derive(Clone, Debug)]
pub struct Generated {
    pub bytes: Vec<u8>,
    pub lines: Vec<Line>,
}
const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
const NAME: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-$";
const SET: &[&[u8]] = &[b"", b"xx", b"!!!", b"$y$j9T$salt$hash", b"!$6$salt$hash", b"$6$", b"X"];
fn pick<'a>(rng: &mut Rng, from: &'a [u8]) -> &'a u8 {
    from.get(usize::try_from(rng.below(from.len() as u64)).expect("small index")).expect("index below the length")
}
fn name(rng: &mut Rng, limits: &Limits, taken: &BTreeSet<Vec<u8>>) -> Vec<u8> {
    loop {
        let most = if rng.chance(50) { u64::from(limits.name) } else { 12 };
        let length = rng.between(1, most);
        let mut name = vec![*pick(rng, FIRST)];
        for _ in 1..length {
            name.push(*pick(rng, NAME));
        }
        if !taken.contains(&name) {
            return name;
        }
    }
}
/// An uninterpreted field: any bytes but the separators.
fn free(rng: &mut Rng) -> Vec<u8> {
    let mut field = Vec::new();
    for _ in 0..rng.between(0, 20) {
        let byte = u8::try_from(rng.below(256)).expect("a byte");
        if byte != b':' && byte != b'\n' {
            field.push(byte);
        }
    }
    field
}
fn id(rng: &mut Rng) -> u32 {
    u32::try_from(match rng.below(8) {
        0 => 0,
        1 => 65_535,
        2..=4 => rng.below(1_000),
        _ => rng.below(65_536),
    })
    .expect("an id")
}
fn password(rng: &mut Rng, file: File) -> (Vec<u8>, Password) {
    let locked: &[&[u8]] = match file {
        File::Passwd | File::Group => &[b"x", b"*"],
        File::Shadow | File::Gshadow => &[b"*", b"!", b"!!", b"!*"],
    };
    if rng.chance(500) {
        let value = locked.get(usize::try_from(rng.below(locked.len() as u64)).expect("small")).expect("in range");
        return (value.to_vec(), Password::Locked);
    }
    if rng.chance(500) {
        let value = SET.get(usize::try_from(rng.below(SET.len() as u64)).expect("small")).expect("in range");
        return (value.to_vec(), Password::Set);
    }
    // A leading `$` is no locked value.
    let mut value = vec![b'$'];
    value.extend(free(rng));
    (value, Password::Set)
}
fn join(fields: &[Vec<u8>]) -> Vec<u8> {
    let mut line = Vec::new();
    for (at, field) in fields.iter().enumerate() {
        if at > 0 {
            line.push(b':');
        }
        line.extend_from_slice(field);
    }
    line.push(b'\n');
    line
}
/// A valid file of the `file` kind, from the generator's state.
#[must_use]
pub fn file(rng: &mut Rng, file: File, limits: &Limits) -> Generated {
    let count = if rng.chance(50) { rng.between(0, 600) } else { rng.between(0, 30) };
    let count = count.min(u64::from(limits.lines));
    let mut taken = BTreeSet::new();
    let mut bytes = Vec::new();
    let mut lines = Vec::new();
    for _ in 0..count {
        let name = name(rng, limits, &taken);
        taken.insert(name.clone());
        let (secret, password) = password(rng, file);
        let (ids, fields) = match file {
            File::Passwd => {
                let (uid, gid) = (id(rng), id(rng));
                let rest = vec![free(rng), free(rng), free(rng)];
                let mut fields = vec![name.clone(), secret, uid.to_string().into_bytes(), gid.to_string().into_bytes()];
                fields.extend(rest);
                (Ids::User { uid, gid }, fields)
            }
            File::Group => {
                let gid = id(rng);
                (Ids::Group { gid }, vec![name.clone(), secret, gid.to_string().into_bytes(), free(rng)])
            }
            File::Shadow => {
                let mut fields = vec![name.clone(), secret];
                for _ in 0..7 {
                    fields.push(free(rng));
                }
                (Ids::Shadow, fields)
            }
            File::Gshadow => (Ids::Shadow, vec![name.clone(), secret, free(rng), free(rng)]),
        };
        bytes.extend(join(&fields));
        lines.push(Line { name: name.into_boxed_slice(), ids, password });
    }
    // The last line may lack its newline.
    if !bytes.is_empty() && rng.chance(200) {
        bytes.pop();
    }
    Generated { bytes, lines }
}
const INTERESTING: &[u8] = b":\n\r0956#+- x\0\xff";
fn point(rng: &mut Rng, length: usize) -> usize {
    usize::try_from(rng.between(0, length as u64)).expect("bounded fixture value")
}
/// Cut, splice, flip bytes, duplicate lines, add colons: one to three of them.
#[must_use]
pub fn mutate(rng: &mut Rng, bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    for _ in 0..rng.between(1, 3) {
        let at = point(rng, bytes.len());
        match rng.below(6) {
            0 => {
                let stop = (at + point(rng, 40)).min(bytes.len());
                bytes.drain(at..stop);
            }
            1 => bytes.truncate(at),
            2 => {
                let stop = (at + point(rng, 60)).min(bytes.len());
                let piece = bytes[at..stop].to_vec();
                let to = point(rng, bytes.len());
                bytes.splice(to..to, piece);
            }
            3 => {
                for _ in 0..rng.between(1, 4) {
                    let at = point(rng, bytes.len());
                    if let Some(byte) = bytes.get_mut(at) {
                        *byte = if rng.chance(500) {
                            *pick(rng, INTERESTING)
                        } else {
                            u8::try_from(rng.below(256)).expect("a byte")
                        };
                    }
                }
            }
            4 => {
                let start = bytes[..at].iter().rposition(|b| *b == b'\n').map_or(0, |n| n + 1);
                let stop = bytes[at..].iter().position(|b| *b == b'\n').map_or(bytes.len(), |n| at + n + 1);
                let mut line = bytes[start..stop].to_vec();
                if !line.ends_with(b"\n") {
                    line.push(b'\n');
                }
                bytes.splice(stop..stop, line);
            }
            _ => {
                let colons = vec![b':'; usize::try_from(rng.between(1, 3)).expect("small")];
                bytes.splice(at..at, colons);
            }
        }
    }
    bytes
}
/// Limits small enough that a mutated file reaches them.
#[must_use]
pub fn small_limits(rng: &mut Rng) -> Limits {
    Limits {
        file: u32::try_from(rng.between(0, 2_000)).expect("small"),
        lines: u32::try_from(rng.between(0, 40)).expect("small"),
        name: u32::try_from(rng.between(0, 20)).expect("small"),
    }
}
