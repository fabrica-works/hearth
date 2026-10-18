use super::fixtures::{self, settings};
use crate::{
    errors::{Error, Kind, Place},
    evaluate::FaultKind,
    vocabulary::Class,
};
use skein_lib::bytes::copy_of;

#[test]
fn a_target_whose_settings_pass_64_mib_fails_alone() {
    let (mut side, targets) = settings::oversized();
    assert!(settings::validate(&mut side, &targets, b"large-prod").is_none());
    assert_eq!(
        fixtures::kept(&side).as_ref(),
        &[Error {
            side: None,
            kind: Kind::Evaluation(FaultKind::Limit),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"hosts/large/large-prod.json"), position: None }),
            also: None,
            target: Some(copy_of(b"large-prod")),
            task: None,
        }]
    );
    assert!(settings::validate(&mut side, &targets, b"small-prod").is_some());
}
