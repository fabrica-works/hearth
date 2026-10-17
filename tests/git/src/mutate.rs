//! Damage records at framing and object-id boundaries.
#[must_use]
pub fn all(stdout: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut start = 0;
    for (at, byte) in stdout.iter().enumerate() {
        if *byte == 0 || *byte == b'\n' {
            let end = at + 1;
            for cut in [start + 1, start + (end - start) / 2, end.saturating_sub(1)] {
                result.push(stdout[..cut].to_vec());
            }
            start = end;
        }
    }
    for at in 0..stdout.len() {
        if (at == 0 || !stdout[at - 1].is_ascii_hexdigit()) && stdout[at].is_ascii_hexdigit() {
            let length = stdout[at..].iter().take_while(|b| b.is_ascii_hexdigit()).count();
            if length == 40 || length == 64 {
                let mut bytes = stdout.to_vec();
                bytes[at] = b'A';
                result.push(bytes);
            }
        }
    }
    result.push(stdout[..stdout.len().saturating_sub(1)].to_vec());
    result.sort();
    result.dedup();
    result
}
