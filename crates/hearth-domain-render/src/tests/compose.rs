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
