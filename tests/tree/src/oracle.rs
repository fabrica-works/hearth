//! A tree file's oracle, written from domain/store.md (3.1, 3.4, 3.5 and 6.3)
//! with nothing of the crate's codec in it: values as a plain tree encoded
//! recursively into bytes, paths ordered by splitting them, and the rules a
//! sequence of items must meet.
use hearth_lib::encode::Digest;
use hearth_protocol_tree::tree::{Entry, Item, Op, Payload};
use skein_lib::sha256;
use std::{cmp::Ordering, collections::BTreeMap};

/// A canonical value (3.1).
#[derive(Clone, Debug)]
pub enum Value {
    None,
    Uint(u64),
    Bytes(Vec<u8>),
    Digest([u8; 32]),
    List(Vec<Value>),
    Map(BTreeMap<Vec<u8>, Value>),
}

fn bytes(value: &[u8]) -> Value {
    Value::Bytes(value.to_vec())
}

/// The value's bytes: a tag, then big-endian counts and lengths (3.1).
#[must_use]
pub fn encode(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    put(value, &mut out);
    out
}
fn put(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::None => out.push(0),
        Value::Uint(number) => {
            out.push(3);
            out.extend_from_slice(&number.to_be_bytes());
        }
        Value::Bytes(text) => {
            out.push(4);
            out.extend_from_slice(&(text.len() as u64).to_be_bytes());
            out.extend_from_slice(text);
        }
        Value::Digest(digest) => {
            out.push(5);
            out.extend_from_slice(digest);
        }
        Value::List(values) => {
            out.push(6);
            out.extend_from_slice(&(values.len() as u64).to_be_bytes());
            for value in values {
                put(value, out);
            }
        }
        Value::Map(entries) => {
            out.push(7);
            out.extend_from_slice(&(entries.len() as u64).to_be_bytes());
            for (key, value) in entries {
                put(&bytes(key), out);
                put(value, out);
            }
        }
    }
}

/// H(tag, values…) (3.2).
#[must_use]
pub fn h(tag: &[u8], values: Vec<Value>) -> Digest {
    let mut all = vec![bytes(b"hearth/1"), bytes(tag)];
    all.extend(values);
    Digest(sha256::digest(&encode(&Value::List(all))))
}

fn digest(digest: &Digest) -> Value {
    Value::Digest(digest.0)
}
/// An entry as 3.4 lists it.
#[must_use]
pub fn entry_value(entry: &Entry) -> Value {
    let (kind, payload) = match &entry.payload {
        Payload::Directory => (1, Value::None),
        Payload::File { size, content } => (2, Value::List(vec![Value::Uint(*size), digest(content)])),
        Payload::Symlink { target } => (3, bytes(target)),
        Payload::Hardlink { path } => (4, bytes(path)),
        Payload::Char { major, minor } => {
            (5, Value::List(vec![Value::Uint(u64::from(*major)), Value::Uint(u64::from(*minor))]))
        }
        Payload::Block { major, minor } => {
            (6, Value::List(vec![Value::Uint(u64::from(*major)), Value::Uint(u64::from(*minor))]))
        }
        Payload::Fifo => (7, Value::None),
        Payload::Socket => (8, Value::None),
    };
    let xattrs = entry.xattrs.iter().map(|xattr| (xattr.name.to_vec(), bytes(&xattr.value))).collect();
    Value::List(vec![
        bytes(&entry.path),
        Value::Uint(kind),
        Value::Uint(u64::from(entry.mode)),
        Value::Uint(u64::from(entry.uid)),
        Value::Uint(u64::from(entry.gid)),
        Value::Map(xattrs),
        payload,
    ])
}
/// The tree digest of entries in tree order (3.4).
#[must_use]
pub fn tree_digest(entries: &[&Entry]) -> Digest {
    h(b"tree", vec![Value::List(entries.iter().map(|entry| entry_value(entry)).collect())])
}
/// The change set's digest (3.5): a put has the entry at its path.
#[must_use]
pub fn changes_digest(parent: Option<Digest>, tree: Digest, changes: &[(Op, &[u8], Option<&Entry>)]) -> Digest {
    let list = changes
        .iter()
        .map(|(op, path, entry)| {
            let number = match op {
                Op::Put => 1,
                Op::Delete => 2,
            };
            Value::List(vec![Value::Uint(number), bytes(path), entry.map_or(Value::None, entry_value)])
        })
        .collect();
    h(b"changes", vec![parent.map_or(Value::None, |parent| digest(&parent)), digest(&tree), Value::List(list)])
}

