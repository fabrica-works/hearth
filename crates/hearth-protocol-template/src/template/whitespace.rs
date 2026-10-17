//! Whitespace at delimiter edges (format.md, 7.2).
pub(crate) const WHITE_SPACE: &[&[u8]] = &[
    b"\t",
    b"\n",
    b"\x0b",
    b"\x0c",
    b"\r",
    b" ",
    b"\xc2\x85",
    b"\xc2\xa0",
    b"\xe1\x9a\x80",
    b"\xe2\x80\x80",
    b"\xe2\x80\x81",
    b"\xe2\x80\x82",
    b"\xe2\x80\x83",
    b"\xe2\x80\x84",
    b"\xe2\x80\x85",
    b"\xe2\x80\x86",
    b"\xe2\x80\x87",
    b"\xe2\x80\x88",
    b"\xe2\x80\x89",
    b"\xe2\x80\x8a",
    b"\xe2\x80\xa8",
    b"\xe2\x80\xa9",
    b"\xe2\x80\xaf",
    b"\xe2\x81\x9f",
    b"\xe3\x80\x80",
];
pub(super) fn leading(text: &[u8], mut at: usize) -> usize {
    for _ in 0..text.len().saturating_sub(at) {
        let rest = text.get(at..).expect("whitespace cursor");
        let mut width = 0;
        for &space in WHITE_SPACE {
            if rest.starts_with(space) {
                width = space.len();
                break;
            }
        }
        if width == 0 {
            break;
        }
        at = at.saturating_add(width);
    }
    at
}
pub(super) fn trailing(text: &[u8], start: usize, mut end: usize, newlines: bool) -> usize {
    for _ in 0..end.saturating_sub(start) {
        let rest = text.get(start..end).expect("whitespace range");
        let mut width = 0;
        for &space in WHITE_SPACE {
            if (newlines || (space != b"\r" && space != b"\n")) && rest.ends_with(space) {
                width = space.len();
                break;
            }
        }
        if width == 0 {
            break;
        }
        end = end.saturating_sub(width);
    }
    end
}
pub(super) fn before(text: &[u8], start: usize, at: usize, kind: u8, marker: Option<u8>) -> usize {
    if marker == Some(b'-') {
        return trailing(text, start, at, true);
    }
    if kind == b'{' || marker == Some(b'+') {
        return at;
    }
    let prefix = trailing(text, 0, at, false);
    if prefix == 0 || text.get(prefix.saturating_sub(1)) == Some(&b'\n') {
        trailing(text, start, at, false)
    } else {
        at
    }
}
pub(super) fn after(text: &[u8], at: usize, kind: u8, marker: Option<u8>) -> usize {
    if marker == Some(b'-') {
        leading(text, at)
    } else if kind != b'{' && marker != Some(b'+') && text.get(at) == Some(&b'\n') {
        at.saturating_add(1)
    } else {
        at
    }
}
