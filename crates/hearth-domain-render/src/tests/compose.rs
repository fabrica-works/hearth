use super::fixtures::{
    composition::{at, int, map, null, string},
    only,
};
use crate::{
    compose::{Composite, Layer, Merge, Provenance},
    limits,
    revision::{Kind, Value},
};

#[test]
fn a_later_layer_s_scalar_replaces_and_keeps_its_own_place() {
    let fleet = map(at(1, 1), &[(b"port", at(1, 2), int(at(1, 10), 22))]);
    let base = map(at(3, 3), &[(b"port", at(3, 4), int(at(3, 12), 10_127))]);
    let mut merge = Merge::new(&limits::SHIPPED);
    let composed = merge.merge(None, &fleet, Layer::Fleet, 0);
    let composed = merge.merge(composed, &base, Layer::Base, 2).expect("map");
    let Composite::Map(pairs) = &composed.kind else { panic!("map") };
    let port = only(pairs);
    assert_eq!(port.value.kind, Composite::Int(10_127));
    assert_eq!(
        port.value.from,
        Provenance { layer: Layer::Base, document: 2, at: crate::revision::Position { line: 3, column: 12 } }
    );
    assert_eq!(composed.from.layer, Layer::Fleet);
}
#[test]
fn null_removes_its_key_at_any_depth_and_never_stays() {
    let fleet = map(
        at(1, 1),
        &[
            (b"a", at(1, 2), int(at(1, 5), 1)),
            (b"b", at(1, 8), map(at(1, 12), &[(b"c", at(1, 13), int(at(1, 16), 2))])),
        ],
    );
    let base = map(
        at(2, 1),
        &[(b"a", at(2, 2), null(at(2, 5))), (b"b", at(2, 8), map(at(2, 12), &[(b"c", at(2, 13), null(at(2, 16)))]))],
    );
    let mut merge = Merge::new(&limits::SHIPPED);
    let composed = merge.merge(None, &fleet, Layer::Fleet, 0);
    let composed = merge.merge(composed, &base, Layer::Base, 1).expect("map");
    let Composite::Map(pairs) = composed.kind else { panic!("map") };
    assert_eq!(only(&pairs).key.as_ref(), b"b");
    assert_eq!(only(&pairs).value.kind, Composite::Map(Box::new([])));
}
const REPLACEMENTS: &[(Kind, Composite)] =
    &[(Kind::Bool(true), Composite::Bool(true)), (Kind::Int(42), Composite::Int(42))];
#[test]
fn a_list_or_a_scalar_patch_replaces_whole() {
    let mut merge = Merge::new(&limits::SHIPPED);
    for (kind, expected) in REPLACEMENTS {
        let target = merge.merge(None, &map(at(1, 1), &[]), Layer::Fleet, 0);
        let composed =
            merge.merge(target, &Value { at: at(2, 1), kind: kind.clone() }, Layer::Base, 1).expect("scalar");
        assert_eq!(&composed.kind, expected);
    }
}
#[test]
fn a_list_patch_replaces_whole_and_keeps_its_elements_places() {
    let mut merge = Merge::new(&limits::SHIPPED);
    let target = merge.merge(None, &string(at(1, 1), b"old"), Layer::Fleet, 0);
    let patch = Value { at: at(2, 1), kind: Kind::List(Box::new([int(at(2, 2), 7)])) };
    let composed = merge.merge(target, &patch, Layer::Base, 1).expect("list");
    let Composite::List(values) = composed.kind else { panic!("list") };
    assert_eq!(only(&values).kind, Composite::Int(7));
    assert_eq!(
        only(&values).from,
        Provenance { layer: Layer::Base, document: 1, at: crate::revision::Position { line: 2, column: 2 } }
    );
}
#[test]
fn an_object_patch_over_a_scalar_makes_a_new_object() {
    let mut merge = Merge::new(&limits::SHIPPED);
    let target = merge.merge(None, &int(at(1, 1), 42), Layer::Fleet, 0);
    let patch = map(at(2, 1), &[(b"gone", at(2, 2), null(at(2, 10)))]);
    let composed = merge.merge(target, &patch, Layer::Base, 1).expect("map");
    assert_eq!(composed.kind, Composite::Map(Box::new([])));
    assert_eq!(
        composed.from,
        Provenance { layer: Layer::Base, document: 1, at: crate::revision::Position { line: 2, column: 1 } }
    );
}
#[test]
fn a_merge_at_the_shipped_depth_fits_its_stack() {
    let mut patch = int(at(1, 1), 42);
    for _ in 0..limits::SHIPPED.depth {
        patch = map(at(1, 1), &[(b"nested", at(1, 2), patch)]);
    }
    let mut merge = Merge::new(&limits::SHIPPED);
    let composed = merge.merge(None, &patch, Layer::Fleet, 0).expect("deep map");
    let composed = merge.merge(Some(composed), &patch, Layer::Base, 1).expect("patched deep map");
    let mut value = &composed;
    for _ in 0..limits::SHIPPED.depth {
        let Composite::Map(pairs) = &value.kind else { panic!("nested map") };
        value = &only(pairs).value;
    }
    assert_eq!(value.kind, Composite::Int(42));
}

