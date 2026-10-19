use super::fake::{
    CHANGES, FILE, FILE_SIZE, KEY, Node, TREE, assert_paths, dir, drive, file, link, root, run, special,
};
use crate::{
    Tree,
    limits::{Limits, SHIPPED},
    vocabulary::{
        Answer, Bound, Cause, Counts, End, Entry, Event, Fault, Item, Kind, Local, Output, Payload, Request, TreeFile,
        Xattr,
    },
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Env, Queue, Time, Wall, bytes};

fn path(path: &[u8]) -> Box<[u8]> {
    bytes::copy_of(path)
}
fn entry(path: &[u8], mode: u32, payload: Payload) -> Entry {
    Entry { path: bytes::copy_of(path), mode, uid: 0, gid: 0, xattrs: Box::default(), payload }
}
fn local(inode: u64) -> Local {
    Local { inode, ctime: (10, 1), mtime: (20, 2) }
}
fn items(entry: Entry, inode: u64) -> [Item; 2] {
    let put = Item::Put { path: bytes::copy_of(&entry.path) };
    [Item::Entry(entry, local(inode)), put]
}
const EMPTY: Counts = Counts { bytes: 0, deletes: 0, entries: 0, files: 0, hashed: 0, puts: 0 };

#[test]
fn a_root_alone_is_one_entry_and_its_put() {
    let run = run(&[root()], SHIPPED);
    let counts = Counts { entries: 1, puts: 1, ..EMPTY };
    let [first, second] = items(entry(b"", 0o755, Payload::Directory), 1);
    let expected = [
        Request::Create,
        Request::Stat { path: path(b"") },
        Request::Attributes { path: path(b"") },
        Request::List { path: path(b""), max: SHIPPED.page },
        Request::Write { items: Box::new([first, second]) },
        Request::Digests { parent_tree: None, entries: 1, changes: 1 },
        Request::Finish { end: End { key: KEY, parent_tree: None, tree: TREE, changes: CHANGES, counts } },
    ];
    assert_eq!(run.requests.as_slice(), expected);
    assert_eq!(
        run.answer,
        Answer::Hashed { tree: TREE, changes: CHANGES, tree_file: TreeFile { sha256: FILE, size: FILE_SIZE }, counts }
    );
}

const ORDER: &[Node] = &[
    root(),
    dir(b"a", 2),
    file(b"a/b", 3, 4),
    file(b"a-b", 4, 0),
    dir(b"b", 5),
    file(b"b/a", 6, 1),
    file(b"b/B", 7, 2),
    special(b"c", 8, Kind::Fifo),
    link(b"d", 9, b"a/b"),
    special(b"e", 10, Kind::Char),
    special(b"f", 11, Kind::Block),
    special(b"g", 12, Kind::Socket),
];

#[test]
fn entries_come_in_tree_order_each_followed_by_its_put() {
    let run = run(ORDER, SHIPPED);
    let expected: &[&[u8]] = &[b"", b"a", b"a/b", b"a-b", b"b", b"b/B", b"b/a", b"c", b"d", b"e", b"f", b"g"];
    assert_paths(&run.paths(), expected);
    assert_paths(&run.puts, expected);
    let Answer::Hashed { counts, .. } = run.answer else { panic!("hashed: {:?}", run.answer) };
    assert_eq!(counts, Counts { bytes: 7, deletes: 0, entries: 12, files: 4, hashed: 7, puts: 12 });
}

#[test]
fn every_kind_has_its_payload() {
    let run = run(ORDER, SHIPPED);
    let payloads = [
        (&b""[..], Payload::Directory),
        (b"a/b", Payload::File { size: 4, content: Digest([3; 32]) }),
        (b"c", Payload::Fifo),
        (b"d", Payload::Symlink { target: path(b"a/b") }),
        (b"e", Payload::Char { major: 4, minor: 2 }),
        (b"f", Payload::Block { major: 4, minor: 2 }),
        (b"g", Payload::Socket),
    ];
    for (wanted, payload) in payloads {
        let found = run.entry(wanted);
        assert_eq!(found.payload, payload, "{wanted:?}");
    }
}

