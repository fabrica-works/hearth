use hearth_protocol_template::{
    expression,
    limits::{self, Decoder},
};
use hearth_template_world::fixtures;
use skein_heap::{Counting, Meter};

#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn compiling_at_the_limits_holds_no_more_than_the_worst_case() {
    let bound = limits::worst_case(Decoder::Expression, &fixtures::LIMITS).expect("a worst case");
    for (text, what, _) in fixtures::at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let compiled = expression::compile(&text, &fixtures::LIMITS);
        meter.check(meter.end(), bound, &what);
        drop(compiled);
    }
}
