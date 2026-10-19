use super::fake::{Node, PARENT, Stream, assert_changes, dir, file, link, merge, merge_with, root, run, special};
use crate::{
    limits::SHIPPED,
    vocabulary::{Answer, Cause, Counts, End, Fault, Item, Kernel, Kind, Local, Refusal, Request},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, bytes};

const EMPTY: Counts = Counts { bytes: 0, deletes: 0, entries: 0, files: 0, hashed: 0, puts: 0 };
/// A change time later than the fake's default of (10, 1).
const LATER: (u64, u32) = (50, 5);

/// The stream a walk of `nodes` leaves for a child.
fn parent_of(nodes: &[Node]) -> Stream {
    Stream::after(&run(nodes, SHIPPED))
}
fn counts(run: &super::fake::Run) -> Counts {
    match &run.answer {
        Answer::Walked { counts, .. } => *counts,
        Answer::Failed { .. } => panic!("the walk failed: {:?}", run.answer),
    }
}
fn asked(run: &super::fake::Run, kind: fn(&Request) -> bool) -> u32 {
    run.count(kind)
}
fn is_hash(request: &Request) -> bool {
    match request {
        Request::HashFile { .. } => true,
        Request::Stat { .. }
        | Request::List { .. }
        | Request::Next { .. }
        | Request::Attributes { .. }
        | Request::Attribute { .. }
        | Request::ReadLink { .. }
        | Request::Create
        | Request::Write { .. }
        | Request::Digests { .. }
        | Request::ParentNext
        | Request::Finish { .. } => false,
    }
}
fn is_attributes(request: &Request) -> bool {
    match request {
        Request::Attributes { .. } => true,
        Request::Stat { .. }
        | Request::List { .. }
        | Request::Next { .. }
        | Request::Attribute { .. }
        | Request::ReadLink { .. }
        | Request::HashFile { .. }
        | Request::Create
        | Request::Write { .. }
        | Request::Digests { .. }
        | Request::ParentNext
        | Request::Finish { .. } => false,
    }
}
fn is_finish(request: &Request) -> bool {
    match request {
        Request::Finish { .. } => true,
        Request::Stat { .. }
        | Request::List { .. }
        | Request::Next { .. }
        | Request::Attributes { .. }
        | Request::Attribute { .. }
        | Request::ReadLink { .. }
        | Request::HashFile { .. }
        | Request::Create
        | Request::Write { .. }
        | Request::Digests { .. }
        | Request::ParentNext => false,
    }
}

const TREE: &[Node] = &[
    root(),
    dir(b"a", 2),
    file(b"a/b", 3, 4),
    file(b"a-b", 4, 6),
    dir(b"b", 5),
    file(b"b/a", 6, 1),
    special(b"c", 8, Kind::Fifo),
    link(b"d", 9, b"a/b"),
];

#[test]
fn a_walk_with_nothing_changed_keeps_every_entry_unread() {
    let stream = parent_of(TREE);
    let run = merge(TREE, &stream, SHIPPED);
    let all: &[(u8, &[u8])] = &[
        (b'=', b""),
        (b'=', b"a"),
        (b'=', b"a/b"),
        (b'=', b"a-b"),
        (b'=', b"b"),
        (b'=', b"b/a"),
        (b'=', b"c"),
        (b'=', b"d"),
    ];
    assert_changes(&run, all);
    assert_eq!(asked(&run, is_hash), 0, "no file is read");
    assert_eq!(asked(&run, is_attributes), 0, "no attribute is listed");
    assert_eq!(counts(&run), Counts { bytes: 11, entries: 8, files: 3, ..EMPTY });
    let digests = Request::Digests { parent_tree: Some(PARENT.tree), entries: 8, changes: 0 };
    assert!(run.requests.as_slice().contains(&digests), "the digests name the parent's tree and no change");
    // The entries are the parent's, byte for byte.
    for (entry, _) in &stream.entries {
        assert_eq!(run.entry(&entry.path), entry);
    }
}

