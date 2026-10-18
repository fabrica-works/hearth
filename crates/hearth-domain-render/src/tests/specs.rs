use super::fixtures::{self, specs};
const KNOWN: crate::call::resolution::Known = crate::call::resolution::Known { digest: fixtures::KEY_A, closure: None };
#[test]
fn a_spec_past_the_budget_fails_its_target() {
    let story = specs::render();
    assert_eq!(fixtures::kept(&story.call.side).as_ref(), &[specs::limit_error(None)]);
    assert_eq!(
        story.call.side.made.get(b"x1-prod".as_slice()),
        Some(&crate::diff::Made::Failed(crate::vocabulary::Class::Configuration))
    );
}
#[test]
fn a_spec_past_a_diffs_budget_fails_that_side() {
    let story = specs::diff();
    assert_eq!(fixtures::kept(&story.call.side).as_ref(), &[specs::limit_error(Some(crate::diff::Side::To))]);
    assert_eq!(story.failures.as_ref(), &[(skein_lib::bytes::copy_of(b"x1-prod"), crate::diff::Side::To)]);
}

#[test]
fn the_shipped_specs_budget_is_64_mib() {
    assert_eq!(crate::limits::SHIPPED.specs, specs::SHIPPED_SPEC_BYTES);
}

#[test]
fn a_render_keeps_only_a_found_closures_digest() {
    let mut side = specs::finding(false);
    side.found(Some(fixtures::diff_record(fixtures::KEY_A, fixtures::changed_closures().before)), false);
    assert_eq!(side.finding.record(&fixtures::KEY_A), Some(&KNOWN));
}
#[test]
fn a_diff_keeps_the_closure_it_compares() {
    let mut side = specs::finding(true);
    let input = fixtures::changed_closures().before;
    side.found(Some(fixtures::diff_record(fixtures::KEY_A, input.clone())), false);
    assert_eq!(side.finding.record(&fixtures::KEY_A).expect("found key").closure.as_deref(), Some(input.as_ref()));
}