#[test]
fn a_directory_is_listed_a_page_at_a_time_and_its_names_are_sorted() {
    const PAGED: &[Node] =
        &[root(), file(b"e", 2, 0), file(b"b", 3, 0), file(b"d", 4, 0), file(b"a", 5, 0), file(b"c", 6, 0)];
    let limits = Limits { page: 2, ..SHIPPED };
    let run = run(PAGED, limits);
    assert_paths(&run.paths(), &[b"", b"a", b"b", b"c", b"d", b"e"]);
    let mut nexts = 0_u32;
    for request in &run.requests {
        if let Request::Next { max } = request {
            assert_eq!(*max, 2);
            nexts = nexts.saturating_add(1);
        }
    }
    assert_eq!(nexts, 2_u32, "five names in pages of two: a list and two nexts");
}

#[test]
fn a_directory_is_written_before_anything_beneath_it_is_read() {
    let run = run(ORDER, SHIPPED);
    let mut written = 0_usize;
    for request in &run.requests {
        match request {
            Request::Write { items } => written = written.saturating_add(items.len()),
            Request::List { path, .. } => {
                // The entries written by now include the directory about to be listed.
                assert!(!run.entries.is_empty(), "{path:?}");
            }
            Request::Stat { .. }
            | Request::Next { .. }
            | Request::Attributes { .. }
            | Request::Attribute { .. }
            | Request::ReadLink { .. }
            | Request::HashFile { .. }
            | Request::Create
            | Request::Digests { .. }
            | Request::Finish { .. } => {}
        }
    }
    assert_eq!(written, 24_usize);
}

#[test]
fn only_the_covered_attributes_are_read_and_they_are_sorted() {
    const ATTRS: &[(&[u8], &[u8])] = &[
        (b"user.z", b"1"),
        (b"trusted.x", b"2"),
        (b"security.capability", b"3"),
        (b"security.selinux", b"4"),
        (b"system.posix_acl_default", b"5"),
        (b"user.a", b""),
        (b"system.posix_acl_access", b"6"),
        (b"system.other", b"7"),
    ];
    let nodes = [root(), file(b"f", 2, 0).with(ATTRS)];
    let run = run(&nodes, SHIPPED);
    let found = run.entry(b"f");
    let names: [&[u8]; 5] =
        [b"security.capability", b"system.posix_acl_access", b"system.posix_acl_default", b"user.a", b"user.z"];
    assert_eq!(found.xattrs.len(), names.len());
    for (xattr, name) in found.xattrs.iter().zip(names) {
        assert_eq!(&*xattr.name, name);
    }
    let Xattr { value, .. } = &found.xattrs[4];
    assert_eq!(&**value, b"1");
    let mut read = 0_u32;
    for request in &run.requests {
        if let Request::Attribute { .. } = request {
            read = read.saturating_add(1);
        }
    }
    assert_eq!(read, 5_u32, "the others are not asked for");
}

#[test]
fn the_mode_is_the_permission_bits() {
    let nodes = [root(), file(b"f", 2, 0).mode(0o106_755)];
    let run = run(&nodes, SHIPPED);
    let found = run.entry(b"f");
    assert_eq!(found.mode, 0o6755);
}

#[test]
fn a_big_batch_is_written_in_pieces_within_the_bound() {
    let limits = Limits { batch: 1_000, ..SHIPPED };
    let run = run(ORDER, limits);
    assert_paths(&run.paths(), &[b"", b"a", b"a/b", b"a-b", b"b", b"b/B", b"b/a", b"c", b"d", b"e", b"f", b"g"]);
    assert!(run.writes.len() > 1, "{:?}", run.writes);
    let mut total = 0_u32;
    for count in &run.writes {
        assert_eq!(count.checked_rem(2), Some(0), "an entry and its put go together");
        total = total.saturating_add(*count);
    }
    assert_eq!(total, 24_u32);
}

/// Steps the walk with `event` and takes its one output.
fn feed(tree: &mut Tree, event: Event) -> Output {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: SHIPPED };
    let mut out = Queue::with_capacity(crate::MAX_OUT);
    crate::step(tree, &env, event, &mut out);
    out.pop().expect("one output a step")
}
fn failed(path: &[u8], cause: Cause) -> Output {
    Output::Answer(Answer::Failed { path: bytes::copy_of(path), cause })
}

