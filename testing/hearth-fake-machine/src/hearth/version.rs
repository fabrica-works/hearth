//! Debian's version ordering (Policy 5.6.12).
use core::cmp::Ordering;

#[must_use]
pub fn compare(a: &[u8], b: &[u8]) -> Ordering {
    let (ae, au, ar) = parts(a);
    let (be, bu, br) = parts(b);
    number(ae, be).then_with(|| component(au, bu)).then_with(|| component(ar, br))
}
fn parts(bytes: &[u8]) -> (&[u8], &[u8], &[u8]) {
    let (epoch, rest) = split(bytes, b':', false).unwrap_or((b"0", bytes));
    let (upstream, revision) = split(rest, b'-', true).unwrap_or((rest, b"0"));
    (epoch, upstream, revision)
}
fn number(a: &[u8], b: &[u8]) -> Ordering {
    let a = zeroes(a);
    let b = zeroes(b);
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}
fn zeroes(bytes: &[u8]) -> &[u8] {
    let count = bytes.iter().take_while(|byte| **byte == b'0').count();
    bytes.get(count..).expect("leading zeroes are in the slice")
}
pub(super) fn split(bytes: &[u8], separator: u8, last: bool) -> Option<(&[u8], &[u8])> {
    let at =
        if last { bytes.iter().rposition(|b| *b == separator)? } else { bytes.iter().position(|b| *b == separator)? };
    let (left, right) = bytes.split_at_checked(at)?;
    Some((left, right.get(1..)?))
}
fn order(byte: Option<&u8>) -> i32 {
    match byte {
        Some(b'~') => -1,
        None | Some(b'0'..=b'9') => 0,
        Some(value) if value.is_ascii_alphabetic() => i32::from(*value),
        Some(value) => i32::from(*value).checked_add(256).expect("one byte plus 256"),
    }
}
fn component(mut a: &[u8], mut b: &[u8]) -> Ordering {
    for _ in 0..=a.len().saturating_add(b.len()) {
        if a.is_empty() && b.is_empty() {
            return Ordering::Equal;
        }
        if a.first().is_some_and(|x| !x.is_ascii_digit()) || b.first().is_some_and(|x| !x.is_ascii_digit()) {
            let result = order(a.first()).cmp(&order(b.first()));
            if result != Ordering::Equal {
                return result;
            }
            a = a.get(1..).unwrap_or_default();
            b = b.get(1..).unwrap_or_default();
        } else {
            let an = a.iter().take_while(|x| x.is_ascii_digit()).count();
            let bn = b.iter().take_while(|x| x.is_ascii_digit()).count();
            let (ad, rest_a) = a.split_at_checked(an).expect("digit prefix");
            let (bd, rest_b) = b.split_at_checked(bn).expect("digit prefix");
            let result = number(ad, bd);
            if result != Ordering::Equal {
                return result;
            }
            a = rest_a;
            b = rest_b;
        }
    }
    unreachable!("each pass consumes at least one byte")
}
