use super::fixtures::{self, additions as inputs};
use crate::{
    additions,
    parts::{
        config::{Line, LineKind, Tmpfile},
        packages::Microcode,
    },
    revision::declaration::Name,
};
use skein_lib::bytes::copy_of;
const RANGES: &[(&[u8], i64, i64)] = &[(b"free", 165_536, 65_536), (b"ci", 100_000, 65_536)];
#[test]
fn subordinate_ranges_become_lines_by_user_name() {
    assert_eq!(
        additions::subordinate(&inputs::accounts_with_ranges(RANGES)).expect("two ranges").as_ref(),
        b"ci:100000:65536\nfree:165536:65536\n",
        "subuid(5), by name"
    );
}
#[test]
fn no_subordinate_range_makes_no_file() {
    assert_eq!(additions::subordinate(&fixtures::kinds::accounts([]).expect("user without range")), None);
}
#[test]
fn a_data_entry_becomes_a_d_line_with_its_mode_and_owners() {
    let entry = fixtures::kinds::data::entry(
        b"journal",
        &[
            (Name::Path, b"/var/log/journal"),
            (Name::Mode, b"2755"),
            (Name::Owner, b"root"),
            (Name::Group, b"systemd-journal"),
        ],
    )
    .expect("data");
    assert_eq!(
        additions::data_lines(core::slice::from_ref(&entry)).as_ref(),
        [additions::DataLine {
            task: copy_of(b"motd"),
            at: fixtures::kinds::FILE_AT,
            tmpfile: Tmpfile {
                path: copy_of(b"/var/log/journal"),
                line: Line {
                    kind: LineKind::D,
                    mode: Some(0o2755),
                    owner: Some(copy_of(b"root")),
                    group: Some(copy_of(b"systemd-journal")),
                    age: None,
                    argument: None
                }
            },
        }]
    );
}
const MICROCODE: &[(Microcode, &[u8])] = &[
    (Microcode::Generic, b"DEBUG=no\nAUTO_MODULES=no\nMICROCODE=generic\nINCLUDE_MODULES=\n"),
    (Microcode::None, b"DEBUG=no\nAUTO_MODULES=no\nMICROCODE=no\nINCLUDE_MODULES=\n"),
];
#[test]
fn the_initramfs_file_uses_the_profiles_microcode() {
    for (row, &(microcode, text)) in MICROCODE.iter().enumerate() {
        assert_eq!(additions::initramfs(&inputs::initramfs(microcode, &[])).as_ref(), text, "row {row}");
    }
}
#[test]
fn modules_are_sorted_and_comma_joined() {
    assert_eq!(
        additions::initramfs(&inputs::initramfs(Microcode::Generic, &[b"nvme", b"btrfs"])).as_ref(),
        b"DEBUG=no\nAUTO_MODULES=no\nMICROCODE=generic\nINCLUDE_MODULES=btrfs,nvme\n"
    );
}
const FILES: &[Option<hearth_lib::encode::Digest>] = &[None, Some(hearth_lib::encode::Digest([7; 32]))];
#[test]
fn the_initramfs_file_digest_is_beside_the_packages_part_not_in_its_digest() {
    for (row, &file) in FILES.iter().enumerate() {
        assert_eq!(
            crate::parts::packages::digest(&inputs::packages_with_file(file)),
            fixtures::THIN_PACKAGES_DIGEST,
            "row {row}"
        );
    }
}
#[test]
fn a_container_has_no_initramfs_file() {
    let packages = fixtures::packages::declarations_packages(
        &fixtures::packages::module_expansion(),
        crate::revision::Runs::Container,
    )
    .expect("container");
    assert_eq!(packages.initramfs, None);
}
