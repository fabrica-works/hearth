use hearth_domain_tree::{
    limits::{Limits, SHIPPED},
    vocabulary::{Answer, Bound, Cause, Fault, Kind, Refusal, Request, TreeFile},
};
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{
    limits as protocol,
    tree::{self, Item},
};
use hearth_tree_world::{
    check, expected,
    fs::{Fs, Node},
    helper::{self, Helper, Walked, tree_counts},
    layers, oracle,
};
use skein_lib::{Rng, sha256};
use std::fmt::Write;

const KEY: Digest = Digest([0x42; 32]);

fn hex(digest: &Digest) -> String {
    let mut text = String::new();
    for byte in digest.0 {
        write!(text, "{byte:02x}").expect("a string takes text");
    }
    text
}
/// Walks `fs`, and checks that what came is what the oracle lists: the digests, the counts, the
/// file and its digest.
fn hashed(fs: &Fs, limits: &Limits, what: &str) -> Walked {
    let walked = helper::walk(fs, limits, KEY);
    let Answer::Walked { tree, changes, tree_file, counts } = &walked.answer else {
        let (path, cause) = walked.failed();
        panic!("{what}: the walk failed at a path of {} bytes: {cause:?}", path.len())
    };
    let items = expected::items(fs, KEY);
    let Some(Item::End(end)) = items.last() else { panic!("the end item is last") };
    assert_eq!(*tree, end.tree, "{what}: the tree digest");
    assert_eq!(*changes, end.changes, "{what}: the change set's digest");
    assert_eq!(tree_counts(counts), end.counts, "{what}: the counts");
    assert!(oracle::items_ok(&items), "{what}: the oracle lists a tree file");
    let read = check::read_tree(&walked.file, &protocol::SHIPPED).unwrap_or_else(|error| panic!("{what}: {error:?}"));
    assert_eq!(read.as_ref(), items.as_slice(), "{what}: the file reads back to what the oracle lists");
    let size = u64::try_from(walked.file.len()).expect("a size");
    assert_eq!(*tree_file, TreeFile { sha256: Digest(sha256::digest(&walked.file)), size }, "{what}: the file");
    let body: Vec<u8> = items.iter().flat_map(oracle::item_bytes).collect();
    let mut file = tree::header(u64::try_from(body.len()).expect("a size")).to_vec();
    file.extend_from_slice(&body);
    assert_eq!(walked.file, file, "{what}: the file's bytes");
    walked
}

#[test]
fn the_empty_layers_root_alone_is_the_tree_digest_of_the_design() {
    let walked = hashed(&Fs::new(), &SHIPPED, "the empty layer");
    let Answer::Walked { tree, counts, .. } = &walked.answer else { unreachable!("checked") };
    assert_eq!(hex(tree), "6022d1fbc6ad38d08d2ae02cefc21e7e6af5a35e6be9d115f0ba267fdb0d005a");
    assert_eq!((counts.entries, counts.puts, counts.deletes, counts.files), (1, 1, 0, 0));
}

#[test]
fn generated_layers_hash_to_the_oracles_digests_and_file() {
    for seed in 0..48 {
        let mut rng = Rng::new(0x7EE0_6000 + seed);
        let fs = layers::layer(&mut rng, 1 + seed * 3);
        hashed(&fs, &SHIPPED, &format!("seed {seed}"));
    }
}

#[test]
fn layers_listed_in_pages_and_written_in_batches_hash_the_same() {
    let mut rng = Rng::new(0x7EE0_6100);
    let fs = layers::layer(&mut rng, 150);
    let whole = hashed(&fs, &SHIPPED, "whole");
    for (page, batch) in [(1, 1), (2, 300), (7, 2_000), (1_024, 1)] {
        let limits = Limits { page, batch, ..SHIPPED };
        let paged = hashed(&fs, &limits, &format!("page {page}, batch {batch}"));
        assert_eq!(paged.file, whole.file, "page {page}, batch {batch}");
    }
}