#[test]
fn an_unchanged_entry_has_the_new_stats_local() {
    let stream = parent_of(TREE);
    let moved: &[Node] = &[root(), file(b"f", 2, 3)];
    let again = merge(moved, &parent_of(moved), SHIPPED);
    assert_changes(&again, &[(b'=', b""), (b'=', b"f")]);
    let mut nodes = [root(), file(b"f", 2, 3)];
    nodes[1].stat.mtime = (99, 9);
    let run = merge(&nodes, &parent_of(moved), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'=', b"f")]);
    let mut locals = 0_u32;
    for item in &run.items {
        if let Item::Entry(entry, local) = item
            && &*entry.path == b"f"
        {
            assert_eq!(*local, Local { inode: 2, ctime: (10, 1), mtime: (99, 9) });
            locals = locals.saturating_add(1);
        }
    }
    assert_eq!(locals, 1_u32);
    assert_eq!(stream.entries.len(), 8);
}

#[test]
fn a_path_only_in_the_walk_is_a_put_and_one_only_in_the_parent_is_a_delete() {
    let before = [root(), dir(b"a", 2), file(b"b", 3, 1), file(b"d", 4, 1), file(b"e", 5, 1)];
    let after =
        [root(), file(b"a", 20, 1), file(b"b", 3, 1), file(b"c", 6, 2), file(b"e", 5, 1).changed(LATER).mode(0o600)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"a"), (b'=', b"b"), (b'+', b"c"), (b'-', b"d"), (b'+', b"e")]);
    assert_eq!(counts(&run), Counts { bytes: 5, deletes: 1, entries: 5, files: 4, hashed: 4, puts: 3 });
    assert_eq!(asked(&run, is_hash), 3, "a, c and e are read; b is not");
}

#[test]
fn a_deleted_directory_lists_every_path_beneath_it() {
    let before = [
        root(),
        file(b"a", 2, 0),
        dir(b"d", 3),
        dir(b"d/a", 4),
        file(b"d/a/b", 5, 1),
        file(b"d/c", 6, 1),
        file(b"z", 7, 0),
    ];
    let after = [root(), file(b"a", 2, 0), file(b"z", 7, 0)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    let expected: &[(u8, &[u8])] =
        &[(b'=', b""), (b'=', b"a"), (b'-', b"d"), (b'-', b"d/a"), (b'-', b"d/a/b"), (b'-', b"d/c"), (b'=', b"z")];
    assert_changes(&run, expected);
    assert_eq!(counts(&run).deletes, 4);
    assert_eq!(counts(&run).puts, 0);
    // The deletes are the walk's last, too: the tail of the parent after the walk's last path.
    let after = [root(), file(b"a", 2, 0)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(
        &run,
        &[(b'=', b""), (b'=', b"a"), (b'-', b"d"), (b'-', b"d/a"), (b'-', b"d/a/b"), (b'-', b"d/c"), (b'-', b"z")],
    );
}

#[test]
fn a_kind_change_at_a_path_is_a_put_and_what_was_beneath_it_is_deleted() {
    let before = [root(), dir(b"x", 2), file(b"x/y", 3, 1), file(b"x/z", 4, 1), file(b"y", 5, 0)];
    let after = [root(), file(b"x", 9, 2), file(b"y", 5, 0)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"x"), (b'-', b"x/y"), (b'-', b"x/z"), (b'=', b"y")]);
    // And the other way: a file becomes a directory with a file in it.
    let run = merge(&before, &parent_of(&after), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"x"), (b'+', b"x/y"), (b'+', b"x/z"), (b'=', b"y")]);
    // Even at the same inode and change time, a different kind is not the same entry.
    let before = [root(), file(b"x", 2, 0)];
    let after = [root(), dir(b"x", 2)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"x")]);
    assert_eq!(asked(&run, is_attributes), 1, "x is read: the root is kept");
}

