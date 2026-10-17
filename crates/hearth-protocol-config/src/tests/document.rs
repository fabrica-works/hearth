use super::fixtures::{STOA, document};
use crate::{
    Fault, Field, Position, Shape,
    document::{Firmware, Members, On, Placement, Runs},
    json::{Entry, Kind, Value},
};
#[test]
fn a_task_value_is_its_settings_object_or_null() {
    const TASKS: &[(&[u8], Position, Position)] = &[
        (b"core", Position { offset: 16, line: 1, column: 17 }, Position { offset: 24, line: 1, column: 25 }),
        (b"wifi", Position { offset: 44, line: 1, column: 45 }, Position { offset: 52, line: 1, column: 53 }),
    ];
    let doc = document(br#"{"_note": "x1", "core": {"hostname": "x1"}, "wifi": null}"#).expect("document");
    assert_eq!(doc.layer.tasks.len(), TASKS.len());
    for (task, expected) in doc.layer.tasks.iter().zip(TASKS) {
        assert_eq!((&*task.name, task.at, task.value.at), *expected);
    }
    assert_eq!(doc.layer.tasks[1].value, Value { at: Position { offset: 52, line: 1, column: 53 }, kind: Kind::Null });
}
#[test]
fn debian_is_kept_as_written_with_its_positions() {
    let value = document(br#"{"debian":{"suite":"forky"}}"#).expect("debian").layer.debian.expect("present");
    assert_eq!(value.at, Position { offset: 10, line: 1, column: 11 });
    let Kind::Map(entries) = value.kind else {
        panic!("object kept");
    };
    assert_eq!(entries[0].key.as_ref(), b"suite");
    assert_eq!(entries[0].at, Position { offset: 11, line: 1, column: 12 });
    assert_eq!(
        entries[0].value,
        Value { at: Position { offset: 19, line: 1, column: 20 }, kind: Kind::String(b"forky".as_slice().into()) }
    );
}

#[test]
fn a_task_value_neither_object_nor_null_is_refused_naming_the_task() {
    const CASES: &[&[u8]] = &[br#"{"core":1}"#, br#"{"core":[]}"#, br#"{"core":false}"#, br#"{"core":"x"}"#];
    for input in CASES {
        let error = document(input).expect_err("settings");
        assert_eq!(error.at, Position { offset: 8, line: 1, column: 9 });
        assert_eq!(error.fault, Fault::Shape(Shape::Settings { task: b"core".as_slice().into() }));
    }
}
#[test]
fn a_key_that_is_no_task_name_is_refused_at_the_key() {
    const CASES: &[&[u8]] = &[br#"{"A":{}}"#, br#"{"a-b":{}}"#, br#"{"":{}}"#, br#"{"1":{}}"#];
    for input in CASES {
        let error = document(input).expect_err("name");
        assert_eq!(error.at, Position { offset: 1, line: 1, column: 2 });
        assert_eq!(error.fault, Fault::Shape(Shape::Name));
    }
}

#[test]
fn a_vm_s_placement_decodes_its_kind_firmware_and_hardware() {
    let doc = document(br#"{ "placement": { "kind": "vm", "firmware": "bios", "hardware": "ionos" } }"#).expect("VM");
    assert_eq!(
        doc.layer.placement,
        Some(Placement {
            at: Position { offset: 15, line: 1, column: 16 },
            kind: Runs::Vm,
            firmware: Some(Firmware::Bios),
            hardware: Some(b"ionos".as_slice().into()),
            on: None,
            mutable: false,
        })
    );
}
#[test]
fn a_container_s_placement_names_its_host_where_it_is_written() {
    let doc = document(br#"{ "placement": { "kind": "container", "on": "rhi-prod" } }"#).expect("container");
    assert_eq!(
        doc.layer.placement,
        Some(Placement {
            at: Position { offset: 15, line: 1, column: 16 },
            kind: Runs::Container,
            firmware: None,
            hardware: None,
            mutable: false,
            on: Some(On { at: Position { offset: 44, line: 1, column: 45 }, target: b"rhi-prod".as_slice().into() }),
        })
    );
}
#[test]
fn mutable_is_false_unless_a_placement_sets_it_true() {
    const CASES: &[(&[u8], bool)] = &[
        (br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"}}"#, false),
        (br#"{"placement":{"kind":"vm","firmware":"bios","hardware":"ionos","mutable":false}}"#, false),
        (br#"{"placement":{"kind":"vm","firmware":"bios","hardware":"ionos","mutable":true}}"#, true),
        (br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen","mutable":true}}"#, true),
    ];
    for &(input, expected) in CASES {
        assert_eq!(
            document(input).expect("placement").layer.placement.expect("present").mutable,
            expected,
            "{input:?}"
        );
    }
}
#[test]
fn a_placement_field_m1_lacks_is_refused_at_its_key() {
    const CASES: &[(&[u8], &[u8])] = &[
        (br#"{"placement":{"ids":0,"kind":"container","on":"x"}}"#, b"ids"),
        (br#"{"placement":{"binds":{},"kind":"container","on":"x"}}"#, b"binds"),
        (br#"{"placement":{"capabilities":[],"kind":"container","on":"x"}}"#, b"capabilities"),
        (br#"{"placement":{"network":"host","kind":"container","on":"x"}}"#, b"network"),
        (br#"{"placement":{"bridge":"br0","kind":"container","on":"x"}}"#, b"bridge"),
        (br#"{"placement":{"mutable":true,"kind":"container","on":"x"}}"#, b"mutable"),
        (br#"{"placement":{"mutable":false,"kind":"container","on":"x"}}"#, b"mutable"),
        (br#"{"placement":{"mutable":null,"kind":"container","on":"x"}}"#, b"mutable"),
    ];
    for &(input, expected) in CASES {
        let error = document(input).expect_err("M1 field");
        assert_eq!(error.at, Position { offset: 14, line: 1, column: 15 });
        let Fault::Shape(Shape::Unknown { field }) = error.fault else { panic!("unknown field: {error:?}") };
        assert_eq!(field.as_ref(), expected, "{input:?}");
    }
}
const FIRMWARE_OR_HARDWARE_ON_A_CONTAINER_IS_REFUSED_AT_ITS_KEY: &[(&[u8], Fault)] = &[
    (
        br#"{"placement":{"firmware":"bios","kind":"container","on":"x"}}"#,
        Fault::Shape(Shape::Misplaced { field: Field::Firmware }),
    ),
    (
        br#"{"placement":{"hardware":"ionos","kind":"container","on":"x"}}"#,
        Fault::Shape(Shape::Misplaced { field: Field::Hardware }),
    ),
    (
        br#"{"placement":{"firmware":null,"kind":"container","on":"x"}}"#,
        Fault::Shape(Shape::Misplaced { field: Field::Firmware }),
    ),
    (
        br#"{"placement":{"hardware":null,"kind":"container","on":"x"}}"#,
        Fault::Shape(Shape::Misplaced { field: Field::Hardware }),
    ),
];
#[test]
fn firmware_or_hardware_on_a_container_is_refused_at_its_key() {
    for (input, expected) in FIRMWARE_OR_HARDWARE_ON_A_CONTAINER_IS_REFUSED_AT_ITS_KEY {
        let error = document(input).expect_err("container field");
        assert_eq!(error.at, Position { offset: 14, line: 1, column: 15 });
        assert_eq!(&error.fault, expected, "{input:?}");
    }
}
#[test]
fn on_for_anything_but_a_container_is_refused_at_its_key() {
    const CASES: &[&[u8]] = &[
        br#"{"placement":{"on":"x","kind":"vm","firmware":"bios","hardware":"ionos"}}"#,
        br#"{"placement":{"on":null,"kind":"metal","firmware":"efi","hardware":"7th-gen"}}"#,
    ];
    for input in CASES {
        let error = document(input).expect_err("host only for a container");
        assert_eq!(error.at, Position { offset: 14, line: 1, column: 15 });
        assert_eq!(error.fault, Fault::Shape(Shape::Misplaced { field: Field::On }), "{input:?}");
    }
}
const A_KIND_OR_FIRMWARE_NOT_LISTED_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], Fault, Position)] = &[
    (
        br#"{"placement":{"kind":"bare"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Kind }),
        Position { offset: 21, line: 1, column: 22 },
    ),
    (
        br#"{"placement":{"kind":"VM"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Kind }),
        Position { offset: 21, line: 1, column: 22 },
    ),
    (
        br#"{"placement":{"firmware":"uefi","kind":"vm","hardware":"ionos"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Firmware }),
        Position { offset: 25, line: 1, column: 26 },
    ),
    (
        br#"{"placement":{"firmware":"BIOS","kind":"metal","hardware":"7th-gen"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Firmware }),
        Position { offset: 25, line: 1, column: 26 },
    ),
];
#[test]
fn a_kind_or_firmware_not_listed_is_refused_at_its_value() {
    for (input, expected, at) in A_KIND_OR_FIRMWARE_NOT_LISTED_IS_REFUSED_AT_ITS_VALUE {
        let error = document(input).expect_err("listed enum");
        assert_eq!(error.at, *at, "{input:?}");
        assert_eq!(&error.fault, expected, "{input:?}");
    }
}
#[test]
fn a_null_placement_is_refused_at_its_value() {
    let error = document(br#"{"placement":null}"#).expect_err("never a removal");
    assert_eq!(error.at, Position { offset: 13, line: 1, column: 14 });
    assert_eq!(error.fault, Fault::Shape(Shape::Placement));
}
#[test]
fn a_nonobject_placement_is_refused_at_its_value() {
    const CASES: &[&[u8]] =
        &[br#"{"placement":false}"#, br#"{"placement":1}"#, br#"{"placement":[]}"#, br#"{"placement":"vm"}"#];
    for input in CASES {
        let error = document(input).expect_err("object");
        assert_eq!(error.at, Position { offset: 13, line: 1, column: 14 });
        assert_eq!(error.fault, Fault::Shape(Shape::Placement), "{input:?}");
    }
}
const A_PLACEMENT_FIELD_OF_THE_WRONG_TYPE_IS_REFUSED_AT_ITS_VALUE: &[(&[u8], Fault, Position)] = &[
    (
        br#"{"placement":{"kind":1}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Kind }),
        Position { offset: 21, line: 1, column: 22 },
    ),
    (
        br#"{"placement":{"firmware":null,"kind":"vm","hardware":"ionos"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Firmware }),
        Position { offset: 25, line: 1, column: 26 },
    ),
    (
        br#"{"placement":{"hardware":{},"kind":"metal","firmware":"efi"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Hardware }),
        Position { offset: 25, line: 1, column: 26 },
    ),
    (
        br#"{"placement":{"on":false,"kind":"container"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::On }),
        Position { offset: 19, line: 1, column: 20 },
    ),
    (
        br#"{"placement":{"mutable":0,"kind":"vm","firmware":"bios","hardware":"ionos"}}"#,
        Fault::Shape(Shape::Wrong { field: Field::Mutable }),
        Position { offset: 24, line: 1, column: 25 },
    ),
];
#[test]
fn a_placement_field_of_the_wrong_type_is_refused_at_its_value() {
    for (input, expected, at) in A_PLACEMENT_FIELD_OF_THE_WRONG_TYPE_IS_REFUSED_AT_ITS_VALUE {
        let error = document(input).expect_err("field type");
        assert_eq!(error.at, *at, "{input:?}");
        assert_eq!(&error.fault, expected, "{input:?}");
    }
}
const A_PLACEMENT_MISSING_WHAT_ITS_KIND_NEEDS_IS_REFUSED_AT_ITS_BRACE: &[(&[u8], Fault)] = &[
    (br#"{"placement":{}}"#, Fault::Shape(Shape::Missing { field: Field::Kind })),
    (br#"{"placement":{"kind":"vm","hardware":"ionos"}}"#, Fault::Shape(Shape::Missing { field: Field::Firmware })),
    (br#"{"placement":{"kind":"metal","firmware":"efi"}}"#, Fault::Shape(Shape::Missing { field: Field::Hardware })),
    (br#"{"placement":{"kind":"vm","firmware":"bios"}}"#, Fault::Shape(Shape::Missing { field: Field::Hardware })),
    (
        br#"{"placement":{"kind":"metal","hardware":"7th-gen"}}"#,
        Fault::Shape(Shape::Missing { field: Field::Firmware }),
    ),
    (br#"{"placement":{"kind":"container"}}"#, Fault::Shape(Shape::Missing { field: Field::On })),
];
#[test]
fn a_placement_missing_what_its_kind_needs_is_refused_at_its_brace() {
    for (input, expected) in A_PLACEMENT_MISSING_WHAT_ITS_KIND_NEEDS_IS_REFUSED_AT_ITS_BRACE {
        let error = document(input).expect_err("whole placement");
        assert_eq!(error.at, Position { offset: 13, line: 1, column: 14 });
        assert_eq!(&error.fault, expected, "{input:?}");
    }
}

#[test]
fn a_base_s_members_decode_in_name_order_each_a_layer() {
    const MEMBERS: &[(&[u8], Position, i64)] = &[
        (b"1", Position { offset: 83, line: 5, column: 5 }, 1),
        (b"2", Position { offset: 120, line: 6, column: 5 }, 2),
        (b"3", Position { offset: 157, line: 7, column: 5 }, 3),
    ];
    let members = document(STOA).expect("stoa").members.expect("members");
    assert_eq!(members.at, Position { offset: 77, line: 4, column: 14 });
    assert_eq!(members.each.len(), 3);
    for (member, expected) in members.each.iter().zip(MEMBERS) {
        let layer = member.layer.as_ref().expect("a layer");
        let [task] = &*layer.tasks else { panic!("one task") };
        let Kind::Map(settings) = &task.value.kind else { panic!("settings") };
        let [index] = &**settings else { panic!("one setting") };
        let Kind::Int(number) = index.value.kind else { panic!("integer") };
        assert_eq!(task.name.as_ref(), b"stoa");
        assert_eq!(index.key.as_ref(), b"index");
        assert_eq!((member.name.as_ref(), member.at, number), *expected);
    }
}
#[test]
fn a_member_set_to_null_is_removed() {
    const CASES: &[(&[u8], &[u8])] = &[
        (br#"{"members":{"1":null}}"#, b"1"),
        (br#"{"members":{"a-2":null}}"#, b"a-2"),
        (br#"{"members":{"a--b":null}}"#, b"a--b"),
        (br#"{"members":{"a":null}}"#, b"a"),
        (br#"{"members":{"0":null}}"#, b"0"),
    ];
    for &(input, expected) in CASES {
        let members = document(input).expect("members").members.expect("present");
        let [member] = &*members.each else { panic!("one removal") };
        assert_eq!(member.name.as_ref(), expected, "{input:?}");
        assert_eq!(member.at, Position { offset: 12, line: 1, column: 13 });
        assert_eq!(member.layer, None, "{input:?}");
    }
}
#[test]
fn a_member_name_out_of_form_is_refused_at_its_key() {
    const CASES: &[&[u8]] = &[
        br#"{"members":{"a-":{}}}"#,
        br#"{"members":{"Stoa":{}}}"#,
        br#"{"members":{"-a":{}}}"#,
        br#"{"members":{"a_b":{}}}"#,
        br#"{"members":{"a.b":{}}}"#,
        br#"{"members":{"":{}}}"#,
    ];
    for input in CASES {
        let error = document(input).expect_err("member name");
        assert_eq!(error.at, Position { offset: 12, line: 1, column: 13 });
        assert_eq!(error.fault, Fault::Shape(Shape::Member), "{input:?}");
    }
}
#[test]
fn members_inside_a_member_are_refused_at_the_key() {
    const CASES: &[&[u8]] = &[br#"{"members":{"1":{"members":{}}}}"#, br#"{"members":{"1":{"members":null}}}"#];
    for input in CASES {
        let error = document(input).expect_err("no nested members");
        assert_eq!(error.at, Position { offset: 17, line: 1, column: 18 });
        assert_eq!(error.fault, Fault::Shape(Shape::Member), "{input:?}");
    }
}
#[test]
fn a_member_s_value_neither_object_nor_null_is_refused() {
    const CASES: &[&[u8]] = &[
        br#"{"members":{"1":false}}"#,
        br#"{"members":{"1":1}}"#,
        br#"{"members":{"1":[]}}"#,
        br#"{"members":{"1":"x"}}"#,
    ];
    for input in CASES {
        let error = document(input).expect_err("member layer or null");
        assert_eq!(error.at, Position { offset: 16, line: 1, column: 17 });
        assert_eq!(error.fault, Fault::Shape(Shape::Members), "{input:?}");
    }
}
#[test]
fn members_that_are_not_an_object_are_refused_at_the_value() {
    const CASES: &[&[u8]] = &[
        br#"{"members":null}"#,
        br#"{"members":1}"#,
        br#"{"members":false}"#,
        br#"{"members":[]}"#,
        br#"{"members":"x"}"#,
    ];
    for input in CASES {
        let error = document(input).expect_err("members object");
        assert_eq!(error.at, Position { offset: 11, line: 1, column: 12 });
        assert_eq!(error.fault, Fault::Shape(Shape::Members), "{input:?}");
    }
}
#[test]
fn a_member_s_layer_holds_its_placement() {
    let members = document(br#"{"members":{"1":{"placement":{"kind":"container","on":"rhi-prod"}}}}"#)
        .expect("members")
        .members
        .expect("present");
    let [member] = &*members.each else { panic!("one member") };
    assert_eq!(
        member.layer.as_ref().expect("layer").placement,
        Some(Placement {
            at: Position { offset: 29, line: 1, column: 30 },
            kind: Runs::Container,
            firmware: None,
            hardware: None,
            mutable: false,
            on: Some(On { at: Position { offset: 54, line: 1, column: 55 }, target: b"rhi-prod".as_slice().into() }),
        })
    );
}
#[test]
fn a_member_s_layer_holds_its_debian() {
    let members =
        document(br#"{"members":{"1":{"debian":{"suite":"forky"}}}}"#).expect("members").members.expect("present");
    let [member] = &*members.each else { panic!("one member") };
    let value = member.layer.as_ref().expect("layer").debian.as_ref().expect("debian");
    assert_eq!(value.at, Position { offset: 26, line: 1, column: 27 });
    let Kind::Map(entries) = &value.kind else { panic!("debian object") };
    let [entry] = &**entries else { panic!("suite") };
    assert_eq!(
        entry,
        &Entry {
            at: Position { offset: 27, line: 1, column: 28 },
            key: b"suite".as_slice().into(),
            value: Value {
                at: Position { offset: 35, line: 1, column: 36 },
                kind: Kind::String(b"forky".as_slice().into())
            },
        }
    );
}
#[test]
fn an_empty_members_object_stays_present() {
    assert_eq!(
        document(br#"{"members":{}}"#).expect("empty cluster").members,
        Some(Members { at: Position { offset: 11, line: 1, column: 12 }, each: Box::new([]) })
    );
}
#[test]
fn a_document_without_members_keeps_them_absent() {
    assert_eq!(document(b"{}").expect("no members").members, None);
}
