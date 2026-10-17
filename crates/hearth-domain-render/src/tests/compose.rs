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
        let settings = crate::compose::compose(
            only(&discovered.targets),
            &documents,
            &crate::schema::Defaults::new(),
            &mut Merge::new(&limits::SHIPPED),
        );
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

#[test]
fn the_defaults_lie_under_every_document_of_a_target() {
    use super::fixtures::schemas::{self, Layers};
    let mut declaration = schemas::typed_declaration(crate::revision::declaration::Form::Int);
    declaration.settings = Box::new([crate::revision::declaration::Spec {
        default: Some(int(at(1, 20), 1)),
        ..schemas::schema_spec(b"x", 0)
    }]);
    let (target, documents, defaults) = schemas::layers(
        &declaration,
        Layers {
            fleet: map(at(2, 1), &[(b"x", at(2, 2), int(at(2, 7), 2))]),
            base: map(at(3, 1), &[]),
            environment: map(at(4, 1), &[]),
            overlay: map(at(5, 1), &[]),
        },
    );
    let settings = crate::compose::compose(&target, &documents, &defaults, &mut Merge::new(&limits::SHIPPED));
    let Composite::Map(entries) = &only(&settings.tasks).settings.kind else { panic!("settings") };
    assert_eq!(
        only(entries).value,
        crate::compose::Composed {
            from: Provenance {
                layer: Layer::Fleet,
                document: documents.find(b"fleet.json").expect("fleet"),
                at: at(2, 7)
            },
            kind: Composite::Int(2)
        }
    );
}
#[test]
fn null_removes_a_default_map_s_entry() {
    use super::fixtures::schemas;
    let (declaration, layers) = schemas::nulled_accounts();
    let (target, documents, defaults) = schemas::layers(&declaration, layers);
    let settings = crate::compose::compose(&target, &documents, &defaults, &mut Merge::new(&limits::SHIPPED));
    let Composite::Map(settings) = &only(&settings.tasks).settings.kind else { panic!("settings") };
    let Composite::Map(accounts) = &only(settings).value.kind else { panic!("accounts") };
    assert_eq!(only(accounts).key.as_ref(), b"bob");
}
#[test]
fn a_nulled_field_gets_its_default_back() {
    use super::fixtures::schemas;
    use crate::evaluate::Value as V;
    let (declaration, layers) = schemas::nulled_accounts();
    let (target, documents, defaults) = schemas::layers(&declaration, layers);
    let settings = crate::compose::compose(&target, &documents, &defaults, &mut Merge::new(&limits::SHIPPED));
    let value = schemas::validate(
        &declaration,
        &only(&settings.tasks).settings,
        &crate::schema::Check::Target { environment: b"prod", targets: &[] },
    )
    .expect("valid accounts");
    let V::Map(settings) = value else { panic!("settings") };
    let Some(V::Map(accounts)) = &only(&settings).value else { panic!("accounts") };
    let Some(V::Map(fields)) = &only(accounts).value else { panic!("account") };
    let mut shell = None;
    for field in fields {
        if field.key.as_ref() == b"shell" {
            shell = field.value.as_ref();
        }
    }
    assert_eq!(shell, Some(&V::String(skein_lib::bytes::copy_of(b"/bin/bash"))));
}

const DEFAULTS: &[&[u8]] = &[b"fleet.json", b"envs/lab.json"];
#[test]
fn null_for_a_task_in_a_default_is_refused_at_its_key() {
    use super::fixtures;
    for (row, path) in DEFAULTS.iter().enumerate() {
        let documents = fixtures::rule_document(path, b"motd", true, false, false);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        crate::compose::rules(&documents, &mut errors);
        let errors = errors.finish().0;
        assert_eq!(only(&errors).kind, crate::errors::Kind::Composition(crate::errors::Composition::Null), "row {row}");
        assert_eq!(only(&errors).at.as_ref().expect("key").position, Some(at(1, 2)), "row {row}");
        assert_eq!(only(&errors).task.as_deref(), Some(b"motd".as_slice()), "row {row}");
    }
}
#[test]
fn a_null_hearth_is_no_compose_error() {
    let documents = super::fixtures::rule_document(b"fleet.json", b"hearth", true, false, false);
    let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
    crate::compose::rules(&documents, &mut errors);
    assert_eq!(errors.finish().0.as_ref(), []);
}
const BASE_PLACEMENTS: &[(&[u8], bool)] =
    &[(b"fleet.json", false), (b"hosts/stoa/stoa.json", false), (b"hosts/stoa/stoa.json", true)];
