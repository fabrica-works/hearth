use super::fixtures;
use crate::{limits, worst_case};

#[test]
fn a_larger_limit_gives_a_larger_worst_case() {
    let mut limits = limits::SHIPPED;
    let smaller = worst_case(&limits).expect("shipped sum fits");
    limits.queue = fixtures::LARGER_QUEUE;
    assert!(worst_case(&limits).expect("larger sum fits") > smaller, "a queue slot retains more heap");
}
