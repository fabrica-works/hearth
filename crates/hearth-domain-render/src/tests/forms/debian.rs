use super::{
    Entry, Fault, Format, List, Schema, Value, at, copy_of, forms, inputs, int, map, revision, schema, string,
};
#[test]
fn debian_fills_its_defaults_and_keeps_main_on() {
    let (value, faults) = inputs::debian(&[]);
    assert_eq!(faults.as_ref(), &[]);
    let Value::Map(entries) = value.expect("sound Debian") else { panic!("object") };
    let [_, components, install, pockets, _, _] = entries.as_ref() else { panic!("six fields") };
    assert_eq!(
        (&components.value, &install.value, &pockets.value),
        (
            &Some(Value::Map(Box::new([Entry { key: copy_of(b"main"), value: Some(Value::Bool(true)) }]))),
            &Some(Value::Bool(false)),
            &Some(Value::Map(Box::new([
                Entry { key: copy_of(b"security"), value: Some(Value::Bool(true)) },
                Entry { key: copy_of(b"updates"), value: Some(Value::Bool(true)) }
            ]))),
        )
    );
}
#[test]
fn an_absent_debian_is_missing_at_the_base_s_document() {
    let mut faults = List::with_capacity(4);
    assert_eq!(schema::debian(None, 2, &mut faults), None);
    assert_eq!(
        faults.as_slice(),
        &[Fault { schema: Schema::Missing { setting: copy_of(b"debian") }, document: 2, at: at(1, 1) }]
    );
}
#[test]
fn debian_s_security_snapshot_is_required_with_its_pocket() {
    let (_, faults) = inputs::debian(&[(
        b"snapshot",
        at(2, 2),
        map(at(2, 7), &[(b"archive", at(2, 8), string(at(2, 18), b"20261001T030405Z"))]),
    )]);
    assert_eq!(faults.as_ref(), &[Fault { schema: Schema::Security, document: 2, at: at(2, 7) }]);
}
#[test]
fn a_security_snapshot_beside_a_pocket_off_is_dropped() {
    let (value, faults) = inputs::debian(&[
        (
            b"pockets",
            at(2, 2),
            map(
                at(2, 7),
                &[(b"security", at(2, 8), revision::Value { at: at(2, 20), kind: revision::Kind::Bool(false) })],
            ),
        ),
        (
            b"snapshot",
            at(2, 26),
            map(
                at(2, 38),
                &[
                    (b"archive", at(2, 39), string(at(2, 50), b"20261001T030405Z")),
                    (b"security", at(2, 70), int(at(2, 82), 7)),
                ],
            ),
        ),
    ]);
    assert_eq!(faults.as_ref(), &[]);
    let Value::Map(entries) = value.expect("sound Debian") else { panic!("object") };
    let [_, _, _, _, snapshot, _] = entries.as_ref() else { panic!("six fields") };
    assert_eq!(
        snapshot.value,
        Some(Value::Map(Box::new([
            Entry { key: copy_of(b"archive"), value: Some(Value::String(copy_of(b"20261001T030405Z"))) },
            Entry { key: copy_of(b"security"), value: None }
        ])))
    );
}
const TIMESTAMPS: &[(&[u8], bool)] = &[
    (b"20261001T030405Z", true),
    (b"99999999T999999Z", true),
    (b"20261001t030405Z", false),
    (b"2026100xT030405Z", false),
    (b"20261001T030405z", false),
    (b"20261001T030405", false),
];
#[test]
fn debian_s_timestamps_follow_snapshot_s_form() {
    for (row, &(written, expected)) in TIMESTAMPS.iter().enumerate() {
        assert_eq!(forms::check(Format::Timestamp, written), expected, "row {row}");
    }
}
const BAD_TIMESTAMPS: &[&[u8]] = &[b"20261001t030405Z", b"2026100xT030405Z"];
#[test]
fn a_debian_snapshot_refuses_a_malformed_timestamp() {
    for (row, &written) in BAD_TIMESTAMPS.iter().enumerate() {
        let (_, faults) = inputs::debian(&[(
            b"snapshot",
            at(2, 2),
            map(
                at(2, 7),
                &[
                    (b"archive", at(2, 8), string(at(2, 18), written)),
                    (b"security", at(2, 40), string(at(2, 52), b"20261001T030405Z")),
                ],
            ),
        )]);
        assert_eq!(
            faults.as_ref(),
            &[Fault { schema: Schema::Malformed { format: Format::Timestamp }, document: 2, at: at(2, 18) }],
            "row {row}"
        );
    }
}
const DEBIAN_NAMES: &[(&[u8], bool)] = &[
    (b"amd64", true),
    (b"trixie", true),
    (b"a+.-9", true),
    (b"a", false),
    (b"-amd64", false),
    (b"Forky", false),
    (&[b'a'; 256], true),
    (&[b'a'; 257], false),
];
#[test]
fn debian_s_suite_and_architecture_follow_their_form() {
    for (row, &(written, expected)) in DEBIAN_NAMES.iter().enumerate() {
        assert_eq!(forms::check(Format::Debian, written), expected, "row {row}");
    }
}
const BAD_NAMES: &[(&[u8], &[u8])] = &[(b"suite", b"Forky"), (b"architecture", b"-amd64")];
#[test]
fn debian_refuses_a_suite_or_architecture_outside_its_form() {
    for (row, &(name, written)) in BAD_NAMES.iter().enumerate() {
        let (_, faults) = inputs::debian(&[(name, at(2, 2), string(at(2, 7), written))]);
        assert_eq!(
            faults.as_ref(),
            &[Fault { schema: Schema::Malformed { format: Format::Debian }, document: 2, at: at(2, 7) }],
            "row {row}"
        );
    }
}
#[test]
fn an_undeclared_debian_field_is_refused_with_the_declared() {
    let (_, faults) = inputs::debian(&[(b"typo", at(2, 2), int(at(2, 7), 1))]);
    assert_eq!(
        faults.as_ref(),
        &[Fault {
            schema: Schema::UndeclaredField {
                field: copy_of(b"typo"),
                declared: Box::new([
                    copy_of(b"architecture"),
                    copy_of(b"components"),
                    copy_of(b"install_recommends"),
                    copy_of(b"pockets"),
                    copy_of(b"snapshot"),
                    copy_of(b"suite")
                ])
            },
            document: 2,
            at: at(2, 2)
        }]
    );
}
const WRONG_FIELDS: &[(&[u8], crate::errors::TypeName)] =
    &[(b"suite", crate::errors::TypeName::String), (b"architecture", crate::errors::TypeName::String)];
