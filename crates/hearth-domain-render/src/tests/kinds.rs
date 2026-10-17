use super::fixtures::{
    self,
    kinds::{self as fixture, directory, file, plan, text, tree},
};
use crate::{
    errors::{Format, Kind, Repository, Schema, TypeName},
    kinds::files::{Make, Source, TreeFile},
    revision::{Oid, declaration::Name},
};
#[test]
fn a_directory_entry_gives_its_files_the_owner_and_group_they_lack() {
    let tree = tree(&[b"/etc/app/conf", b"/etc/app/secret"]);
    let plan = plan(
        &tree,
        &[
            file(b"/etc/app/", [text(Name::Owner, b"app"), text(Name::Group, b"app"), text(Name::Mode, b"0750")]),
            file(b"/etc/app/secret", [text(Name::Owner, b"root")]),
        ],
        &[],
    )
    .expect("valid plan");
    assert_eq!(
        fixture::owners(&plan).as_ref(),
        [
            (b"/etc/app".as_slice(), b"app".as_slice(), b"app".as_slice()),
            (b"/etc/app/conf", b"app", b"app"),
            (b"/etc/app/secret", b"root", b"app")
        ]
    );
}
#[test]
fn a_directory_entrys_mode_is_the_directorys_only() {
    let plan =
        plan(&tree(&[b"/etc/app/conf"]), &[file(b"/etc/app/", [text(Name::Mode, b"0750")])], &[]).expect("valid plan");
    let mut entries = skein_lib::List::with_capacity(u32::try_from(plan.installs.len()).expect("plan count"));
    for install in &plan.installs {
        entries.push(&install.make).expect("plan count");
    }
    assert_eq!(
        entries.as_slice(),
        [&Make::Directory { mode: 0o750 }, &Make::Static { sha256: hearth_lib::encode::Digest([1; 32]), mode: 0o644 }]
    );
}
#[test]
fn an_entry_for_a_path_the_tree_lacks_is_a_repository_error() {
    let faults = plan(&[], &[file(b"/missing", [])], &[]).expect_err("missing tree");
    assert_eq!(fixtures::only(&faults).kind, Kind::Repository(Repository::NoTree));
}
#[test]
fn a_link_entry_over_a_tree_path_is_twice() {
    let faults = plan(&tree(&[b"/etc/app"]), &[file(b"/etc/app", [text(Name::Link, b"elsewhere")])], &[])
        .expect_err("two sources");
    assert_eq!(fixtures::only(&faults).kind, Kind::Repository(Repository::Destination));
}
#[test]
fn two_sources_on_one_entry_err_at_the_second() {
    let faults = plan(&[], &[file(b"/etc/app", [text(Name::Link, b"elsewhere"), directory(true)])], &[])
        .expect_err("two sources");
    assert_eq!(
        fixtures::only(&faults),
        &crate::entries::Fault {
            kind: Kind::Schema(Schema::Source { first: Name::Link, second: Name::Directory }),
            at: fixture::FIELD_AT,
            also: None
        }
    );
}
#[test]
fn an_entry_its_if_dropped_leaves_its_tree_path_out() {
    let plan = plan(&tree(&[b"/etc/app"]), &[], &[b"/etc/app"]).expect("valid plan");
    assert_eq!(plan.installs.as_ref(), []);
}
#[test]
fn the_nearest_directory_entry_governs_owner_and_group() {
    let plan = plan(
        &tree(&[b"/etc/app/conf"]),
        &[
            file(b"/etc/", [text(Name::Owner, b"outer"), text(Name::Group, b"outer")]),
            file(b"/etc/app/", [text(Name::Owner, b"inner")]),
        ],
        &[],
    )
    .expect("valid plan");
    assert_eq!(
        fixture::owners(&plan).as_ref(),
        [
            (b"/etc".as_slice(), b"outer".as_slice(), b"outer".as_slice()),
            (b"/etc/app", b"inner", b"outer"),
            (b"/etc/app/conf", b"inner", b"outer")
        ]
    );
}
#[test]
fn a_dropped_directory_entry_drops_a_kept_entry_beneath_it() {
    let plan =
        plan(&tree(&[b"/etc/app/conf"]), &[file(b"/etc/app/conf", [text(Name::Owner, b"app")])], &[b"/etc/app/"])
            .expect("valid plan");
    assert_eq!(plan.installs.as_ref(), []);
}
const SOURCES: &[(Name, &[u8])] = &[(Name::Template, b"template"), (Name::Copy, b"copy"), (Name::Link, b"link")];
#[test]
fn a_directory_key_with_a_source_field_errs_undeclared() {
    for (row, &(name, expected)) in SOURCES.iter().enumerate() {
        let faults = plan(&tree(&[b"/etc/app/conf"]), &[file(b"/etc/app/", [text(name, b"source")])], &[])
            .expect_err("directory source");
        assert_eq!(
            fixtures::only(&faults).kind,
            Kind::Schema(Schema::UndeclaredField { field: expected.into(), declared: Box::new([]) }),
            "row {row}"
        );
    }
}
#[test]
fn a_directory_key_rejects_the_directory_source_field() {
    let faults =
        plan(&tree(&[b"/etc/app/conf"]), &[file(b"/etc/app/", [directory(true)])], &[]).expect_err("directory source");
    assert_eq!(
        fixtures::only(&faults).kind,
        Kind::Schema(Schema::UndeclaredField { field: b"directory".as_slice().into(), declared: Box::new([]) })
    );
}
const NON_DIRECTORIES: &[&[u8]] = &[b"/etc/app", b"/etc/app/conf/"];
#[test]
fn a_key_ends_in_a_slash_exactly_when_it_names_a_tree_directory() {
    for (row, path) in NON_DIRECTORIES.iter().enumerate() {
        let faults = plan(&tree(&[b"/etc/app/conf"]), &[file(path, [])], &[]).expect_err("wrong path kind");
        assert_eq!(fixtures::only(&faults).kind, Kind::Repository(Repository::NoTree), "row {row}");
    }
}
#[test]
fn directory_false_is_wrong() {
    let faults = plan(&[], &[file(b"/etc/app", [directory(false)])], &[]).expect_err("false directory");
    assert_eq!(fixtures::only(&faults).kind, Kind::Schema(Schema::Wrong { expected: TypeName::Bool }));
}
const BAD_LINKS: &[&[u8]] = &[b"", b"a\0b", &[b'x'; fixture::LINK_MAX + 1]];
#[test]
fn a_link_target_of_no_bytes_a_nul_or_4096_bytes_is_refused() {
    for (row, target) in BAD_LINKS.iter().enumerate() {
        let faults = plan(&[], &[file(b"/link", [text(Name::Link, target)])], &[]).expect_err("invalid link");
        assert_eq!(fixtures::only(&faults).kind, Kind::Schema(Schema::Malformed { format: Format::Link }), "row {row}");
    }
}
#[test]
fn a_link_target_at_the_limit_is_accepted() {
    assert!(crate::kinds::files::link(&[b'x'; fixture::LINK_MAX]));
}
#[test]
fn tree_etc_x_j2_j2_renders_to_etc_x_j2() {
    let tree = [TreeFile {
        path: b"/etc/x.j2".as_slice().into(),
        source: Source::Template {
            oid: Oid(b"oid".as_slice().into()),
            path: b"tasks/motd/tree/etc/x.j2.j2".as_slice().into(),
        },
        mode: 0o644,
    }];
    let plan = plan(&tree, &[], &[]).expect("template plan");
    assert_eq!(fixtures::only(&plan.installs).path.as_ref(), b"/etc/x.j2");
}

mod sources;

mod accounts;

mod directives;
