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
        reply @ (Reply::Spawned { .. }
        | Reply::Sent(_)
        | Reply::Took(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_)
        | Reply::Read(_)
        | Reply::Done
        | Reply::Stat(_)
        | Reply::Listed(_)) => {
            panic!("expected Opened, got {reply:?}")
        }
    }
}
pub fn done(machine: &mut Machine, pid: Pid, op: Op) -> Result<(), crate::linux::Refusal> {
    match machine.call(pid, op)? {
        Reply::Done => Ok(()),
        reply @ (Reply::Spawned { .. }
        | Reply::Sent(_)
        | Reply::Took(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_)
        | Reply::Opened(_)
        | Reply::Read(_)
        | Reply::Stat(_)
        | Reply::Listed(_)) => {
            panic!("expected Done, got {reply:?}")
        }
    }
}
pub fn listing(machine: &mut Machine, pid: Pid, dir: Handle, most: u32) -> Vec<(crate::linux::Kind, Box<[u8]>)> {
    match machine.call(pid, Op::List { dir, most }).expect("list succeeds") {
        Reply::Listed(entries) => entries.into(),
        reply @ (Reply::Spawned { .. }
        | Reply::Sent(_)
        | Reply::Took(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_)
        | Reply::Opened(_)
        | Reply::Read(_)
        | Reply::Done
        | Reply::Stat(_)) => {
            panic!("expected Listed, got {reply:?}")
        }
    }
}
pub fn read(machine: &mut Machine, pid: Pid, file: Handle, at: u64, most: u32) -> Box<[u8]> {
    match machine.call(pid, Op::Read { file, at, most }).expect("read succeeds") {
        Reply::Read(bytes) => bytes,
        reply @ (Reply::Spawned { .. }
        | Reply::Sent(_)
        | Reply::Took(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_)
        | Reply::Opened(_)
        | Reply::Done
        | Reply::Stat(_)
        | Reply::Listed(_)) => {
            panic!("expected Read, got {reply:?}")
        }
    }
}
pub fn whole() -> crate::Machine {
    let mut machine = crate::Machine::new(SEED, Sizes::tiny());
    machine.linux_mut().lay(&[
        Item::directory(b"/srv", 0o755).owner(BUILDER),
        Item::directory(b"/usr", 0o755),
        Item::directory(b"/usr/bin", 0o755),
    ]);
    machine
}
pub fn builder(machine: &mut crate::Machine) -> Pid {
    machine.linux_mut().login(BUILDER, b"/srv")
}
pub fn install_git(machine: &mut crate::Machine) {
    machine.install(b"/usr/bin/git", crate::hearth::Tool::Git);
}
pub fn spawn(
    machine: &mut crate::Machine,
    parent: Pid,
    program: &[u8],
    args: &[&[u8]],
    pipes: &[crate::linux::Pipe],
) -> (Pid, Box<[Handle]>) {
    let root = open(machine.linux_mut(), parent, b"/", How::Directory);
    let op = Op::Spawn(crate::linux::Spawn {
        program: program.into(),
        args: args.iter().map(|a| Box::from(*a)).collect(),
        env: Box::default(),
        root,
        dir: b"srv".as_slice().into(),
        pipes: pipes.into(),
    });
    let reply = machine.call(parent, op).expect("spawn succeeds");
    spawned(reply)
}
pub fn spawned(reply: Reply) -> (Pid, Box<[Handle]>) {
    match reply {
        Reply::Spawned { child, pipes } => (child, pipes),
        Reply::Opened(_)
        | Reply::Read(_)
        | Reply::Done
        | Reply::Stat(_)
        | Reply::Listed(_)
        | Reply::Sent(_)
        | Reply::Took(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_) => panic!("expected Spawned, got {reply:?}"),
    }
}
pub fn sent(machine: &mut crate::Machine, pid: Pid, pipe: Handle, bytes: &[u8]) -> Result<u32, crate::linux::Refusal> {
    match machine.call(pid, Op::Send { pipe, bytes: bytes.into() })? {
        Reply::Sent(length) => Ok(length),
        Reply::Opened(_)
        | Reply::Read(_)
        | Reply::Done
        | Reply::Stat(_)
        | Reply::Listed(_)
        | Reply::Spawned { .. }
        | Reply::Took(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_) => panic!("expected Sent"),
    }
}
pub fn took(machine: &mut crate::Machine, pid: Pid, pipe: Handle, most: u32) -> Box<[u8]> {
    match machine.call(pid, Op::Take { pipe, most }).expect("take succeeds") {
        Reply::Took(bytes) => bytes,
        Reply::Opened(_)
        | Reply::Read(_)
        | Reply::Done
        | Reply::Stat(_)
        | Reply::Listed(_)
        | Reply::Spawned { .. }
        | Reply::Sent(_)
        | Reply::Empty
        | Reply::Ended
        | Reply::Running
        | Reply::Exited(_) => panic!("expected Took"),
    }
}

