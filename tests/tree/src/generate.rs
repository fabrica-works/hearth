//! Valid records generated from injected randomness, framed by their writers.
use crate::check::Record;
use hearth_lib::encode::{Digest, Item};
use hearth_protocol_tree::{
    cache_file::{CacheFile, Origin},
    limits::Limits,
    resolution::{Costs, Index, Inputs, Line, Made, Package, Resolution},
    store::{Range, Store},
};
use skein_lib::{Duration, Rng, Wall};

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
pub fn one_of_each(rng: &mut Rng, limits: &Limits) -> [Record; 3] {
    let store = Store { layout: 1, made: Wall::from_nanos(rng.next_u64()), range: None, uid: 1_000 };
    let packages = u32::try_from(rng.between(0, 64)).expect("small closure");
    [Record::Store(store), cache_file(rng, limits), Record::Resolution(resolution(rng, limits, packages))]
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
