use hearth_protocol_mmdebstrap::limits::{self, Limits};
pub const LIMITS: Limits = limits::SHIPPED;
#[must_use]
pub fn at_the_limits(limits: &Limits) -> Vec<u8> {
    let name = "a".repeat(usize::try_from(limits.name).expect("bounded fixture value"));
    let mut bytes = format!(
        "Inst {name} ({} Debian [{name}])\n",
        "v".repeat(usize::try_from(limits.name).expect("bounded fixture value"))
    )
    .into_bytes();
    bytes.extend(std::iter::repeat_n(b'x', usize::try_from(limits.line).expect("bounded fixture value") + 1));
    bytes.push(b'\n');
    for _ in 0..limits.tail_lines {
        bytes.extend_from_slice(
            format!("E: {}\n", "x".repeat(usize::try_from(limits.tail_line).expect("tail size fits"))).as_bytes(),
        );
    }
    bytes.extend_from_slice(format!("E: Unable to locate package {name}\n").as_bytes());
    bytes
}

/// All selected payloads reach their limits; an ignored field fills the stanza.
#[must_use]
pub fn index_at_the_limits(limits: &Limits) -> Vec<u8> {
    let name = "a".repeat(usize::try_from(limits.name).expect("name fits"));
    let filename = "f".repeat(usize::try_from(limits.filename).expect("filename fits"));
    let mut text = format!(
        "Package: {name}\nVersion: {name}\nArchitecture: {name}\nFilename: {filename}\nSize: 18446744073709551615\nSHA256: {}\nProvides: ",
        "f".repeat(64)
    );
    let ignored = usize::try_from(limits.stanza)
        .expect("stanza fits")
        .checked_sub(text.len() + 2)
        .expect("room for ignored field");
    text.push_str(&"x".repeat(ignored));
    text.push_str("\n\n");
    text.into_bytes()
}
