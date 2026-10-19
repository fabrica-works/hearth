//! A pull reader of a tree file: the caller feeds the bytes at the next item,
//! and the file is never held whole (domain/store.md, 6.3).

use super::{
    CHANGE, END, ENTRY, End, Entry, Item, Kernel, Local, MODE, OWNER, Op, Payload, TARGET, VALUE, VERSION, XATTRS,
    Xattr,
    path::{PATH, path_valid, tree_order},
};
use crate::{
    fields::Fields,
    frame::{self, Kind, Unreadable},
    layer,
    limits::Limits,
};
use alloc::boxed::Box;
use core::cmp::Ordering;
use skein_lib::{List, Reader as Bytes, bytes};

/// What the next bytes hold.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Next {
    /// An item, and the bytes it took from the front.
    Item(Item, u32),
    /// The item needs this many bytes from the front, which the caller
    /// has not yet fed.
    Need(u32),
}

/// Where the reader is in the file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Header,
    Items,
    Ended,
}

/// A tree file's reader: its state is the last path it saw, and the bytes
/// it has taken. The decoder is bounded by the tree kind's map key
/// (domain/store.md, 6.1).
#[derive(Debug)]
pub struct Reader {
    limits: Limits,
    phase: Phase,
    length: u64,
    taken: u64,
    last: Option<Box<[u8]>>,
    /// The last item was an entry, which its put may follow.
    open: bool,
    failed: Option<Unreadable>,
}

impl Reader {
    #[must_use]
    pub fn new(limits: &Limits) -> Reader {
        Reader {
            limits: Limits { key: limits.tree.key, ..*limits },
            phase: Phase::Header,
            length: 0,
            taken: 0,
            last: None,
            open: false,
            failed: None,
        }
    }

    /// Checks the header, the body's length against its bound first, and
    /// returns that length.
    pub fn header(&mut self, header: &[u8; 24]) -> Result<u64, Unreadable> {
        if let Some(error) = self.failed {
            return Err(error);
        }
        if self.phase != Phase::Header {
            return self.fail(Unreadable::Record);
        }
        match frame::check_header(header, Kind::Tree, &self.limits) {
            Ok(length) => {
                self.length = length;
                self.phase = Phase::Items;
                Ok(length)
            }
            Err(error) => self.fail(error),
        }
    }

    /// Whether the end item was read, and the body ends with it.
    #[must_use]
    pub fn ended(&self) -> bool {
        self.phase == Phase::Ended
    }

    /// The item at the front of `bytes`, which is the file from the item's
    /// first byte on, or the bytes it needs. The item's length is checked
    /// before its value is decoded. An error ends the reader.
    pub fn next(&mut self, bytes: &[u8]) -> Result<Next, Unreadable> {
        if let Some(error) = self.failed {
            return Err(error);
        }
        match self.take(bytes) {
            Ok(next) => Ok(next),
            Err(error) => self.fail(error),
        }
    }

    fn fail<T>(&mut self, error: Unreadable) -> Result<T, Unreadable> {
        self.failed = Some(error);
        Err(error)
    }

    fn take(&mut self, input: &[u8]) -> Result<Next, Unreadable> {
        match self.phase {
            Phase::Header => return Err(Unreadable::Missing(b"header")),
            Phase::Ended => return Err(Unreadable::Record),
            Phase::Items => {}
        }
        let left = self.length.checked_sub(self.taken).expect("the items stay within the body");
        if left == 0 {
            return Err(Unreadable::Missing(b"end"));
        }
        if left < 5 {
            return Err(self.past(5));
        }
        let Some(head) = input.get(..5) else {
            return Ok(Next::Need(5));
        };
        let mut head = Bytes::new(head);
        let tag = head.u8().expect("the head has 5 bytes");
        let value = head.u32().expect("the head has 5 bytes");
        if tag != ENTRY && tag != CHANGE && tag != END {
            return Err(Unreadable::Tag { found: tag });
        }
        if value > self.limits.tree.item {
            return Err(Unreadable::Limit(b"item"));
        }
        let Some(size) = value.checked_add(5) else {
            return Err(Unreadable::Limit(b"item"));
        };
        if u64::from(size) > left {
            return Err(self.past(size));
        }
        let Some(whole) = input.get(..usize::try_from(size).expect("a u32 fits in memory")) else {
            return Ok(Next::Need(size));
        };
        let body = whole.get(5..).expect("the item has its head");
        let item = match tag {
            ENTRY => self.read_entry(body)?,
            CHANGE => self.read_change(body)?,
            END => self.read_end(body)?,
            found => return Err(Unreadable::Tag { found }),
        };
        self.place(&item, size)?;
        Ok(Next::Item(item, size))
    }

