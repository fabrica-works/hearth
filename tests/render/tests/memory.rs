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
    fixtures::memory::each(bound, fixtures::at_the_limits(), |expression, roots| {
        let mut evaluator = Evaluator::new(&limits::SHIPPED);
        let mut budget = Budget::new(&limits::SHIPPED);
        let (context, bindings) = (roots.context(), Bindings::with_capacity(0));
        let evaluated = evaluate::expression(&mut evaluator, expression, &context, &bindings, &mut budget);
        drop(evaluated);
        drop(evaluator);
    });
}

#[test]
fn running_templates_at_the_limits_holds_no_more_than_the_worst_case() {
    let bound = evaluate::template::worst_case(&limits::SHIPPED).expect("the shipped run has a worst case");
    fixtures::memory::each(bound, fixtures::templates_at_the_limits(), |template, roots| {
        let rendered = evaluate::template::run(
            template,
            &roots.context(),
            None,
            &limits::SHIPPED,
            u64::from(limits::SHIPPED.rendered_file),
        );
        drop(rendered);
    });
}

#[test]
fn expanding_at_the_limits_holds_no_more_than_the_worst_case() {
    let limits = fixtures::EXPANSION_LIMITS;
    let bound = hearth_domain_render::entries::worst_case(&limits).expect("expansion bound");
    fixtures::memory::each(bound, fixtures::entries_at_the_limits(), |declaration, roots| {
        let expanded =
            hearth_domain_render::entries::expand(declaration, &roots.context(), &limits, &mut { limits.expanded });
        drop(expanded);
    });
}

#[test]
fn collecting_errors_at_the_limits_holds_no_more_than_the_worst_case() {
    use hearth_domain_render::errors::Collector;
    let bound = Collector::worst_case(&limits::SHIPPED).expect("the shipped collector has a worst case");
    let meter = Meter::new();
    meter.start();
    let mut collector = Collector::new(&limits::SHIPPED);
    let step = meter.end();
    meter.check(step, bound, &"empty collector");
    for index in 0..limits::SHIPPED.errors {
        meter.start();
        collector.push(fixtures::error_at_budget(fixtures::ERROR_RECORD_BYTES));
        let step = meter.end();
        meter.check(step, bound, &index);
    }
    meter.start();
    collector.push(fixtures::error_at_budget(fixtures::ERROR_FIELDS_BYTES));
    let step = meter.end();
    meter.check(step, bound, &"first excluded error");
}

#[test]
fn decoding_at_the_limits_holds_no_more_than_the_worst_case() {
    let repository = fixtures::decoded_templates_repository();
    fixtures::checked_decoding(&repository, fixtures::decoded_bound(), &fixtures::newline_template());
}

#[test]
fn rendering_every_target_holds_one_targets_settings() {
    let repository = fixtures::settings::every_target();
    let bound = hearth_domain_render::compose::worst_case(&fixtures::settings::LIMITS).expect("composition bound");
    let run = hearth_render_world::parent::checked(
        &repository,
        fixtures::settings::MEMORY_SEED,
        fixtures::settings::LIMITS,
        bound,
        skein_world::Memory::Checked,
    );
    assert_eq!(run.outcome, hearth_domain_render::vocabulary::Outcome::Done);
    assert_eq!(run.answers.len(), fixtures::settings::LIMITS.targets as usize);
}

#[test]
fn decoding_comment_only_templates_holds_no_more_than_the_worst_case() {
    let repository = fixtures::decoded_comments_repository();
    fixtures::checked_decoding(&repository, fixtures::decoded_bound(), &fixtures::comment_template());
}

#[test]
fn rendering_specs_at_the_limits_holds_no_more_than_the_worst_case() {
    let repository = fixtures::specs::repository();
    let run = hearth_render_world::parent::checked(
        &repository,
        fixtures::specs::MEMORY_SEED,
        fixtures::specs::LIMITS,
        fixtures::specs::bound(),
        skein_world::Memory::Checked,
    );
    assert_eq!(run.outcome, hearth_domain_render::vocabulary::Outcome::Failed);
    assert!(
        run.answers.iter().any(|answer| matches!(answer, hearth_domain_render::vocabulary::Answer::Spec(_))),
        "the leading specs fit before the next target fails"
    );
}
