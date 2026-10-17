use super::fixtures::document;
use crate::{
    Fault, Field, Position, Shape,
    document::{Firmware, On, Placement, Runs},
    json::{Kind, Value},
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
fn placement_and_members_are_refused_until_they_are_decoded() {
    let error = document(br#"{"members":{}}"#).expect_err("reserved");
    assert_eq!(error.at, Position { offset: 1, line: 1, column: 2 });
    assert_eq!(error.fault, Fault::Shape(Shape::Reserved));
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