#[test]
fn a_second_hash_is_busy_and_the_walk_goes_on() {
    let mut tree = Tree::new(&SHIPPED);
    assert_eq!(feed(&mut tree, Event::Hash { key: KEY }), Output::Request(Request::Create));
    assert_eq!(feed(&mut tree, Event::Hash { key: KEY }), failed(b"", Cause::Busy));
    assert_eq!(feed(&mut tree, Event::Done), Output::Request(Request::Stat { path: path(b"") }));
}

#[test]
fn an_answer_nothing_waits_for_fails_the_walk_at_its_path() {
    let mut tree = Tree::new(&SHIPPED);
    assert_eq!(feed(&mut tree, Event::Done), failed(b"", Cause::Unexpected));
    assert_eq!(feed(&mut tree, Event::Hash { key: KEY }), Output::Request(Request::Create));
    assert_eq!(feed(&mut tree, Event::Done), Output::Request(Request::Stat { path: path(b"") }));
    assert_eq!(feed(&mut tree, Event::Value { bytes: path(b"x") }), failed(b"", Cause::Unexpected));
    assert_eq!(feed(&mut tree, Event::Done), failed(b"", Cause::Unexpected), "and the walk is over");
}

#[test]
fn a_walk_after_a_failed_walk_starts_clean() {
    let mut tree = Tree::new(&SHIPPED);
    for fault in [Fault::Permission, Fault::NoSpace] {
        assert_eq!(feed(&mut tree, Event::Hash { key: KEY }), Output::Request(Request::Create));
        assert_eq!(feed(&mut tree, Event::Done), Output::Request(Request::Stat { path: path(b"") }));
        let cause = if fault == Fault::Permission { Cause::Permission } else { Cause::NoSpace };
        assert_eq!(feed(&mut tree, Event::Failed { fault }), failed(b"", cause));
    }
}

/// The path a request names: the failure of its answer is reported there.
fn named(request: &Request, listed: &[u8]) -> Box<[u8]> {
    match request {
        Request::Stat { path }
        | Request::List { path, .. }
        | Request::Attributes { path }
        | Request::Attribute { path, .. }
        | Request::ReadLink { path }
        | Request::HashFile { path, .. } => bytes::copy_of(path),
        Request::Next { .. } => bytes::copy_of(listed),
        Request::Write { items } => match items.first() {
            Some(Item::Entry(entry, _)) => bytes::copy_of(&entry.path),
            Some(Item::Put { path }) => bytes::copy_of(path),
            None => panic!("a write holds items"),
        },
        Request::Create | Request::Digests { .. } | Request::Finish { .. } => path(b""),
    }
}

const ALL: &[Node] = &[
    root(),
    dir(b"d", 2),
    file(b"d/f", 3, 4).with(&[(b"user.k", b"v")]),
    link(b"d/l", 4, b"f"),
    special(b"p", 5, Kind::Fifo),
    file(b"x", 6, 1).linked(2),
    file(b"y", 6, 1).linked(2),
];

#[test]
fn an_io_failure_at_any_request_ends_the_walk_naming_that_requests_path() {
    let clean = run(ALL, SHIPPED);
    let count = clean.requests.len();
    assert!(count > 20, "{count} requests");
    for fault in [Fault::NotFound, Fault::Permission, Fault::NoSpace, Fault::Bound, Fault::Other] {
        let cause = match fault {
            Fault::NotFound => Cause::NotFound,
            Fault::Permission => Cause::Permission,
            Fault::NoSpace => Cause::NoSpace,
            Fault::Bound => Cause::Bound(Bound::Item),
            Fault::Other => Cause::Io,
        };
        let mut listed: Box<[u8]> = path(b"");
        for at in 0..count {
            let request = clean.requests.get(at).expect("in range");
            if let Request::List { path, .. } = request {
                listed = bytes::copy_of(path);
            }
            let broken = drive(ALL, SHIPPED, Some(at), fault);
            let (found, why) = broken.failed();
            assert_eq!(found, &*named(request, &listed), "request {at}: {request:?}");
            assert_eq!(why, cause, "request {at}: {request:?}");
            assert_eq!(broken.requests.len(), at + 1, "nothing is asked after the failure");
        }
    }
}
