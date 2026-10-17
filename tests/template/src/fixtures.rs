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
pub const SMALL: Shape = Shape { width: 16 };
pub const WIDE: Shape = Shape { width: 260 };
#[must_use]
pub fn limits(rng: &mut Rng) -> Limits {
    if rng.below(4) != 0 {
        SHIPPED
    } else {
        Limits {
            expression: u32::try_from(rng.between(1, 256)).expect("bounded by fixture or checked text"),
            nesting: u32::try_from(rng.between(1, 8)).expect("bounded by fixture or checked text"),
        }
    }
}
#[must_use]
pub fn kind(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::TooLong => "long",
        ErrorKind::Character => "character",
        ErrorKind::Unterminated => "unterminated",
        ErrorKind::Escape => "escape",
        ErrorKind::Integer => "integer",
        ErrorKind::Expected(_) => "expected",
        ErrorKind::TooDeep => "deep",
        ErrorKind::Misplaced => "misplaced",
        ErrorKind::UnknownFilter => "filter",
        ErrorKind::Arity { .. } => "arity",
    }
}
pub const EVERY_KIND: &[&str] =
    &["long", "character", "unterminated", "escape", "integer", "expected", "deep", "misplaced", "filter", "arity"];
pub fn assert_fell(fell: &BTreeSet<&str>) {
    for kind in EVERY_KIND {
        assert!(fell.contains(kind), "missing {kind}");
    }
}
