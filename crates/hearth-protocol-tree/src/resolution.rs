//! Resolution records, kind 4 at version 1 (store.md, 6.4; render.md, 5.4).

use crate::{
    fields::Fields,
    frame::{self, Kind, Refused, Unreadable},
    limits::Limits,
};
use alloc::boxed::Box;
use hearth_lib::encode::{Digest, Encoder, Item};
use skein_lib::{Duration, List, Stack, Wall, bytes};

/// A resolution, with inputs kept item for item and digests as written.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolution {
    pub closure: Box<[Package]>,
    pub digest: Digest,
    pub inputs: Inputs,
    pub key: Digest,
    pub made: Made,
    pub sources: Box<[Line]>,
}
/// The key's four values, as the packages part holds them (render.md, 5.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inputs {
    pub apt: Box<[Item]>,
    pub request: Box<[Item]>,
    pub sources: Box<[Item]>,
    pub variant: Box<[Item]>,
}
/// A source line, in the sources' order, with the indices kept in the cache.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub uri: Box<[u8]>,
    pub suite: Box<[u8]>,
    pub components: Box<[Box<[u8]>]>,
    pub indices: Box<[Index]>,
}
/// An index's path relative to the suite, its size, and its SHA-256.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Index {
    pub path: Box<[u8]>,
    pub size: u64,
    pub sha256: Digest,
}
/// A closure package; `source` is its source line's index.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Package {
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub version: Box<[u8]>,
    pub source: u32,
    pub filename: Box<[u8]>,
    pub size: u64,
    pub sha256: Digest,
}
/// How this record was made, outside its key and digest.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Made {
    pub apt: Box<[u8]>,
    pub at: Wall,
    pub costs: Costs,
    pub mmdebstrap: Box<[u8]>,
}
/// Bytes fetched and the run's duration.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Costs {
    pub fetched: u64,
    pub took: Duration,
}

