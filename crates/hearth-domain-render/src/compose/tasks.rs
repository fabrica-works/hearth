//! Presence keeps source indexes; settings live only for their reader.
use super::{Composed, Composite, Layer, Merge, Present, Provenance, Settings, Target, layers};
use crate::{
    evaluate::ELEMENT,
    revision::{self, Documents, Kind, Position},
    schema::Defaults,
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Stack, bytes};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Index {
    Written { document: u32, member: Option<u32>, task: u32, layer: Layer },
    Hearth { document: u32 },
}
impl Index {
    pub(crate) fn name(self, documents: &Documents) -> &[u8] {
        match self {
            Self::Written { document, member, task, .. } => {
                &source(documents, document, member)
                    .tasks
                    .get(usize::try_from(task).expect("task index"))
                    .expect("present task")
                    .name
            }
            Self::Hearth { .. } => b"hearth",
        }
    }
    fn from(self, documents: &Documents) -> Provenance {
        match self {
            Self::Written { document, member, task, layer } => Provenance {
                document,
                layer,
                at: source(documents, document, member)
                    .tasks
                    .get(usize::try_from(task).expect("task index"))
                    .expect("present task")
                    .at,
            },
            Self::Hearth { document } => {
                Provenance { document, layer: Layer::BaseEnv, at: Position { line: 1, column: 1 } }
            }
        }
    }
}
fn source(documents: &Documents, document: u32, member: Option<u32>) -> &revision::Layer {
    let document = documents.get(document).expect("presence document");
    match member {
        Some(index) => document
            .members
            .as_ref()
            .expect("members")
            .each
            .get(usize::try_from(index).expect("member index"))
            .expect("member")
            .layer
            .as_ref()
            .expect("present member"),
        None => &document.layer,
    }
}
pub(crate) fn indexes(target: &Target, documents: &Documents) -> Box<[Index]> {
    let mut count = 1_u32;
    for (_, source, layer) in layers(target, documents).into_iter().flatten() {
        if [Layer::Base, Layer::Member, Layer::BaseEnv, Layer::BaseEnvMember].contains(&layer) {
            count = count.saturating_add(u32::try_from(source.tasks.len()).expect("task count"));
        }
    }
    let mut present = Map::with_capacity(count);
    for (document, source, layer) in layers(target, documents).into_iter().flatten() {
        if ![Layer::Base, Layer::Member, Layer::BaseEnv, Layer::BaseEnvMember].contains(&layer) {
            continue;
        }
        let member = if [Layer::Member, Layer::BaseEnvMember].contains(&layer) {
            let mut index = None;
            for (ordinal, member) in documents
                .get(document)
                .expect("member document")
                .members
                .as_ref()
                .expect("members")
                .each
                .iter()
                .enumerate()
            {
                if Some(member.name.as_ref()) == target.member.as_deref() {
                    index = Some(u32::try_from(ordinal).expect("member count"));
                    break;
                }
            }
            index
        } else {
            None
        };
        for (task, value) in source.tasks.iter().enumerate() {
            if value.value.kind == Kind::Null {
                present.remove(value.name.as_ref());
            } else {
                present
                    .insert(
                        value.name.clone(),
                        Index::Written { document, member, task: u32::try_from(task).expect("task count"), layer },
                    )
                    .expect("presence count");
            }
        }
    }
    let container = match super::placement(target, documents) {
        Some(placement) => placement.kind == revision::Runs::Container,
        None => false,
    };
    if container {
        present.remove(b"hearth".as_slice());
    } else if !present.contains_key(b"hearth".as_slice()) {
        present
            .insert(bytes::copy_of(b"hearth"), Index::Hearth { document: target.base_env })
            .expect("implicit task room");
    }
    let mut indexes = List::with_capacity(present.len());
    for (_, index) in &present {
        indexes.push(*index).expect("presence count");
    }
    indexes.into_boxed()
}
pub(crate) fn task(
    target: &Target,
    index: Index,
    documents: &Documents,
    defaults: &Defaults,
    merge: &mut Merge,
) -> Option<Present> {
    let name = index.name(documents);
    let from = index.from(documents);
    let mut settings =
        Some(defaults.layer(name).cloned().unwrap_or(Composed { from, kind: Composite::Map(Box::new([])) }));
    for (document, source, layer) in layers(target, documents).into_iter().flatten() {
        for task in &source.tasks {
            if task.name.as_ref() == name {
                settings = merge.merge(settings, &task.value, layer, document);
            }
        }
    }
    Some(Present {
        name: bytes::copy_of(name),
        from,
        settings: settings?,
        implicit: match index {
            Index::Hearth { .. } => true,
            Index::Written { .. } => false,
        },
    })
}
pub(crate) fn bounded(
    target: &Target,
    documents: &Documents,
    defaults: &Defaults,
    merge: &mut Merge,
    max: u64,
) -> Option<Settings> {
    settings(target, documents, defaults, merge, max)
}
pub(super) fn settings(
    target: &Target,
    documents: &Documents,
    defaults: &Defaults,
    merge: &mut Merge,
    mut left: u64,
) -> Option<Settings> {
    let indexes = indexes(target, documents);
    let mut tasks = List::with_capacity(u32::try_from(indexes.len()).expect("task count"));
    for index in indexes {
        let Some(task) = task(target, index, documents, defaults, merge) else { continue };
        left = left.checked_sub(ELEMENT.saturating_add(u64::try_from(task.name.len()).expect("task name")))?;
        left = charge(&task.settings, left, merge.stack.capacity())?;
        tasks.push(task).expect("task count");
    }
    Some(Settings { tasks: tasks.into_boxed() })
}
fn charge(value: &Composed, mut left: u64, depth: u32) -> Option<u64> {
    #[derive(Debug)]
    enum Children<'a> {
        List(core::slice::Iter<'a, Composed>),
        Map(core::slice::Iter<'a, super::Pair>),
    }
    let mut stack = Stack::with_capacity(depth);
    let mut next = Some(value);
    for _ in 0..left.saturating_mul(2).saturating_add(u64::from(depth)).saturating_add(1) {
        if let Some(value) = next.take() {
            let bytes = match &value.kind {
                Composite::Bool(_) | Composite::Int(_) => 0,
                Composite::String(value) => u64::try_from(value.len()).expect("string bytes"),
                Composite::List(values) => {
                    stack.push(Children::List(values.iter())).expect("decoded depth");
                    u64::try_from(values.len()).expect("list count").saturating_mul(ELEMENT)
                }
                Composite::Map(values) => {
                    stack.push(Children::Map(values.iter())).expect("decoded depth");
                    let mut size = 0_u64;
                    for pair in values {
                        size = size
                            .saturating_add(ELEMENT)
                            .saturating_add(u64::try_from(pair.key.len()).expect("key bytes"));
                    }
                    size
                }
            };
            left = left.checked_sub(bytes)?;
        }
        let Some(mut children) = stack.pop() else { return Some(left) };
        let more = match &mut children {
            Children::List(values) => {
                next = values.next();
                next.is_some()
            }
            Children::Map(values) => {
                next = match values.next() {
                    Some(pair) => Some(&pair.value),
                    None => None,
                };
                next.is_some()
            }
        };
        if more {
            stack.push(children).expect("popped frame fits");
        }
    }
    None
}
