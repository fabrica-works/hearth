use crate::linux::{At, Handle, How, Ids, Item, Machine, Op, Pid, Reply, Sizes};
pub const SEED: u64 = 26;
pub const BUILDER: Ids = Ids { user: 1000, group: 1000 };
pub const OTHER: Ids = Ids { user: 1001, group: 1001 };
pub const DONE: [Result<(), crate::linux::Refusal>; 1] = [Ok(())];
pub fn machine() -> Machine {
    let mut machine = Machine::new(SEED, Sizes::tiny());
    machine.lay(&[Item::directory(b"/srv", 0o755).owner(BUILDER)]);
    machine
}
pub fn open(machine: &mut Machine, pid: Pid, path: &[u8], how: How) -> Handle {
    match machine.call(pid, Op::Open { at: At::Root, path: path.into(), how }).expect("open succeeds") {
        Reply::Opened(handle) => handle,
        reply @ (Reply::Read(_) | Reply::Done | Reply::Stat(_) | Reply::Listed(_)) => {
            panic!("expected Opened, got {reply:?}")
        }
    }
}
pub fn done(machine: &mut Machine, pid: Pid, op: Op) -> Result<(), crate::linux::Refusal> {
    match machine.call(pid, op)? {
        Reply::Done => Ok(()),
        reply @ (Reply::Opened(_) | Reply::Read(_) | Reply::Stat(_) | Reply::Listed(_)) => {
            panic!("expected Done, got {reply:?}")
        }
    }
}
pub fn listing(machine: &mut Machine, pid: Pid, dir: Handle, most: u32) -> Vec<(crate::linux::Kind, Box<[u8]>)> {
    match machine.call(pid, Op::List { dir, most }).expect("list succeeds") {
        Reply::Listed(entries) => entries.into(),
        reply @ (Reply::Opened(_) | Reply::Read(_) | Reply::Done | Reply::Stat(_)) => {
            panic!("expected Listed, got {reply:?}")
        }
    }
}
pub fn read(machine: &mut Machine, pid: Pid, file: Handle, at: u64, most: u32) -> Box<[u8]> {
    match machine.call(pid, Op::Read { file, at, most }).expect("read succeeds") {
        Reply::Read(bytes) => bytes,
        reply @ (Reply::Opened(_) | Reply::Done | Reply::Stat(_) | Reply::Listed(_)) => {
            panic!("expected Read, got {reply:?}")
        }
    }
}
