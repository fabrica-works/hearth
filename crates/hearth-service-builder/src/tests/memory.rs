use super::fixtures;
use crate::{limits, worst_case};

#[test]
fn the_shipped_limits_have_a_worst_case_within_1_gib() {
    let limits = limits::SHIPPED;
    let bytes = worst_case(&limits).expect("shipped sum fits");
    assert!(bytes <= fixtures::BUILDER_BYTES, "shipped worst case is {bytes} bytes");
}

#[test]
fn a_larger_limit_gives_a_larger_worst_case() {
    let mut limits = limits::SHIPPED;
    let smaller = worst_case(&limits).expect("shipped sum fits");
    limits.queue = fixtures::LARGER_QUEUE;
    assert!(worst_case(&limits).expect("larger sum fits") > smaller, "a queue slot retains more heap");
}
