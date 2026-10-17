use hearth_protocol_template::{
    expression,
    limits::{self, Decoder},
};
use hearth_template_world::fixtures;
use skein_heap::{Counting, Meter, Span};

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

#[test]
fn compiling_templates_and_strings_at_the_limits_holds_no_more_than_the_worst_case() {
    let limits = &fixtures::LIMITS;
    let template_bound = limits::worst_case(Decoder::Template, limits).expect("template bound");
    let string_bound =
        limits::worst_case(Decoder::Interpolated { length: limits.template }, limits).expect("string bound");
    for (text, what, _) in fixtures::templates_at_the_limits(limits) {
        let span = Span::start();
        let compiled = hearth_protocol_template::template::compile(&text, limits);
        assert!(span.end().peak <= i64::try_from(template_bound).expect("bound fits i64"), "{what}");
        drop(compiled);
        let span = Span::start();
        let compiled = hearth_protocol_template::interpolation::compile(&text, limits);
        assert!(span.end().peak <= i64::try_from(string_bound).expect("bound fits i64"), "{what}");
        drop(compiled);
    }
}