#[test]
fn an_unchanged_directory_is_listed_and_a_changed_file_beneath_it_is_a_put() {
    let before = [root(), dir(b"d", 2), file(b"d/f", 3, 4), file(b"d/g", 4, 4), file(b"e", 5, 1)];
    let after =
        [root(), dir(b"d", 2), file(b"d/f", 3, 4).changed(LATER).mode(0o755), file(b"d/g", 4, 4), file(b"e", 5, 1)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'=', b"d"), (b'+', b"d/f"), (b'=', b"d/g"), (b'=', b"e")]);
    let mut listed = 0_u32;
    for request in &run.requests {
        if let Request::List { path, .. } = request
            && &**path == b"d"
        {
            listed = listed.saturating_add(1);
        }
    }
    assert_eq!(listed, 1_u32, "the unchanged directory is descended into");
    assert_eq!(asked(&run, is_attributes), 1, "only d/f is read");
    assert_eq!(counts(&run).hashed, 4);
}

#[test]
fn an_entry_read_again_and_equal_to_the_parents_is_no_change() {
    let before = [root(), file(b"f", 2, 5), dir(b"d", 3), link(b"l", 4, b"f")];
    // A touch: the change and modification times move, nothing else does.
    let after =
        [root(), file(b"f", 2, 5).changed(LATER), dir(b"d", 3).changed(LATER), link(b"l", 4, b"f").changed(LATER)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'=', b"d"), (b'=', b"f"), (b'=', b"l")]);
    assert_eq!(counts(&run), Counts { bytes: 5, entries: 4, files: 1, hashed: 5, ..EMPTY });
    assert_eq!(asked(&run, is_hash), 1, "the touched file was read");
    // The new local index is kept: the next walk finds these unchanged.
    let next = merge(&after, &Stream::after(&run), SHIPPED);
    assert_eq!(counts(&next).hashed, 0);
}

#[test]
fn a_reused_inode_with_a_new_change_time_and_a_new_inode_at_the_same_time_are_read() {
    let before = [root(), file(b"a", 2, 3), file(b"b", 3, 3)];
    let after = [root(), file(b"a", 2, 3).changed(LATER).holds(77), file(b"b", 4, 3).holds(78)];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"a"), (b'+', b"b")]);
    assert_eq!(counts(&run), Counts { bytes: 6, entries: 3, files: 2, hashed: 6, puts: 2, ..EMPTY });
    assert_eq!(run.entry(b"a").payload, crate::vocabulary::Payload::File { size: 3, content: Digest([77; 32]) });
}

#[test]
fn the_parents_entry_at_a_path_is_not_the_childs_when_the_childs_is_read_and_differs() {
    let before = [root(), file(b"f", 2, 1).with(&[(b"user.k", b"1")])];
    let after = [root(), file(b"f", 2, 1).changed(LATER).with(&[(b"user.k", b"2")])];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"f")]);
    let after = [root(), file(b"f", 2, 1).changed(LATER).with(&[(b"user.k", b"1")])];
    assert_changes(&merge(&after, &parent_of(&before), SHIPPED), &[(b'=', b""), (b'=', b"f")]);
}

#[test]
fn hard_links_unchanged_are_met_and_a_later_read_path_agrees_with_them() {
    let before = [root(), file(b"a", 3, 5).linked(2), file(b"b", 3, 5).linked(2)];
    let stream = parent_of(&before);
    let kept = merge(&before, &stream, SHIPPED);
    assert_changes(&kept, &[(b'=', b""), (b'=', b"a"), (b'=', b"b")]);
    assert_eq!(asked(&kept, is_hash), 0);
    assert_eq!(counts(&kept), Counts { bytes: 5, entries: 3, files: 1, ..EMPTY });
    // The later path is read, the first is kept: it is a hard link to the kept first path.
    let later = [root(), file(b"a", 3, 5).linked(2), file(b"b", 3, 5).linked(2).changed(LATER)];
    let run = merge(&later, &stream, SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'=', b"a"), (b'=', b"b")]);
    assert_eq!(asked(&run, is_hash), 0, "a later link is never hashed");
    assert_eq!(run.entry(b"b").payload, crate::vocabulary::Payload::Hardlink { path: bytes::copy_of(b"a") });
    // And it must agree with what the unchanged first path has.
    let odd = [root(), file(b"a", 3, 5).linked(2), file(b"b", 3, 5).linked(2).changed(LATER).mode(0o600)];
    let run = merge(&odd, &stream, SHIPPED);
    assert_eq!(run.failed(), (&b"b"[..], Cause::Refused(Refusal::Link)));
    // An unchanged later path is not checked, nor read: its change time is its inode's.
    let moved = [root(), file(b"a", 3, 5).linked(2).changed(LATER), file(b"b", 3, 5).linked(2)];
    let run = merge(&moved, &stream, SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'=', b"a"), (b'=', b"b")]);
    assert_eq!(asked(&run, is_hash), 1, "the first path is read, and it is the same file");
}

