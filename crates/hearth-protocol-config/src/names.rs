//! Names in configuration documents (format.md, 2.4).
#[must_use]
pub fn identifier(name: &[u8]) -> bool {
    let Some(first) = name.first() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'_' {
            return false;
        }
    }
    true
}

/// A base, environment or member name (format.md, 2.4).
#[must_use]
pub fn member(name: &[u8]) -> bool {
    let Some(first) = name.first() else {
        return false;
    };
    if (!first.is_ascii_lowercase() && !first.is_ascii_digit()) || name.last() == Some(&b'-') {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
