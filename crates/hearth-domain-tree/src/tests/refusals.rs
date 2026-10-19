use super::fake::{Node, dir, file, link, root, run};
use crate::{
    limits::SHIPPED,
    vocabulary::{Cause, Refusal},
};

/// Each row: a layer, the path it fails at, and why.
fn refused(nodes: &[Node], at: &[u8], why: Refusal) {
    let run = run(nodes, SHIPPED);
    let (path, cause) = run.failed();
    assert_eq!((path, cause), (at, Cause::Refused(why)), "{nodes:?}", nodes = nodes.len());
}

#[test]
fn a_root_that_is_not_a_directory_is_refused() {
    refused(&[file(b"", 1, 0).mode(0o755)], b"", Refusal::RootKind);
    refused(&[link(b"", 1, b"x")], b"", Refusal::RootKind);
}

#[test]
fn a_root_that_is_not_0755_is_refused() {
    for mode in [0o700, 0o750, 0o775, 0o1755, 0o4755, 0o644, 0] {
        refused(&[root().mode(mode)], b"", Refusal::RootMode);
    }
}

#[test]
fn a_root_owned_by_anyone_but_zero_is_refused() {
    refused(&[root().owner(1, 0)], b"", Refusal::RootOwner);
    refused(&[root().owner(0, 1)], b"", Refusal::RootOwner);
    refused(&[root().owner(65_536, 65_536)], b"", Refusal::RootOwner);
}

#[test]
fn an_owner_past_65535_is_refused_at_its_path() {
    refused(&[root(), dir(b"d", 2), file(b"d/f", 3, 0).owner(65_536, 0)], b"d/f", Refusal::Owner);
    refused(&[root(), file(b"f", 2, 0).owner(0, 65_536)], b"f", Refusal::Owner);
    refused(&[root(), dir(b"d", 2).owner(70_000, 70_000)], b"d", Refusal::Owner);
    let run = run(&[root(), file(b"f", 2, 0).owner(65_535, 65_535)], SHIPPED);
    assert_eq!(run.entries.len(), 2, "65535 is the last owner");
}

#[test]
fn an_entry_on_another_device_is_refused_at_its_path() {
    refused(&[root(), dir(b"d", 2), dir(b"d/sub", 3).device(2)], b"d/sub", Refusal::Device);
    refused(&[root(), file(b"f", 2, 0).device(9)], b"f", Refusal::Device);
}

#[test]
fn a_file_whose_size_changed_between_its_stat_and_its_hash_is_refused() {
    refused(&[root(), file(b"f", 2, 10).reads(11)], b"f", Refusal::Size);
    refused(&[root(), file(b"f", 2, 10).reads(9)], b"f", Refusal::Size);
    refused(&[root(), file(b"f", 2, 0).reads(1)], b"f", Refusal::Size);
}

#[test]
fn a_name_that_is_not_a_component_is_refused_at_its_directory() {
    for name in [&b"."[..], b"..", b"a\0b"] {
        refused(&[root(), file(name, 2, 0)], b"", Refusal::Name);
    }
}

#[test]
fn a_name_listed_twice_is_refused() {
    refused(&[root(), file(b"a", 2, 0), file(b"a", 3, 0)], b"", Refusal::Name);
}

#[test]
fn an_attribute_listed_twice_is_refused_at_its_entry() {
    refused(&[root(), file(b"f", 2, 0).with(&[(b"user.a", b"1"), (b"user.a", b"2")])], b"f", Refusal::Attribute);
}
