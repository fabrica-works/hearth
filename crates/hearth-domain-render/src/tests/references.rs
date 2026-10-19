use super::fixtures::{self, Reference, ReferenceFields, key, name, op};
use crate::{
    errors::{Root, Schema},
    evaluate::{Missing, Op, template::Instruction},
    schema::references,
};
use alloc::boxed::Box;

const SETTINGS: &[(Missing, &[u8])] = &[(Missing::Error, b"greting"), (Missing::Undefined, b"greting")];
#[test]
fn a_settings_reference_must_be_a_declared_setting() {
    for (row, &(missing, setting)) in SETTINGS.iter().enumerate() {
        let expression = fixtures::expression([name(0, b"settings", missing), key(9, setting, missing)]);
        assert_eq!(
            fixtures::check_reference(&expression),
            Some((
                Schema::Undeclared {
                    setting: Box::from(&b"greting"[..]),
                    declared: Box::new([Box::from(&b"greeting"[..]), Box::from(&b"port"[..])]),
                },
                9
            )),
            "row {row}: protocol/format.md, 6.4"
        );
    }
}
const LISTED: &[Reference] = &[
    Reference { root: b"tasks", task: b"core", setting: Some(b"domain") },
    Reference { root: b"exports", task: b"mail", setting: None },
];
#[test]
fn tasks_and_exports_references_in_reads_pass() {
    for (row, &reference) in LISTED.iter().enumerate() {
        assert_eq!(fixtures::check_reference(&fixtures::reference_expression(reference)), None, "row {row}");
    }
}
const UNLISTED: &[(Reference, ReferenceFields<'static>)] = &[
    (
        Reference { root: b"tasks", task: b"core", setting: Some(b"port") },
        (b"", Root::Tasks, b"core", Some(b"port"), 0),
    ),
    (Reference { root: b"exports", task: b"core", setting: None }, (b"", Root::Exports, b"core", None, 0)),
];
#[test]
fn tasks_and_exports_references_must_be_in_reads() {
    for (row, &(reference, expected)) in UNLISTED.iter().enumerate() {
        let fault = fixtures::check_reference(&fixtures::reference_expression(reference)).expect("unlisted read");
        let Schema::Unlisted { reader, root, task, setting } = &fault.0 else { panic!("unlisted read: {fault:?}") };
        assert_eq!((reader.as_ref(), *root, task.as_ref(), setting.as_deref(), fault.1), expected, "row {row}");
    }
}
const DYNAMIC: &[(&[u8], fixtures::ReferenceFault)] = &[
    (b"tasks", Some((Schema::Dynamic { root: Root::Tasks }, 0))),
    (b"exports", Some((Schema::Dynamic { root: Root::Exports }, 0))),
];
#[test]
fn tasks_and_exports_take_only_literal_names() {
    for (row, (root, expected)) in DYNAMIC.iter().enumerate() {
        let expression = fixtures::expression([
            name(0, root, Missing::Error),
            name(6, b"key", Missing::Error),
            op(5, Op::Item { missing: Missing::Error }),
        ]);
        assert_eq!(fixtures::check_reference(&expression), *expected, "row {row}");
    }
}
const DEFINED: &[bool] = &[false, true];
#[test]
fn a_tasks_name_under_is_defined_passes_for_an_optional_read_only() {
    for (row, &negated) in DEFINED.iter().enumerate() {
        let test = if negated { Op::NotDefined } else { Op::Defined };
        let expression = fixtures::expression([
            name(0, b"tasks", Missing::Undefined),
            key(6, b"wifi", Missing::Undefined),
            op(11, test),
        ]);
        assert_eq!(fixtures::check_reference(&expression), None, "row {row}");
    }
}
const REQUIRED_READS: &[(&[u8], ReferenceFields<'static>)] =
    &[(b"core", (b"", Root::Tasks, b"core", None, 0)), (b"nope", (b"", Root::Tasks, b"nope", None, 0))];
#[test]
fn a_required_or_unknown_read_under_is_defined_is_unlisted() {
    for (row, &(task, expected)) in REQUIRED_READS.iter().enumerate() {
        let expression = fixtures::expression([
            name(0, b"tasks", Missing::Undefined),
            key(6, task, Missing::Undefined),
            op(11, Op::Defined),
        ]);
        let fault = fixtures::check_reference(&expression).expect("unlisted read");
        let Schema::Unlisted { reader, root, task: read, setting } = &fault.0 else {
            panic!("unlisted read: {fault:?}")
        };
        assert_eq!((reader.as_ref(), *root, read.as_ref(), setting.as_deref(), fault.1), expected, "row {row}");
    }
}
#[test]
fn a_dynamic_settings_key_is_not_checked() {
    let expression = fixtures::expression([
        name(0, b"settings", Missing::Error),
        name(9, b"key", Missing::Error),
        op(8, Op::Item { missing: Missing::Error }),
    ]);
    assert_eq!(fixtures::check_reference(&expression), None);
}
#[test]
fn every_instructions_expression_of_a_template_is_checked() {
    let expression = fixtures::reference_typo();
    for (row, does) in fixtures::reference_instructions(expression).into_iter().enumerate() {
        let template = fixtures::template(&[0], [Instruction { at: 0, does }]);
        let (_, offset) = fixtures::template_references(&template).expect("schema fault");
        assert_eq!(offset, fixtures::REFERENCE_OFFSET, "row {row}");
    }
}
const DECLARATION_PLACES: &[(fixtures::ReferencePlace, u32)] = &[
    (fixtures::ReferencePlace::Guard, 20),
    (fixtures::ReferencePlace::If, 20),
    (fixtures::ReferencePlace::ForEach, 20),
    (fixtures::ReferencePlace::Key, 21),
    (fixtures::ReferencePlace::Text, 21),
    (fixtures::ReferencePlace::Typed, 21),
    (fixtures::ReferencePlace::Strings, 21),
    (fixtures::ReferencePlace::Range, 21),
];
#[test]
fn every_declarations_expression_is_checked_at_its_string() {
    for (row, &(place, column)) in DECLARATION_PLACES.iter().enumerate() {
        let mut faults = skein_lib::List::with_capacity(0);
        references::declaration(&fixtures::reference_declaration(place), 7, &mut faults);
        assert_eq!(
            faults.as_slice(),
            &[crate::schema::Fault {
                schema: Schema::Undeclared { setting: Box::from(&b"greting"[..]), declared: Box::new([]) },
                document: 7,
                at: crate::revision::Position { line: 3, column },
            }],
            "row {row}"
        );
    }
}
#[test]
fn a_typo_in_an_untaken_branch_of_a_template_is_found_at_its_offset() {
    let template = fixtures::template(
        &[0],
        [
            fixtures::branch(0, fixtures::expression([op(0, Op::Boolean(false))]), 1),
            fixtures::output(12, fixtures::reference_typo()),
        ],
    );
    let (_, offset) = fixtures::template_references(&template).expect("schema fault");
    assert_eq!(offset, fixtures::REFERENCE_OFFSET);
}
const LITERAL_SETTINGS: &[&[u8]] = &[b"greeting", b"port"];
#[test]
fn a_literal_settings_item_is_a_declared_setting() {
    for (row, &setting) in LITERAL_SETTINGS.iter().enumerate() {
        let expression = fixtures::expression([
            name(0, b"settings", Missing::Error),
            op(9, Op::String(skein_lib::bytes::copy_of(setting))),
            op(8, Op::Item { missing: Missing::Error }),
        ]);
        assert_eq!(fixtures::check_reference(&expression), None, "row {row}");
    }
}
const TASK_VALUES: &[Op] = &[Op::Filter(crate::evaluate::Filter::Tojson), Op::Item { missing: Missing::Error }];
#[test]
fn a_tasks_value_without_a_literal_setting_is_dynamic() {
    for (row, suffix) in TASK_VALUES.iter().enumerate() {
        let expression = fixtures::expression([
            name(0, b"tasks", Missing::Error),
            key(6, b"core", Missing::Error),
            op(11, suffix.clone()),
        ]);
        assert_eq!(
            fixtures::check_reference(&expression),
            Some((Schema::Dynamic { root: Root::Tasks }, 0)),
            "row {row}"
        );
    }
}
#[test]
fn a_reference_under_is_defined_is_checked_too() {
    let expression = fixtures::expression([
        name(12, b"settings", Missing::Undefined),
        key(fixtures::REFERENCE_OFFSET, b"greting", Missing::Undefined),
        op(29, Op::Defined),
    ]);
    let (_, offset) = fixtures::check_reference(&expression).expect("schema fault");
    assert_eq!(offset, fixtures::REFERENCE_OFFSET);
}