#[test]
fn a_debian_field_of_another_type_is_refused() {
    for (row, (name, expected)) in WRONG_FIELDS.iter().enumerate() {
        let (_, faults) = inputs::debian(&[(name, at(2, 2), int(at(2, 7), 1))]);
        assert_eq!(
            faults.as_ref(),
            &[Fault { schema: Schema::Wrong { expected: expected.clone() }, document: 2, at: at(2, 7) }],
            "row {row}"
        );
    }
}
#[test]
fn an_unknown_component_is_refused_at_its_key() {
    let (_, faults) = inputs::debian(&[(
        b"components",
        at(2, 2),
        map(at(2, 7), &[(b"typo", at(2, 8), revision::Value { at: at(2, 16), kind: revision::Kind::Bool(true) })]),
    )]);
    assert_eq!(
        faults.as_ref(),
        &[Fault {
            schema: Schema::Enum {
                value: copy_of(b"typo"),
                listed: Box::new([
                    copy_of(b"contrib"),
                    copy_of(b"main"),
                    copy_of(b"non-free"),
                    copy_of(b"non-free-firmware")
                ])
            },
            document: 2,
            at: at(2, 8)
        }]
    );
}
const MAIN_OFF: &[(revision::Kind, revision::Position)] =
    &[(revision::Kind::Bool(false), at(2, 16)), (revision::Kind::Null, at(2, 2))];
#[test]
fn components_with_main_off_are_refused() {
    for (row, (kind, position)) in MAIN_OFF.iter().enumerate() {
        let (_, faults) = inputs::debian(&[(
            b"components",
            at(2, 2),
            map(at(2, 7), &[(b"main", at(2, 8), revision::Value { at: at(2, 16), kind: kind.clone() })]),
        )]);
        assert_eq!(faults.as_ref(), &[Fault { schema: Schema::Main, document: 2, at: *position }], "row {row}");
    }
}
#[test]
fn components_without_main_keep_it_on() {
    let (value, faults) = inputs::debian(&[(
        b"components",
        at(2, 2),
        map(at(2, 7), &[(b"contrib", at(2, 8), revision::Value { at: at(2, 16), kind: revision::Kind::Bool(true) })]),
    )]);
    assert_eq!(faults.as_ref(), &[]);
    let Value::Map(entries) = value.expect("sound Debian") else { panic!("object") };
    let [_, components, _, _, _, _] = entries.as_ref() else { panic!("six fields") };
    assert_eq!(
        components.value,
        Some(Value::Map(Box::new([
            Entry { key: copy_of(b"contrib"), value: Some(Value::Bool(true)) },
            Entry { key: copy_of(b"main"), value: Some(Value::Bool(true)) }
        ])))
    );
}
