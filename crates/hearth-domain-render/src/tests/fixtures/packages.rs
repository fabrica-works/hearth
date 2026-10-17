use crate::{
    entries::{Concrete, Expanded, Expansion, Group},
    parts::packages::*,
    revision::declaration::{Map, Name},
};
use skein_lib::{List, bytes::copy_of};
pub const MIN_PACKAGE_NAME: usize = 2;
pub const MAX_PACKAGE_NAME: usize = 256;
pub fn thin_packages() -> Packages {
    Packages {
        initramfs_file: None,
        sources: Sources {
            suite: copy_of(b"forky"),
            architecture: copy_of(b"amd64"),
            components: Box::new([Component::Main]),
            archive: copy_of(b"20261001T030405Z"),
            updates: false,
            security: None,
        },
        request: Box::new([]),
        variant: Variant::Apt,
        preseeds: Box::new([]),
        initramfs: Some(Initramfs { modules: Box::new([]), microcode: Microcode::Generic }),
        apt: Apt { install_recommends: false },
    }
}
pub fn bytes(value: &[u8]) -> hearth_lib::encode::Item {
    hearth_lib::encode::Item::Bytes(copy_of(value))
}
pub fn package(name: &[u8], component: Option<&[u8]>) -> Concrete {
    let fields = match component {
        Some(value) => &[(Name::Component, value)][..],
        None => &[],
    };
    let expansion = super::concrete_expansion(Map::Packages, &[(name, fields)]);
    super::only(&super::only(&super::only(&expansion.tasks).groups).entries).clone()
}
type PackageRow<'a> = (&'a [u8], &'a [u8], Option<&'a [u8]>);
pub const PACKAGE_NAME_BYTES: &[u8; MAX_PACKAGE_NAME + 1] = &[b'a'; MAX_PACKAGE_NAME + 1];
pub fn package_expansion(rows: &[PackageRow<'_>]) -> Expansion {
    let mut tasks = List::with_capacity(u32::try_from(rows.len()).expect("fixture count"));
    for &(task, name, component) in rows {
        tasks
            .push(Expanded {
                task: copy_of(task),
                groups: Box::new([Group { map: Map::Packages, entries: Box::new([package(name, component)]) }]),
            })
            .expect("fixture count");
    }
    Expansion { tasks: tasks.into_boxed() }
}
pub fn built_packages(rows: &[PackageRow<'_>]) -> Result<Packages, Box<[crate::errors::Error]>> {
    build(&super::checked_debian(), &super::containers::placement(None), &package_expansion(rows), None)
}
pub fn packages_root(components: &[(&[u8], bool)], security: bool, timestamp: &[u8]) -> crate::evaluate::Value {
    use super::composition::{at, map, string};
    use crate::revision::{Kind, Value};
    let mut fields = List::with_capacity(u32::try_from(components.len()).expect("fixture count"));
    for &(name, enabled) in components {
        fields.push((name, at(1, 1), Value { at: at(1, 1), kind: Kind::Bool(enabled) })).expect("fixture count");
    }
    super::forms::debian(&[
        (b"components", at(1, 1), map(at(1, 1), fields.as_slice())),
        (
            b"pockets",
            at(1, 1),
            map(at(1, 1), &[(b"security", at(1, 1), Value { at: at(1, 1), kind: Kind::Bool(security) })]),
        ),
        (
            b"snapshot",
            at(1, 1),
            map(
                at(1, 1),
                &[
                    (b"archive", at(1, 1), string(at(1, 1), b"20261001T030405Z")),
                    (b"security", at(1, 1), string(at(1, 1), timestamp)),
                ],
            ),
        ),
    ])
    .0
    .expect("valid fixed schema")
}
pub const MIN_MODULE_NAME: usize = 1;
pub const MAX_MODULE_NAME: usize = 55;
pub const MODULE_NAME_BYTES: &[u8; MAX_MODULE_NAME + 1] = &[b'a'; MAX_MODULE_NAME + 1];
pub fn preseed(question: &[u8], fields: &[(Name, &[u8])]) -> Concrete {
    let expansion = super::concrete_expansion(Map::Preseeds, &[(question, fields)]);
    super::only(&super::only(&super::only(&expansion.tasks).groups).entries).clone()
}
pub fn module(name: &[u8]) -> Concrete {
    let expansion = super::concrete_expansion(Map::Initramfs, &[(name, &[])]);
    super::only(&super::only(&super::only(&expansion.tasks).groups).entries).clone()
}
pub fn preseed_answer() -> Preseed {
    Preseed {
        question: copy_of(b"tzdata/Areas"),
        owner: copy_of(b"tzdata"),
        kind: PreseedKind::Select,
        value: copy_of(b"Europe"),
    }
}
pub fn declarations_packages(
    expansion: &Expansion,
    kind: crate::revision::Runs,
) -> Result<Packages, Box<[crate::errors::Error]>> {
    let mut placement = super::containers::placement(None);
    placement.kind = kind;
    build(&super::checked_debian(), &placement, expansion, None)
}
pub fn module_expansion() -> Expansion {
    let mut expansion = super::concrete_expansion(Map::Initramfs, &[(b"nvme", &[]), (b"btrfs", &[])]);
    let task = super::only(&expansion.tasks).clone();
    expansion.tasks = Box::new([task.clone(), Expanded { task: copy_of(b"other"), groups: task.groups }]);
    expansion
}

pub const PRESEED_FIELDS: &[(Name, &[u8])] = &[(Name::Type, b"select"), (Name::Value, b"Europe")];
pub const MIN_CMDLINE_ORDER: i64 = 0;
pub const DEFAULT_CMDLINE_ORDER: u16 = 500;
pub const MAX_CMDLINE_ORDER: i64 = 999;
pub fn cmdline(key: &[u8], param: Option<&[u8]>, order: Option<i64>) -> Concrete {
    let fields = match param {
        Some(param) => &[(Name::Param, param)][..],
        None => &[],
    };
    let expansion = super::concrete_expansion(Map::Cmdline, &[(key, fields)]);
    let mut entry = super::only(&super::only(&super::only(&expansion.tasks).groups).entries).clone();
    if let Some(order) = order {
        let mut fields = List::with_capacity(2);
        for field in entry.fields {
            fields.push(field).expect("fixture count");
        }
        fields
            .push(crate::entries::Made {
                name: Name::Order,
                at: super::FIELD_AT,
                value: crate::entries::Fielded::Int(order),
            })
            .expect("fixture count");
        entry.fields = fields.into_boxed();
    }
    entry
}
type ParameterRow<'a> = (&'a [u8], &'a [u8], &'a [u8], Option<i64>);
pub fn boot_expansion(rows: &[ParameterRow<'_>]) -> Expansion {
    let mut tasks = List::with_capacity(u32::try_from(rows.len()).expect("fixture count"));
    for &(task, key, param, order) in rows {
        tasks
            .push(Expanded {
                task: copy_of(task),
                groups: Box::new([Group { map: Map::Cmdline, entries: Box::new([cmdline(key, Some(param), order)]) }]),
            })
            .expect("fixture count");
    }
    Expansion { tasks: tasks.into_boxed() }
}
pub fn built_boot(rows: &[ParameterRow<'_>]) -> crate::parts::boot::Boot {
    crate::parts::boot::build(&boot_expansion(rows), &super::containers::placement(None)).expect("valid parameters")
}

#[must_use]
pub fn parts_changed(before: crate::parts::PartRecord, after: crate::parts::PartRecord) -> crate::diff::TargetChange {
    use crate::{
        diff::{self, Made, Needs},
        parts::Part,
    };
    let before = super::spec(b"x1-prod", &[Part { digest: hearth_lib::encode::Digest([1; 32]), record: before }]);
    let mut after = super::spec(b"x1-prod", &[Part { digest: hearth_lib::encode::Digest([2; 32]), record: after }]);
    after.digest = hearth_lib::encode::Digest([3; 32]);
    diff::compare(Some(&Made::Spec(Box::new(before))), Some(&Made::Spec(Box::new(after))), Needs::Known)
}
