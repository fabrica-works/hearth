//! Layer records, kind 2 at version 1 (store.md, 6.2).

use crate::{
    fields::Fields,
    frame::{self, Kind, Refused, Unreadable},
    limits::Limits,
};
use alloc::boxed::Box;
use hearth_lib::encode::{Digest, Encoder};
use skein_lib::{Duration, List, Wall};

/// A layer's record: what the store knows of it without its tree file.
/// `placed` is `Some` exactly for a software layer, `scan` is `None` exactly
/// for the empty one, and `inputs` and `made.packages` follow `kind`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub changes: Digest,
    pub costs: Box<[Step]>,
    pub counts: Counts,
    pub inputs: Inputs,
    pub key: Digest,
    pub kind: LayerKind,
    pub made: Made,
    pub parent: Option<Digest>,
    pub placed: Option<Box<[Digest]>>,
    pub scan: Option<Digest>,
    pub subvolume: Subvolume,
    pub tools: Box<[Tool]>,
    pub tree: Digest,
    pub tree_file: TreeFile,
}
/// A layer's kind, written as its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LayerKind {
    Empty,
    Packages,
    Software,
    Config,
}
/// The parts a layer's key is made of: a map of the kind's fields.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Inputs {
    Empty,
    Packages { part: Digest, resolution: Digest },
    Software { part: Digest },
    Config { part: Digest },
}
/// `<key>.tree`'s SHA-256 and size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TreeFile {
    pub sha256: Digest,
    pub size: u64,
}
/// The subvolume's uuids, as read after the snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Subvolume {
    pub parent_uuid: Option<[u8; 16]>,
    pub uuid: [u8; 16],
}
/// The tree's and the change set's counts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Counts {
    pub bytes: u64,
    pub deletes: u64,
    pub entries: u64,
    pub files: u64,
    pub hashed: u64,
    pub puts: u64,
}
/// A tool the layer's build ran, with its version.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tool {
    pub name: Box<[u8]>,
    pub version: Box<[u8]>,
}
/// A build step's name, a map key of `costs`, and its cost.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Step {
    pub name: Box<[u8]>,
    pub cost: Cost,
}
/// One step's cost: times in nanoseconds, memory and bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cost {
    pub cpu: Duration,
    pub fetched: u64,
    pub hashed: u64,
    pub memory: u64,
    pub wall: Duration,
    pub written: u64,
}
/// How this record was made: `by` is `built` in M2.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Made {
    pub at: Wall,
    pub packages: Option<Line>,
}
/// A packages layer's line of reconciles; `line` is the `at` of the last
/// scratch build in it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub base: Option<Digest>,
    pub depth: u64,
    pub how: How,
    pub line: Wall,
}
/// A packages layer's way of being made.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum How {
    Scratch,
    Reconcile,
}

impl LayerKind {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Empty => b"empty",
            Self::Packages => b"packages",
            Self::Software => b"software",
            Self::Config => b"config",
        }
    }
}
impl Inputs {
    const fn kind(&self) -> LayerKind {
        match self {
            Self::Empty => LayerKind::Empty,
            Self::Packages { .. } => LayerKind::Packages,
            Self::Software { .. } => LayerKind::Software,
            Self::Config { .. } => LayerKind::Config,
        }
    }
}
impl How {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Scratch => b"scratch",
            Self::Reconcile => b"reconcile",
        }
    }
}

