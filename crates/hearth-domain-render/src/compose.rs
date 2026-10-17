//! Target discovery and whole-placement replacement (format.md, 3.1, 3.4).
use crate::{
    limits::Limits,
    limits::SHIPPED,
    revision::{self, Class, Entry},
    revision::{Documents, Kind, Position, Value},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Target {
    pub name: Box<[u8]>,
    pub base: Box<[u8]>,
    pub environment: Box<[u8]>,
    pub member: Option<Box<[u8]>>,
    pub fleet: Option<u32>,
    pub base_document: Option<u32>,
    pub environment_document: u32,
    pub base_env: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TooMany;
pub fn targets(entries: &[Entry]) -> Result<Box<[Target]>, TooMany> {
    let documents = Documents::new(entries);
    let capacity = u32::try_from(entries.len()).expect("listing's configured capacity");
    let mut environments = Map::with_capacity(capacity);
    for entry in entries {
        match revision::class(&entry.path) {
            Class::Environment { env } => {
                environments.insert(bytes::copy_of(env), entry).expect("listing capacity");
            }
            Class::Fleet | Class::Host { .. } | Class::Declaration { .. } | Class::Tree { .. } | Class::Other => {}
        }
    }
    let mut discovered = Map::with_capacity(SHIPPED.targets);
    for entry in entries {
        match revision::class(&entry.path) {
            Class::Host { base, name } => {
                if name == base {
                    continue;
                }
                let Some(suffix) = name.strip_prefix(base) else {
                    continue;
                };
                let Some(env) = suffix.strip_prefix(b"-") else {
                    continue;
                };
                let Some(environment) = environments.get(env) else {
                    continue;
                };
                let target = Target {
                    name: bytes::copy_of(name),
                    base: bytes::copy_of(base),
                    environment: bytes::copy_of(env),
                    member: None,
                    fleet: documents.find(b"fleet.json"),
                    base_document: base_document(&documents, base),
                    environment_document: documents.find(&environment.path).expect("environment listed"),
                    base_env: documents.find(&entry.path).expect("host listed"),
                };
                if discovered.insert(target.name.clone(), target).is_err() {
                    return Err(TooMany);
                }
            }
            Class::Fleet
            | Class::Environment { .. }
            | Class::Declaration { .. }
            | Class::Tree { .. }
            | Class::Other => {}
        }
    }
    let mut targets = List::with_capacity(discovered.len());
    for (_, target) in &discovered {
        targets.push(target.clone()).expect("discovered count");
    }
    Ok(targets.into_boxed())
}
fn base_document(documents: &Documents, base: &[u8]) -> Option<u32> {
    for (index, entry) in documents.entries().iter().enumerate() {
        match revision::class(&entry.path) {
            Class::Host { base: name, name: host } if base == name && name == host => return u32::try_from(index).ok(),
            Class::Fleet
            | Class::Environment { .. }
            | Class::Host { .. }
            | Class::Declaration { .. }
            | Class::Tree { .. }
            | Class::Other => {}
        }
    }
    None
}
/// Ordered layers and each value's origin (render.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Layer {
    Defaults,
    Fleet,
    Base,
    Member,
    Environment,
    BaseEnv,
    BaseEnvMember,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Provenance {
    pub layer: Layer,
    pub document: u32,
    pub at: Position,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Composed {
    pub from: Provenance,
    pub kind: Composite,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Composite {
    Bool(bool),
    Int(i64),
    String(Box<[u8]>),
    List(Box<[Composed]>),
    Map(Box<[Pair]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pair {
    pub key: Box<[u8]>,
    pub from: Provenance,
    pub value: Composed,
}
#[derive(Debug)]
enum Frame {
    Map {
        from: Provenance,
        target: Map<Box<[u8]>, Pair>,
        patch: Box<[revision::Pair]>,
        next: usize,
        key: Option<(Box<[u8]>, Provenance)>,
    },
    List {
        from: Provenance,
        patch: Box<[Value]>,
        next: usize,
        out: List<Composed>,
    },
}
/// RFC 7386 on a stack bounded by JSON nesting (render.md, 8.3).
#[derive(Debug)]
pub struct Merge {
    stack: skein_lib::Stack<Frame>,
    steps: u64,
}
impl Merge {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            stack: skein_lib::Stack::with_capacity(limits.depth.saturating_add(1)),
            steps: u64::from(limits.document).saturating_mul(3).saturating_add(3),
        }
    }
    pub fn merge(&mut self, target: Option<Composed>, patch: &Value, layer: Layer, document: u32) -> Option<Composed> {
        assert!(self.stack.is_empty(), "previous merge finished");
        let mut pending = Some((target, patch.clone()));
        let mut finished = None;
        for _ in 0..self.steps {
            if let Some((target, patch)) = pending.take() {
                let from = Provenance { layer, document, at: patch.at };
                match patch.kind {
                    Kind::Null => {
                        finished = None;
                    }
                    Kind::Bool(value) => {
                        finished = Some(Composed { from, kind: Composite::Bool(value) });
                    }
                    Kind::Int(value) => {
                        finished = Some(Composed { from, kind: Composite::Int(value) });
                    }
                    Kind::String(value) => {
                        finished = Some(Composed { from, kind: Composite::String(value) });
                    }
                    Kind::List(patch) => {
                        let out = List::with_capacity(u32::try_from(patch.len()).expect("bounded list"));
                        self.stack.push(Frame::List { from, patch, next: 0, out }).expect("decoded nesting");
                    }
                    Kind::Map(patch) => {
                        let (from, pairs) = map_target(target, from);
                        let mut target = Map::with_capacity(
                            u32::try_from(pairs.len().saturating_add(patch.len())).expect("bounded pairs"),
                        );
                        for pair in pairs {
                            target.insert(pair.key.clone(), pair).expect("pair count");
                        }
                        self.stack
                            .push(Frame::Map { from, target, patch, next: 0, key: None })
                            .expect("decoded nesting");
                    }
                }
            }
            let Some(mut frame) = self.stack.pop() else {
                return finished;
            };
            match &mut frame {
                Frame::Map { from, target, patch, next, key } => {
                    if let Some((name, origin)) = key.take()
                        && let Some(value) = finished.take()
                    {
                        target.insert(name.clone(), Pair { key: name, from: origin, value }).expect("pair capacity");
                    }
                    if let Some(pair) = patch.get_mut(*next) {
                        *next = next.saturating_add(1);
                        let value = core::mem::replace(&mut pair.value, Value { at: pair.at, kind: Kind::Null });
                        let previous = target.remove(pair.key.as_ref());
                        let previous = match previous {
                            Some(pair) => Some(pair.value),
                            None => None,
                        };
                        *key = Some((pair.key.clone(), Provenance { layer, document, at: pair.at }));
                        pending = Some((previous, value));
                    } else {
                        let mut pairs = List::with_capacity(target.len());
                        for (_, pair) in &*target {
                            pairs.push(pair.clone()).expect("pair count");
                        }
                        finished = Some(Composed { from: *from, kind: Composite::Map(pairs.into_boxed()) });
                        continue;
                    }
                }
                Frame::List { from: _, patch, next, out } => {
                    if let Some(value) = finished.take() {
                        out.push(value).expect("list count");
                    }
                    if let Some(value) = patch.get_mut(*next) {
                        *next = next.saturating_add(1);
                        pending = Some((None, core::mem::replace(value, Value { at: value.at, kind: Kind::Null })));
                    } else {
                        let Frame::List { from, out, .. } = frame else { unreachable!("list frame") };
                        finished = Some(Composed { from, kind: Composite::List(out.into_boxed()) });
                        continue;
                    }
                }
            }
            self.stack.push(frame).expect("popped frame fits");
        }
        unreachable!("decoded document bounds the merge walk")
    }
}
fn map_target(target: Option<Composed>, from: Provenance) -> (Provenance, Box<[Pair]>) {
    match target {
        Some(Composed { from, kind: Composite::Map(pairs) }) => (from, pairs),
        Some(Composed {
            kind: Composite::Bool(_) | Composite::Int(_) | Composite::String(_) | Composite::List(_),
            ..
        })
        | None => (from, Box::new([])),
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Settings {
    pub tasks: Box<[Present]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Present {
    pub name: Box<[u8]>,
    pub from: Provenance,
    pub settings: Composed,
}
/// Presence uses host layers alone; settings include fleet and environment.
pub fn compose(target: &Target, documents: &Documents, merge: &mut Merge) -> Settings {
    let mut presence = None;
    let mut settings = None;
    for (index, layer) in [
        (target.fleet, Layer::Fleet),
        (target.base_document, Layer::Base),
        (Some(target.environment_document), Layer::Environment),
        (Some(target.base_env), Layer::BaseEnv),
    ] {
        if let Some(index) = index
            && let Some(document) = documents.get(index)
        {
            let mut pairs = List::with_capacity(u32::try_from(document.layer.tasks.len()).expect("bounded tasks"));
            for task in &document.layer.tasks {
                pairs
                    .push(revision::Pair { key: task.name.clone(), at: task.at, value: task.value.clone() })
                    .expect("task count");
            }
            let patch = Value { at: document.at, kind: Kind::Map(pairs.into_boxed()) };
            if layer == Layer::Base || layer == Layer::BaseEnv {
                presence = merge.merge(presence, &patch, layer, index);
            }
            settings = merge.merge(settings, &patch, layer, index);
        }
    }
    let origin = Provenance { layer: Layer::Base, document: 0, at: Position { line: 1, column: 1 } };
    let (_, present) = map_target(presence, origin);
    let (_, settings) = map_target(settings, origin);
    let mut tasks = List::with_capacity(u32::try_from(present.len()).expect("task count"));
    for pair in present {
        for settings in &settings {
            if pair.key == settings.key {
                tasks
                    .push(Present { name: pair.key.clone(), from: pair.from, settings: settings.value.clone() })
                    .expect("present count");
            }
        }
    }
    Settings { tasks: tasks.into_boxed() }
}
