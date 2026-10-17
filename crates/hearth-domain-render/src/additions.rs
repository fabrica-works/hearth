//! Files and tmpfiles lines render contributes (render.md, 3.6, 4.2).
use crate::{
    kinds::{
        data,
        users::{Accounts, UserKind},
    },
    parts::{
        config::{Line, LineKind, Tmpfile},
        packages::{Initramfs, Microcode},
    },
    revision::Position,
};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Map, Writer};
#[must_use]
pub fn subordinate(accounts: &Accounts) -> Option<Box<[u8]>> {
    let mut ranges = Map::with_capacity(u32::try_from(accounts.users.len()).expect("user bound"));
    let mut length = 0_usize;
    for user in &accounts.users {
        if let UserKind::Sysusers { subordinate: Some((base, count)), .. } = &user.kind {
            let base = Decimal::of(u64::from(*base));
            let count = Decimal::of(u64::from(*count));
            length = length
                .saturating_add(user.name.len())
                .saturating_add(base.as_bytes().len())
                .saturating_add(count.as_bytes().len())
                .saturating_add(3);
            ranges.insert(user.name.clone(), (base, count)).expect("user bound");
        }
    }
    if ranges.is_empty() {
        return None;
    }
    let mut writer = Writer::new(length);
    for (name, (base, count)) in &ranges {
        for bytes in [name.as_ref(), b":", base.as_bytes(), b":", count.as_bytes(), b"\n"] {
            writer.put(bytes).expect("range length");
        }
    }
    Some(writer.finish())
}
/// A generated line retains its declaration's claimant for collision checks.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataLine {
    pub task: Box<[u8]>,
    pub at: Position,
    pub tmpfile: Tmpfile,
}
#[must_use]
pub fn data_lines(entries: &[data::Entry]) -> Box<[DataLine]> {
    let mut lines = List::with_capacity(u32::try_from(entries.len()).expect("entry bound"));
    for entry in entries {
        lines
            .push(DataLine {
                task: entry.task.clone(),
                at: entry.at,
                tmpfile: Tmpfile {
                    path: entry.path.clone(),
                    line: Line {
                        kind: LineKind::D,
                        mode: Some(entry.mode),
                        owner: Some(entry.owner.clone()),
                        group: Some(entry.group.clone()),
                        age: None,
                        argument: None,
                    },
                },
            })
            .expect("entry bound");
    }
    lines.into_boxed()
}
#[must_use]
pub fn initramfs(initramfs: &Initramfs) -> Box<[u8]> {
    let mut modules = Map::with_capacity(u32::try_from(initramfs.modules.len()).expect("module bound"));
    let mut length = 0_usize;
    for module in &initramfs.modules {
        if !modules.contains_key(module) {
            length = length.saturating_add(module.len()).saturating_add(1);
            modules.insert(module.clone(), ()).expect("module bound");
        }
    }
    let microcode: &[u8] = match initramfs.microcode {
        Microcode::None => b"no",
        Microcode::Generic => b"generic",
    };
    let header: &[u8] = b"DEBUG=no\nAUTO_MODULES=no\nMICROCODE=";
    let include: &[u8] = b"\nINCLUDE_MODULES=";
    let mut writer = Writer::new(
        header
            .len()
            .saturating_add(microcode.len())
            .saturating_add(include.len())
            .saturating_add(length)
            .saturating_add(usize::from(modules.is_empty())),
    );
    for bytes in [header, microcode, include] {
        writer.put(bytes).expect("configuration length");
    }
    for (index, (module, ())) in modules.iter().enumerate() {
        if index != 0 {
            writer.put(b",").expect("separator length");
        }
        writer.put(module).expect("module length");
    }
    writer.put(b"\n").expect("LF length");
    writer.finish()
}
