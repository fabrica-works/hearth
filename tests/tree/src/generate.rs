//! Valid records generated from injected randomness, framed by their writers.
use crate::{check::Record, oracle};
use hearth_lib::encode::{Digest, Item};
use hearth_protocol_tree::{
    cache_file::{CacheFile, Origin},
    layer::{self, Cost, Counts, How, Layer, LayerKind, Step, Subvolume, Tool, TreeFile},
    limits::Limits,
    resolution::{Costs, Index, Inputs, Line, Made, Package, Resolution},
    store::{Range, Store},
    tree::{End, Entry, Item as TreeItem, Kernel, Local, Op, Payload, Xattr},
};
use skein_lib::{Duration, Rng, Wall};
use std::collections::BTreeSet;

fn digest(rng: &mut Rng) -> Digest {
    let mut value = [0; 32];
    for word in value.chunks_exact_mut(8) {
        word.copy_from_slice(&rng.next_u64().to_be_bytes());
    }
    Digest(value)
}
fn bytes(rng: &mut Rng, maximum: u32) -> Box<[u8]> {
    let length = usize::try_from(rng.between(0, u64::from(maximum.min(64)))).expect("small length");
    (0..length).map(|_| u8::try_from(rng.below(256)).expect("a byte")).collect()
}
fn text(value: &[u8]) -> Item {
    Item::Bytes(value.into())
}

#[must_use]
pub fn small(rng: &mut Rng, limits: &Limits) -> Record {
    if rng.chance(500) {
        Record::Store(Store {
            layout: 1,
            made: Wall::from_nanos(rng.next_u64()),
            range: rng.chance(500).then_some(Range { base: 100_000, count: 65_536 }),
            uid: u32::try_from(rng.below(u64::from(u32::MAX))).expect("a uid"),
        })
    } else {
        cache_file(rng, limits)
    }
}
fn cache_file(rng: &mut Rng, limits: &Limits) -> Record {
    let origin = match rng.below(4) {
        0 => Origin::Fetch { uri: bytes(rng, limits.cache_file.path) },
        1 => Origin::Index { resolution: digest(rng) },
        2 => Origin::Blob { oid: bytes(rng, limits.cache_file.oid) },
        _ => Origin::Rendered { path: bytes(rng, limits.cache_file.path) },
    };
    Record::CacheFile(CacheFile {
        at: Wall::from_nanos(rng.next_u64()),
        origin,
        sha256: digest(rng),
        size: rng.next_u64(),
        took: Duration::from_nanos(rng.next_u64()),
    })
}

#[must_use]
pub fn one_of_each(rng: &mut Rng, limits: &Limits) -> [Record; 5] {
    let store = Store { layout: 1, made: Wall::from_nanos(rng.next_u64()), range: None, uid: 1_000 };
    let packages = u32::try_from(rng.between(0, 64)).expect("small closure");
    [
        Record::Store(store),
        Record::Layer(Box::new(layer(rng, limits))),
        Record::Tree(tree_file(rng, 12).into()),
        cache_file(rng, limits),
        Record::Resolution(resolution(rng, limits, packages)),
    ]
}

