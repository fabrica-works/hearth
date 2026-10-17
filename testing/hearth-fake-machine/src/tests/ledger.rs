use super::fixtures::{self, BUILDER};
use crate::linux::{At, How, Item, Op, Sizes};
const CHANGES: [u64; 2] = [1, 2];
const WRITTEN: [(bool, crate::linux::Ids, bool, u64, Option<crate::linux::Refusal>); 1] =
    [(true, BUILDER, true, 1, None)];
#[test]
fn each_change_to_a_file_moves_its_counter_and_lands_in_the_ledger() {
    let mut machine = fixtures::machine();
    let node = machine.lay(&[Item::file(b"/srv/file", 0o666, b"abc")])[0];
    let pid = machine.login(BUILDER, b"/srv");
    let file = fixtures::open(&mut machine, pid, b"/srv/file", How::Write);
    drop(machine.ledger());
    for &expected in &CHANGES {
        machine.call(pid, Op::Write { file, at: 0, bytes: b"x".as_slice().into() }).unwrap();
        assert_eq!(machine.facts(b"/srv/file").unwrap().changes, expected);
        let observed: Vec<_> = machine
            .ledger()
            .map(|entry| (entry.pid == pid, entry.ids, entry.node == Some(node), entry.bytes, entry.refused))
            .collect();
        assert_eq!(observed, WRITTEN);
    }
}
#[test]
#[should_panic(expected = "ledger")]
fn a_full_ledger_asserts() {
    let sizes = Sizes { ledger: 1, ..Sizes::tiny() };
    let mut machine = crate::linux::Machine::new(fixtures::SEED, sizes);
    let pid = machine.login(BUILDER, b"/");
    machine.call(pid, Op::Open { at: At::Root, path: b"/".as_slice().into(), how: How::Directory }).unwrap();
    machine.call(pid, Op::Open { at: At::Root, path: b"/".as_slice().into(), how: How::Directory }).unwrap();
}
const REFUSED: [(u64, Option<crate::linux::Refusal>); 1] = [(0, Some(crate::linux::Refusal::NotFound))];
#[test]
fn a_refusal_is_recorded_once_without_moving_bytes() {
    let mut machine = fixtures::machine();
    let pid = machine.login(BUILDER, b"/srv");
    machine.call(pid, Op::Open { at: At::Root, path: b"/srv/missing".as_slice().into(), how: How::Read }).unwrap_err();
    let observed: Vec<_> = machine.ledger().map(|entry| (entry.bytes, entry.refused)).collect();
    assert_eq!(observed, REFUSED);
}
const TRANSFERRED: [u64; 2] = [3, 2];
#[test]
fn the_ledger_records_each_pipe_s_bytes_moved() {
    use crate::linux::{Pipe, Way};
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, pipes) =
        fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 0, way: Way::In }]);
    let input = machine.linux().descriptor(child, 0);
    drop(machine.linux_mut().ledger());
    fixtures::sent(&mut machine, parent, pipes[0], b"abc").unwrap();
    fixtures::took(&mut machine, child, input, 2);
    let observed: Vec<_> = machine.linux_mut().ledger().map(|entry| entry.bytes).collect();
    assert_eq!(observed, TRANSFERRED);
}