// Pinned forky index, 20261017T143929Z.
pub const MAWK: &[u8] = b"Package: mawk\n\
Version: 1.3.4.20260302-2\n\
Installed-Size: 299\n\
Maintainer: Boyuan Yang <byang@debian.org>\n\
Architecture: amd64\n\
Provides: awk\n\
Depends: libc6 (>= 2.38)\n\
Description: Pattern scanning and text processing language\n\
Multi-Arch: foreign\n\
Homepage: https://invisible-island.net/mawk/\n\
Description-md5: 05965bfcd1a2333e963d9f1945d161fc\n\
Tag: devel::interpreter, implemented-in::c, interface::commandline,\n\
 role::program, scope::utility, use::filtering, use::scanning,\n\
 works-with::text\n\
Section: interpreters\n\
Priority: required\n\
Filename: pool/main/m/mawk/mawk_1.3.4.20260302-2_amd64.deb\n\
Size: 144700\n\
MD5sum: 24d7eedf2197e29402ff8c06abd465af\n\
SHA256: 6fcd463ea272c05627d5159716336b53bfa14b35b1ac7ba29a878f0ee1352419\n\
\n\
";
pub const MAWK_KEPT: &[u8] = b"Package: mawk\n\
Version: 1.3.4.20260302-2\n\
Installed-Size: 299\n\
Architecture: amd64\n\
Provides: awk\n\
Depends: libc6 (>= 2.38)\n\
Section: interpreters\n\
Priority: required\n\
Filename: pool/main/m/mawk/mawk_1.3.4.20260302-2_amd64.deb\n\
Size: 144700\n\
SHA256: 6fcd463ea272c05627d5159716336b53bfa14b35b1ac7ba29a878f0ee1352419\n\
\n\
";

// Pinned forky index, 20261017T143929Z.
pub const TINY_INITRAMFS: &[u8] = b"Package: tiny-initramfs\n\
Version: 0.1-8\n\
Installed-Size: 35\n\
Maintainer: Debian QA Group <packages@qa.debian.org>\n\
Architecture: all\n\
Provides: linux-initramfs-tool\n\
Depends: tiny-initramfs-core (>= 0.1-8), tiny-initramfs-core (<< 0.1-8+b+~)\n\
Conflicts: dracut, initramfs-tools, live-tools\n\
Description: Minimalistic initramfs implementation (automation)\n\
Homepage: https://github.com/chris-se/tiny-initramfs/\n\
Description-md5: f21681110ae703bb6849e64e50836f0b\n\
Section: utils\n\
Priority: optional\n\
Filename: pool/main/t/tiny-initramfs/tiny-initramfs_0.1-8_all.deb\n\
Size: 13420\n\
MD5sum: d0561fcf207289e483b4f210b502b504\n\
SHA256: bad1f0b5e68fd252b511a5c02f3d162b04a4cae66a0635581154fb8f5566b9c6\n\
\n\
";
