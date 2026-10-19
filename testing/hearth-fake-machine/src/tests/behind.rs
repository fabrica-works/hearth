use super::fixtures;
use crate::{
    behind::{answer, refusal},
    linux::{Item, Refusal},
};
use skein_io::kernel::{Error, Kind, OpenHow, Stat};
use skein_sim::{Ask, Program, Reply};
use skein_world::{Cut, Machine};

#[test]
fn a_file_made_through_the_seam_reads_back_and_stats_as_laid() {
    let mut behind = fixtures::behind();
    let root = fixtures::directory(&mut behind, b"/srv/store");
    let file = fixtures::seam_file(&mut behind, root, b"made", OpenHow::Create { mode: Some(0o640) });
    assert_eq!(
        answer(&mut behind, Ask::Write { file, at: 0, bytes: b"hello".as_slice().into() }),
        Ok(Reply::Done),
        "written"
    );
    assert_eq!(answer(&mut behind, Ask::Close { file }), Ok(Reply::Done), "closed before reading");
    let file = fixtures::seam_file(&mut behind, root, b"made", OpenHow::Read);
    assert_eq!(
        answer(&mut behind, Ask::Read { file, at: 0, len: 10 }),
        Ok(Reply::Read(b"hello".as_slice().into())),
        "bytes read back"
    );
    assert_eq!(
        answer(&mut behind, Ask::Stat { file }),
        Ok(Reply::Stat(Stat { kind: Kind::File, size: 5, mode: 0o640, owner: 1000, links: 1 })),
        "mode, ownership and length"
    );
}
const REFUSALS: &[(Refusal, Error)] = &[
    (Refusal::NotFound, Error::NotFound),
    (Refusal::Exists, Error::Exists),
    (Refusal::NotADirectory, Error::NotADirectory),
    (Refusal::IsADirectory, Error::IsADirectory),
    (Refusal::NotEmpty, Error::NotEmpty),
    (Refusal::Permission, Error::Permission),
    (Refusal::NotPermitted, Error::Other(1)),
    (Refusal::Escape, Error::Escape),
    (Refusal::Loop, Error::Other(40)),
    (Refusal::NameTooLong, Error::NameTooLong),
    (Refusal::Invalid, Error::InvalidArgument),
    (Refusal::BrokenPipe, Error::BrokenPipe),
    (Refusal::Busy, Error::Busy),
];
#[test]
fn every_refusal_maps_to_the_kernel_error_of_its_name() {
    for (refused, expected) in REFUSALS {
        assert_eq!(refusal(*refused), *expected, "{refused:?}");
    }
}
#[test]
fn a_listing_resumes_where_it_stopped_and_keeps_to_the_room() {
    let mut behind = fixtures::behind();
    behind.machine_mut().linux_mut().lay(&[
        Item::file(b"/srv/store/a", 0o644, b""),
        Item::file(b"/srv/store/b", 0o644, b""),
        Item::file(b"/srv/store/c", 0o644, b""),
    ]);
    let dir = fixtures::directory(&mut behind, b"/srv/store");
    let mut names = Vec::new();
    for _ in 0_u32..3 {
        let Reply::Listed(page) = answer(&mut behind, Ask::List { dir, most: 10, room: 256 }).expect("list") else {
            panic!("List answers Listed")
        };
        assert_eq!(page.len(), 1, "stop before less than 255 bytes remain");
        names.extend(page.into_iter().map(|(_, name)| name));
    }
    names.sort();
    assert_eq!(
        names,
        vec![Box::from(b"a".as_slice()), Box::from(b"b".as_slice()), Box::from(b"c".as_slice())],
        "each entry exactly once"
    );
    assert_eq!(
        answer(&mut behind, Ask::List { dir, most: 10, room: 256 }),
        Ok(Reply::Listed(vec![])),
        "end stays reached"
    );
}
#[test]
fn a_spawn_of_an_unclaimed_file_exits_127() {
    let mut behind = fixtures::behind();
    let root = fixtures::directory(&mut behind, b"/");
    assert_eq!(
        answer(&mut behind, fixtures::seam_spawn(root, b"/usr/bin/unclaimed")),
        Ok(Reply::Program(Program::Exit(127))),
        "domain/testing.md, 3.5"
    );
}
#[test]
fn a_spawn_of_a_missing_file_is_not_found() {
    let mut behind = fixtures::behind();
    let root = fixtures::directory(&mut behind, b"/");
    assert_eq!(
        answer(&mut behind, fixtures::seam_spawn(root, b"/usr/bin/missing")),
        Err(Error::NotFound),
        "domain/testing.md, 3.5"
    );
}
#[test]
fn a_write_changes_the_digest_of_machines_laid_alike() {
    let mut first = fixtures::behind();
    let mut second = fixtures::behind();
    let one = fixtures::directory(&mut first, b"/srv/store");
    let two = fixtures::directory(&mut second, b"/srv/store");
    let file = fixtures::seam_file(&mut first, one, b"made", OpenHow::Create { mode: None });
    let other = fixtures::seam_file(&mut second, two, b"made", OpenHow::Create { mode: None });
    assert_eq!(file, other, "the same seam identities");
    assert_eq!(first.state_digest(), second.state_digest(), "laid and opened alike");
    assert_eq!(
        answer(&mut first, Ask::Write { file, at: 0, bytes: b"changed".as_slice().into() }),
        Ok(Reply::Done),
        "written"
    );
    assert_ne!(first.state_digest(), second.state_digest(), "a write changes the machine digest");
}
#[test]
fn the_ledger_does_not_move_the_digest() {
    let mut behind = fixtures::behind();
    fixtures::directory(&mut behind, b"/srv/store");
    let before = behind.state_digest();
    assert_eq!(behind.machine_mut().linux_mut().ledger().count(), 1, "one open to drain");
    assert_eq!(behind.state_digest(), before, "the referee's ledger is excluded");
}
#[test]
fn a_lock_is_busy_through_a_second_handle_until_the_first_closes() {
    let mut behind = fixtures::behind();
    let first = fixtures::directory(&mut behind, b"/srv/store");
    let second = fixtures::directory(&mut behind, b"/srv/store");
    assert_eq!(answer(&mut behind, Ask::Lock { file: first }), Ok(Reply::Done), "kernel.md, 6.1: taken");
    assert_eq!(answer(&mut behind, Ask::Lock { file: second }), Err(Error::Busy), "another open holds it");
    assert_eq!(answer(&mut behind, Ask::Close { file: first }), Ok(Reply::Done), "closed");
    assert_eq!(answer(&mut behind, Ask::Lock { file: second }), Ok(Reply::Done), "it went with its handle");
}
#[test]
fn a_kill_cut_releases_the_lock_it_held() {
    let mut behind = fixtures::behind();
    let first = fixtures::directory(&mut behind, b"/srv/store");
    let second = fixtures::directory(&mut behind, b"/srv/store");
    assert_eq!(answer(&mut behind, Ask::Lock { file: first }), Ok(Reply::Done), "taken");
    behind.cut(Cut::Kill, &[first], 7);
    assert_eq!(answer(&mut behind, Ask::Lock { file: second }), Ok(Reply::Done), "Kill closes the holder");
}
#[test]
#[should_panic(expected = "M2")]
fn a_power_loss_cut_asserts_m2() {
    fixtures::behind().cut(Cut::PowerLoss, &[], 7);
}
