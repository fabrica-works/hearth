use super::fixtures::{
    composition::{at, int, map, string},
    forms as inputs, schemas,
};
use crate::{
    errors::{Format, Schema},
    evaluate::{Entry, Value},
    limits::{self, Limits},
    revision::{
        self, Mode,
        declaration::{Form, Spec},
    },
    schema::{self, Check, Fault, forms},
};
use skein_lib::{List, bytes::copy_of};
const FORMS: &[(Format, &[u8], bool)] = &[
    (Format::Path, b"/etc/motd", true),
    (Format::Path, b"/", true),
    (Format::Path, b"", false),
    (Format::Path, b"etc/motd", false),
    (Format::Path, b"/a//b", false),
    (Format::Path, b"/a/./b", false),
    (Format::Path, b"/a/../b", false),
    (Format::Path, b"/a/", false),
    (Format::Mode, b"644", true),
    (Format::Mode, b"7777", true),
    (Format::Mode, b"64", false),
    (Format::Mode, b"0844", false),
    (Format::Mode, b"17777", false),
    (Format::Duration, b"14d", true),
    (Format::Duration, b"106751991167300d", true),
    (Format::Duration, b"106751991167301d", false),
    (Format::Duration, b"5", false),
    (Format::Duration, b"05s", false),
    (Format::Duration, b"0s", false),
    (Format::Duration, b"5w", false),
    (Format::Secret, b"users/free.passwd", true),
    (Format::Secret, b"users/../x", false),
    (Format::Secret, b"Users/x", false),
    (Format::Secret, b"a//b", false),
    (Format::Secret, b".", false),
    (Format::File, b"lab-ca/lab.crt", true),
    (Format::File, b"/a", false),
    (Format::File, b"a/", false),
    (Format::File, b"a/../b", false),
];
#[test]
fn strings_follow_their_type_s_form() {
    for (row, &(format, written, expected)) in FORMS.iter().enumerate() {
        assert_eq!(forms::check(format, written), expected, "{format:?} row {row}");
    }
}
#[test]
fn a_duration_is_its_seconds() {
    assert_eq!(
        inputs::validate(Form::Duration, &inputs::setting(b"14d"), &Check::Default, &[], &limits::SHIPPED),
        Ok(Value::Map(Box::new([Entry { key: copy_of(b"x"), value: Some(Value::Int(1_209_600)) }])))
    );
}
const LENGTHS: &[(usize, bool)] = &[(255, true), (256, false)];
#[test]
fn a_secret_name_is_at_most_255_bytes() {
    for &(length, expected) in LENGTHS {
        assert_eq!(
            forms::check(Format::Secret, inputs::NAME_BYTES.get(..length).expect("name boundary")),
            expected,
            "{length}"
        );
    }
}
const KEYS: &[(usize, bool)] = &[(0, false), (1, true), (255, true), (256, false)];
#[test]
fn a_map_key_is_one_to_255_bytes() {
    for &(length, expected) in KEYS {
        assert_eq!(
            forms::check(Format::Key, inputs::NAME_BYTES.get(..length).expect("name boundary")),
            expected,
            "{length}"
        );
    }
}
const MAP_KEYS: &[(&[u8], Format)] = &[(b"", Format::Key), (&[b'x'; 256], Format::Key)];
#[test]
fn a_map_setting_refuses_keys_outside_their_form() {
    let mut declaration = schemas::typed_declaration(Form::Map(1));
    declaration.types = Box::new([
        revision::declaration::Type { at: at(1, 1), form: Form::Map(1) },
        revision::declaration::Type { at: at(1, 2), form: Form::Int },
    ]);
    for (row, &(key, format)) in MAP_KEYS.iter().enumerate() {
        let value = schemas::composed(&map(
            at(2, 1),
            &[(b"x", at(2, 2), map(at(2, 7), &[(key, at(2, 8), int(at(2, 12), 1))]))],
        ));
        assert_eq!(
            schemas::validate(&declaration, &value, &Check::Default),
            Err(Box::<[Fault]>::from([Fault { schema: Schema::Malformed { format }, document: 2, at: at(2, 8) }])),
            "row {row}"
        );
    }
}
#[test]
fn a_target_must_be_of_the_environment() {
    let names = inputs::environment_targets();
    assert_eq!(
        inputs::validate(
            Form::Target,
            &inputs::setting(b"x9-prod"),
            &Check::Target { environment: b"prod", targets: &names[..] },
            &[],
            &limits::SHIPPED
        ),
        Err(Box::<[Fault]>::from([Fault {
            schema: Schema::Target { name: copy_of(b"x9-prod"), environment: copy_of(b"prod") },
            document: 2,
            at: at(2, 7)
        }]))
    );
}
#[test]
fn a_listed_target_is_accepted() {
    let names = inputs::environment_targets();
    assert_eq!(
        inputs::validate(
            Form::Target,
            &inputs::setting(b"x2-prod"),
            &Check::Target { environment: b"prod", targets: &names[..] },
            &[],
            &limits::SHIPPED
        ),
        Ok(Value::Map(Box::new([Entry { key: copy_of(b"x"), value: Some(Value::String(copy_of(b"x2-prod"))) }])))
    );
}
const NONFILES: &[Mode] = &[Mode::Link, Mode::Submodule];
#[test]
fn a_file_must_be_listed_as_a_file() {
    for (row, &mode) in NONFILES.iter().enumerate() {
        assert_eq!(
            inputs::validate(
                Form::File,
                &inputs::setting(b"lab-ca/lab.crt"),
                &Check::Default,
                &inputs::listing(mode, 8),
                &limits::SHIPPED
            ),
            Err(Box::<[Fault]>::from([Fault {
                schema: Schema::File { path: copy_of(b"lab-ca/lab.crt") },
                document: 2,
                at: at(2, 7)
            }])),
            "row {row}"
        );
    }
}
const FILES: &[Mode] = &[Mode::File, Mode::Executable];
#[test]
fn regular_and_executable_files_are_accepted() {
    for (row, &mode) in FILES.iter().enumerate() {
        assert_eq!(
            inputs::validate(
                Form::File,
                &inputs::setting(b"lab-ca/lab.crt"),
                &Check::Default,
                &inputs::listing(mode, 8),
                &Limits { static_file: 8, ..limits::SHIPPED }
            ),
            Ok(Value::Map(Box::new([Entry {
                key: copy_of(b"x"),
                value: Some(Value::String(copy_of(b"lab-ca/lab.crt")))
            }]))),
            "row {row}"
        );
    }
}
#[test]
fn a_file_past_the_static_limit_is_refused_at_its_setting() {
    assert_eq!(
        inputs::validate(
            Form::File,
            &inputs::setting(b"lab-ca/lab.crt"),
            &Check::Default,
            &inputs::listing(Mode::File, 9),
            &Limits { static_file: 8, ..limits::SHIPPED }
        ),
        Err(Box::<[Fault]>::from([Fault {
            schema: Schema::Large { path: copy_of(b"lab-ca/lab.crt"), max: 8 },
            document: 2,
            at: at(2, 7)
        }]))
    );
}
#[test]
fn a_default_failing_its_form_is_refused_once_a_revision() {
    let mut declaration = schemas::typed_declaration(Form::Path);
    declaration.settings =
        Box::new([Spec { default: Some(string(at(1, 9), b"bin/sh")), ..schemas::schema_spec(b"x", 0) }]);
    let (_, faults) = schemas::defaults(&declaration);
    assert_eq!(
        faults.as_ref(),
        &[(copy_of(b"task"), Fault { schema: Schema::Malformed { format: Format::Path }, document: 0, at: at(1, 9) })]
    );
}
mod debian;

#[test]
fn a_listed_file_default_is_validated_at_the_revision() {
    let mut declaration = schemas::typed_declaration(Form::File);
    declaration.settings =
        Box::new([Spec { default: Some(string(at(1, 9), b"lab-ca/lab.crt")), ..schemas::schema_spec(b"x", 0) }]);
    let (_, faults) = schemas::defaults_with_listing(&declaration, &inputs::listing(Mode::File, 8));
    assert_eq!(faults.as_ref(), &[]);
}
#[test]
fn a_missing_file_default_is_refused_once_a_revision() {
    let mut declaration = schemas::typed_declaration(Form::File);
    declaration.settings =
        Box::new([Spec { default: Some(string(at(1, 9), b"lab-ca/lab.crt")), ..schemas::schema_spec(b"x", 0) }]);
    let (_, faults) = schemas::defaults(&declaration);
    assert_eq!(
        faults.as_ref(),
        &[(
            copy_of(b"task"),
            Fault { schema: Schema::File { path: copy_of(b"lab-ca/lab.crt") }, document: 0, at: at(1, 9) }
        )]
    );
}
