use hearth_domain_render::{
    evaluate::{self, Bindings, Budget, Evaluator},
    limits,
};
use hearth_render_world::fixtures;
use skein_heap::{Counting, Meter};

#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn evaluating_at_the_limits_holds_no_more_than_the_worst_case() {
    let bound = evaluate::worst_case(&limits::SHIPPED).expect("the shipped limits have a worst case");
    let roots = fixtures::wide_roots();
    for (expression, what) in fixtures::at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let mut evaluator = Evaluator::new(&limits::SHIPPED);
        let mut budget = Budget::new(&limits::SHIPPED);
        let (context, bindings) = (roots.context(), Bindings::with_capacity(0));
        let evaluated = evaluate::expression(&mut evaluator, &expression, &context, &bindings, &mut budget);
        drop(evaluated);
        drop(evaluator);
        let step = meter.end();
        meter.check(step, bound, &what);
    }
}

#[test]
fn running_templates_at_the_limits_holds_no_more_than_the_worst_case() {
    let bound = evaluate::template::worst_case(&limits::SHIPPED).expect("the shipped run has a worst case");
    let roots = fixtures::wide_roots();
    for (template, what) in fixtures::templates_at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let rendered = evaluate::template::run(
            &template,
            &roots.context(),
            &limits::SHIPPED,
            u64::from(limits::SHIPPED.rendered_file),
        );
        drop(rendered);
        let step = meter.end();
        meter.check(step, bound, &what);
    }
}

#[test]
fn expanding_at_the_limits_holds_no_more_than_the_worst_case() {
    let limits = fixtures::EXPANSION_LIMITS;
    let bound = hearth_domain_render::entries::worst_case(&limits).expect("expansion bound");
    let roots = fixtures::wide_roots();
    for (declaration, what) in fixtures::entries_at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let expanded =
            hearth_domain_render::entries::expand(&declaration, &roots.context(), &limits, &mut { limits.expanded });
        drop(expanded);
        let step = meter.end();
        meter.check(step, bound, &what);
    }
}
