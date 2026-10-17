use crate::{
    errors::{Format, Schema},
    kinds::preseeds,
    parts::packages::{self, Microcode},
    revision::{
        Runs,
        declaration::{Map, Name},
    },
    tests::fixtures,
};
use fixtures::PRESEED_FIELDS as SELECT;
use skein_lib::bytes::copy_of;
#[test]
fn the_preseeds_leave_the_resolution_inputs_alone() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.preseeds = Box::new([fixtures::preseed_answer()]);
    assert_eq!(packages::inputs(&input), packages::inputs(&edited));
    assert_ne!(packages::digest(&input), packages::digest(&edited));
}
#[test]
fn the_initramfs_modules_leave_the_resolution_inputs_alone() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.initramfs.as_mut().expect("metal").modules = Box::new([copy_of(b"btrfs")]);
    assert_eq!(packages::inputs(&input), packages::inputs(&edited));
    assert_ne!(packages::digest(&input), packages::digest(&edited));
}
#[test]
fn the_microcode_choice_leaves_the_resolution_inputs_alone() {
    let input = fixtures::thin_packages();
    let mut edited = input.clone();
    edited.initramfs.as_mut().expect("metal").microcode = Microcode::None;
    assert_eq!(packages::inputs(&input), packages::inputs(&edited));
    assert_ne!(packages::digest(&input), packages::digest(&edited));
}
#[test]
fn a_preseed_owner_defaults_to_the_questions_first_segment() {
    let preseed = preseeds::entry(&fixtures::preseed(b"tzdata/Areas", SELECT)).expect("valid answer");
    assert_eq!(preseed.owner.as_ref(), b"tzdata");
}
type Fields = &'static [(Name, &'static [u8])];
const BAD_NAMES: &[(&[u8], Fields)] = &[
    (b"tzdata", SELECT),
    (b"a b/Areas", SELECT),
    (b"/Areas", SELECT),
    (b"tzdata/", SELECT),
    (b"tzdata/Areas", &[(Name::Type, b"select"), (Name::Value, b"Europe"), (Name::Owner, b"a b")]),
    (b"tzdata/Areas", &[(Name::Type, b"select"), (Name::Value, b"Europe"), (Name::Owner, b"a/b")]),
];
#[test]
fn a_preseed_question_or_owner_out_of_form_is_malformed() {
    for (row, &(question, fields)) in BAD_NAMES.iter().enumerate() {
        assert_eq!(
            preseeds::entry(&fixtures::preseed(question, fields)).expect_err("invalid name").0,
            Schema::Malformed { format: Format::Preseed },
            "row {row}"
        );
    }
}
const BAD_VALUES: &[&[u8]] = &[b"Eu\nrope", b"Europe\t", b"Europe\x00", b"Europe\x7f", b"Europe\\"];
#[test]
fn a_preseed_value_with_a_control_byte_or_a_final_backslash_is_malformed() {
    for (row, value) in BAD_VALUES.iter().enumerate() {
        assert_eq!(
            preseeds::entry(&fixtures::preseed(b"tzdata/Areas", &[(Name::Type, b"select"), (Name::Value, value)])),
            Err((Schema::Malformed { format: Format::Preseed }, fixtures::FIELD_AT)),
            "row {row}"
        );
    }
}
const BOOLEANS: &[(&[u8], bool)] =
    &[(b"true", true), (b"false", true), (b"yes", false), (b"True", false), (b"", false)];
#[test]
fn a_boolean_preseed_is_true_or_false() {
    for (row, &(value, valid)) in BOOLEANS.iter().enumerate() {
        assert_eq!(
            preseeds::entry(&fixtures::preseed(b"tzdata/Areas", &[(Name::Type, b"boolean"), (Name::Value, value)]))
                .is_ok(),
            valid,
            "row {row}"
        );
    }
}
const MODULE_LENGTHS: &[(usize, bool)] = &[
    (fixtures::MIN_MODULE_NAME - 1, false),
    (fixtures::MIN_MODULE_NAME, true),
    (fixtures::MAX_MODULE_NAME, true),
    (fixtures::MAX_MODULE_NAME + 1, false),
];
const BAD_MODULES: &[&[u8]] = &[b"a.b", b"a/b", b"a b", b"a+b"];
#[test]
fn a_module_name_out_of_form_or_past_55_bytes_is_malformed() {
    for (row, name) in BAD_MODULES.iter().enumerate() {
        assert_eq!(
            preseeds::module(&fixtures::module(name)),
            Err((Schema::Malformed { format: Format::Module }, fixtures::composition::at(1, 14))),
            "row {row}"
        );
    }
}
#[test]
fn module_names_have_the_kernels_length_bounds() {
    for (row, &(length, valid)) in MODULE_LENGTHS.iter().enumerate() {
        assert_eq!(
            preseeds::module(&fixtures::module(fixtures::MODULE_NAME_BYTES.get(..length).expect("module bound")))
                .is_ok(),
            valid,
            "row {row}"
        );
    }
}
const MISSING: &[(Fields, &[u8])] = &[(&[(Name::Value, b"Europe")], b"type"), (&[(Name::Type, b"select")], b"value")];
#[test]
fn a_preseed_without_type_or_value_is_a_missing_field() {
    for (row, &(fields, field)) in MISSING.iter().enumerate() {
        assert_eq!(
            preseeds::entry(&fixtures::preseed(b"tzdata/Areas", fields)),
            Err((Schema::MissingField { field: copy_of(field) }, fixtures::composition::at(1, 14))),
            "row {row}"
        );
    }
}
const UNLISTED: &[&[u8]] = &[b"password", b"text"];
#[test]
fn an_unlisted_preseed_type_is_an_enum_fault() {
    for (row, kind) in UNLISTED.iter().enumerate() {
        assert_eq!(
            preseeds::entry(&fixtures::preseed(b"tzdata/Areas", &[(Name::Type, kind), (Name::Value, b"Europe")])),
            Err((
                Schema::Enum {
                    value: copy_of(kind),
                    listed: Box::new([
                        copy_of(b"select"),
                        copy_of(b"multiselect"),
                        copy_of(b"string"),
                        copy_of(b"boolean")
                    ])
                },
                fixtures::FIELD_AT
            )),
            "row {row}"
        );
    }
}
#[test]
fn modules_are_a_sorted_union_without_repeats() {
    let packages = fixtures::declarations_packages(&fixtures::module_expansion(), Runs::Metal).expect("valid modules");
    assert_eq!(packages.initramfs.expect("metal").modules.as_ref(), [copy_of(b"btrfs"), copy_of(b"nvme")]);
}
#[test]
fn a_container_has_no_initramfs_whatever_tasks_declare() {
    let packages =
        fixtures::declarations_packages(&fixtures::module_expansion(), Runs::Container).expect("valid container");
    assert_eq!(packages.initramfs, None);
}
const MICROCODE: &[(Runs, Microcode)] = &[(Runs::Metal, Microcode::Generic), (Runs::Vm, Microcode::None)];
#[test]
fn the_microcode_is_generic_on_metal_and_none_on_a_vm() {
    for (row, &(kind, microcode)) in MICROCODE.iter().enumerate() {
        let packages = fixtures::declarations_packages(&fixtures::concrete_expansion(Map::Initramfs, &[]), kind)
            .expect("valid placement");
        assert_eq!(packages.initramfs.expect("metal or VM").microcode, microcode, "row {row}");
    }
}