#[test]
fn a_link_added_to_an_unchanged_file_reads_both_paths() {
    let before = [root(), file(b"a", 3, 5), file(b"z", 4, 1)];
    let after = [
        root(),
        file(b"0", 3, 5).linked(2).changed(LATER),
        file(b"a", 3, 5).linked(2).changed(LATER),
        file(b"z", 4, 1),
    ];
    let run = merge(&after, &parent_of(&before), SHIPPED);
    assert_changes(&run, &[(b'=', b""), (b'+', b"0"), (b'+', b"a"), (b'=', b"z")]);
    assert_eq!(run.entry(b"a").payload, crate::vocabulary::Payload::Hardlink { path: bytes::copy_of(b"0") });
    assert_eq!(counts(&run), Counts { bytes: 6, entries: 4, files: 2, hashed: 5, puts: 2, ..EMPTY });
}

#[test]
fn the_end_item_names_the_parent_and_copies_its_kernels() {
    let kernels: Box<[Kernel]> = Box::new([Kernel {
        version: bytes::copy_of(b"6.12"),
        kernel: Digest([1; 32]),
        modules: Digest([2; 32]),
        initramfs: Digest([3; 32]),
    }]);
    let mut stream = parent_of(TREE);
    stream.boot = kernels.clone();
    let run = merge(TREE, &stream, SHIPPED);
    let mut ends = 0_u32;
    for request in &run.requests {
        if let Request::Finish { end: End { parent_tree, boot, .. } } = request {
            assert_eq!(*parent_tree, Some(PARENT.tree));
            assert_eq!(*boot, kernels);
            ends = ends.saturating_add(1);
        }
    }
    assert_eq!(ends, 1_u32);
    let alone = run_without_parent();
    assert!(alone.is_empty(), "a walk with no parent has no kernels");
}
fn run_without_parent() -> Box<[Kernel]> {
    let run = run(TREE, SHIPPED);
    for request in &run.requests {
        if let Request::Finish { end } = request {
            assert_eq!(end.parent_tree, None);
            return end.boot.clone();
        }
    }
    panic!("the walk finished")
}

/// The stream with its entries taken in the order of `order`, which may leave some out or take one twice.
fn picked(good: &Stream, order: &List<u32>) -> Stream {
    let mut entries = List::with_capacity(order.len());
    for at in order {
        let entry = good.entries.get(usize::try_from(*at).expect("small")).expect("an entry of the stream");
        assert!(entries.push(entry.clone()).is_ok(), "room");
    }
    Stream { entries: entries.into_boxed(), ..good.clone() }
}
fn in_order(count: u32) -> List<u32> {
    let mut order = List::with_capacity(count.saturating_add(1));
    for at in 0..count {
        assert!(order.push(at).is_ok(), "room");
    }
    order
}

#[test]
fn a_parent_stream_out_of_tree_order_fails_the_walk() {
    let good = parent_of(TREE);
    let n = u32::try_from(good.entries.len()).expect("small");
    assert!(n >= 4, "a stream to damage");
    // Swapped neighbours, a repeated entry, and the root last.
    for swap in 1..n - 1 {
        let mut order = in_order(n);
        *order.get_mut(swap).expect("in range") = swap + 1;
        *order.get_mut(swap + 1).expect("in range") = swap;
        let run = merge(TREE, &picked(&good, &order), SHIPPED);
        assert_eq!(run.failed().1, Cause::Parent, "entries {swap} and {} swapped", swap + 1);
    }
    for repeat in 0..n {
        let mut order = List::with_capacity(n + 1);
        for at in 0..n {
            assert!(order.push(at).is_ok(), "room");
            if at == repeat {
                assert!(order.push(at).is_ok(), "room");
            }
        }
        let run = merge(TREE, &picked(&good, &order), SHIPPED);
        assert_eq!(run.failed().1, Cause::Parent, "entry {repeat} twice");
    }
    let mut order = List::with_capacity(n);
    for at in 1..n {
        assert!(order.push(at).is_ok(), "room");
    }
    assert!(order.push(0).is_ok(), "room");
    let run = merge(TREE, &picked(&good, &order), SHIPPED);
    assert_eq!(run.failed().1, Cause::Parent, "the root last");
}