fn uuid(rng: &mut Rng) -> [u8; 16] {
    let mut value = [0; 16];
    for word in value.chunks_exact_mut(8) {
        word.copy_from_slice(&rng.next_u64().to_be_bytes());
    }
    value
}
/// A name that sorts after every name made with a smaller `number`.
fn ordered(rng: &mut Rng, number: u64, maximum: u32) -> Box<[u8]> {
    let mut name = format!("{number:04}").into_bytes();
    name.extend_from_slice(&bytes(rng, maximum.saturating_sub(4)));
    name.into_boxed_slice()
}
/// A layer record of a random kind, its lists a few items long.
#[must_use]
pub fn layer(rng: &mut Rng, limits: &Limits) -> Layer {
    let l = limits.layer;
    let kind = match rng.below(4) {
        0 => LayerKind::Empty,
        1 => LayerKind::Packages,
        2 => LayerKind::Software,
        _ => LayerKind::Config,
    };
    let part = digest(rng);
    let (inputs, packages) = match kind {
        LayerKind::Empty => (layer::Inputs::Empty, None),
        LayerKind::Packages => (
            layer::Inputs::Packages { part, resolution: digest(rng) },
            Some(layer::Line {
                base: rng.chance(500).then(|| digest(rng)),
                depth: rng.next_u64(),
                how: if rng.chance(500) { How::Scratch } else { How::Reconcile },
                line: Wall::from_nanos(rng.next_u64()),
            }),
        ),
        LayerKind::Software => (layer::Inputs::Software { part }, None),
        LayerKind::Config => (layer::Inputs::Config { part }, None),
    };
    let tools = (0..rng.between(0, 8))
        .map(|number| Tool { name: ordered(rng, number, l.name), version: bytes(rng, l.name) })
        .collect();
    let costs =
        (0..rng.between(0, 6)).map(|number| Step { name: ordered(rng, number, limits.key), cost: cost(rng) }).collect();
    let placed = (kind == LayerKind::Software)
        .then(|| (0..rng.between(0, 64)).map(|_| digest(rng)).collect::<Vec<_>>().into_boxed_slice());
    Layer {
        changes: digest(rng),
        costs,
        counts: Counts {
            bytes: rng.next_u64(),
            deletes: rng.next_u64(),
            entries: rng.next_u64(),
            files: rng.next_u64(),
            hashed: rng.next_u64(),
            puts: rng.next_u64(),
        },
        inputs,
        key: digest(rng),
        kind,
        made: layer::Made { at: Wall::from_nanos(rng.next_u64()), packages },
        parent: rng.chance(500).then(|| digest(rng)),
        placed,
        scan: (kind != LayerKind::Empty).then(|| digest(rng)),
        subvolume: Subvolume { parent_uuid: rng.chance(500).then(|| uuid(rng)), uuid: uuid(rng) },
        tools,
        tree: digest(rng),
        tree_file: TreeFile { sha256: digest(rng), size: rng.next_u64() },
    }
}
fn cost(rng: &mut Rng) -> Cost {
    Cost {
        cpu: Duration::from_nanos(rng.next_u64()),
        fetched: rng.next_u64(),
        hashed: rng.next_u64(),
        memory: rng.next_u64(),
        wall: Duration::from_nanos(rng.next_u64()),
        written: rng.next_u64(),
    }
}

/// A software layer with every list full and every name at its limit.
#[must_use]
pub fn layer_at_its_limits(limits: &Limits) -> Layer {
    let l = limits.layer;
    let mut rng = Rng::new(0x7EE0_FFFE);
    let mut record = layer(&mut rng, limits);
    let name = |number: u64, longest: u32| {
        let mut name = vec![b'n'; usize::try_from(longest).expect("name fits memory")];
        name[..8].copy_from_slice(&number.to_be_bytes());
        name.into_boxed_slice()
    };
    record.kind = LayerKind::Software;
    record.inputs = layer::Inputs::Software { part: digest(&mut rng) };
    record.made.packages = None;
    record.scan = Some(digest(&mut rng));
    record.placed = Some((0..l.placed).map(|_| digest(&mut rng)).collect());
    record.tools = (0..l.tools)
        .map(|number| Tool {
            name: name(u64::from(number), l.name),
            version: vec![b'v'; usize::try_from(l.name).expect("name fits memory")].into(),
        })
        .collect();
    record.costs =
        (0..l.steps).map(|number| Step { name: name(u64::from(number), limits.key), cost: cost(&mut rng) }).collect();
    record
}