pub fn read(bytes: &[u8], limits: &Limits) -> Result<Resolution, Unreadable> {
    let mut f = Fields::new(frame::body(bytes, Kind::Resolution, limits)?, limits);
    let (mut closure, mut digest, mut inputs, mut key, mut made, mut sources) = (None, None, None, None, None, None);
    for _ in 0..f.map(b"")? {
        match f.key()?.as_ref() {
            b"closure" => closure = Some(read_closure(&mut f, limits)?),
            b"digest" => digest = Some(f.digest(b"digest")?),
            b"inputs" => inputs = Some(read_inputs(&mut f, limits)?),
            b"key" => key = Some(f.digest(b"key")?),
            b"made" => made = Some(read_made(&mut f, limits)?),
            b"sources" => sources = Some(read_sources(&mut f, limits)?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    f.finish()?;
    let record = Resolution {
        closure: closure.ok_or(Unreadable::Missing(b"closure"))?,
        digest: digest.ok_or(Unreadable::Missing(b"digest"))?,
        inputs: inputs.ok_or(Unreadable::Missing(b"inputs"))?,
        key: key.ok_or(Unreadable::Missing(b"key"))?,
        made: made.ok_or(Unreadable::Missing(b"made"))?,
        sources: sources.ok_or(Unreadable::Missing(b"sources"))?,
    };
    for package in &record.closure {
        if usize::try_from(package.source).expect("a u32 fits in memory") >= record.sources.len() {
            return Err(Unreadable::Limit(b"source"));
        }
    }
    Ok(record)
}
fn read_closure(f: &mut Fields<'_>, limits: &Limits) -> Result<Box<[Package]>, Unreadable> {
    let count = f.list(b"closure", limits.resolution.packages)?;
    let mut result: List<Package> = List::with_capacity(count);
    for _ in 0..count {
        f.tuple(b"package", 7)?;
        let package = Package {
            name: f.bytes(b"name", limits.resolution.name)?,
            architecture: f.bytes(b"architecture", limits.resolution.name)?,
            version: f.bytes(b"version", limits.resolution.name)?,
            source: f.id(b"source")?,
            filename: f.bytes(b"filename", limits.resolution.path)?,
            size: f.uint(b"size")?,
            sha256: f.digest(b"sha256")?,
        };
        if let Some(previous) = result.last()
            && !above(previous, &package)
        {
            return Err(Unreadable::Order(b"closure"));
        }
        result.push(package).expect("the checked count has room");
    }
    Ok(result.into_boxed())
}
fn above(previous: &Package, package: &Package) -> bool {
    previous.name < package.name || (previous.name == package.name && previous.architecture < package.architecture)
}
fn read_inputs(f: &mut Fields<'_>, limits: &Limits) -> Result<Inputs, Unreadable> {
    let (mut apt, mut request, mut sources, mut variant) = (None, None, None, None);
    let mut left = limits.resolution.inputs;
    for _ in 0..f.map(b"inputs")? {
        let name: &'static [u8] = match f.key()?.as_ref() {
            b"apt" => b"apt",
            b"request" => b"request",
            b"sources" => b"sources",
            b"variant" => b"variant",
            _ => return Err(Unreadable::Unknown),
        };
        let value = f.value(name, left, u32::MAX)?;
        let shaped = match value.first() {
            Some(Item::Map(_)) => name == b"apt" || name == b"sources",
            Some(Item::List(_)) => name == b"request",
            Some(Item::Bytes(_)) => name == b"variant",
            Some(Item::None | Item::False | Item::True | Item::Uint(_) | Item::Digest(_)) | None => false,
        };
        if !shaped {
            return Err(Unreadable::Shape(name));
        }
        left = left.saturating_sub(u32::try_from(value.len()).expect("input items fit their u32 limit"));
        match input(&value, name, limits) {
            Ok(()) => {}
            Err(Refused::Limit(field)) => return Err(Unreadable::Limit(field)),
            Err(Refused::Order(field)) => return Err(Unreadable::Order(field)),
            Err(Refused::Bound { .. }) => unreachable!("input checks no frame bound"),
        }
        match name {
            b"apt" => apt = Some(value),
            b"request" => request = Some(value),
            b"sources" => sources = Some(value),
            b"variant" => variant = Some(value),
            _ => unreachable!("the input field name was checked"),
        }
    }
    Ok(Inputs {
        apt: apt.ok_or(Unreadable::Missing(b"apt"))?,
        request: request.ok_or(Unreadable::Missing(b"request"))?,
        sources: sources.ok_or(Unreadable::Missing(b"sources"))?,
        variant: variant.ok_or(Unreadable::Missing(b"variant"))?,
    })
}
fn input(value: &[Item], name: &'static [u8], limits: &Limits) -> Result<(), Refused> {
    let Some(first) = value.first() else {
        return Err(Refused::Limit(name));
    };
    match first {
        Item::Map(_) if name == b"apt" || name == b"sources" => Ok(()),
        Item::Bytes(text) if name == b"variant" && value.len() == 1 => length(text.len(), limits.resolution.name, name),
        Item::List(count) if name == b"request" => {
            if *count > u64::from(limits.resolution.packages)
                || usize::try_from(*count).ok() != value.len().checked_sub(1)
            {
                return Err(Refused::Limit(name));
            }
            let mut previous: Option<&[u8]> = None;
            for item in value.get(1..).expect("the head exists") {
                match item {
                    Item::Bytes(text) => {
                        length(text.len(), limits.resolution.name, b"request")?;
                        if let Some(last) = previous
                            && last >= text.as_ref()
                        {
                            return Err(Refused::Order(b"request"));
                        }
                        previous = Some(text);
                    }
                    Item::None
                    | Item::False
                    | Item::True
                    | Item::Uint(_)
                    | Item::Digest(_)
                    | Item::Map(_)
                    | Item::List(_) => return Err(Refused::Limit(name)),
                }
            }
            Ok(())
        }
        Item::None
        | Item::False
        | Item::True
        | Item::Uint(_)
        | Item::Digest(_)
        | Item::Map(_)
        | Item::List(_)
        | Item::Bytes(_) => Err(Refused::Limit(name)),
    }
}
fn read_sources(f: &mut Fields<'_>, limits: &Limits) -> Result<Box<[Line]>, Unreadable> {
    let count = f.list(b"sources", limits.resolution.lines)?;
    let mut lines = List::with_capacity(count);
    for _ in 0..count {
        f.tuple(b"line", 4)?;
        let uri = f.bytes(b"uri", limits.resolution.path)?;
        let suite = f.bytes(b"suite", limits.resolution.name)?;
        let count = f.list(b"components", limits.resolution.inputs)?;
        let mut components = List::with_capacity(count);
        for _ in 0..count {
            components.push(f.bytes(b"component", limits.resolution.name)?).expect("the checked count has room");
        }
        let count = f.list(b"indices", limits.resolution.indices)?;
        let mut indices = List::with_capacity(count);
        for _ in 0..count {
            f.tuple(b"index", 3)?;
            let index = Index {
                path: f.bytes(b"path", limits.resolution.path)?,
                size: f.uint(b"size")?,
                sha256: f.digest(b"sha256")?,
            };
            indices.push(index).expect("the checked count has room");
        }
        lines
            .push(Line { uri, suite, components: components.into_boxed(), indices: indices.into_boxed() })
            .expect("the checked count has room");
    }
    Ok(lines.into_boxed())
}
fn read_made(f: &mut Fields<'_>, limits: &Limits) -> Result<Made, Unreadable> {
    let (mut apt, mut at, mut costs, mut mmdebstrap) = (None, None, None, None);
    for _ in 0..f.map(b"made")? {
        match f.key()?.as_ref() {
            b"apt" => apt = Some(f.bytes(b"apt", limits.resolution.name)?),
            b"at" => at = Some(f.wall(b"at")?),
            b"costs" => costs = Some(read_costs(f)?),
            b"mmdebstrap" => mmdebstrap = Some(f.bytes(b"mmdebstrap", limits.resolution.name)?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(Made {
        apt: apt.ok_or(Unreadable::Missing(b"apt"))?,
        at: at.ok_or(Unreadable::Missing(b"at"))?,
        costs: costs.ok_or(Unreadable::Missing(b"costs"))?,
        mmdebstrap: mmdebstrap.ok_or(Unreadable::Missing(b"mmdebstrap"))?,
    })
}
fn read_costs(f: &mut Fields<'_>) -> Result<Costs, Unreadable> {
    let (mut fetched, mut took) = (None, None);
    for _ in 0..f.map(b"costs")? {
        match f.key()?.as_ref() {
            b"fetched" => fetched = Some(f.uint(b"fetched")?),
            b"took" => took = Some(f.duration(b"took")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(Costs {
        fetched: fetched.ok_or(Unreadable::Missing(b"fetched"))?,
        took: took.ok_or(Unreadable::Missing(b"took"))?,
    })
}
fn length(found: usize, limit: u32, name: &'static [u8]) -> Result<(), Refused> {
    if found > usize::try_from(limit).expect("a u32 fits in memory") { Err(Refused::Limit(name)) } else { Ok(()) }
}

pub fn write(record: &Resolution, limits: &Limits) -> Result<Box<[u8]>, Refused> {
    validate(record, limits)?;
    let mut measure = Encoder::measuring();
    encode(record, &mut measure);
    let mut writer = Encoder::writing(measure.measured());
    encode(record, &mut writer);
    frame::write(Kind::Resolution, &writer.written(), limits)
}
fn validate(record: &Resolution, limits: &Limits) -> Result<(), Refused> {
    let l = limits.resolution;
    if limits.key < 10 {
        return Err(Refused::Limit(b"key"));
    }
    let mut depth = 3;
    for line in &record.sources {
        depth = if line.indices.is_empty() { depth.max(4) } else { 5 };
    }
    if limits.depth < depth {
        return Err(Refused::Limit(b"depth"));
    }
    length(record.closure.len(), l.packages, b"closure")?;
    length(record.sources.len(), l.lines, b"sources")?;
    let mut previous = None;
    for package in &record.closure {
        length(package.name.len(), l.name, b"name")?;
        length(package.architecture.len(), l.name, b"architecture")?;
        length(package.version.len(), l.name, b"version")?;
        length(package.filename.len(), l.path, b"filename")?;
        if let Some(last) = previous
            && !above(last, package)
        {
            return Err(Refused::Order(b"closure"));
        }
        previous = Some(package);
        if usize::try_from(package.source).expect("a u32 fits in memory") >= record.sources.len() {
            return Err(Refused::Limit(b"source"));
        }
    }
    for line in &record.sources {
        length(line.uri.len(), l.path, b"uri")?;
        length(line.suite.len(), l.name, b"suite")?;
        length(line.components.len(), l.inputs, b"components")?;
        for component in &line.components {
            length(component.len(), l.name, b"component")?;
        }
        length(line.indices.len(), l.indices, b"indices")?;
        for index in &line.indices {
            length(index.path.len(), l.path, b"path")?;
        }
    }
    length(record.made.apt.len(), l.name, b"apt")?;
    length(record.made.mmdebstrap.len(), l.name, b"mmdebstrap")?;
    let mut left = l.inputs;
    for (name, value) in [
        (b"apt".as_slice(), &record.inputs.apt),
        (b"request", &record.inputs.request),
        (b"sources", &record.inputs.sources),
        (b"variant", &record.inputs.variant),
    ] {
        length(value.len(), left, name)?;
        left = left.saturating_sub(u32::try_from(value.len()).expect("the input fits its u32 limit"));
        input(value, name, limits)?;
        input_shape(value, name, limits)?;
    }
    Ok(())
}
#[derive(Debug)]
struct OpenInput {
    left: u64,
    map: bool,
    key: bool,
    last: Option<Box<[u8]>>,
}
fn input_shape(value: &[Item], name: &'static [u8], limits: &Limits) -> Result<(), Refused> {
    let mut stack: Stack<OpenInput> = Stack::with_capacity(limits.depth.saturating_sub(2).min(62));
    let mut placed = false;
    for item in value {
        let mut is_key = false;
        if let Some(top) = stack.top_mut() {
            is_key = top.map && top.key;
            if is_key {
                match item {
                    Item::Bytes(key) => {
                        length(key.len(), limits.key, name)?;
                        if let Some(last) = &top.last
                            && last >= key
                        {
                            return Err(Refused::Order(name));
                        }
                        top.last = Some(bytes::copy_of(key));
                    }
                    Item::None
                    | Item::False
                    | Item::True
                    | Item::Uint(_)
                    | Item::Digest(_)
                    | Item::List(_)
                    | Item::Map(_) => return Err(Refused::Limit(name)),
                }
            }
            top.left = top.left.checked_sub(1).ok_or(Refused::Limit(name))?;
            if top.map {
                top.key = !top.key;
            }
        } else if placed {
            return Err(Refused::Limit(name));
        } else {
            placed = true;
        }
        if !is_key {
            let open = match item {
                Item::List(count) => Some(OpenInput { left: *count, map: false, key: false, last: None }),
                Item::Map(count) => Some(OpenInput {
                    left: count.checked_mul(2).ok_or(Refused::Limit(name))?,
                    map: true,
                    key: true,
                    last: None,
                }),
                Item::None | Item::False | Item::True | Item::Uint(_) | Item::Digest(_) | Item::Bytes(_) => None,
            };
            if let Some(open) = open
                && stack.push(open).is_err()
            {
                return Err(Refused::Limit(name));
            }
        }
        for _ in 0..stack.capacity() {
            match stack.top() {
                Some(OpenInput { left: 0, .. }) => drop(stack.pop()),
                Some(OpenInput { .. }) | None => break,
            }
        }
    }
    if !placed || !stack.is_empty() {
        return Err(Refused::Limit(name));
    }
    Ok(())
}
pub(crate) fn encode(record: &Resolution, out: &mut Encoder) {
    out.map(6);
    out.key(b"closure");
    out.list(u64::try_from(record.closure.len()).expect("bounded packages"));
    for package in &record.closure {
        out.list(7);
        out.bytes(&package.name);
        out.bytes(&package.architecture);
        out.bytes(&package.version);
        out.uint(u64::from(package.source));
        out.bytes(&package.filename);
        out.uint(package.size);
        out.digest(&package.sha256);
    }
    out.key(b"digest");
    out.digest(&record.digest);
    out.key(b"inputs");
    out.map(4);
    for (name, value) in [
        (b"apt".as_slice(), &record.inputs.apt),
        (b"request", &record.inputs.request),
        (b"sources", &record.inputs.sources),
        (b"variant", &record.inputs.variant),
    ] {
        out.key(name);
        for item in value {
            out.item(item);
        }
    }
    out.key(b"key");
    out.digest(&record.key);
    out.key(b"made");
    out.map(4);
    out.key(b"apt");
    out.bytes(&record.made.apt);
    out.key(b"at");
    out.uint(record.made.at.as_nanos());
    out.key(b"costs");
    out.map(2);
    out.key(b"fetched");
    out.uint(record.made.costs.fetched);
    out.key(b"took");
    out.uint(record.made.costs.took.as_nanos());
    out.key(b"mmdebstrap");
    out.bytes(&record.made.mmdebstrap);
    out.key(b"sources");
    out.list(u64::try_from(record.sources.len()).expect("bounded lines"));
    for line in &record.sources {
        out.list(4);
        out.bytes(&line.uri);
        out.bytes(&line.suite);
        out.list(u64::try_from(line.components.len()).expect("bounded components"));
        for component in &line.components {
            out.bytes(component);
        }
        out.list(u64::try_from(line.indices.len()).expect("bounded indices"));
        for index in &line.indices {
            out.list(3);
            out.bytes(&index.path);
            out.uint(index.size);
            out.digest(&index.sha256);
        }
    }
}