#[test]
fn a_parent_stream_that_is_not_the_records_fails_the_walk_whatever_the_walk_found() {
    let good = parent_of(TREE);
    let wrong = [
        Stream { tree: Digest([0; 32]), ..good.clone() },
        Stream { sha256: Digest([0; 32]), ..good.clone() },
        Stream { size: good.size + 1, ..good.clone() },
        Stream { size: 0, ..good.clone() },
    ];
    for stream in &wrong {
        let run = merge(TREE, stream, SHIPPED);
        assert_eq!(run.failed(), (&b""[..], Cause::Parent));
        assert_eq!(run.count(is_finish), 0, "nothing is finished");
    }
    // Also when the walk reached its end long before the stream did.
    let run = merge(&[root()], &Stream { size: 1, ..good }, SHIPPED);
    assert_eq!(run.failed(), (&b""[..], Cause::Parent));
}

#[test]
fn an_event_of_the_parent_nothing_waits_for_fails_the_walk() {
    use crate::{Tree, step};
    use skein_lib::{Env, Queue, Time, Wall};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: SHIPPED };
    let mut tree = Tree::new(&SHIPPED);
    let end = crate::vocabulary::Event::ParentEnd {
        tree: PARENT.tree,
        boot: Box::default(),
        sha256: PARENT.tree_file.sha256,
        size: PARENT.tree_file.size,
    };
    let mut out = Queue::with_capacity(crate::MAX_OUT);
    step(&mut tree, &env, end, &mut out);
    let crate::vocabulary::Output::Answer(Answer::Failed { cause, .. }) = out.pop().expect("an output") else {
        panic!("a failure")
    };
    assert_eq!(cause, Cause::Unexpected);
}

#[test]
fn an_io_failure_at_any_request_of_a_walk_against_a_parent_ends_it_at_that_requests_path() {
    let before =
        [root(), dir(b"d", 2), file(b"d/f", 3, 4), file(b"d/g", 4, 1), file(b"x", 5, 1), file(b"y", 6, 0).linked(2)];
    let after = [
        root(),
        dir(b"d", 2),
        file(b"d/f", 3, 4).changed(LATER).mode(0o600),
        file(b"w", 7, 2),
        file(b"y", 6, 0).linked(2),
    ];
    let stream = parent_of(&before);
    let clean = merge(&after, &stream, SHIPPED);
    assert_changes(
        &clean,
        &[(b'=', b""), (b'=', b"d"), (b'+', b"d/f"), (b'-', b"d/g"), (b'+', b"w"), (b'-', b"x"), (b'=', b"y")],
    );
    let count = clean.requests.len();
    assert!(count > 15, "{count} requests");
    for fault in [Fault::NotFound, Fault::Permission, Fault::NoSpace, Fault::Bound, Fault::Other] {
        for at in 0..count {
            let request = clean.requests.get(at).expect("in range");
            let broken = merge_with(&after, &stream, SHIPPED, Some(at), fault);
            let (path, _) = broken.failed();
            assert_eq!(broken.requests.len(), at + 1, "nothing is asked after the failure, at {at}: {request:?}");
            if let Request::ParentNext = request {
                // Asked while merging to the next path the walk visits, or at the end.
                let mut wanted: &[u8] = b"";
                for later in clean.requests.iter().skip(usize::try_from(at + 1).expect("small")) {
                    if let Request::Stat { path } = later {
                        wanted = path;
                        break;
                    }
                }
                assert_eq!(path, wanted, "at {at}");
            }
        }
    }
}
