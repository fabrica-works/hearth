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
    bytes
}
