//! A layer's life past the index: staging, sealing and discarding (domain/store.md, 4.2).
//!
//! One request is in flight at a time: the store sends the next of a chain's steps when the
//! answer to the last one comes, and answers the build when the chain ends.
use crate::{
    Store,
    limits::Limits,
    vocabulary::{Answer, Event, Helped, Helper, Layer, LayerInputs, Output, Record, Renaming, Request, Step, Walk},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Env, Queue, Wall};

/// A staged layer, until it is sealed.
///
/// A second `Stage` while a layer is staged is refused, not queued (4.6): a build makes one
/// layer after another, so one slot holds it. Builds of different keys at once (4.6) need a
/// table here, with the task that runs them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Staged {
    key: Digest,
    uuid: [u8; 16],
    parent_uuid: [u8; 16],
}
/// What the request in flight is waiting for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Next {
    Created,
    Hashed,
    Walked,
    Frozen,
    TreeRenamed,
    Written,
    Published,
    Snapshotted,
}
/// The chain in progress: a stage, a seal, or the empty layer's making before a stage.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Working {
    /// What a failure is reported as: `Stage` or `Seal`.
    step: Step,
    next: Next,
    /// The layer whose steps these are: the empty layer's, the one sealed, or the one snapshotted.
    key: Digest,
    parent: Option<Digest>,
    inputs: LayerInputs,
    uuid: [u8; 16],
    parent_uuid: Option<[u8; 16]>,
    walk: Option<Walk>,
    at: Wall,
    /// The stage that waits for the empty layer: its key and its parent, the empty key.
    then: Option<(Digest, Digest)>,
}
impl Working {
    const fn new(step: Step, next: Next, key: Digest, at: Wall) -> Self {
        Self {
            step,
            next,
            key,
            parent: None,
            inputs: LayerInputs::Empty,
            uuid: [0; 16],
            parent_uuid: None,
            walk: None,
            at,
            then: None,
        }
    }
}
/// What the request in flight came back as.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Heard {
    Created([u8; 16]),
    Snapshotted {
        uuid: [u8; 16],
        parent_uuid: [u8; 16],
    },
    Hashed(Walk),
    Walked(Walk),
    Frozen,
    Done,
    /// A failure, or an answer that no request of the chain can have.
    Other,
}

/// Step 1: the writable snapshot of the parent, after the empty layer if it is the parent
/// and is not yet published (4.1).
pub(super) fn stage(store: &mut Store, env: &Env<Limits>, key: Digest, parent: Digest, out: &mut Queue<Output>) {
    let empty = crate::layers::empty();
    let making = parent == empty && !store.scan.layers.contains_key(&empty);
    let needed = if making { 2 } else { 1 };
    let room = store.scan.layers.len().saturating_add(needed) <= store.scan.layers.capacity();
    let parented = making || store.scan.layers.contains_key(&parent);
    if !store.usable
        || store.working.is_some()
        || store.staged.is_some()
        || store.scan.layers.contains_key(&key)
        || !room
        || !parented
    {
        out.push(Output::Answer(Answer::Unusable { step: Step::Stage }));
    } else if making {
        store.working =
            Some(Working { then: Some((key, parent)), ..Working::new(Step::Stage, Next::Created, empty, env.wall) });
        out.push(Output::Request(Request::Create { key: empty }));
    } else {
        snapshot(store, &Working::new(Step::Stage, Next::Snapshotted, key, env.wall), parent, out);
    }
}
fn snapshot(store: &mut Store, working: &Working, parent: Digest, out: &mut Queue<Output>) {
    store.working = Some(*working);
    out.push(Output::Request(Request::Helper(Helper::Snapshot { parent, staging: working.key })));
}

/// Steps 3 to 6: the walk against the parent's tree file, the freeze, the record, the publication.
pub(super) fn seal(
    store: &mut Store,
    env: &Env<Limits>,
    key: Digest,
    parent: Digest,
    inputs: LayerInputs,
    out: &mut Queue<Output>,
) {
    let staged = match store.staged {
        Some(staged) if staged.key == key && store.working.is_none() => staged,
        Some(_) | None => {
            out.push(Output::Answer(Answer::Unusable { step: Step::Seal }));
            return;
        }
    };
    store.working = Some(Working {
        parent: Some(parent),
        inputs,
        uuid: staged.uuid,
        parent_uuid: Some(staged.parent_uuid),
        ..Working::new(Step::Seal, Next::Walked, key, env.wall)
    });
    out.push(Output::Request(Request::Helper(Helper::Walk { layer: key, parent: Some(parent) })));
}

/// Transitional: h04c replaces this with the drop of the staging subvolume.
pub(super) fn discard(out: &mut Queue<Output>) {
    out.push(Output::Answer(Answer::Unusable { step: Step::Discard }));
}