#[test]
fn placement_in_fleet_json_or_a_base_is_refused() {
    for (row, &(path, member)) in BASE_PLACEMENTS.iter().enumerate() {
        let documents = super::fixtures::rule_document(path, b"", false, true, member);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        crate::compose::rules(&documents, &mut errors);
        let errors = errors.finish().0;
        let error = only(&errors);
        assert_eq!(error.kind, crate::errors::Kind::Composition(crate::errors::Composition::Placement), "row {row}");
        assert_eq!(error.at.as_ref().expect("placement").position, Some(at(1, 2)), "row {row}");
    }
}
#[test]
fn members_in_fleet_json_or_an_environment_is_refused() {
    for (row, path) in DEFAULTS.iter().enumerate() {
        let documents = super::fixtures::rule_document(path, b"", false, false, true);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        crate::compose::rules(&documents, &mut errors);
        let errors = errors.finish().0;
        assert_eq!(
            only(&errors).kind,
            crate::errors::Kind::Composition(crate::errors::Composition::Members),
            "row {row}"
        );
        assert_eq!(only(&errors).at.as_ref().expect("members").position, Some(at(1, 11)), "row {row}");
    }
}
#[test]
fn a_target_without_placement_fails_with_its_error() {
    let documents = super::fixtures::member_documents(None, None);
    let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
    let discovered = crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("targets");
    let error =
        crate::compose::placed(only(&discovered.targets), &documents, &discovered.targets).expect_err("unplaced");
    assert_eq!(error.kind, crate::errors::Kind::Composition(crate::errors::Composition::Unplaced));
    assert_eq!(error.at.expect("base-env").path.as_ref(), b"hosts/stoa/stoa-lab.json");
}
type HostCase = (&'static [u8], (&'static str, Option<&'static [u8]>));
const HOSTS: &[HostCase] = &[
    (b"absent-lab", ("unknown", None)),
    (b"rhi-prod", ("elsewhere", Some(b"prod"))),
    (b"chat-lab", ("container", None)),
];
#[test]
fn a_container_s_on_names_a_non_container_of_its_environment() {
    for (row, (on, expected)) in HOSTS.iter().enumerate() {
        let documents = super::fixtures::container_documents(on);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        let discovered = crate::compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("targets");
        let mut target = None;
        for each in &discovered.targets {
            if each.name.as_ref() == b"stoa-lab" {
                target = Some(each);
            }
        }
        let target = target.expect("container");
        let error = crate::compose::placed(target, &documents, &discovered.targets).expect_err("invalid host");
        let crate::errors::Kind::Composition(crate::errors::Composition::On { on: name, host }) = &error.kind else {
            panic!("on error")
        };
        assert_eq!(name.as_ref(), *on, "row {row}");
        let host = match host {
            crate::errors::Host::Unknown => ("unknown", None),
            crate::errors::Host::Container => ("container", None),
            crate::errors::Host::Elsewhere { environment } => ("elsewhere", Some(environment.as_ref())),
        };
        assert_eq!(&host, expected, "row {row}");
        assert_eq!(error.at.expect("on").position, Some(at(1, 43)), "row {row}");
    }
}
const TASK_DOCUMENTS: &[(&[u8], bool)] = &[
    (b"fleet.json", false),
    (b"envs/lab.json", false),
    (b"hosts/stoa/stoa.json", false),
    (b"hosts/stoa/stoa.json", true),
];
#[test]
fn a_task_named_that_does_not_exist_is_refused_at_its_key() {
    for (row, &(path, member)) in TASK_DOCUMENTS.iter().enumerate() {
        let documents = super::fixtures::rule_document(path, b"missing", false, false, member);
        let mut errors = crate::errors::Collector::new(&limits::SHIPPED);
        crate::compose::rules(&documents, &mut errors);
        let errors = errors.finish().0;
        assert_eq!(only(&errors).kind, crate::errors::Kind::Composition(crate::errors::Composition::Task), "row {row}");
        assert_eq!(only(&errors).at.as_ref().expect("task").position, Some(at(1, 2)), "row {row}");
        assert_eq!(only(&errors).task.as_deref(), Some(b"missing".as_slice()), "row {row}");
    }
}

use super::fixtures::containers;
#[test]
fn containers_are_the_targets_placed_on_a_host_in_name_order() {
    let mut fleet = containers::fleet();
    fleet.targets.reverse();
    let containers = containers::on(&fleet, b"rhi-prod").expect("valid containers");
    assert_eq!(containers::names(&containers).as_ref(), containers::RHI_PROD_CONTAINERS);
}
#[test]
fn a_target_placed_on_another_host_is_not_listed() {
    let containers = containers::on(&containers::fleet(), b"rhi-lab").expect("valid containers");
    assert_eq!(containers::names(&containers).as_ref(), [b"mail-lab".as_slice()]);
}
#[test]
fn a_target_nothing_is_placed_on_has_an_empty_containers_map() {
    assert_eq!(containers::root(&containers::fleet(), b"empty-prod"), crate::evaluate::Value::Map(Box::new([])));
}
#[test]
fn a_containers_placement_map_leaves_absent_fields_out() {
    assert_eq!(
        containers::root(&containers::fleet(), b"rhi-lab"),
        crate::evaluate::Value::Map(Box::new([crate::evaluate::Entry {
            key: Box::from(b"mail-lab".as_slice()),
            value: Some(crate::evaluate::Value::Map(Box::new([
                crate::evaluate::Entry {
                    key: Box::from(b"kind".as_slice()),
                    value: Some(crate::evaluate::Value::String(Box::from(b"container".as_slice())))
                },
                crate::evaluate::Entry {
                    key: Box::from(b"on".as_slice()),
                    value: Some(crate::evaluate::Value::String(Box::from(b"rhi-lab".as_slice())))
                },
            ]))),
        }]))
    );
}
#[test]
fn a_base_envs_placement_replaces_the_environments_for_containers() {
    let fleet = containers::fleet();
    let documents = super::fixtures::with_document(
        &fleet.documents,
        b"envs/prod.json",
        containers::document(Some(containers::placement(Some(b"empty-prod")))),
    );
    let fleet = containers::discovered(documents);
    let containers = containers::on(&fleet, b"rhi-prod").expect("valid containers");
    assert_eq!(containers::names(&containers).as_ref(), containers::RHI_PROD_CONTAINERS);
}
#[test]
fn a_container_with_a_refused_placement_fails_its_host() {
    let fleet = containers::discovered(super::fixtures::container_documents(b"vm-lab"));
    let error = containers::on(&fleet, b"stoa-lab").expect_err("a container cannot host chat");
    assert_eq!(
        error.kind,
        crate::errors::Kind::Composition(crate::errors::Composition::On {
            on: Box::from(b"stoa-lab".as_slice()),
            host: crate::errors::Host::Container
        })
    );
    assert_eq!(error.target.as_deref(), Some(b"stoa-lab".as_slice()));
}
