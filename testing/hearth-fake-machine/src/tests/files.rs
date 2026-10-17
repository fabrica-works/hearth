use super::fixtures::{self, BUILDER, DONE, OTHER};
use crate::linux::{At, Facts, How, Item, Kind, Op, Refusal};

const CREATED: [Facts; 1] =
    [Facts { kind: Kind::File, size: 0, mode: 0o644, user: 1000, group: 1000, links: 1, changes: 0 }];
#[test]
fn a_file_created_takes_its_mode_less_the_umask_and_its_creator() {
    let mut machine = fixtures::machine();
    let pid = machine.login(BUILDER, b"/srv");
    fixtures::open(&mut machine, pid, b"/srv/new", How::Create { mode: None });
    assert_eq!(machine.facts(b"/srv/new"), Some(CREATED[0]));
}
const ESCAPES: [(&[u8], Refusal); 2] = [(b"../elsewhere", Refusal::Escape), (b"/elsewhere", Refusal::Escape)];
#[test]
fn a_path_that_climbs_above_its_handle_is_refused_as_an_escape() {
    let mut machine = fixtures::machine();
    let pid = machine.login(BUILDER, b"/srv");
    let dir = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    for &(path, expected) in &ESCAPES {
        assert_eq!(
            machine.call(pid, Op::Open { at: At::Beneath(dir), path: path.into(), how: How::Read }),
            Err(expected),
            "{path:?}"
        );
    }
}
const NO_FOLLOW: [(How, Refusal); 4] = [
    (How::ReadNoFollow, Refusal::Loop),
    (How::DirectoryNoFollow, Refusal::Loop),
    (How::PathNoFollow, Refusal::Loop),
    (How::WriteNoFollow, Refusal::Loop),
];
#[test]
fn a_link_on_the_way_is_refused_when_the_open_follows_none() {
    let mut machine = fixtures::machine();
    machine.lay(&[Item::link(b"/srv/link", b".")]);
    let pid = machine.login(BUILDER, b"/srv");
    for &(how, expected) in &NO_FOLLOW {
        assert_eq!(
            machine.call(pid, Op::Open { at: At::Root, path: b"/srv/link/child".as_slice().into(), how }),
            Err(expected),
            "{how:?}"
        );
    }
}
const DENIED: [Refusal; 1] = [Refusal::Permission];
#[test]
fn reading_a_file_whose_other_bits_deny_it_is_refused() {
    let mut machine = fixtures::machine();
    machine.lay(&[Item::file(b"/srv/private", 0o640, b"hidden").owner(BUILDER)]);
    let pid = machine.login(OTHER, b"/srv");
    assert_eq!(
        machine.call(pid, Op::Open { at: At::Root, path: b"/srv/private".as_slice().into(), how: How::Read }),
        Err(DENIED[0])
    );
}
const RENAMES: [(&[u8], Result<(), Refusal>); 2] = [(b"to/moved", Err(Refusal::Permission)), (b"from/kept", Ok(()))];
#[test]
fn a_rename_into_another_directory_needs_write_on_the_moved_directory() {
    let mut machine = fixtures::machine();
    machine.lay(&[
        Item::directory(b"/srv/from", 0o755).owner(BUILDER),
        Item::directory(b"/srv/from/moved", 0o555).owner(BUILDER),
        Item::directory(b"/srv/to", 0o755).owner(BUILDER),
    ]);
    let pid = machine.login(BUILDER, b"/srv");
    let dir = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    for &(to, expected) in &RENAMES {
        assert_eq!(
            fixtures::done(
                &mut machine,
                pid,
                Op::Rename { from_dir: dir, from: b"from/moved".as_slice().into(), to_dir: dir, to: to.into() }
            ),
            expected,
            "{to:?}"
        );
    }
}
const LISTED: [(&[u8], Kind); 3] = [(b"b", Kind::File), (b"a", Kind::File), (b"c", Kind::File)];
#[test]
fn a_listing_comes_back_in_an_order_drawn_from_the_seed() {
    let mut machine = fixtures::machine();
    machine.lay(&[
        Item::file(b"/srv/a", 0o644, b""),
        Item::file(b"/srv/b", 0o644, b""),
        Item::file(b"/srv/c", 0o644, b""),
    ]);
    let pid = machine.login(BUILDER, b"/srv");
    let dir = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    let mut entries = fixtures::listing(&mut machine, pid, dir, 1);
    entries.extend(fixtures::listing(&mut machine, pid, dir, 2));
    let observed: Vec<_> = entries.iter().map(|(kind, name)| (name.as_ref(), *kind)).collect();
    assert_eq!(observed, LISTED);
}
const STICKY: [(crate::linux::Ids, Result<(), Refusal>); 2] = [(OTHER, Err(Refusal::NotPermitted)), (BUILDER, Ok(()))];
#[test]
fn a_sticky_directory_lets_only_an_owner_remove_an_entry() {
    let mut machine = fixtures::machine();
    machine.lay(&[Item::directory(b"/tmp", 0o1777), Item::file(b"/tmp/owned", 0o644, b"").owner(BUILDER)]);
    for &(ids, expected) in &STICKY {
        let pid = machine.login(ids, b"/tmp");
        let dir = fixtures::open(&mut machine, pid, b"/tmp", How::Directory);
        assert_eq!(
            fixtures::done(&mut machine, pid, Op::Remove { dir, path: b"owned".as_slice().into(), directory: false }),
            expected,
            "{ids:?}"
        );
    }
}
const LOOPS: [(u32, Option<Refusal>); 2] = [(40, None), (41, Some(Refusal::Loop))];
#[test]
fn a_chain_of_forty_one_links_is_refused_as_a_loop() {
    for &(count, expected) in &LOOPS {
        let mut machine = fixtures::machine();
        machine.lay(&[Item::file(b"/srv/end", 0o644, b"")]);
        for index in (0..count).rev() {
            let target = if index.checked_add(1).unwrap() == count {
                "end".into()
            } else {
                format!("link{}", index.checked_add(1).unwrap())
            };
            machine.lay(&[Item::link(format!("/srv/link{index}").as_bytes(), target.as_bytes())]);
        }
        let pid = machine.login(BUILDER, b"/srv");
        assert_eq!(
            machine.call(pid, Op::Open { at: At::Root, path: b"/srv/link0".as_slice().into(), how: How::Read }).err(),
            expected,
            "{count}"
        );
    }
}
const ROOT: [crate::linux::Ids; 1] = [crate::linux::Ids { user: 0, group: 0 }];
#[test]
#[should_panic(expected = "no process runs as uid 0")]
fn a_login_as_uid_zero_asserts() {
    let mut machine = fixtures::machine();
    machine.login(ROOT[0], b"/");
}
const LOCKS: [Result<(), Refusal>; 4] = [Ok(()), Ok(()), Err(Refusal::Busy), Ok(())];
#[test]
fn a_lock_is_busy_through_another_handle_until_the_first_closes() {
    let mut machine = fixtures::machine();
    let pid = machine.login(BUILDER, b"/srv");
    let first = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    let second = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    assert_eq!(fixtures::done(&mut machine, pid, Op::Lock { file: first }), LOCKS[0]);
    assert_eq!(fixtures::done(&mut machine, pid, Op::Lock { file: first }), LOCKS[1]);
    assert_eq!(fixtures::done(&mut machine, pid, Op::Lock { file: second }), LOCKS[2]);
    machine.call(pid, Op::Close { handle: first }).unwrap();
    assert_eq!(fixtures::done(&mut machine, pid, Op::Lock { file: second }), LOCKS[3]);
}
#[test]
#[should_panic(expected = "a handle it issued")]
fn a_handle_the_process_does_not_hold_asserts() {
    let mut machine = fixtures::machine();
    let owner = machine.login(BUILDER, b"/srv");
    let other = machine.login(OTHER, b"/srv");
    let handle = fixtures::open(&mut machine, owner, b"/srv", How::Directory);
    machine.call(other, Op::Stat { handle }).unwrap();
}
const CONTENTS: [&[u8]; 1] = [b"changed"];
#[test]
fn a_hard_link_and_an_open_handle_share_contents_after_a_rename() {
    let mut machine = fixtures::machine();
    machine.lay(&[Item::file(b"/srv/file", 0o666, b"initial"), Item::hard_link(b"/srv/alias", b"/srv/file")]);
    let pid = machine.login(BUILDER, b"/srv");
    let file = fixtures::open(&mut machine, pid, b"/srv/file", How::Write);
    let dir = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    machine
        .call(
            pid,
            Op::Rename {
                from_dir: dir,
                from: b"file".as_slice().into(),
                to_dir: dir,
                to: b"renamed".as_slice().into(),
            },
        )
        .unwrap();
    machine.call(pid, Op::Write { file, at: 0, bytes: b"changed".as_slice().into() }).unwrap();
    assert_eq!(machine.contents(b"/srv/alias"), Some(CONTENTS[0]));
}
const READS: [(u64, u32, &[u8]); 3] = [(0, 2, b"ab"), (2, 9, b"c"), (9, 2, b"")];
#[test]
fn reads_return_the_requested_range_or_end_of_file() {
    let mut machine = fixtures::machine();
    machine.lay(&[Item::file(b"/srv/file", 0o644, b"abc")]);
    let pid = machine.login(BUILDER, b"/srv");
    let file = fixtures::open(&mut machine, pid, b"/srv/file", How::Read);
    for &(at, most, expected) in &READS {
        assert_eq!(fixtures::read(&mut machine, pid, file, at, most).as_ref(), expected, "{at}");
    }
}
const TRUNCATED: [(u64, &[u8]); 2] = [(5, b"abc\0\0"), (1, b"a")];
#[test]
fn truncation_zeros_an_extension_and_removes_a_tail() {
    let mut machine = fixtures::machine();
    machine.lay(&[Item::file(b"/srv/file", 0o666, b"abc")]);
    let pid = machine.login(BUILDER, b"/srv");
    let file = fixtures::open(&mut machine, pid, b"/srv/file", How::Write);
    for &(length, expected) in &TRUNCATED {
        assert_eq!(fixtures::done(&mut machine, pid, Op::Truncate { file, length }), DONE[0]);
        assert_eq!(machine.contents(b"/srv/file"), Some(expected), "{length}");
    }
}

const DIRECTORY_LINKS: [u32; 3] = [2, 3, 2];
#[test]
fn a_directory_counts_its_children_s_parent_links() {
    let mut machine = fixtures::machine();
    let pid = machine.login(BUILDER, b"/srv");
    let dir = fixtures::open(&mut machine, pid, b"/srv", How::Directory);
    assert_eq!(machine.facts(b"/srv").unwrap().links, DIRECTORY_LINKS[0]);
    machine.call(pid, Op::MakeDirectory { dir, path: b"child".as_slice().into(), mode: 0o777 }).unwrap();
    assert_eq!(machine.facts(b"/srv").unwrap().links, DIRECTORY_LINKS[1]);
    machine.call(pid, Op::Remove { dir, path: b"child".as_slice().into(), directory: true }).unwrap();
    assert_eq!(machine.facts(b"/srv").unwrap().links, DIRECTORY_LINKS[2]);
}
