use hearth_protocol_template::{
    ErrorKind,
    limits::{Limits, SHIPPED},
};
use skein_lib::Rng;
use std::collections::BTreeSet;
pub const LIMITS: Limits = SHIPPED;
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub width: u32,
}
pub const SMALL_TEMPLATES: Shape = Shape { width: 8 };
pub const WIDE_TEMPLATES: Shape = Shape { width: 48 };
pub const TEMPLATE_KINDS: &[&str] = &[
    "long",
    "character",
    "unterminated",
    "escape",
    "integer",
    "expected",
    "deep",
    "misplaced",
    "filter",
    "arity",
    "test",
    "utf8",
    "return",
    "unclosed",
    "statement",
];
pub const SMALL: Shape = Shape { width: 16 };
pub const WIDE: Shape = Shape { width: 260 };
#[must_use]
pub fn limits(rng: &mut Rng) -> Limits {
    if rng.below(4) != 0 {
        SHIPPED
    } else {
        Limits {
            expression: u32::try_from(rng.between(1, 256)).expect("bounded by fixture or checked text"),
            template: u32::try_from(rng.between(64, 4096)).expect("tiny template limit"),
            nesting: u32::try_from(rng.between(1, 8)).expect("bounded by fixture or checked text"),
        }
    }
}
#[must_use]
pub fn kind(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Utf8 => "utf8",
        ErrorKind::Return => "return",
        ErrorKind::Unclosed => "unclosed",
        ErrorKind::UnknownStatement => "statement",
        ErrorKind::TooLong => "long",
        ErrorKind::Character => "character",
        ErrorKind::Unterminated => "unterminated",
        ErrorKind::Escape => "escape",
        ErrorKind::Integer => "integer",
        ErrorKind::Expected(_) => "expected",
        ErrorKind::TooDeep => "deep",
        ErrorKind::Misplaced => "misplaced",
        ErrorKind::UnknownFilter => "filter",
        ErrorKind::UnknownTest => "test",
        ErrorKind::Arity { .. } => "arity",
    }
}
pub const EVERY_KIND: &[&str] = &[
    "long",
    "character",
    "unterminated",
    "escape",
    "integer",
    "expected",
    "deep",
    "misplaced",
    "filter",
    "arity",
    "test",
];
pub fn assert_fell(fell: &BTreeSet<&str>, kinds: &[&str]) {
    for kind in kinds {
        assert!(fell.contains(kind), "missing {kind}");
    }
}

type LimitSpec =
    (&'static str, &'static str, usize, &'static str, &'static str, &'static str, &'static str, Result<(), u32>);
pub type LimitText = (Vec<u8>, &'static str, Result<(), u32>);
// Head, repeated opening, count, middle, repeated closing, tail, label, outcome.
const LIMIT_TEXTS: &[LimitSpec] = &[
    ("[0", ",0", 2046, "", "", "] ", "4096-byte list", Ok(())),
    ("[0", ",0", 2046, "", "", "]  ", "one byte past the list", Err(4096)),
    ("", "(", 64, "a", ")", "", "64 groups", Ok(())),
    ("", "(", 65, "a", ")", "", "65 groups", Err(64)),
    ("", "[", 64, "a", "]", "", "64 lists", Ok(())),
    ("", "[", 65, "a", "]", "", "65 lists", Err(64)),
    ("", "a[", 64, "a", "]", "", "64 indices", Ok(())),
    ("", "a[", 65, "a", "]", "", "65 indices", Err(129)),
    ("", "not ", 64, "a", "", "", "64 negations", Ok(())),
    ("", "not ", 65, "a", "", "", "65 negations", Err(256)),
    ("", "a if c else ", 64, "b", "", "", "64 else branches", Ok(())),
    ("", "a if c else ", 65, "b", "", "", "65 else branches", Err(770)),
    ("'", "a", 4094, "", "", "'", "longest string", Ok(())),
    ("'", "a", 4095, "", "", "'", "one byte past the string", Err(4096)),
    ("a", ".a", 2047, "", "", " ", "longest chain", Ok(())),
    ("a", ".a", 2048, "", "", "", "one byte past the chain", Err(4096)),
    ("", "a", 4096, "", "", "", "longest name", Ok(())),
    ("", "a", 4097, "", "", "", "one byte past the name", Err(4096)),
    ("['a'", ",'a'", 1022, "", "", "]   ", "many retained strings", Ok(())),
];
#[must_use]
pub fn at_the_limits() -> Vec<LimitText> {
    LIMIT_TEXTS
        .iter()
        .map(|&(head, open, count, middle, close, tail, what, outcome)| {
            let text = [
                head.as_bytes(),
                open.repeat(count).as_bytes(),
                middle.as_bytes(),
                close.repeat(count).as_bytes(),
                tail.as_bytes(),
            ]
            .concat();
            (text, what, outcome)
        })
        .collect()
}
