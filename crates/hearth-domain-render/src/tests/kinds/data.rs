use super::super::fixtures::{
    self,
    kinds::{self as fixture, data},
};
use crate::{
    errors::{Format, Kind, Schema},
    parts::{
        self, PartRecord,
        data::{DataChange, DataClass, DataKeyed, DataValue},
    },
    revision::{
        Position,
        declaration::{Map, Name},
    },
};
#[test]
fn a_data_entry_defaults_to_the_data_class_and_0755() {
    assert_eq!(
        data::entry(b"journal", &[(Name::Path, b"/var/log/journal")]).expect("entry"),
        crate::kinds::data::Entry {
            name: b"journal".as_slice().into(),
            task: b"motd".as_slice().into(),
            at: fixture::FILE_AT,
            class: DataClass::Data,
            path: b"/var/log/journal".as_slice().into(),
            mode: 0o755,
            owner: b"root".as_slice().into(),
            group: b"root".as_slice().into()
        }
    );
}
const INVALID_NAMES: &[&[u8]] = &[b"", b"A", b"1a", b"a.b", b"a_b", &[b'a'; data::MAX_NAME + 1]];
#[test]
fn a_data_name_follows_2_4s_form() {
    for (row, name) in INVALID_NAMES.iter().enumerate() {
        assert_eq!(
            data::entry(name, &[(Name::Path, b"/var/lib/app")]).expect_err("name").kind,
            Kind::Schema(Schema::Malformed { format: Format::DataName }),
            "row {row}"
        );
    }
}
const VALID_NAMES: &[&[u8]] = &[b"a", b"a-0", b"a-", &[b'a'; data::MAX_NAME]];
#[test]
fn valid_data_names_are_kept() {
    for (row, name) in VALID_NAMES.iter().enumerate() {
        assert_eq!(
            data::entry(name, &[(Name::Path, b"/var/lib/app")]).expect("name").name.as_ref(),
            *name,
            "row {row}"
        );
    }
}
#[test]
fn a_path_is_required() {
    assert_eq!(
        data::entry(b"journal", &[]).expect_err("path").kind,
        Kind::Schema(Schema::MissingField { field: b"path".as_slice().into() })
    );
}
#[test]
fn no_entry_makes_an_empty_data_part() {
    assert_eq!(
        parts::data::build(&fixtures::concrete_expansion(Map::Data, &[])).expect("empty"),
        parts::data::Data { entries: Box::new([]) }
    );
}
#[test]
fn a_65th_data_entry_errs_above_at_its_name() {
    assert_eq!(
        fixtures::only(&parts::data::build(&data::many(data::MAX_ENTRIES + 1)).expect_err("too many")),
        &crate::errors::Error {
            side: None,
            kind: Kind::Schema(Schema::Above { max: i64::from(data::MAX_ENTRIES) }),
            class: crate::vocabulary::Class::Configuration,
            at: Some(crate::errors::Place {
                path: b"tasks/motd/task.json".as_slice().into(),
                position: Some(Position { line: 65, column: 5 })
            }),
            also: None,
            target: None,
            task: Some(b"motd".as_slice().into())
        }
    );
}
#[test]
fn a_data_change_alone_rebuilds_slot_only() {
    let change = fixtures::packages::parts_changed(
        PartRecord::Data(data::part(DataClass::Core)),
        PartRecord::Data(data::part(DataClass::Data)),
    );
    assert_eq!(
        change,
        crate::diff::TargetChange::Changed {
            rebuild: crate::diff::Rebuild::SlotOnly,
            parts: Box::new([crate::diff::PartChange::Data(DataChange {
                entries: Box::new([DataKeyed::Changed {
                    key: b"iwd".as_slice().into(),
                    before: DataValue { class: DataClass::Core, path: b"/var/lib/iwd".as_slice().into() },
                    after: DataValue { class: DataClass::Data, path: b"/var/lib/iwd".as_slice().into() }
                }])
            })])
        }
    );
}
const CLASSES: &[(&[u8], DataClass)] =
    &[(b"core", DataClass::Core), (b"data", DataClass::Data), (b"cache", DataClass::Cache)];
#[test]
fn each_data_class_is_typed() {
    for (row, &(name, class)) in CLASSES.iter().enumerate() {
        assert_eq!(
            data::entry(b"app", &[(Name::Path, b"/var/lib/app"), (Name::Class, name)]).expect("class").class,
            class,
            "row {row}"
        );
    }
}
const BAD_PATHS: &[&[u8]] = &[b"relative", b"/var/../app", b"/var//app"];
#[test]
fn data_paths_follow_the_path_form() {
    for (row, &path) in BAD_PATHS.iter().enumerate() {
        assert_eq!(
            data::entry(b"app", &[(Name::Path, path)]).expect_err("path").kind,
            Kind::Schema(Schema::Malformed { format: Format::Path }),
            "row {row}"
        );
    }
}
#[test]
fn an_unknown_data_class_lists_the_three_classes() {
    assert_eq!(
        data::entry(b"app", &[(Name::Path, b"/var/lib/app"), (Name::Class, b"cold")]).expect_err("class").kind,
        Kind::Schema(Schema::Enum {
            value: b"cold".as_slice().into(),
            listed: Box::new([b"core".as_slice().into(), b"data".as_slice().into(), b"cache".as_slice().into()])
        })
    );
}

#[test]
fn the_data_entry_limit_is_inclusive() {
    let part = parts::data::build(&data::many(data::MAX_ENTRIES)).expect("at limit");
    assert_eq!(u32::try_from(part.entries.len()).expect("bounded entries"), data::MAX_ENTRIES);
}
const BAD_MODES: &[&[u8]] = &[b"0888", b"64", b"10000"];
#[test]
fn data_modes_follow_the_mode_form() {
    for (row, &mode) in BAD_MODES.iter().enumerate() {
        assert_eq!(
            data::entry(b"app", &[(Name::Path, b"/var/lib/app"), (Name::Mode, mode)]).expect_err("mode").kind,
            Kind::Schema(Schema::Malformed { format: Format::Mode }),
            "row {row}"
        );
    }
}

#[test]
fn the_empty_data_digest_matches_the_independent_vector() {
    assert_eq!(
        parts::data::digest(&parts::data::Data { entries: Box::new([]) }),
        fixtures::digest("2d4765200bab698719b36d2feeeb9662babcfb1b0d00caa7d0e6187387068461")
    );
}