#[test]
fn a_symlink_with_two_links_and_devices_and_attributes_of_every_family() {
    let mut fs = Fs::new();
    fs.add(b"d", Node::new(Kind::Directory, 2).attr(b"user.dir", b"x"));
    let link = fs.add(b"d/link", Node::symlink(3, b"../target").attr(b"security.capability", b"cap"));
    fs.link(b"another", link);
    fs.add(b"c", Node::device(Kind::Char, 4, 1, 3));
    fs.add(b"b", Node::device(Kind::Block, 5, 8, 0));
    fs.add(
        b"f",
        Node::file(6, b"contents")
            .moded(0o4755)
            .owned(65_535, 7)
            .attr(b"system.posix_acl_access", b"a")
            .attr(b"system.posix_acl_default", b"d")
            .attr(b"user.b", b"2")
            .attr(b"user.a", b"1")
            .attr(b"security.selinux", b"dropped")
            .attr(b"trusted.x", b"dropped"),
    );
    fs.add(b"fifo", Node::new(Kind::Fifo, 7));
    fs.add(b"sock", Node::new(Kind::Socket, 8));
    fs.shuffle(&mut Rng::new(5));
    let walked = hashed(&fs, &SHIPPED, "every kind");
    let items = check::read_tree(&walked.file, &protocol::SHIPPED).expect("a tree file");
    let hard = items
        .iter()
        .find_map(|item| match item {
            Item::Entry(entry, _) if &*entry.path == b"d/link" => Some(&entry.payload),
            Item::Entry(..) | Item::Change { .. } | Item::End(_) => None,
        })
        .expect("listed");
    assert_eq!(*hard, tree::Payload::Hardlink { path: b"another".as_slice().into() });
}

#[test]
fn wide_and_deep_layers_hash_to_the_oracles() {
    let mut rng = Rng::new(0x7EE0_6200);
    hashed(&layers::wide_and_deep(&mut rng, 3_000, 200), &SHIPPED, "wide and deep");
}

fn failed(walked: &Walked, path: &[u8], cause: Cause) {
    let (found, why) = walked.failed();
    assert_eq!(why, cause, "at a path of {} bytes", found.len());
    assert!(found == path, "the path of {} bytes is not the {} expected", found.len(), path.len());
}
fn refused(fs: &Fs, path: &[u8], why: Refusal) {
    failed(&helper::walk(fs, &SHIPPED, KEY), path, Cause::Refused(why));
}
fn bound(fs: &Fs, limits: &Limits, path: &[u8], which: Bound) {
    failed(&helper::walk(fs, limits, KEY), path, Cause::Bound(which));
}

#[test]
fn each_refusal_fails_at_its_path() {
    let mut fs = Fs::new();
    fs.root_mut().mode = 0o700;
    refused(&fs, b"", Refusal::RootMode);
    let mut fs = Fs::new();
    fs.root_mut().uid = 5;
    refused(&fs, b"", Refusal::RootOwner);
    let mut fs = Fs::new();
    *fs.root_mut() = Node::file(1, b"");
    refused(&fs, b"", Refusal::RootKind);

    let mut fs = Fs::new();
    fs.add(b"d", Node::new(Kind::Directory, 2));
    fs.add(b"d/f", Node::file(3, b"x").owned(65_536, 0));
    refused(&fs, b"d/f", Refusal::Owner);

    let mut fs = Fs::new();
    fs.add(b"sub", Node::new(Kind::Directory, 2).on(77));
    refused(&fs, b"sub", Refusal::Device);

    let mut fs = Fs::new();
    let grown = fs.add(b"f", Node::file(2, b"abc"));
    fs.node_mut(grown).grown = 1;
    refused(&fs, b"f", Refusal::Size);

    // Two nodes of one inode that disagree about it: the later path in tree order is refused.
    let mut fs = Fs::new();
    fs.add(b"a", Node::file(2, b"x"));
    fs.add(b"b", Node::file(2, b"x").moded(0o600));
    refused(&fs, b"b", Refusal::Link);
    let mut fs = Fs::new();
    fs.add(b"a", Node::file(2, b"x").attr(b"user.k", b"1"));
    fs.add(b"b", Node::file(2, b"x").attr(b"user.k", b"2"));
    refused(&fs, b"b", Refusal::Link);
    // Attributes the walk drops do not count.
    let mut fs = Fs::new();
    fs.add(b"a", Node::file(2, b"x").attr(b"trusted.k", b"1"));
    fs.add(b"b", Node::file(2, b"x").attr(b"trusted.k", b"2"));
    hashed(&fs, &SHIPPED, "dropped attributes");
}