#[must_use]
pub fn resolution(rng: &mut Rng, limits: &Limits, packages: u32) -> Resolution {
    let l = limits.resolution;
    let lines = u32::try_from(rng.between(1, u64::from(l.lines.min(3)))).expect("small sources");
    let closure = (0..packages.min(l.packages))
        .map(|number| Package {
            name: format!("p{number:010}").into_bytes().into_boxed_slice(),
            architecture: b"amd64".as_slice().into(),
            version: bytes(rng, l.name),
            source: u32::try_from(rng.below(u64::from(lines))).expect("line index"),
            filename: bytes(rng, l.path),
            size: rng.next_u64(),
            sha256: digest(rng),
        })
        .collect();
    let sources = (0..lines).map(|_| line(rng, limits)).collect();
    let apt =
        [Item::Map(1), text(b"options"), Item::List(3), Item::None, Item::True, Item::Uint(rng.next_u64())].into();
    Resolution {
        closure,
        digest: digest(rng),
        key: digest(rng),
        sources,
        inputs: Inputs {
            apt,
            request: [Item::List(0)].into(),
            sources: [Item::Map(1), text(b"suite"), text(b"forky")].into(),
            variant: [text(b"apt")].into(),
        },
        made: Made {
            apt: bytes(rng, l.name),
            at: Wall::from_nanos(rng.next_u64()),
            costs: Costs { fetched: rng.next_u64(), took: Duration::from_nanos(rng.next_u64()) },
            mmdebstrap: bytes(rng, l.name),
        },
    }
}
fn line(rng: &mut Rng, limits: &Limits) -> Line {
    let l = limits.resolution;
    let count = rng.between(0, u64::from(l.indices.min(3)));
    Line {
        uri: bytes(rng, l.path),
        suite: bytes(rng, l.name),
        components: [b"main".as_slice().into()].into(),
        indices: (0..count)
            .map(|_| Index { path: bytes(rng, l.path), size: rng.next_u64(), sha256: digest(rng) })
            .collect(),
    }
}

/// All packages, lines and indices, with filenames filling the body's bound.
#[must_use]
pub fn resolution_at_its_limits(limits: &Limits) -> Resolution {
    let mut rng = Rng::new(0x7EE0_FFFF);
    let mut record = resolution(&mut rng, limits, 0);
    record.closure = (0..limits.resolution.packages)
        .map(|number| Package {
            name: format!("p{number:010}").into_bytes().into_boxed_slice(),
            architecture: b"amd64".as_slice().into(),
            version: b"1".as_slice().into(),
            source: 0,
            filename: Box::new([]),
            size: 42,
            sha256: Digest([3; 32]),
        })
        .collect();
    let l = limits.resolution;
    record.sources = (0..l.lines)
        .map(|_| {
            let mut source = line(&mut rng, limits);
            source.indices = (0..l.indices)
                .map(|_| Index { path: b"InRelease".as_slice().into(), size: 42, sha256: digest(&mut rng) })
                .collect();
            source
        })
        .collect();
    let frame = hearth_protocol_tree::resolution::write(&record, limits).expect("the initial record fits");
    let target = usize::try_from(l.body).expect("body fits memory") - 512;
    let mut extra = target - (frame.len() - 24);
    for package in &mut record.closure {
        let grow = extra.min(usize::try_from(l.path).expect("path fits memory") - package.filename.len());
        package.filename = vec![b'p'; package.filename.len() + grow].into_boxed_slice();
        extra -= grow;
    }
    assert_eq!(extra, 0, "filenames fill the body to within 1 KiB");
    record
}

/// Names that sort differently as a component than as bytes: `/` is between
/// `.` and `0`, and a space, `-` and `.` come before it.
const NAMES: &[&[u8]] = &[
    b"a",
    b"b",
    b"a-b",
    b"a b",
    b"a.b",
    b"a_b",
    b"ab",
    b"a0",
    b"-",
    b".a",
    b"a.",
    b"..a",
    b"A",
    b"~",
    b"z",
    b"\xc3\xa9",
    b"\xff",
    b"0",
];
fn name(rng: &mut Rng) -> Vec<u8> {
    if rng.chance(300) {
        let length = rng.between(1, 3);
        (0..length).map(|_| b'a' + u8::try_from(rng.below(3)).expect("a letter")).collect()
    } else {
        NAMES[usize::try_from(rng.below(NAMES.len() as u64)).expect("an index")].to_vec()
    }
}
fn join(directory: &[u8], name: &[u8]) -> Vec<u8> {
    if directory.is_empty() { name.to_vec() } else { [directory, b"/", name].concat() }
}
fn id(rng: &mut Rng) -> u32 {
    if rng.chance(700) { 0 } else { u32::try_from(rng.below(65_536)).expect("an owner") }
}
fn xattrs(rng: &mut Rng, limits: &Limits) -> Box<[Xattr]> {
    let names: [&[u8]; 5] =
        [b"security.capability", b"system.posix_acl_access", b"system.posix_acl_default", b"user.a", b"user.b"];
    let count = if rng.chance(700) { 0 } else { rng.between(1, 4) };
    let mut chosen = BTreeSet::new();
    for _ in 0..count {
        chosen.insert(names[usize::try_from(rng.below(5)).expect("an index")]);
    }
    if rng.chance(50) {
        chosen.insert(&[b'u'; 255][..]);
    }
    chosen
        .into_iter()
        .map(|name| {
            assert!(name.len() <= usize::try_from(limits.tree.key).expect("small"), "a name within the key bound");
            Xattr { name: name.into(), value: bytes(rng, 65_536) }
        })
        .collect()
}
fn local(rng: &mut Rng) -> Local {
    Local {
        inode: rng.next_u64(),
        ctime: (rng.next_u64(), u32::try_from(rng.below(1_000_000_000)).expect("nanoseconds")),
        mtime: (rng.next_u64(), u32::try_from(rng.below(1_000_000_000)).expect("nanoseconds")),
    }
}