    /// An item of `size` bytes would pass the body's end.
    fn past(&self, size: u32) -> Unreadable {
        Unreadable::Length { length: self.length, after: self.taken.saturating_add(u64::from(size)) }
    }

    /// Checks the order of an item against the last, and takes its bytes.
    fn place(&mut self, item: &Item, size: u32) -> Result<(), Unreadable> {
        match item {
            Item::Entry(entry, _) => {
                self.after(&entry.path, b"path")?;
                self.open = true;
            }
            Item::Change { op: Op::Put, path } => {
                // A put stands for the entry just before it.
                let same = match &self.last {
                    Some(last) => self.open && last == path,
                    None => false,
                };
                if !same {
                    return Err(Unreadable::Order(b"change"));
                }
                self.open = false;
            }
            Item::Change { op: Op::Delete, path } => {
                self.after(path, b"change")?;
                self.open = false;
            }
            Item::End(_) => {
                self.phase = Phase::Ended;
                self.open = false;
            }
        }
        self.taken = self.taken.checked_add(u64::from(size)).expect("a bounded body fits in a u64");
        if self.phase == Phase::Ended && self.taken != self.length {
            return Err(Unreadable::Length { length: self.length, after: self.taken });
        }
        Ok(())
    }

    /// A path of the entries and the deletes comes strictly after the last.
    fn after(&mut self, path: &[u8], field: &'static [u8]) -> Result<(), Unreadable> {
        if let Some(last) = &self.last
            && tree_order(last, path) != Ordering::Less
        {
            return Err(Unreadable::Order(field));
        }
        self.last = Some(bytes::copy_of(path));
        Ok(())
    }

    fn read_entry(&self, body: &[u8]) -> Result<Item, Unreadable> {
        let mut f = Fields::new(body, &self.limits);
        f.tuple(b"entry", 2)?;
        let entry = read_entry(&mut f)?;
        f.tuple(b"local", 5)?;
        let inode = f.uint(b"inode")?;
        let ctime = (f.uint(b"ctime")?, f.id(b"ctime")?);
        let mtime = (f.uint(b"mtime")?, f.id(b"mtime")?);
        f.finish()?;
        Ok(Item::Entry(entry, Local { inode, ctime, mtime }))
    }

    fn read_change(&self, body: &[u8]) -> Result<Item, Unreadable> {
        let mut f = Fields::new(body, &self.limits);
        f.tuple(b"change", 2)?;
        let op = match f.uint(b"op")? {
            1 => Op::Put,
            2 => Op::Delete,
            _ => return Err(Unreadable::Shape(b"op")),
        };
        let path = read_path(&mut f, b"path")?;
        f.finish()?;
        Ok(Item::Change { op, path })
    }

    fn read_end(&self, body: &[u8]) -> Result<Item, Unreadable> {
        let mut f = Fields::new(body, &self.limits);
        let (mut boot, mut changes, mut counts, mut key, mut parent_tree, mut tree) =
            (None, None, None, None, None, None);
        for _ in 0..f.map(b"end")? {
            match f.key()?.as_ref() {
                b"boot" => boot = Some(read_boot(&mut f, &self.limits)?),
                b"changes" => changes = Some(f.digest(b"changes")?),
                b"counts" => counts = Some(layer::read_counts(&mut f)?),
                b"key" => key = Some(f.digest(b"key")?),
                b"parent_tree" => parent_tree = Some(f.digest_or_none(b"parent_tree")?),
                b"tree" => tree = Some(f.digest(b"tree")?),
                _ => return Err(Unreadable::Unknown),
            }
        }
        f.finish()?;
        Ok(Item::End(End {
            key: key.ok_or(Unreadable::Missing(b"key"))?,
            parent_tree: parent_tree.ok_or(Unreadable::Missing(b"parent_tree"))?,
            tree: tree.ok_or(Unreadable::Missing(b"tree"))?,
            changes: changes.ok_or(Unreadable::Missing(b"changes"))?,
            counts: counts.ok_or(Unreadable::Missing(b"counts"))?,
            boot: boot.ok_or(Unreadable::Missing(b"boot"))?,
        }))
    }
}