#[test]
fn each_bound_fails_at_its_path() {
    let at = |limits: Limits| limits;
    let mut fs = Fs::new();
    for name in ["a", "b", "c"] {
        fs.add(name.as_bytes(), Node::file(10, b""));
    }
    bound(&fs, &at(Limits { entries: 3, ..SHIPPED }), b"c", Bound::Entries);
    bound(&fs, &at(Limits { names: 2, ..SHIPPED }), b"", Bound::Names);
    bound(&fs, &at(Limits { name: 0, ..SHIPPED }), b"", Bound::Name);

    let mut deep = Fs::new();
    let mut path = Vec::new();
    for level in 0..257 {
        path.extend_from_slice(if level == 0 { b"d" } else { b"/d" });
        deep.add(&path, Node::new(Kind::Directory, 2 + level));
    }
    // The shipped depth: 256 directories below the root, and the 257th is past it.
    bound(&deep, &SHIPPED, &path, Bound::Depth);
    path.truncate(path.len() - 2);
    let mut fits = Fs::new();
    let mut at = Vec::new();
    for level in 0..256 {
        at.extend_from_slice(if level == 0 { b"d" } else { b"/d" });
        fits.add(&at, Node::new(Kind::Directory, 2 + level));
    }
    hashed(&fits, &SHIPPED, "256 deep");

    // A path of 4096 bytes is the longest; one more fails.
    let long = |size: usize| {
        let mut fs = Fs::new();
        let mut path: Vec<u8> = Vec::new();
        loop {
            let used = if path.is_empty() { 0 } else { path.len() + 1 };
            let rest = size - used;
            if rest <= 255 {
                if !path.is_empty() {
                    path.push(b'/');
                }
                path.extend_from_slice(&vec![b'y'; rest]);
                fs.add(&path, Node::file(5, b""));
                return (fs, path);
            }
            if !path.is_empty() {
                path.push(b'/');
            }
            path.extend_from_slice(&[b'x'; 254]);
            fs.add(&path, Node::new(Kind::Directory, 1000 + u64::try_from(path.len()).expect("small")));
        }
    };
    let (fs, path) = long(4_096);
    assert_eq!(path.len(), 4_096);
    hashed(&fs, &SHIPPED, "a path of 4096 bytes");
    let (fs, path) = long(4_097);
    bound(&fs, &SHIPPED, &path, Bound::Path);

    // A target of 4096 bytes is the longest.
    let mut fs = Fs::new();
    fs.add(b"l", Node::symlink(2, &[b't'; 4_096]));
    hashed(&fs, &SHIPPED, "a target of 4096 bytes");
    let mut fs = Fs::new();
    fs.add(b"l", Node::symlink(2, &[b't'; 4_097]));
    bound(&fs, &SHIPPED, b"l", Bound::Target);

    // 64 attributes of 65,536 bytes each are the most, and an item that large is written.
    let mut full = Node::file(2, b"x");
    for number in 0..64 {
        full = full.attr(format!("user.{number:0>250}").as_bytes(), &vec![b'v'; 65_536]);
    }
    let mut fs = Fs::new();
    fs.add(b"f", full.clone());
    hashed(&fs, &SHIPPED, "attributes at their limits");
    let mut fs = Fs::new();
    fs.add(b"f", full.clone().attr(b"user.one-more", b""));
    bound(&fs, &SHIPPED, b"f", Bound::Attributes);
    let mut fs = Fs::new();
    fs.add(b"f", Node::file(2, b"x").attr(&[b"user.", &[b'n'; 251][..]].concat(), b""));
    bound(&fs, &SHIPPED, b"f", Bound::AttributeName);
    let mut fs = Fs::new();
    fs.add(b"f", Node::file(2, b"x").attr(b"user.v", &vec![b'v'; 65_537]));
    bound(&fs, &SHIPPED, b"f", Bound::AttributeValue);

    // Linked inodes and their first paths.
    let mut fs = Fs::new();
    let first = fs.add(b"a", Node::file(2, b""));
    fs.link(b"a2", first);
    let second = fs.add(b"b", Node::file(3, b""));
    fs.link(b"b2", second);
    bound(&fs, &Limits { linked: 1, ..SHIPPED }, b"b", Bound::Linked);
    bound(&fs, &Limits { linked_paths: 1, ..SHIPPED }, b"b", Bound::LinkedPaths);

    // Names held across the stack.
    let mut fs = Fs::new();
    fs.add(b"a", Node::new(Kind::Directory, 2));
    for name in ["x", "y", "z"] {
        fs.add(format!("a/{name}").as_bytes(), Node::file(10, b""));
    }
    fs.add(b"b", Node::file(3, b""));
    bound(&fs, &Limits { held: 3, ..SHIPPED }, b"a", Bound::Held);
}