fn device(rng: &mut Rng) -> (u32, u32) {
    (u32::try_from(rng.below(256)).expect("small"), u32::try_from(rng.below(1 << 20)).expect("small"))
}
/// A leaf's payload: a hard link names one of the earlier `files`.
fn payload(rng: &mut Rng, files: &[Vec<u8>]) -> Payload {
    match rng.below(8) {
        0 => Payload::Directory,
        1..=3 => Payload::File { size: rng.below(1 << 20), content: digest(rng) },
        4 => Payload::Symlink { target: bytes(rng, 64) },
        5 if !files.is_empty() => {
            let at = usize::try_from(rng.below(files.len() as u64)).expect("an index");
            Payload::Hardlink { path: files[at].clone().into() }
        }
        5 => Payload::Fifo,
        6 => {
            let (major, minor) = device(rng);
            Payload::Char { major, minor }
        }
        _ if rng.chance(500) => Payload::Socket,
        _ => {
            let (major, minor) = device(rng);
            Payload::Block { major, minor }
        }
    }
}

/// A tree file's items for about `size` entries: paths of every shape, the
/// eight kinds, hard links to earlier files, puts after some entries and
/// deletes between them, and an end item whose digests the oracle made.
#[must_use]
pub fn tree_file(rng: &mut Rng, size: u64) -> Vec<TreeItem> {
    let limits = &crate::fixtures::limits();
    let mut paths = BTreeSet::from([Vec::new()]);
    let mut directories = vec![Vec::new()];
    for _ in 0..size.saturating_mul(4) {
        if paths.len() as u64 >= size.max(1) {
            break;
        }
        let parent = directories[usize::try_from(rng.below(directories.len() as u64)).expect("an index")].clone();
        let path = join(&parent, &name(rng));
        if paths.insert(path.clone()) && rng.chance(350) {
            directories.push(path);
        }
    }
    let mut ordered: Vec<Vec<u8>> = paths.iter().cloned().collect();
    ordered.sort_by(|a, b| oracle::order(a, b));
    let mut gone = Vec::new();
    for number in 0..rng.between(0, 3) {
        let parent = directories[usize::try_from(rng.below(directories.len() as u64)).expect("an index")].clone();
        let path = join(&parent, format!("~gone{number}").as_bytes());
        for child in 0..rng.between(0, 2) {
            gone.push(join(&path, format!("c{child}").as_bytes()));
        }
        gone.push(path);
    }
    let mut slots: Vec<(Vec<u8>, bool)> = ordered.iter().map(|path| (path.clone(), true)).collect();
    slots.extend(gone.iter().map(|path| (path.clone(), false)));
    slots.sort_by(|a, b| oracle::order(&a.0, &b.0));
    let mut items = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut changes: Vec<(Op, Vec<u8>, Option<usize>)> = Vec::new();
    let mut files: Vec<Vec<u8>> = Vec::new();
    let (mut bytes_total, mut file_count) = (0_u64, 0_u64);
    for (path, is_entry) in slots {
        if !is_entry {
            items.push(TreeItem::Change { op: Op::Delete, path: path.clone().into() });
            changes.push((Op::Delete, path, None));
            continue;
        }
        let has_children = directories.contains(&path)
            && ordered.iter().any(|other| {
                other.len() > path.len() && other.starts_with(&path) && (path.is_empty() || other[path.len()] == b'/')
            });
        let payload = if has_children || path.is_empty() { Payload::Directory } else { payload(rng, &files) };
        if let Payload::File { size, .. } = &payload {
            files.push(path.clone());
            bytes_total = bytes_total.saturating_add(*size);
            file_count += 1;
        }
        let entry = Entry {
            path: path.clone().into(),
            mode: u32::try_from(rng.below(0o10000)).expect("a mode"),
            uid: id(rng),
            gid: id(rng),
            xattrs: xattrs(rng, limits),
            payload,
        };
        items.push(TreeItem::Entry(entry.clone(), local(rng)));
        entries.push(entry);
        if rng.chance(350) {
            items.push(TreeItem::Change { op: Op::Put, path: path.clone().into() });
            changes.push((Op::Put, path, Some(entries.len() - 1)));
        }
    }
    let tree_refs: Vec<&Entry> = entries.iter().collect();
    let tree = oracle::tree_digest(&tree_refs);
    let parent_tree = rng.chance(500).then(|| digest(rng));
    let listed: Vec<(Op, &[u8], Option<&Entry>)> =
        changes.iter().map(|(op, path, at)| (*op, path.as_slice(), at.map(|at| &entries[at]))).collect();
    let changes_digest = oracle::changes_digest(parent_tree, tree, &listed);
    let mut version = 0_u64;
    let boot = (0..rng.between(0, 3))
        .map(|_| {
            version += rng.between(1, 9);
            Kernel {
                version: format!("6.{version:03}.0-amd64").into_bytes().into(),
                kernel: digest(rng),
                modules: digest(rng),
                initramfs: digest(rng),
            }
        })
        .collect();
    items.push(TreeItem::End(End {
        key: digest(rng),
        parent_tree,
        tree,
        changes: changes_digest,
        counts: Counts {
            bytes: bytes_total,
            deletes: changes.iter().filter(|change| change.0 == Op::Delete).count() as u64,
            entries: entries.len() as u64,
            files: file_count,
            hashed: rng.below(bytes_total.max(1)),
            puts: changes.iter().filter(|change| change.0 == Op::Put).count() as u64,
        },
        boot,
    }));
    items
}

