use crate::world::End;
use hearth_protocol_mmdebstrap::limits::Limits;
use skein_lib::Rng;
use std::fmt::Write;
#[must_use]
pub fn output(rng: &mut Rng, limits: &Limits) -> Vec<u8> {
    let mut text = String::new();
    for i in 0..rng.between(1, 12) {
        match rng.between(0, 9) {
            0 => text.push_str("I: downloading packages with apt...\n"),
            1 => text.push_str("Get:1 http://example.invalid/ forky InRelease\n"),
            2 => text.push_str("Conf aa (1 Debian [amd64])\n"),
            3 => writeln!(text, "Fetched {}.3 kB in 1s", rng.between(0, 1000)).expect("writing a String"),
            5 => text.push_str(std::str::from_utf8(&failures(rng)).expect("ASCII failures")),
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
    if bytes.is_empty() {
        return Vec::new();
    }
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

#[must_use]
pub fn failures(rng: &mut Rng) -> Vec<u8> {
    const LINES: &[&str] = &[
        "I: running apt-get update...",
        "E: setup failed: E: Unable to locate package denied-package",
        "E: apt-get update failed: process exited with 100",
        "W: hooklistener errored out",
        "I: PID 12 exited with exit code 1",
        "E: mmdebstrap failed to run",
        "Err:1 URI 404  Not Found",
        "E: URI does not have a Release file",
        "E: Failed to fetch URI Could not resolve host",
        "E: Failed to fetch URI Hash Sum mismatch",
        "E: No space left on device",
        "E: Unable to locate package absent-package",
        "E: Package 'virtual-package' has no installation candidate",
        "The following packages have unmet dependencies:",
        " aa : Depends: bb",
        "However the following packages replace it:",
        "  aa",
        "E: ignored diagnostic",
        "\tcontinued\rtext",
        "not admitted",
        " orphan",
        "dpkg: error: install failed",
        " continuation",
    ];
    let mut text = String::new();
    for _ in 0..rng.between(1, 32) {
        let at =
            usize::try_from(rng.between(0, u64::try_from(LINES.len() - 1).expect("short list"))).expect("small index");
        text.push_str(LINES[at]);
        text.push('\n');
    }
    text.into_bytes()
}

/// Valid selected fields around ignored, sometimes multi-chunk fields.
#[must_use]
pub fn index(rng: &mut Rng, limits: &Limits) -> Vec<u8> {
    let mut text = String::new();
    for at in 0..rng.between(1, 6) {
        writeln!(text, "Package: pkg{at}\nVersion: 1.{}-2\nArchitecture: amd64", rng.between(0, 99))
            .expect("String write");
        writeln!(text, "Filename: pool/main/p/pkg{at}/pkg{at}.deb\nSize: {}", rng.between(0, 1_000_000))
            .expect("String write");
        text.push_str("SHA256: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae\nDescription: ignored\n continuation\n");
        if rng.chance(100) {
            text.push_str("Provides: ");
            text.push_str(&"x".repeat(usize::try_from(limits.chunk).expect("chunk fits") + 17));
            text.push('\n');
        }
        text.push_str(if rng.chance(500) { " \t\n\n" } else { "\n" });
    }
    text.into_bytes()
}
