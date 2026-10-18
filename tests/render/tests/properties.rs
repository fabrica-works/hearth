use hearth_domain_render::vocabulary::Outcome;
use hearth_render_world::fixtures::properties as inputs;
use hearth_render_world::{fixtures, repositories::Edit};
#[test]
fn a_generated_repository_renders_without_errors() {
    for (case, generated) in inputs::cases() {
        let run = inputs::run(&generated, case);
        assert_eq!(run.outcome, Outcome::Done, "case {case}: {:?}", run.answers);
        assert_eq!(run.answers.len(), generated.model.targets.len(), "case {case}: every target renders");
    }
}
#[test]
fn a_reordered_object_changes_no_digest() {
    inputs::unchanged(&Edit::Reorder);
}
#[test]
fn reformatted_whitespace_changes_no_digest() {
    inputs::unchanged(&Edit::Respace);
}
#[test]
fn an_edited_note_changes_no_digest() {
    inputs::unchanged(&Edit::Note);
}
#[test]
fn every_seed_gives_the_same_digests_0() {
    inputs::deterministic(0);
}
#[test]
fn every_seed_gives_the_same_digests_1() {
    inputs::deterministic(1);
}
#[test]
fn every_seed_gives_the_same_digests_2() {
    inputs::deterministic(2);
}

#[test]
fn a_setting_only_its_own_template_reads_changes_its_target_alone() {
    inputs::locality(&Edit::Own(inputs::PROD_TARGET.into()));
}
#[test]
fn an_unread_setting_changes_no_digest() {
    inputs::locality(&Edit::Unread(inputs::PROD_TARGET.into()));
}
#[test]
fn an_exported_setting_changes_every_target_of_its_environment() {
    inputs::locality(&Edit::Exported(inputs::PROD_TARGET.into()));
}
#[test]
fn a_production_edit_never_changes_a_lab_spec() {
    for (case, generated) in inputs::cases() {
        let before = inputs::environment(&inputs::digests(&inputs::run(&generated, case)), inputs::LAB);
        for edit in inputs::production_edits() {
            let edited = inputs::edited(&generated, &edit, case);
            assert_eq!(
                inputs::environment(&inputs::digests(&inputs::run(&edited, case)), inputs::LAB),
                before,
                "case {case}: {edit:?}"
            );
        }
    }
}
#[test]
fn a_task_edit_changes_every_target_that_has_it() {
    inputs::locality(&Edit::Task);
}
#[test]
fn a_lab_environment_edit_leaves_every_production_spec() {
    inputs::locality(&Edit::Env(inputs::LAB.into()));
}
#[test]
fn a_mutated_repository_renders_its_healthy_targets() {
    for case in inputs::CASES {
        let (generated, mutated) = inputs::mutations(case);
        inputs::mutation_property(&generated, &mutated, case);
    }
}
#[test]
fn a_mutated_repository_gives_sorted_bounded_errors() {
    for case in inputs::CASES {
        let (_, mutated) = inputs::mutations(case);
        let run = inputs::run(&mutated, case);
        let errors = inputs::errors(&run);
        assert!(!errors.is_empty(), "case {case}");
        assert!(errors.len() <= usize::try_from(inputs::LIMITS.errors).expect("errors"), "case {case}");
        assert!(errors.iter().map(|error| inputs::error_key(error)).is_sorted(), "case {case}");
        assert_eq!(run.left_out, 0, "case {case}: all errors fit");
    }
}
#[test]
fn targets_asked_in_any_order_give_the_same_specs() {
    for (case, generated) in inputs::cases() {
        let repository = fixtures::repository_of(&generated.leaves);
        let digests = inputs::digests(&inputs::run(&generated, case));
        for order in inputs::ORDERS {
            let targets = inputs::ordered_targets(&generated, *order, case);
            assert_eq!(
                inputs::digests(&inputs::render(&repository, &targets, case)),
                digests,
                "case {case}: {order:?}"
            );
        }
    }
}
#[test]
fn a_collision_names_the_same_pair_whichever_task_arrives_first_0() {
    inputs::collision_order(0);
}
#[test]
fn a_collision_names_the_same_pair_whichever_task_arrives_first_1() {
    inputs::collision_order(1);
}
#[test]
fn a_collision_names_the_same_pair_whichever_task_arrives_first_2() {
    inputs::collision_order(2);
}

#[test]
fn a_mutated_repository_fails_the_call() {
    for case in inputs::CASES {
        let (_, mutated) = inputs::mutations(case);
        assert_eq!(inputs::run(&mutated, case).outcome, Outcome::Failed, "case {case}");
    }
}
