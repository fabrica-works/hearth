use super::fake::{Node, Run, dir, file, link, root, run, special};
use crate::{
    limits::SHIPPED,
    vocabulary::{Cause, Entry, Kind, Payload, Refusal, Request},
};
use skein_lib::bytes;

fn entry<'a>(run: &'a Run, path: &[u8]) -> &'a Entry {
    run.entry(path)
}

#[test]
fn the_first_path_in_tree_order_is_the_file_and_later_ones_point_at_it() {
    const NODES: &[Node] = &[
        root(),
        dir(b"b", 2),
        file(b"b/z", 3, 5).linked(3),
        file(b"a", 3, 5).linked(3),
        file(b"c", 3, 5).linked(3),
        file(b"lone", 4, 1),
    ];
    let run = run(NODES, SHIPPED);
    assert_eq!(entry(&run, b"a").payload, Payload::File { size: 5, content: hearth_lib::encode::Digest([3; 32]) });
    assert_eq!(entry(&run, b"b/z").payload, Payload::Hardlink { path: bytes::copy_of(b"a") });
    assert_eq!(entry(&run, b"c").payload, Payload::Hardlink { path: bytes::copy_of(b"a") });
    let mut hashed = 0_u32;
    for request in &run.requests {
        if let Request::HashFile { .. } = request {
            hashed = hashed.saturating_add(1);
        }
    }
    assert_eq!(hashed, 2_u32, "the first path and the lone file are read; later links never");
}

#[test]
fn a_symlink_with_two_links_is_a_link_and_then_a_hard_link() {
    const NODES: &[Node] = &[root(), link(b"a", 2, b"t").linked(2), link(b"b", 2, b"t").linked(2)];
    let run = run(NODES, SHIPPED);
    assert_eq!(entry(&run, b"a").payload, Payload::Symlink { target: bytes::copy_of(b"t") });
    assert_eq!(entry(&run, b"b").payload, Payload::Hardlink { path: bytes::copy_of(b"a") });
    let mut read = 0_u32;
    for request in &run.requests {
        if let Request::ReadLink { .. } = request {
            read = read.saturating_add(1);
        }
    }
    assert_eq!(read, 1_u32);
}

#[test]
fn devices_and_fifos_may_be_linked_too_and_a_directory_never_is() {
    const NODES: &[Node] = &[root(), special(b"a", 2, Kind::Char).linked(2), special(b"b", 2, Kind::Char).linked(2)];
    let chars = run(NODES, SHIPPED);
    assert_eq!(entry(&chars, b"b").payload, Payload::Hardlink { path: bytes::copy_of(b"a") });
    let dirs = [root(), dir(b"a", 2).linked(2), dir(b"b", 3).linked(2)];
    let walked = run(&dirs, SHIPPED);
    assert_eq!(walked.entries.len(), 3);
    assert_eq!(entry(&walked, b"b").payload, Payload::Directory);
}

#[test]
fn a_later_link_keeps_its_own_mode_owners_and_attributes_when_they_agree() {
    const ATTRS: &[(&[u8], &[u8])] = &[(b"user.k", b"v"), (b"trusted.k", b"ignored")];
    const NODES: &[Node] = &[
        root(),
        file(b"a", 2, 1).linked(2).mode(0o640).owner(3, 4).with(ATTRS),
        file(b"b", 2, 1).linked(2).mode(0o640).owner(3, 4).with(&[(b"user.k", b"v")]),
    ];
    let run = run(NODES, SHIPPED);
    let later = entry(&run, b"b");
    assert_eq!((later.mode, later.uid, later.gid), (0o640_u32, 3_u32, 4_u32));
    assert_eq!(later.xattrs.len(), 1);
}

#[test]
fn a_later_link_that_disagrees_is_refused_at_its_path() {
    const ATTRS: &[(&[u8], &[u8])] = &[(b"user.k", b"v")];
    const OTHER: &[(&[u8], &[u8])] = &[(b"user.k", b"w")];
    let rows: [(Node, &str); 5] = [
        (file(b"b", 2, 1).linked(2).mode(0o600), "mode"),
        (file(b"b", 2, 1).linked(2).owner(1, 0), "uid"),
        (file(b"b", 2, 1).linked(2).owner(0, 1), "gid"),
        (file(b"b", 2, 1).linked(2), "attributes missing"),
        (file(b"b", 2, 1).linked(2).with(OTHER), "value"),
    ];
    for (later, what) in rows {
        let nodes = [root(), file(b"a", 2, 1).linked(2).with(ATTRS), later];
        let run = run(&nodes, SHIPPED);
        assert_eq!(run.failed(), (&b"b"[..], Cause::Refused(Refusal::Link)), "{what}");
    }
}