#[test]
fn a_base_with_members_makes_one_target_per_member() {
    use super::fixtures;
    let documents = fixtures::member_documents(Some(fixtures::members(&[(b"2", false), (b"1", false)])), None);
    let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
    let discovered = crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("targets");
    assert_eq!(discovered.names.as_ref(), &[Box::from(b"stoa-1-lab".as_slice()), Box::from(b"stoa-2-lab".as_slice())]);
}
#[test]
fn a_base_env_adds_removes_and_changes_members() {
    use super::fixtures;
    let documents = fixtures::member_documents(
        Some(fixtures::members(&[(b"1", false), (b"2", false), (b"3", true)])),
        Some(fixtures::members(&[(b"1", false), (b"2", true), (b"4", false)])),
    );
    let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
    let discovered = crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("targets");
    assert_eq!(discovered.names.as_ref(), &[Box::from(b"stoa-1-lab".as_slice()), Box::from(b"stoa-4-lab".as_slice())]);
}
const MEMBER_LAYERS: &[([Option<i64>; 5], Composite)] = &[
    ([Some(1), None, None, None, None], Composite::Int(1)),
    ([Some(1), Some(2), None, None, None], Composite::Int(2)),
    ([Some(1), Some(2), Some(3), None, None], Composite::Int(3)),
    ([Some(1), Some(2), Some(3), Some(4), None], Composite::Int(4)),
    ([Some(1), Some(2), Some(3), Some(4), Some(5)], Composite::Int(5)),
];
#[test]
fn a_member_s_layers_follow_its_base_s_and_base_env_s() {
    use super::fixtures;
    for (row, (values, expected)) in MEMBER_LAYERS.iter().enumerate() {
        let documents = fixtures::member_layers(*values);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        let discovered = crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("targets");
        let settings =
            crate::compose::compose(only(&discovered.targets), &documents, &mut Merge::new(&limits::SHIPPED));
        let Composite::Map(pairs) = &only(&settings.tasks).settings.kind else { panic!("task settings") };
        assert_eq!(&only(pairs).value.kind, expected, "row {row}: member layer order");
    }
}
const LONG_MEMBERS: &[&[u8]] = &[
    b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
];
#[test]
fn a_target_name_past_63_bytes_is_refused() {
    use super::fixtures;
    for (row, &name) in LONG_MEMBERS.iter().enumerate() {
        let input = (name, false);
        let documents = fixtures::member_documents(Some(fixtures::members(core::slice::from_ref(&input))), None);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        let discovered = crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("bounded count");
        assert_eq!(discovered.targets.as_ref(), [], "row {row}");
        assert_eq!(
            only(&errors.finish().0).kind,
            crate::errors::Kind::Composition(crate::errors::Composition::Long),
            "row {row}"
        );
    }
}
#[test]
fn two_targets_of_one_name_fail_both_with_one_error() {
    use super::fixtures;
    use crate::{
        errors::{Class, Composition, Error, Kind, Place},
        revision::{Document, Layer, Position},
    };
    use skein_lib::bytes::copy_of;
    let input = (b"1".as_slice(), false);
    let documents = fixtures::member_documents(Some(fixtures::members(core::slice::from_ref(&input))), None);
    let documents = fixtures::with_document(
        &documents,
        b"hosts/stoa-1/stoa-1-lab.json",
        Document { at: at(1, 1), members: None, layer: Layer { placement: None, tasks: Box::new([]) } },
    );
    let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
    assert_eq!(
        crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("bounded count").targets.as_ref(),
        []
    );
    assert_eq!(
        errors.finish().0.as_ref(),
        &[Error {
            side: None,
            kind: Kind::Composition(Composition::Duplicate),
            class: Class::Configuration,
            at: Some(Place { path: copy_of(b"hosts/stoa-1/stoa-1-lab.json"), position: None }),
            also: Some(Place {
                path: copy_of(b"hosts/stoa/stoa.json"),
                position: Some(Position { line: 1, column: 12 })
            }),
            target: Some(copy_of(b"stoa-1-lab")),
            task: None,
        }]
    );
}
