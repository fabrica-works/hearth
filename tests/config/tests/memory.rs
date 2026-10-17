use hearth_config_world::fixtures;
use hearth_protocol_config::{json, limits::Limits};
use skein_heap::Counting;
#[path = "../src/fixtures/metered.rs"]
mod measured;
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn each_step_holds_no_more_than_the_worst_case_under_tiny_limits() {
    let limits = Limits { document: 4096, depth: 4, keys: 8, settings: 8 };
    let bound = json::worst_case(&limits).expect("valid limits");
    for (document, what) in fixtures::at_the_limits(&limits) {
        assert!(measured::metered(&document, &limits, bound, what) > 0, "{what}");
    }
}

#[test]
fn declarations_hold_no_more_than_their_worst_case_at_the_limits() {
    // Enough depth to exercise nested type frames, with cheap object and setting limits.
    let limits = Limits { document: 4096, depth: 16, keys: 8, settings: 8 };
    let bound = hearth_protocol_config::declaration::worst_case(&limits).expect("valid limits");
    for (document, what) in fixtures::declarations_at_the_limits(&limits) {
        assert!(measured::metered_declaration(&document, &limits, bound, what) > 0, "{what}");
    }
}