/// The answer to the request in flight, and the next request of its chain.
pub(super) fn step(store: &mut Store, event: Event, out: &mut Queue<Output>) {
    let working = store.working.expect("a chain is in flight");
    let key = working.key;
    let heard = heard(event);
    let next = match (working.next, heard) {
        // The builder's BTRFS_IOC_GET_SUBVOL_INFO read of the new subvolume (4.2, step 1)
        // will replace the uuids taken here from the helper's answer, and the empty layer's from
        // its creation.
        (Next::Created, Heard::Created(uuid)) => {
            Some((Working { uuid, next: Next::Hashed, ..working }, Request::Helper(Helper::Hash { layer: key })))
        }
        (Next::Hashed, Heard::Hashed(walk)) | (Next::Walked, Heard::Walked(walk)) => Some((
            Working { walk: Some(walk), next: Next::Frozen, ..working },
            Request::Helper(Helper::Freeze { layer: key }),
        )),
        (Next::Frozen, Heard::Frozen) => {
            Some((Working { next: Next::TreeRenamed, ..working }, Request::Rename { key, rename: Renaming::Tree }))
        }
        (Next::TreeRenamed, Heard::Done) => Some((
            Working { next: Next::Written, ..working },
            Request::WriteLayer { record: Box::new(record(&working)) },
        )),
        (Next::Written, Heard::Done) => {
            Some((Working { next: Next::Published, ..working }, Request::Rename { key, rename: Renaming::Publish }))
        }
        (Next::Published, Heard::Done) => return published(store, &working, out),
        (Next::Snapshotted, Heard::Snapshotted { uuid, parent_uuid }) => {
            store.working = None;
            store.staged = Some(Staged { key, uuid, parent_uuid });
            out.push(Output::Answer(Answer::Staged { key }));
            return;
        }
        (
            Next::Created
            | Next::Hashed
            | Next::Walked
            | Next::Frozen
            | Next::TreeRenamed
            | Next::Written
            | Next::Published
            | Next::Snapshotted,
            Heard::Created(_)
            | Heard::Snapshotted { .. }
            | Heard::Hashed(_)
            | Heard::Walked(_)
            | Heard::Frozen
            | Heard::Done
            | Heard::Other,
        ) => None,
    };
    if let Some((working, request)) = next {
        store.working = Some(working);
        out.push(Output::Request(request));
    } else {
        // Whatever the chain made stays for the discard (h04c), and nothing is published.
        store.working = None;
        out.push(Output::Answer(Answer::Unusable { step: working.step }));
    }
}
/// The layer is a published subvolume with its record: the index has it, and the chain goes on
/// to the stage it was made for, or ends.
fn published(store: &mut Store, working: &Working, out: &mut Queue<Output>) {
    let record = record(working);
    store.scan.layers.insert(working.key, record.layer).expect("room was checked at stage");
    if let Some((key, parent)) = working.then {
        snapshot(store, &Working::new(Step::Stage, Next::Snapshotted, key, working.at), parent, out);
    } else {
        store.working = None;
        store.staged = None;
        out.push(Output::Answer(Answer::Sealed { key: working.key }));
    }
}
/// The record of the layer in hand, once its walk is known (6.2).
fn record(working: &Working) -> Record {
    let walk = working.walk.expect("the walk precedes the record");
    Record {
        layer: Layer {
            key: working.key,
            parent: working.parent,
            inputs: working.inputs,
            tree: walk.tree,
            uuid: working.uuid,
            at: working.at,
        },
        walk,
        parent_uuid: working.parent_uuid,
    }
}
fn heard(event: Event) -> Heard {
    match event {
        Event::Created { uuid } => Heard::Created(uuid),
        Event::Done => Heard::Done,
        Event::Helped { answer } => match answer {
            Helped::Snapshotted { uuid, parent_uuid } => Heard::Snapshotted { uuid, parent_uuid },
            Helped::Walked(walk) => Heard::Walked(walk),
            Helped::Hashed(walk) => Heard::Hashed(walk),
            Helped::Frozen => Heard::Frozen,
            Helped::Thawed | Helped::Applied | Helped::Emptied { .. } | Helped::Deleted | Helped::Failed { .. } => {
                Heard::Other
            }
        },
        Event::Failed
        | Event::Record { .. }
        | Event::Room
        | Event::Unwritable { .. }
        | Event::Find { .. }
        | Event::Resolution { .. }
        | Event::Want { .. }
        | Event::Stage { .. }
        | Event::Seal { .. }
        | Event::Discard { .. }
        | Event::Layer { .. }
        | Event::Unreadable
        | Event::Arrived { .. }
        | Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Page { .. } => Heard::Other,
    }
}