pub fn read(bytes: &[u8], limits: &Limits) -> Result<Layer, Unreadable> {
    let mut f = Fields::new(frame::body(bytes, Kind::Layer, limits)?, limits);
    let (mut changes, mut costs, mut counts, mut inputs, mut key, mut kind, mut made) =
        (None, None, None, None, None, None, None);
    let (mut parent, mut placed, mut scan, mut subvolume, mut tools, mut tree, mut tree_file) =
        (None, None, None, None, None, None, None);
    for _ in 0..f.map(b"")? {
        match f.key()?.as_ref() {
            b"changes" => changes = Some(f.digest(b"changes")?),
            b"costs" => costs = Some(read_costs(&mut f, limits)?),
            b"counts" => counts = Some(read_counts(&mut f)?),
            b"inputs" => inputs = Some(read_inputs(&mut f)?),
            b"key" => key = Some(f.digest(b"key")?),
            b"kind" => kind = Some(read_kind(&mut f)?),
            b"made" => made = Some(read_made(&mut f)?),
            b"parent" => parent = Some(f.digest_or_none(b"parent")?),
            b"placed" => placed = Some(read_placed(&mut f, limits)?),
            b"scan" => scan = Some(f.digest(b"scan")?),
            b"subvolume" => subvolume = Some(read_subvolume(&mut f)?),
            b"tools" => tools = Some(read_tools(&mut f, limits)?),
            b"tree" => tree = Some(f.digest(b"tree")?),
            b"tree_file" => tree_file = Some(read_tree_file(&mut f)?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    f.finish()?;
    let kind: LayerKind = kind.ok_or(Unreadable::Missing(b"kind"))?;
    let inputs = inputs.ok_or(Unreadable::Missing(b"inputs"))?;
    let made = made.ok_or(Unreadable::Missing(b"made"))?;
    Ok(Layer {
        changes: changes.ok_or(Unreadable::Missing(b"changes"))?,
        costs: costs.ok_or(Unreadable::Missing(b"costs"))?,
        counts: counts.ok_or(Unreadable::Missing(b"counts"))?,
        inputs: agree_inputs(kind, inputs)?,
        key: key.ok_or(Unreadable::Missing(b"key"))?,
        kind,
        made: Made { at: made.at, packages: agree_made(kind, &made)? },
        parent: parent.ok_or(Unreadable::Missing(b"parent"))?,
        placed: agree_placed(kind, placed)?,
        scan: agree_scan(kind, scan)?,
        subvolume: subvolume.ok_or(Unreadable::Missing(b"subvolume"))?,
        tools: tools.ok_or(Unreadable::Missing(b"tools"))?,
        tree: tree.ok_or(Unreadable::Missing(b"tree"))?,
        tree_file: tree_file.ok_or(Unreadable::Missing(b"tree_file"))?,
    })
}
fn read_kind(f: &mut Fields<'_>) -> Result<LayerKind, Unreadable> {
    let name = f.bytes(b"kind", u32::MAX)?;
    match name.as_ref() {
        b"empty" => Ok(LayerKind::Empty),
        b"packages" => Ok(LayerKind::Packages),
        b"software" => Ok(LayerKind::Software),
        b"config" => Ok(LayerKind::Config),
        _ => Err(Unreadable::Shape(b"kind")),
    }
}
/// The inputs map as read, before the kind says which fields it must hold.
struct RawInputs {
    part: Option<Digest>,
    resolution: Option<Digest>,
}
fn read_inputs(f: &mut Fields<'_>) -> Result<RawInputs, Unreadable> {
    let (mut part, mut resolution) = (None, None);
    for _ in 0..f.map(b"inputs")? {
        match f.key()?.as_ref() {
            b"part" => part = Some(f.digest(b"part")?),
            b"resolution" => resolution = Some(f.digest(b"resolution")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(RawInputs { part, resolution })
}
fn agree_inputs(kind: LayerKind, raw: RawInputs) -> Result<Inputs, Unreadable> {
    let part = raw.part.ok_or(Unreadable::Missing(b"part"));
    match kind {
        LayerKind::Empty => match (raw.part, raw.resolution) {
            (None, None) => Ok(Inputs::Empty),
            (Some(_), _) => Err(Unreadable::Shape(b"part")),
            (None, Some(_)) => Err(Unreadable::Shape(b"resolution")),
        },
        LayerKind::Packages => {
            Ok(Inputs::Packages { part: part?, resolution: raw.resolution.ok_or(Unreadable::Missing(b"resolution"))? })
        }
        LayerKind::Software | LayerKind::Config => {
            let part = part?;
            if raw.resolution.is_some() {
                return Err(Unreadable::Shape(b"resolution"));
            }
            Ok(if kind == LayerKind::Software { Inputs::Software { part } } else { Inputs::Config { part } })
        }
    }
}
fn agree_placed(kind: LayerKind, placed: Option<Box<[Digest]>>) -> Result<Option<Box<[Digest]>>, Unreadable> {
    match (kind, placed) {
        (LayerKind::Software, Some(placed)) => Ok(Some(placed)),
        (LayerKind::Software, None) => Err(Unreadable::Missing(b"placed")),
        (LayerKind::Empty | LayerKind::Packages | LayerKind::Config, None) => Ok(None),
        (LayerKind::Empty | LayerKind::Packages | LayerKind::Config, Some(_)) => Err(Unreadable::Shape(b"placed")),
    }
}
fn agree_scan(kind: LayerKind, scan: Option<Digest>) -> Result<Option<Digest>, Unreadable> {
    match (kind, scan) {
        (LayerKind::Empty, None) => Ok(None),
        (LayerKind::Empty, Some(_)) => Err(Unreadable::Shape(b"scan")),
        (LayerKind::Packages | LayerKind::Software | LayerKind::Config, Some(scan)) => Ok(Some(scan)),
        (LayerKind::Packages | LayerKind::Software | LayerKind::Config, None) => Err(Unreadable::Missing(b"scan")),
    }
}
fn read_placed(f: &mut Fields<'_>, limits: &Limits) -> Result<Box<[Digest]>, Unreadable> {
    let count = f.list(b"placed", limits.layer.placed)?;
    let mut result = List::with_capacity(count);
    for _ in 0..count {
        result.push(f.digest(b"placed")?).expect("the checked count has room");
    }
    Ok(result.into_boxed())
}
fn read_tree_file(f: &mut Fields<'_>) -> Result<TreeFile, Unreadable> {
    f.tuple(b"tree_file", 2)?;
    Ok(TreeFile { sha256: f.digest(b"tree_file")?, size: f.uint(b"tree_file")? })
}
fn read_subvolume(f: &mut Fields<'_>) -> Result<Subvolume, Unreadable> {
    let (mut parent_uuid, mut uuid) = (None, None);
    for _ in 0..f.map(b"subvolume")? {
        match f.key()?.as_ref() {
            b"parent_uuid" => parent_uuid = Some(f.uuid_or_none(b"parent_uuid")?),
            b"uuid" => uuid = Some(f.uuid(b"uuid")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(Subvolume {
        parent_uuid: parent_uuid.ok_or(Unreadable::Missing(b"parent_uuid"))?,
        uuid: uuid.ok_or(Unreadable::Missing(b"uuid"))?,
    })
}
fn read_counts(f: &mut Fields<'_>) -> Result<Counts, Unreadable> {
    let (mut bytes, mut deletes, mut entries, mut files, mut hashed, mut puts) = (None, None, None, None, None, None);
    for _ in 0..f.map(b"counts")? {
        match f.key()?.as_ref() {
            b"bytes" => bytes = Some(f.uint(b"bytes")?),
            b"deletes" => deletes = Some(f.uint(b"deletes")?),
            b"entries" => entries = Some(f.uint(b"entries")?),
            b"files" => files = Some(f.uint(b"files")?),
            b"hashed" => hashed = Some(f.uint(b"hashed")?),
            b"puts" => puts = Some(f.uint(b"puts")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(Counts {
        bytes: bytes.ok_or(Unreadable::Missing(b"bytes"))?,
        deletes: deletes.ok_or(Unreadable::Missing(b"deletes"))?,
        entries: entries.ok_or(Unreadable::Missing(b"entries"))?,
        files: files.ok_or(Unreadable::Missing(b"files"))?,
        hashed: hashed.ok_or(Unreadable::Missing(b"hashed"))?,
        puts: puts.ok_or(Unreadable::Missing(b"puts"))?,
    })
}
fn read_tools(f: &mut Fields<'_>, limits: &Limits) -> Result<Box<[Tool]>, Unreadable> {
    let count = f.list(b"tools", limits.layer.tools)?;
    let mut result: List<Tool> = List::with_capacity(count);
    for _ in 0..count {
        f.tuple(b"tool", 2)?;
        let tool =
            Tool { name: f.bytes(b"name", limits.layer.name)?, version: f.bytes(b"version", limits.layer.name)? };
        if let Some(previous) = result.last()
            && previous.name >= tool.name
        {
            return Err(Unreadable::Order(b"tools"));
        }
        result.push(tool).expect("the checked count has room");
    }
    Ok(result.into_boxed())
}
fn read_costs(f: &mut Fields<'_>, limits: &Limits) -> Result<Box<[Step]>, Unreadable> {
    let count = f.map(b"costs")?;
    if count > u64::from(limits.layer.steps) {
        return Err(Unreadable::Limit(b"costs"));
    }
    let mut result = List::with_capacity(u32::try_from(count).expect("the count is within a u32 limit"));
    for _ in 0..count {
        let name = f.key()?;
        let cost = read_cost(f)?;
        result.push(Step { name, cost }).expect("the checked count has room");
    }
    Ok(result.into_boxed())
}
fn read_cost(f: &mut Fields<'_>) -> Result<Cost, Unreadable> {
    let (mut cpu, mut fetched, mut hashed, mut memory, mut wall, mut written) = (None, None, None, None, None, None);
    for _ in 0..f.map(b"cost")? {
        match f.key()?.as_ref() {
            b"cpu" => cpu = Some(f.duration(b"cpu")?),
            b"fetched" => fetched = Some(f.uint(b"fetched")?),
            b"hashed" => hashed = Some(f.uint(b"hashed")?),
            b"memory" => memory = Some(f.uint(b"memory")?),
            b"wall" => wall = Some(f.duration(b"wall")?),
            b"written" => written = Some(f.uint(b"written")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    Ok(Cost {
        cpu: cpu.ok_or(Unreadable::Missing(b"cpu"))?,
        fetched: fetched.ok_or(Unreadable::Missing(b"fetched"))?,
        hashed: hashed.ok_or(Unreadable::Missing(b"hashed"))?,
        memory: memory.ok_or(Unreadable::Missing(b"memory"))?,
        wall: wall.ok_or(Unreadable::Missing(b"wall"))?,
        written: written.ok_or(Unreadable::Missing(b"written"))?,
    })
}
/// The made map as read: its packages fields may be present, absent or partial.
struct RawMade {
    at: Wall,
    base: Option<Digest>,
    has_base: bool,
    depth: Option<u64>,
    how: Option<How>,
    line: Option<Wall>,
}
fn read_made(f: &mut Fields<'_>) -> Result<RawMade, Unreadable> {
    let (mut at, mut base, mut has_base, mut by, mut depth, mut how, mut line) =
        (None, None, false, false, None, None, None);
    for _ in 0..f.map(b"made")? {
        match f.key()?.as_ref() {
            b"at" => at = Some(f.wall(b"at")?),
            b"base" => {
                base = f.digest_or_none(b"base")?;
                has_base = true;
            }
            b"by" => {
                if f.bytes(b"by", u32::MAX)?.as_ref() != b"built" {
                    return Err(Unreadable::Shape(b"by"));
                }
                by = true;
            }
            b"depth" => depth = Some(f.uint(b"depth")?),
            b"how" => {
                how = Some(match f.bytes(b"how", u32::MAX)?.as_ref() {
                    b"scratch" => How::Scratch,
                    b"reconcile" => How::Reconcile,
                    _ => return Err(Unreadable::Shape(b"how")),
                });
            }
            b"line" => line = Some(f.wall(b"line")?),
            _ => return Err(Unreadable::Unknown),
        }
    }
    if !by {
        return Err(Unreadable::Missing(b"by"));
    }
    Ok(RawMade { at: at.ok_or(Unreadable::Missing(b"at"))?, base, has_base, depth, how, line })
}
fn agree_made(kind: LayerKind, raw: &RawMade) -> Result<Option<Line>, Unreadable> {
    if kind != LayerKind::Packages {
        if raw.has_base {
            return Err(Unreadable::Shape(b"base"));
        }
        if raw.depth.is_some() {
            return Err(Unreadable::Shape(b"depth"));
        }
        if raw.how.is_some() {
            return Err(Unreadable::Shape(b"how"));
        }
        if raw.line.is_some() {
            return Err(Unreadable::Shape(b"line"));
        }
        return Ok(None);
    }
    Ok(Some(Line {
        base: if raw.has_base { raw.base } else { return Err(Unreadable::Missing(b"base")) },
        depth: raw.depth.ok_or(Unreadable::Missing(b"depth"))?,
        how: raw.how.ok_or(Unreadable::Missing(b"how"))?,
        line: raw.line.ok_or(Unreadable::Missing(b"line"))?,
    }))
}

pub fn write(record: &Layer, limits: &Limits) -> Result<Box<[u8]>, Refused> {
    validate(record, limits)?;
    let mut measure = Encoder::measuring();
    encode(record, &mut measure);
    let mut writer = Encoder::writing(measure.measured());
    encode(record, &mut writer);
    frame::write(Kind::Layer, &writer.written(), limits)
}
fn length(found: usize, limit: u32, name: &'static [u8]) -> Result<(), Refused> {
    if found > usize::try_from(limit).expect("a u32 fits in memory") { Err(Refused::Limit(name)) } else { Ok(()) }
}
fn validate(record: &Layer, limits: &Limits) -> Result<(), Refused> {
    let l = limits.layer;
    if limits.key < 11 {
        return Err(Refused::Limit(b"key"));
    }
    if limits.depth < if record.costs.is_empty() && record.tools.is_empty() { 2 } else { 3 } {
        return Err(Refused::Limit(b"depth"));
    }
    if record.inputs.kind() != record.kind {
        return Err(Refused::Limit(b"inputs"));
    }
    if record.placed.is_some() != (record.kind == LayerKind::Software) {
        return Err(Refused::Limit(b"placed"));
    }
    if record.scan.is_none() != (record.kind == LayerKind::Empty) {
        return Err(Refused::Limit(b"scan"));
    }
    if record.made.packages.is_some() != (record.kind == LayerKind::Packages) {
        return Err(Refused::Limit(b"made"));
    }
    if let Some(placed) = &record.placed {
        length(placed.len(), l.placed, b"placed")?;
    }
    length(record.tools.len(), l.tools, b"tools")?;
    let mut previous: Option<&Tool> = None;
    for tool in &record.tools {
        length(tool.name.len(), l.name, b"name")?;
        length(tool.version.len(), l.name, b"version")?;
        if let Some(last) = previous
            && last.name >= tool.name
        {
            return Err(Refused::Order(b"tools"));
        }
        previous = Some(tool);
    }
    length(record.costs.len(), l.steps, b"costs")?;
    let mut previous: Option<&Step> = None;
    for step in &record.costs {
        length(step.name.len(), limits.key, b"key")?;
        if let Some(last) = previous
            && last.name >= step.name
        {
            return Err(Refused::Order(b"costs"));
        }
        previous = Some(step);
    }
    Ok(())
}
pub(crate) fn encode(record: &Layer, out: &mut Encoder) {
    let optional = u64::from(record.placed.is_some()).checked_add(u64::from(record.scan.is_some()));
    out.map(optional.expect("two optional fields fit").checked_add(12).expect("fourteen fields fit"));
    out.key(b"changes");
    out.digest(&record.changes);
    out.key(b"costs");
    encode_costs(&record.costs, out);
    out.key(b"counts");
    out.map(6);
    out.key(b"bytes");
    out.uint(record.counts.bytes);
    out.key(b"deletes");
    out.uint(record.counts.deletes);
    out.key(b"entries");
    out.uint(record.counts.entries);
    out.key(b"files");
    out.uint(record.counts.files);
    out.key(b"hashed");
    out.uint(record.counts.hashed);
    out.key(b"puts");
    out.uint(record.counts.puts);
    out.key(b"inputs");
    match &record.inputs {
        Inputs::Empty => out.map(0),
        Inputs::Packages { part, resolution } => {
            out.map(2);
            out.key(b"part");
            out.digest(part);
            out.key(b"resolution");
            out.digest(resolution);
        }
        Inputs::Software { part } | Inputs::Config { part } => {
            out.map(1);
            out.key(b"part");
            out.digest(part);
        }
    }
    out.key(b"key");
    out.digest(&record.key);
    out.key(b"kind");
    out.bytes(record.kind.name());
    out.key(b"made");
    encode_made(&record.made, out);
    out.key(b"parent");
    match &record.parent {
        Some(parent) => out.digest(parent),
        None => out.none(),
    }
    if let Some(placed) = &record.placed {
        out.key(b"placed");
        out.list(u64::try_from(placed.len()).expect("bounded placed"));
        for digest in placed {
            out.digest(digest);
        }
    }
    if let Some(scan) = &record.scan {
        out.key(b"scan");
        out.digest(scan);
    }
    out.key(b"subvolume");
    out.map(2);
    out.key(b"parent_uuid");
    match &record.subvolume.parent_uuid {
        Some(uuid) => out.bytes(uuid),
        None => out.none(),
    }
    out.key(b"uuid");
    out.bytes(&record.subvolume.uuid);
    out.key(b"tools");
    out.list(u64::try_from(record.tools.len()).expect("bounded tools"));
    for tool in &record.tools {
        out.list(2);
        out.bytes(&tool.name);
        out.bytes(&tool.version);
    }
    out.key(b"tree");
    out.digest(&record.tree);
    out.key(b"tree_file");
    out.list(2);
    out.digest(&record.tree_file.sha256);
    out.uint(record.tree_file.size);
}
fn encode_costs(costs: &[Step], out: &mut Encoder) {
    out.map(u64::try_from(costs.len()).expect("bounded steps"));
    for step in costs {
        out.key(&step.name);
        out.map(6);
        out.key(b"cpu");
        out.uint(step.cost.cpu.as_nanos());
        out.key(b"fetched");
        out.uint(step.cost.fetched);
        out.key(b"hashed");
        out.uint(step.cost.hashed);
        out.key(b"memory");
        out.uint(step.cost.memory);
        out.key(b"wall");
        out.uint(step.cost.wall.as_nanos());
        out.key(b"written");
        out.uint(step.cost.written);
    }
}
fn encode_made(made: &Made, out: &mut Encoder) {
    out.map(match &made.packages {
        Some(_) => 6,
        None => 2,
    });
    out.key(b"at");
    out.uint(made.at.as_nanos());
    if let Some(line) = &made.packages {
        out.key(b"base");
        match &line.base {
            Some(base) => out.digest(base),
            None => out.none(),
        }
    }
    out.key(b"by");
    out.bytes(b"built");
    if let Some(line) = &made.packages {
        out.key(b"depth");
        out.uint(line.depth);
        out.key(b"how");
        out.bytes(line.how.name());
        out.key(b"line");
        out.uint(line.line.as_nanos());
    }
}
