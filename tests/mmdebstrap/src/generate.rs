use crate::world::End;
use hearth_protocol_mmdebstrap::limits::Limits;
use skein_lib::Rng;
use std::fmt::Write;
#[must_use]
pub fn output(rng: &mut Rng, limits: &Limits) -> Vec<u8> {
    let mut text = String::new();
    for i in 0..rng.between(1, 12) {
        match rng.between(0, 7) {
            0 => text.push_str("I: downloading packages with apt...\n"),
            1 => text.push_str("Get:1 http://example.invalid/ forky InRelease\n"),
            2 => text.push_str("Conf aa (1 Debian [amd64])\n"),
            3 => writeln!(text, "Fetched {}.3 kB in 1s", rng.between(0, 1000)).expect("writing a String"),
            4 => {
                text.push_str(&"x".repeat(usize::try_from(limits.line).expect("bounded fixture value") + 5));
                text.push('\n');
            }
            _ => {
                let installed = if rng.chance(500) { "[0.1] " } else { "" };
                let origins = [
                    "Debian:testing",
                    "Debian:testing, Debian:unstable",
                    "Debian:testing, Debian:unstable, Local:stable",
                ];
                let origin = origins[usize::try_from(rng.between(0, 2)).expect("bounded fixture value")];
                let suffix = if rng.chance(500) { " [aa :amd64]" } else { "" };
                writeln!(text, "Inst pkg{i} {installed}(1.2-3 {origin} [amd64]){suffix}").expect("writing a String");
            }
        }
    }
    text.into_bytes()
}
#[must_use]
pub fn mutate(rng: &mut Rng, bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    let at = usize::try_from(rng.between(0, u64::try_from(bytes.len()).expect("bounded fixture value").max(1)))
        .expect("bounded fixture value");
    match rng.between(0, 4) {
        0 => {
            let stop = bytes[at..].iter().position(|b| *b == b'\n').map_or(bytes.len(), |n| at + n + 1);
            bytes.drain(at..stop);
        }
        1 => bytes.truncate(at),
        2 => {
            let line = bytes[..at].to_vec();
            bytes.splice(at..at, line);
        }
        _ => {
            if let Some(byte) = bytes.get_mut(at) {
                *byte = u8::try_from(rng.between(0, 255)).expect("bounded fixture value");
            }
        }
    }
    bytes
}
#[must_use]
pub fn end(rng: &mut Rng, bytes: &[u8]) -> End {
    let at = usize::try_from(rng.between(0, u64::try_from(bytes.len()).expect("bounded fixture value").max(1)))
        .expect("bounded fixture value");
    match rng.between(0, 5) {
        0 => End::Cut(at),
        1 => End::Break(at),
        _ => End::Whole,
    }
}
