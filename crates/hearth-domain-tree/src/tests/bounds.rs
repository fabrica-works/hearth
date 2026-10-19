use super::fake::{Node, dir, file, link, root, run};
use crate::{
    limits::{Limits, SHIPPED, worst_case},
    vocabulary::{Bound, Cause},
};

fn bound(nodes: &[Node], limits: Limits, at: &[u8], which: Bound) {
    let run = run(nodes, limits);
    let (path, cause) = run.failed();
    assert_eq!((path, cause), (at, Cause::Bound(which)));
}

#[test]
fn more_entries_than_the_bound_fail_at_the_first_past_it() {
    let nodes = [root(), file(b"a", 2, 0), file(b"b", 3, 0), file(b"c", 4, 0)];
    bound(&nodes, Limits { entries: 3, ..SHIPPED }, b"c", Bound::Entries);
    assert_eq!(run(&nodes, Limits { entries: 4, ..SHIPPED }).entries.len(), 4);
}

#[test]
fn more_names_in_a_directory_than_the_bound_fail_at_the_directory() {
    let nodes = [root(), dir(b"d", 2), file(b"d/a", 3, 0), file(b"d/b", 4, 0), file(b"d/c", 5, 0)];
    bound(&nodes, Limits { names: 2, ..SHIPPED }, b"d", Bound::Names);
    assert_eq!(run(&nodes, Limits { names: 3, ..SHIPPED }).entries.len(), 5);
}

#[test]
fn directories_nested_past_the_depth_fail_at_the_first_past_it() {
    let nodes = [root(), dir(b"a", 2), dir(b"a/b", 3), dir(b"a/b/c", 4)];
    bound(&nodes, Limits { depth: 2, ..SHIPPED }, b"a/b/c", Bound::Depth);
    assert_eq!(run(&nodes, Limits { depth: 3, ..SHIPPED }).entries.len(), 4);
    bound(&[root(), dir(b"a", 2)], Limits { depth: 0, ..SHIPPED }, b"a", Bound::Depth);
}

#[test]
fn more_names_held_across_the_stack_than_the_bound_fail_at_the_directory_being_listed() {
    let nodes = [root(), dir(b"a", 2), file(b"a/x", 3, 0), file(b"a/y", 4, 0), file(b"a/z", 5, 0), file(b"b", 6, 0)];
    // Root holds a and b; a is taken, so b is held while a lists three.
    bound(&nodes, Limits { held: 3, ..SHIPPED }, b"a", Bound::Held);
    assert_eq!(run(&nodes, Limits { held: 4, ..SHIPPED }).entries.len(), 6);
}

#[test]
fn a_name_past_its_bound_fails_at_its_directory() {
    bound(&[root(), file(b"abcd", 2, 0)], Limits { name: 3, ..SHIPPED }, b"", Bound::Name);
}

#[test]
fn a_path_past_its_bound_fails_naming_it() {
    let nodes = [root(), dir(b"aa", 2), file(b"aa/b", 3, 0)];
    bound(&nodes, Limits { path: 3, ..SHIPPED }, b"aa/b", Bound::Path);
    assert_eq!(run(&nodes, Limits { path: 4, ..SHIPPED }).entries.len(), 3);
}

#[test]
fn a_link_target_past_its_bound_fails_whether_its_stat_says_so_or_not() {
    bound(&[root(), link(b"l", 2, b"abcd")], Limits { target: 3, ..SHIPPED }, b"l", Bound::Target);
    // A stat that says 2 bytes of a target that reads as 4.
    let mut short = link(b"l", 2, b"abcd");
    short.stat.size = 2;
    bound(&[root(), short], Limits { target: 3, ..SHIPPED }, b"l", Bound::Target);
    assert_eq!(run(&[root(), link(b"l", 2, b"abc")], Limits { target: 3, ..SHIPPED }).entries.len(), 2);
}

#[test]
fn attributes_past_their_bounds_fail_at_the_entry() {
    let three: &[(&[u8], &[u8])] = &[(b"user.a", b""), (b"user.b", b""), (b"user.c", b"")];
    bound(&[root(), file(b"f", 2, 0).with(three)], Limits { attributes: 2, ..SHIPPED }, b"f", Bound::Attributes);
    assert_eq!(run(&[root(), file(b"f", 2, 0).with(three)], Limits { attributes: 3, ..SHIPPED }).entries.len(), 2);
    let long: &[(&[u8], &[u8])] = &[(b"user.long", b"")];
    bound(&[root(), file(b"f", 2, 0).with(long)], Limits { attribute_name: 8, ..SHIPPED }, b"f", Bound::AttributeName);
    let big: &[(&[u8], &[u8])] = &[(b"user.a", b"abc")];
    bound(&[root(), file(b"f", 2, 0).with(big)], Limits { attribute_value: 2, ..SHIPPED }, b"f", Bound::AttributeValue);
    // An attribute list longer than the bound, covered or not.
    let others: &[(&[u8], &[u8])] = &[(b"trusted.a", b""), (b"trusted.b", b""), (b"trusted.c", b"")];
    bound(&[root(), file(b"f", 2, 0).with(others)], Limits { listed: 2, ..SHIPPED }, b"f", Bound::Attributes);
}

#[test]
fn linked_inodes_past_their_bounds_fail_at_the_first_path_past_them() {
    let nodes = [root(), file(b"a", 2, 0).linked(2), file(b"b", 3, 0).linked(2), file(b"c", 2, 0).linked(2)];
    bound(&nodes, Limits { linked: 1, ..SHIPPED }, b"b", Bound::Linked);
    assert_eq!(run(&nodes, Limits { linked: 2, ..SHIPPED }).entries.len(), 4);
    let long = [root(), file(b"a", 2, 0).linked(2), file(b"bbb", 3, 0).linked(2)];
    bound(&long, Limits { linked_paths: 3, ..SHIPPED }, b"bbb", Bound::LinkedPaths);
    assert_eq!(run(&long, Limits { linked_paths: 4, ..SHIPPED }).entries.len(), 3);
}

#[test]
fn a_page_past_the_one_asked_for_is_not_an_answer() {
    // The fake answers pages of the size asked; a limit changed under it is the only way past.
    let nodes = [root(), file(b"a", 2, 0), file(b"b", 3, 0)];
    let run = run(&nodes, Limits { page: 1, ..SHIPPED });
    assert_eq!(run.entries.len(), 3);
}

#[test]
fn the_worst_case_is_a_sum_that_fits_or_nothing() {
    let shipped = worst_case(&SHIPPED).expect("the shipped limits fit a u64");
    assert!(shipped > 1_048_576, "{shipped}");
    assert_eq!(worst_case(&Limits { held: u32::MAX, name: u32::MAX, ..SHIPPED }), None);
    assert_eq!(worst_case(&Limits { depth: u32::MAX, ..SHIPPED }), None);
}
