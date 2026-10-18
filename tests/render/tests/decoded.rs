use hearth_domain_render::vocabulary::{Answer, Outcome};
use hearth_render_world::fixtures;

#[test]
fn four_large_documents_render_within_the_decoded_budget() {
    let run = fixtures::large_decoded_run(false);
    assert_eq!(run.outcome, Outcome::Done);
    let Answer::Spec(spec) = fixtures::only(&run.answers) else { panic!("one spec") };
    assert_eq!(spec.meta.target.as_ref(), b"x1-prod");
}

const DECODED_BOUND: hearth_domain_render::errors::Kind =
    hearth_domain_render::errors::Kind::Decode(hearth_domain_render::errors::Decode::TooLarge {
        limit: hearth_domain_render::errors::Limit::Decoded,
        max: fixtures::DECODED_LIMITS.decoded,
    });
#[test]
fn the_fifth_large_document_refuses_its_revision_path() {
    let run = fixtures::large_decoded_run(true);
    assert_eq!(
        run.answers.as_slice(),
        &[Answer::Error(fixtures::configuration_error(DECODED_BOUND, b"envs/e.json", None))]
    );
}