fn read_path(f: &mut Fields<'_>, field: &'static [u8]) -> Result<Box<[u8]>, Unreadable> {
    let path = f.bytes(field, PATH)?;
    if path_valid(&path) { Ok(path) } else { Err(Unreadable::Shape(field)) }
}
fn read_id(f: &mut Fields<'_>, field: &'static [u8], largest: u32) -> Result<u32, Unreadable> {
    let value = f.id(field)?;
    if value > largest { Err(Unreadable::Limit(field)) } else { Ok(value) }
}
fn read_entry(f: &mut Fields<'_>) -> Result<Entry, Unreadable> {
    f.tuple(b"entry", 7)?;
    let path = read_path(f, b"path")?;
    let kind = f.uint(b"kind")?;
    let mode = read_id(f, b"mode", MODE)?;
    let uid = read_id(f, b"uid", OWNER)?;
    let gid = read_id(f, b"gid", OWNER)?;
    let xattrs = read_xattrs(f)?;
    let payload = match kind {
        1 => {
            f.none(b"payload")?;
            Payload::Directory
        }
        2 => {
            f.tuple(b"payload", 2)?;
            Payload::File { size: f.uint(b"size")?, content: f.digest(b"content")? }
        }
        3 => Payload::Symlink { target: f.bytes(b"target", TARGET)? },
        4 => Payload::Hardlink { path: read_path(f, b"payload")? },
        5 | 6 => {
            f.tuple(b"payload", 2)?;
            let (major, minor) = (f.id(b"major")?, f.id(b"minor")?);
            if kind == 5 { Payload::Char { major, minor } } else { Payload::Block { major, minor } }
        }
        7 => {
            f.none(b"payload")?;
            Payload::Fifo
        }
        8 => {
            f.none(b"payload")?;
            Payload::Socket
        }
        _ => return Err(Unreadable::Shape(b"kind")),
    };
    Ok(Entry { path, mode, uid, gid, xattrs, payload })
}
fn read_xattrs(f: &mut Fields<'_>) -> Result<Box<[Xattr]>, Unreadable> {
    let count = f.map(b"xattrs")?;
    if count > u64::from(XATTRS) {
        return Err(Unreadable::Limit(b"xattrs"));
    }
    let mut result = List::with_capacity(u32::try_from(count).expect("the count is within a u32 limit"));
    for _ in 0..count {
        let name = f.key()?;
        let value = f.bytes(b"value", VALUE)?;
        result.push(Xattr { name, value }).expect("the checked count has room");
    }
    Ok(result.into_boxed())
}
fn read_boot(f: &mut Fields<'_>, limits: &Limits) -> Result<Box<[Kernel]>, Unreadable> {
    let count = f.list(b"boot", limits.tree.kernels)?;
    let mut result: List<Kernel> = List::with_capacity(count);
    for _ in 0..count {
        let (mut initramfs, mut kernel, mut modules, mut version) = (None, None, None, None);
        for _ in 0..f.map(b"kernel")? {
            match f.key()?.as_ref() {
                b"initramfs" => initramfs = Some(f.digest(b"initramfs")?),
                b"kernel" => kernel = Some(f.digest(b"kernel")?),
                b"modules" => modules = Some(f.digest(b"modules")?),
                b"version" => version = Some(f.bytes(b"version", VERSION)?),
                _ => return Err(Unreadable::Unknown),
            }
        }
        let kernel = Kernel {
            version: version.ok_or(Unreadable::Missing(b"version"))?,
            kernel: kernel.ok_or(Unreadable::Missing(b"kernel"))?,
            modules: modules.ok_or(Unreadable::Missing(b"modules"))?,
            initramfs: initramfs.ok_or(Unreadable::Missing(b"initramfs"))?,
        };
        if let Some(previous) = result.last()
            && previous.version >= kernel.version
        {
            return Err(Unreadable::Order(b"boot"));
        }
        result.push(kernel).expect("the checked count has room");
    }
    Ok(result.into_boxed())
}
