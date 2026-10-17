use super::fixtures;
use crate::{
    compose::{self, Layer, Merge},
    limits,
};

#[test]
fn six_layers_compose_debian_in_format_md_3_2s_order() {
    let (documents, target) =
        fixtures::debian_layers([Some("fleet"), Some("base"), Some("member"), Some("env"), Some("base-env"), None]);
    let composed = compose::debian(&target, &documents, &mut Merge::new(&limits::SHIPPED)).expect("five set it");
    assert_eq!(
        fixtures::suite(&composed),
        Some((&b"base-env"[..], Layer::BaseEnv)),
        "3.2: base-env, last but its member"
    );
}
#[test]
fn a_null_debian_removes_what_earlier_layers_set() {
    let (documents, target) = fixtures::debian_layers([Some("fleet"), None, None, None, Some("null"), None]);
    assert_eq!(compose::debian(&target, &documents, &mut Merge::new(&limits::SHIPPED)), None);
}
#[test]
fn the_last_member_layer_sets_the_debian_suite() {
    let (documents, target) = fixtures::debian_layers([
        Some("fleet"),
        Some("base"),
        Some("member"),
        Some("env"),
        Some("base-env"),
        Some("member-env"),
    ]);
    let composed = compose::debian(&target, &documents, &mut Merge::new(&limits::SHIPPED)).expect("six set it");
    assert_eq!(fixtures::suite(&composed), Some((&b"member-env"[..], Layer::BaseEnvMember)));
}

#[test]
fn the_debian_root_reaches_every_tasks_roots() {
    let (mut side, checked) = fixtures::debian_guarded_tasks();
    assert!(crate::call::guards::check(
        &side.documents,
        &mut side.errors,
        &limits::SHIPPED,
        &fixtures::meta(),
        &checked,
        &fixtures::checked_debian()
    ));
}
#[test]
fn an_if_and_an_interpolated_key_read_debian_suite() {
    let mut roots = fixtures::Roots::x1_prod();
    roots.debian = fixtures::checked_debian();
    let mut left = limits::SHIPPED.expanded;
    let expanded = crate::entries::expand(&fixtures::debian_entry(), &roots.context(), &limits::SHIPPED, &mut left)
        .expect("debian is rooted");
    assert_eq!(fixtures::expanded_keys(&expanded).as_ref(), &[b"trixie".as_slice()]);
}
