use hearth_domain_render::vocabulary::Outcome;
use hearth_render_world::fixtures::properties as inputs;
use hearth_render_world::repositories::Edit;
#[test]
fn a_generated_repository_renders_without_errors() {
    for case in inputs::CASES {
        let generated = inputs::generated(case);
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
