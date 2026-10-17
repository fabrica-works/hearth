//! Target discovery and whole-placement replacement (format.md, 3.1, 3.4).
use crate::{
    limits::Limits,
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
#[derive(Debug)]
struct Maker {
    target: Target,
    at: crate::errors::Place,
}
type Makers = Map<(Box<[u8]>, Box<[u8]>, u32, u32), Maker>;
#[derive(Debug)]
pub struct Discovery {
    pub targets: Box<[Target]>,
    pub names: Box<[Box<[u8]>]>,
}
pub fn targets(
    documents: &Documents,
    limits: &Limits,
    errors: &mut crate::errors::Collector,
) -> Result<Discovery, TooMany> {
    let mut discovered = Makers::with_capacity(limits.targets);
    for (index, entry) in documents.entries().iter().enumerate() {
        let Class::Host { base, name } = revision::class(&entry.path) else { continue };
        if name == base {
            continue;
        }
        let Some(suffix) = name.strip_prefix(base) else { continue };
        let Some(env) = suffix.strip_prefix(b"-") else { continue };
        let mut environment = None;
        for (index, entry) in documents.entries().iter().enumerate() {
            if revision::class(&entry.path) == (Class::Environment { env }) {
                environment = Some(u32::try_from(index).expect("bounded listing"));
            }
        }
        let Some(environment_document) = environment else { continue };
        let target = Target {
            name: bytes::copy_of(name),
            base: bytes::copy_of(base),
            environment: bytes::copy_of(env),
            member: None,
            fleet: documents.find(b"fleet.json"),
            base_document: base_document(documents, base),
            environment_document,
            base_env: u32::try_from(index).expect("bounded listing"),
        };
        let base_members = members(documents, target.base_document);
        let env_members = members(documents, Some(target.base_env));
        if base_members.is_none() && env_members.is_none() {
            admit(&mut discovered, target, crate::errors::Place { path: entry.path.clone(), position: None }, errors)?;
            continue;
        }
        for (document, each) in [(target.base_document, base_members), (Some(target.base_env), env_members)] {
            let Some(each) = each else { continue };
            for member in &each.each {
                if document == target.base_document && member_in(env_members, &member.name).is_some() {
                    continue;
                }
                if member.layer.is_none() {
                    continue;
                }
                let mut made = target.clone();
                made.member = Some(member.name.clone());
                made.name = target_name(base, &member.name, env);
                let at = crate::errors::Place {
                    path: bytes::copy_of(documents.path(document.expect("member document"))),
                    position: Some(member.at),
                };
                admit(&mut discovered, made, at, errors)?;
            }
        }
    }
    let mut targets: List<Target> = List::with_capacity(discovered.len());
    let mut names = List::with_capacity(discovered.len());
    let mut previous: Option<&Maker> = None;
    for (_, maker) in &discovered {
        if let Some(first) = previous
            && first.target.name == maker.target.name
        {
            if let Some(target) = targets.as_slice().last()
                && target.name == maker.target.name
            {
                targets.truncate(targets.len().saturating_sub(1));
                let mut error = composition(&first.target, first.at.clone(), crate::errors::Composition::Duplicate);
                error.also = Some(maker.at.clone());
                errors.push(error);
            }
        } else {
            names.push(maker.target.name.clone()).expect("target limit");
            if maker.target.name.len() <= 63 {
                targets.push(maker.target.clone()).expect("target limit");
            }
        }
        previous = Some(maker);
    }
    Ok(Discovery { targets: targets.into_boxed(), names: names.into_boxed() })
}
fn admit(
    makers: &mut Makers,
    target: Target,
    at: crate::errors::Place,
    errors: &mut crate::errors::Collector,
) -> Result<(), TooMany> {
    if target.name.len() > 63 {
        errors.push(composition(&target, at.clone(), crate::errors::Composition::Long));
    }
    let position = at.position.unwrap_or(Position { line: 0, column: 0 });
    let key = (target.name.clone(), at.path.clone(), position.line, position.column);
    let max = makers.capacity();
    if makers.insert(key, Maker { target, at: at.clone() }).is_err() {
        errors.push(crate::errors::refused(
            &at.path,
            at.position,
            crate::errors::Decode::TooLarge { limit: crate::errors::Limit::Targets, max: u64::from(max) },
            None,
        ));
        return Err(TooMany);
    }
    Ok(())
}
fn composition(target: &Target, at: crate::errors::Place, why: crate::errors::Composition) -> crate::errors::Error {
    crate::errors::Error {
        side: None,
        kind: crate::errors::Kind::Composition(why),
        class: crate::vocabulary::Class::Configuration,
        at: Some(at),
        also: None,
        target: Some(target.name.clone()),
        task: None,
    }
}
fn members(documents: &Documents, document: Option<u32>) -> Option<&revision::Members> {
    documents.get(document?)?.members.as_ref()
}
fn member_in<'a>(members: Option<&'a revision::Members>, name: &[u8]) -> Option<&'a revision::Member> {
    let mut found = None;
    for member in &members?.each {
        if member.name.as_ref() == name {
            found = Some(member);
            break;
        }
    }
    found
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
            | Class::Template { .. }
            | Class::Script { .. }
            | Class::Readme { .. }
            | Class::Refused { .. }
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
    for (index, source, layer) in layers(target, documents).into_iter().flatten() {
        let mut pairs = List::with_capacity(u32::try_from(source.tasks.len()).expect("bounded tasks"));
        for task in &source.tasks {
            pairs
                .push(revision::Pair { key: task.name.clone(), at: task.at, value: task.value.clone() })
                .expect("task count");
        }
        let patch = Value { at: documents.get(index).expect("layer document").at, kind: Kind::Map(pairs.into_boxed()) };
        if [Layer::Base, Layer::Member, Layer::BaseEnv, Layer::BaseEnvMember].contains(&layer) {
            presence = merge.merge(presence, &patch, layer, index);
        }
        settings = merge.merge(settings, &patch, layer, index);
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

/// Listing rules apply once to the revision (format.md, 2.1 to 2.4).
pub(crate) fn listing(
    entries: &[Entry],
    limits: &Limits,
    errors: &mut crate::errors::Collector,
) -> Result<Box<[bool]>, ()> {
    use crate::errors::{self, Repository};
    let capacity = u32::try_from(entries.len()).expect("bounded listing");
    let mut tasks = Map::with_capacity(capacity);
    let mut destinations: Map<Box<[u8]>, Box<[u8]>> = Map::with_capacity(capacity);
    let mut ordered = Map::with_capacity(capacity);
    let mut invalid = List::with_capacity(capacity);
    for (index, entry) in entries.iter().enumerate() {
        invalid.push(false).expect("listing capacity");
        ordered
            .insert(entry.path.clone(), (u32::try_from(index).expect("bounded listing"), entry))
            .expect("listing capacity");
    }
    for (_, (index, entry)) in &ordered {
        let class = revision::class(&entry.path);
        let task = revision::task(&entry.path);
        if let Some(task) = task {
            tasks.insert(bytes::copy_of(task), (entry.path.clone(), *index)).expect("listing capacity");
        }
        let why = listing_refusal(entry, &class, entries);
        if let Some(why) = why {
            *invalid.get_mut(*index).expect("listed index") = true;
            errors::push(errors, errors::repository(&entry.path, task, why));
        }
        if let Class::Tree { .. } | Class::Script { .. } = class {
            let destination = entry.path.strip_suffix(b".j2").unwrap_or(&entry.path);
            if let Some(first) = destinations.get(destination) {
                let mut error = errors::repository(&entry.path, task, Repository::Destination);
                error.also = Some(errors::Place { path: first.clone(), position: None });
                *invalid.get_mut(*index).expect("listed index") = true;
                errors::push(errors, error);
            } else {
                destinations.insert(bytes::copy_of(destination), entry.path.clone()).expect("listing capacity");
            }
        }
    }
    for (task, (path, index)) in &tasks {
        let mut declared = false;
        for entry in entries {
            if let Some(path) = entry.path.strip_prefix(b"tasks/") {
                declared |= path.strip_prefix(task.as_ref()) == Some(b"/task.json".as_slice());
            }
        }
        if !declared {
            *invalid.get_mut(*index).expect("listed index") = true;
            errors::push(errors, errors::repository(path, Some(task), Repository::Declaration));
        }
    }
    if tasks.len() > limits.tasks {
        if let Some((_, (path, _))) = tasks.iter().nth(usize::try_from(limits.tasks).expect("u32 fits")) {
            errors::push(
                errors,
                errors::refused(
                    path,
                    None,
                    errors::Decode::TooLarge { limit: errors::Limit::Tasks, max: u64::from(limits.tasks) },
                    None,
                ),
            );
        }
        return Err(());
    }
    Ok(invalid.into_boxed())
}

fn listing_refusal(entry: &Entry, class: &Class<'_>, entries: &[Entry]) -> Option<crate::errors::Repository> {
    use crate::{errors::Repository, revision::Mode};
    let mut why = match class {
        Class::Refused { why, .. } => Some(why.clone()),
        Class::Host { base, name } if name != base => {
            let env = name.strip_prefix(*base).expect("host prefix").strip_prefix(b"-").expect("environment prefix");
            let mut found = false;
            for entry in entries {
                if let Class::Environment { env: listed } = revision::class(&entry.path) {
                    found |= listed == env;
                }
            }
            if found { None } else { Some(Repository::Environment { env: bytes::copy_of(env) }) }
        }
        Class::Fleet
        | Class::Environment { .. }
        | Class::Host { .. }
        | Class::Declaration { .. }
        | Class::Tree { .. }
        | Class::Template { .. }
        | Class::Script { .. }
        | Class::Readme { .. }
        | Class::Other => None,
    };
    let reads = entry.path.as_ref() == b"fleet.json"
        || entry.path.starts_with(b"envs/")
        || entry.path.starts_with(b"hosts/")
        || entry.path.starts_with(b"tasks/");
    if reads && entry.mode == Mode::Submodule {
        why = Some(Repository::Submodule);
    } else if entry.mode == Mode::Link
        && match class {
            Class::Fleet
            | Class::Environment { .. }
            | Class::Host { .. }
            | Class::Declaration { .. }
            | Class::Template { .. } => true,
            Class::Tree { path, .. } | Class::Script { name: path, .. } => path.ends_with(b".j2"),
            Class::Refused { .. } | Class::Readme { .. } | Class::Other => false,
        }
    {
        why = Some(Repository::Link);
    }
    why
}

fn target_name(base: &[u8], member: &[u8], env: &[u8]) -> Box<[u8]> {
    let mut name =
        skein_lib::Writer::new(base.len().saturating_add(member.len()).saturating_add(env.len()).saturating_add(2));
    for part in [base, b"-", member, b"-", env] {
        name.put(part).expect("name capacity");
    }
    name.finish()
}
fn layer<'a>(
    documents: &'a Documents,
    index: Option<u32>,
    member: Option<&[u8]>,
    kind: Layer,
) -> Option<(u32, &'a revision::Layer, Layer)> {
    let index = index?;
    let document = documents.get(index)?;
    if (kind == Layer::Member || kind == Layer::BaseEnvMember) && member.is_none() {
        return None;
    }
    let layer = match member {
        Some(name) => member_in(document.members.as_ref(), name)?.layer.as_ref()?,
        None => &document.layer,
    };
    Some((index, layer, kind))
}
fn layers<'a>(target: &Target, documents: &'a Documents) -> [Option<(u32, &'a revision::Layer, Layer)>; 6] {
    [
        layer(documents, target.fleet, None, Layer::Fleet),
        layer(documents, target.base_document, None, Layer::Base),
        layer(documents, target.base_document, target.member.as_deref(), Layer::Member),
        layer(documents, Some(target.environment_document), None, Layer::Environment),
        layer(documents, Some(target.base_env), None, Layer::BaseEnv),
        layer(documents, Some(target.base_env), target.member.as_deref(), Layer::BaseEnvMember),
    ]
}
#[must_use]
pub fn placement(target: &Target, documents: &Documents) -> Option<revision::Placement> {
    let mut placement = None;
    for (_, source, kind) in layers(target, documents).into_iter().flatten() {
        if kind != Layer::Member
            && let Some(value) = &source.placement
        {
            placement = Some(value.clone());
        }
    }
    placement
}