#[test]
fn an_item_the_writer_refuses_fails_the_walk_naming_the_batchs_first_path() {
    let mut fs = Fs::new();
    fs.add(b"f", Node::file(2, b"x").attr(b"user.big", &[b'v'; 2_000]));
    let mut limits = protocol::SHIPPED;
    limits.tree.item = 1_000;
    let mut helper = Helper::new(&fs);
    helper.limit(limits);
    let walked = helper::walk_with(helper, &SHIPPED, KEY);
    failed(&walked, b"", Cause::Bound(Bound::Item));
}

#[test]
fn an_io_failure_at_any_request_ends_the_walk_naming_that_requests_path() {
    let mut rng = Rng::new(0x7EE0_6300);
    let fs = layers::layer(&mut rng, 24);
    let clean = helper::walk(&fs, &SHIPPED, KEY);
    for fault in [Fault::NotFound, Fault::Permission, Fault::NoSpace, Fault::Bound, Fault::Other] {
        for at in 0..clean.requests.len() {
            let mut helper = Helper::new(&fs);
            helper.fault = Some((at, fault));
            let walked = helper::walk_with(helper, &SHIPPED, KEY);
            let (path, cause) = walked.failed();
            assert_eq!(path, named(&clean.requests, at), "request {at}: {:?}", clean.requests[at]);
            let wanted = match fault {
                Fault::NotFound => Cause::NotFound,
                Fault::Permission => Cause::Permission,
                Fault::NoSpace => Cause::NoSpace,
                Fault::Bound => Cause::Bound(Bound::Item),
                Fault::Other => Cause::Io,
            };
            assert_eq!(cause, wanted, "request {at}");
            assert_eq!(walked.requests.len(), at + 1, "nothing is asked after a failure");
        }
    }
}

/// The path the failure of request `at` of a walk is reported at.
fn named(requests: &[Request], at: usize) -> &[u8] {
    match &requests[at] {
        Request::Stat { path }
        | Request::List { path, .. }
        | Request::Attributes { path }
        | Request::Attribute { path, .. }
        | Request::ReadLink { path }
        | Request::HashFile { path, .. } => path,
        Request::Next { .. } => requests[..at]
            .iter()
            .rev()
            .find_map(|request| match request {
                Request::List { path, .. } => Some(path.as_ref()),
                Request::Stat { .. }
                | Request::Next { .. }
                | Request::Attributes { .. }
                | Request::Attribute { .. }
                | Request::ReadLink { .. }
                | Request::HashFile { .. }
                | Request::Create
                | Request::Write { .. }
                | Request::ParentNext
                | Request::Digests { .. }
                | Request::Finish { .. } => None,
            })
            .expect("a directory is open"),
        Request::Write { items } => match &items[0] {
            hearth_domain_tree::vocabulary::Item::Entry(entry, _) => &entry.path,
            hearth_domain_tree::vocabulary::Item::Put { path }
            | hearth_domain_tree::vocabulary::Item::Delete { path } => path,
        },
        Request::Create | Request::ParentNext | Request::Digests { .. } | Request::Finish { .. } => b"",
    }
}