/// The longest entry: path, target and 64 attributes, each name and value at
/// its limit.
#[must_use]
pub fn entry_at_its_limits(limits: &Limits) -> TreeItem {
    let component = vec![b'p'; 255];
    let path = vec![component.as_slice(); 16].join(&b'/');
    assert_eq!(path.len(), 4_095, "sixteen components of 255 bytes");
    let xattrs = (0..64_u32)
        .map(|number| {
            let mut name = vec![b'n'; usize::try_from(limits.tree.key).expect("small")];
            name[..4].copy_from_slice(&number.to_be_bytes());
            Xattr { name: name.into(), value: vec![b'v'; 65_536].into() }
        })
        .collect();
    TreeItem::Entry(
        Entry {
            path: path.into(),
            mode: 0o7777,
            uid: 65_535,
            gid: 65_535,
            xattrs,
            payload: Payload::Symlink { target: vec![b't'; 4_096].into() },
        },
        Local { inode: u64::MAX, ctime: (u64::MAX, 999_999_999), mtime: (u64::MAX, 999_999_999) },
    )
}
/// The end item with all its kernels, each version at its limit.
#[must_use]
pub fn end_at_its_limits(limits: &Limits) -> TreeItem {
    let mut rng = Rng::new(0x7EE0_FFFD);
    let boot = (0..limits.tree.kernels)
        .map(|number| {
            let mut version = vec![b'v'; 255];
            version[..4].copy_from_slice(&number.to_be_bytes());
            Kernel {
                version: version.into(),
                kernel: digest(&mut rng),
                modules: digest(&mut rng),
                initramfs: digest(&mut rng),
            }
        })
        .collect();
    TreeItem::End(End {
        key: digest(&mut rng),
        parent_tree: Some(digest(&mut rng)),
        tree: digest(&mut rng),
        changes: digest(&mut rng),
        counts: Counts {
            bytes: u64::MAX,
            deletes: u64::MAX,
            entries: u64::MAX,
            files: u64::MAX,
            hashed: u64::MAX,
            puts: u64::MAX,
        },
        boot,
    })
}
