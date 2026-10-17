use super::fixtures;
use crate::{
    keys::{self, Keys},
    vocabulary::{Outcome, Output, Request},
};
use hearth_lib::encode::Digest;
use skein_lib::{Queue, Token};
#[test]
fn the_resolution_key_is_h_over_its_four_inputs_as_items() {
    assert_eq!(
        keys::resolution(&fixtures::key_inputs()),
        fixtures::digest("01ea811afc99308a9fcc336c944966c7f9dd98857b27592fe00a1fb84bd49d00")
    );
}
#[test]
fn the_layer_keys_chain_packages_software_and_config() {
    assert_eq!(
        keys::layers(&Digest([0; 32]), &Digest([1; 32]), &Digest([2; 32]), &Digest([3; 32])),
        Keys {
            packages: fixtures::digest("22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d"),
            software: fixtures::digest("f96c81da124d998b00e4cea2ec54f3e46a9dec976cbcf12680feb2925e3af4c0"),
            config: fixtures::digest("a56d06bfeaffd29b71c3a735c3c1057b43fff1f4f1fddf12c90080a3dda45b2e"),
        }
    );
}
#[test]
fn an_empty_output_part_enters_as_its_record_with_empty_fields() {
    assert_eq!(
        keys::part(&fixtures::spec(b"x1-prod", &[]), b"software"),
        fixtures::digest("656d135b92c33782dc40e4dc874400fa4d4b666e9e466b3644075a81279f74a6")
    );
}
#[test]
fn each_distinct_key_is_found_once_at_its_first_holder_in_name_order() {
    let mut side = fixtures::finding_side();
    let mut out = Queue::with_capacity(1);
    drop(side.resolved_answer(&mut out));
    assert_eq!(out.pop(), Some(Output::Request(Request::Find { request: Token::new(5), key: fixtures::KEY_A })));
    side.found(None, false);
    drop(side.resolved_answer(&mut out));
    assert_eq!(out.pop(), Some(Output::Request(Request::Find { request: Token::new(5), key: fixtures::KEY_B })));
    side.found(None, false);
    drop(side.resolved_answer(&mut out));
    assert_eq!(out.pop(), None);
}
#[test]
fn a_declined_find_leaves_the_key_needed() {
    let mut side = fixtures::finding_side();
    let mut out = Queue::with_capacity(1);
    drop(side.resolved_answer(&mut out));
    drop(out.pop());
    side.found(None, true);
    drop(side.resolved_answer(&mut out));
    drop(out.pop());
    side.found(None, false);
    let Some(crate::vocabulary::Answer::Spec(spec)) = side.resolved_answer(&mut out) else { panic!("spec") };
    assert_eq!(spec.resolution.as_deref(), Some(&keys::Resolution::Needed { key: fixtures::KEY_A }));
}
#[test]
fn a_declined_find_makes_the_outcome_try_again() {
    let mut side = fixtures::finding_side();
    let mut out = Queue::with_capacity(1);
    drop(side.resolved_answer(&mut out));
    drop(out.pop());
    side.found(None, true);
    assert_eq!(side.outcome, Outcome::TryAgain);
}