/// The bytes of an item (6.3): tag, length and value.
#[must_use]
pub fn item_bytes(item: &Item) -> Vec<u8> {
    let (tag, value) = match item {
        Item::Entry(entry, local) => (
            1,
            Value::List(vec![
                entry_value(entry),
                Value::List(vec![
                    Value::Uint(local.inode),
                    Value::Uint(local.ctime.0),
                    Value::Uint(u64::from(local.ctime.1)),
                    Value::Uint(local.mtime.0),
                    Value::Uint(u64::from(local.mtime.1)),
                ]),
            ]),
        ),
        Item::Change { op, path } => (
            2,
            Value::List(vec![
                Value::Uint(match op {
                    Op::Put => 1,
                    Op::Delete => 2,
                }),
                bytes(path),
            ]),
        ),
        Item::End(end) => {
            let boot = end
                .boot
                .iter()
                .map(|kernel| {
                    Value::Map(BTreeMap::from([
                        (b"version".to_vec(), bytes(&kernel.version)),
                        (b"kernel".to_vec(), digest(&kernel.kernel)),
                        (b"modules".to_vec(), digest(&kernel.modules)),
                        (b"initramfs".to_vec(), digest(&kernel.initramfs)),
                    ]))
                })
                .collect();
            let counts = BTreeMap::from([
                (b"bytes".to_vec(), Value::Uint(end.counts.bytes)),
                (b"deletes".to_vec(), Value::Uint(end.counts.deletes)),
                (b"entries".to_vec(), Value::Uint(end.counts.entries)),
                (b"files".to_vec(), Value::Uint(end.counts.files)),
                (b"hashed".to_vec(), Value::Uint(end.counts.hashed)),
                (b"puts".to_vec(), Value::Uint(end.counts.puts)),
            ]);
            (
                3,
                Value::Map(BTreeMap::from([
                    (b"key".to_vec(), digest(&end.key)),
                    (b"parent_tree".to_vec(), end.parent_tree.as_ref().map_or(Value::None, digest)),
                    (b"tree".to_vec(), digest(&end.tree)),
                    (b"changes".to_vec(), digest(&end.changes)),
                    (b"counts".to_vec(), Value::Map(counts)),
                    (b"boot".to_vec(), Value::List(boot)),
                ])),
            )
        }
    };
    let value = encode(&value);
    let mut out = vec![tag];
    out.extend_from_slice(&u32::try_from(value.len()).expect("a small item").to_be_bytes());
    out.extend_from_slice(&value);
    out
}

/// A path's components: the root's has none.
fn components(path: &[u8]) -> Vec<&[u8]> {
    if path.is_empty() { Vec::new() } else { path.split(|&byte| byte == b'/').collect() }
}
/// Tree order (3.4): component by component, each bytewise.
#[must_use]
pub fn order(a: &[u8], b: &[u8]) -> Ordering {
    components(a).cmp(&components(b))
}
/// A canonical path (3.4).
#[must_use]
pub fn valid(path: &[u8]) -> bool {
    path.len() <= 4_096
        && (path.is_empty()
            || path
                .split(|&byte| byte == b'/')
                .all(|part| !part.is_empty() && part != b"." && part != b".." && !part.contains(&0)))
}

/// Whether one item is writable and readable on its own: paths, owners, modes
/// and attributes within their bounds (3.4, 5.6).
#[must_use]
pub fn item_ok(item: &Item) -> bool {
    match item {
        Item::Entry(entry, _) => {
            let payload = match &entry.payload {
                Payload::Symlink { target } => target.len() <= 4_096,
                Payload::Hardlink { path } => valid(path),
                Payload::Directory
                | Payload::File { .. }
                | Payload::Char { .. }
                | Payload::Block { .. }
                | Payload::Fifo
                | Payload::Socket => true,
            };
            let ordered = entry.xattrs.windows(2).all(|pair| pair[0].name < pair[1].name);
            valid(&entry.path)
                && entry.mode <= 0o7777
                && entry.uid <= 65_535
                && entry.gid <= 65_535
                && entry.xattrs.len() <= 64
                && ordered
                && entry.xattrs.iter().all(|xattr| xattr.name.len() <= 255 && xattr.value.len() <= 65_536)
                && payload
        }
        Item::Change { path, .. } => valid(path),
        Item::End(end) => {
            end.boot.len() <= 8
                && end.boot.windows(2).all(|pair| pair[0].version < pair[1].version)
                && end.boot.iter().all(|kernel| kernel.version.len() <= 255)
        }
    }
}

/// Whether items are a tree file's (6.3): entries and deletes strictly in tree
/// order, a put right after its own path's entry, and the end item last.
#[must_use]
pub fn items_ok(items: &[Item]) -> bool {
    let mut last: Option<&[u8]> = None;
    let mut open = false;
    for (at, item) in items.iter().enumerate() {
        if !item_ok(item) {
            return false;
        }
        match item {
            Item::Entry(entry, _) => {
                if last.is_some_and(|last| order(last, &entry.path) != Ordering::Less) {
                    return false;
                }
                last = Some(&entry.path);
                open = true;
            }
            Item::Change { op: Op::Put, path } => {
                if !(open && last == Some(path)) {
                    return false;
                }
                open = false;
            }
            Item::Change { op: Op::Delete, path } => {
                if last.is_some_and(|last| order(last, path) != Ordering::Less) {
                    return false;
                }
                last = Some(path);
                open = false;
            }
            Item::End(_) => return at + 1 == items.len(),
        }
    }
    false
}
