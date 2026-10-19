//! Paged settlement of startup debris (domain/store.md, 4.4 and 8.2).
use crate::{
    Phase, Store, index,
    limits::Limits,
    resolutions,
    vocabulary::{Answer, Directory, Entry, Event, Layer, Limit, Output, Removal, Request, Shape, Step, Stray},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Env, Map, Queue};
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Pass {
    Top,
    Resolutions,
    Layers,
    Records,
    Files,
    Empty,
    Settle,
    Closing,
}
#[derive(Debug, Hash)]
pub(super) struct Scan {
    pass: Pass,
    pub(super) index: index::Index,
    pub(super) resolutions: Map<Digest, ()>,
    pub(super) layers: Map<Digest, Layer>,
    loading: Option<Digest>,
    pub(super) layer: Option<(Digest, Box<[u8]>)>,
    full: Answer,
    entries: Box<[Entry]>,
    at: u32,
    last: bool,
    removing: bool,
    after: Option<Digest>,
    pairs: u32,
    strays: u32,
    first: Option<Stray>,
}
impl Scan {
    pub fn new(limits: &Limits) -> Self {
        Self {
            pass: Pass::Top,
            index: index::Index::with_capacity(limits.cache_files.saturating_mul(2)),
            resolutions: Map::with_capacity(limits.resolutions),
            layers: Map::with_capacity(limits.layers),
            loading: None,
            layer: None,
            full: full(limits, Limit::CacheFiles),
            entries: Box::new([]),
            at: 0,
            last: true,
            removing: false,
            after: None,
            pairs: 0,
            strays: 0,
            first: None,
        }
    }
    #[cfg(test)]
    pub(super) fn indexed(&self, digest: &Digest) -> bool {
        self.index.get(digest) == Some(&true)
    }
    #[cfg(test)]
    pub(super) fn resolved(&self, key: &Digest) -> bool {
        self.resolutions.get(key).is_some()
    }
    /// Reports a name once among the scan's strays, by count and the first (8.2, step 7).
    pub(super) fn refuse(&mut self, directory: Directory, name: Box<[u8]>) {
        self.strays = self.strays.saturating_add(1);
        if self.first.is_none() {
            self.first = Some(Stray { directory, name });
        }
    }
    fn directory(&self) -> Directory {
        match self.pass {
            Pass::Top => Directory::Top,
            Pass::Resolutions => Directory::Resolutions,
            Pass::Layers => Directory::Layers,
            Pass::Records | Pass::Files | Pass::Empty | Pass::Settle | Pass::Closing => Directory::Cache,
        }
    }
}
pub(super) fn begin(store: &mut Store, limits: Limits, out: &mut Queue<Output>) {
    store.phase = Phase::Scanning;
    store.scan = Scan::new(&limits);
    out.push(Output::Request(Request::List { directory: Directory::Top, max: limits.page }));
}
pub(super) fn step(store: &mut Store, env: &Env<Limits>, event: Event, out: &mut Queue<Output>) {
    let scan = &mut store.scan;
    match event {
        Event::Page { entries, last } => {
            scan.entries = entries;
            scan.at = 0;
            scan.last = last;
        }
        Event::Done => {
            scan.removing = false;
            if scan.pass == Pass::Closing {
                finish(store, store.scan.full.clone(), out);
                return;
            }
            if scan.pass == Pass::Empty {
                scan.pass = Pass::Settle;
            }
            if scan.pass == Pass::Settle
                && let Some(digest) = scan.after
            {
                scan.index.remove(&digest);
            }
        }
        Event::Failed => {
            let step = if scan.removing {
                Step::Remove
            } else if scan.loading.is_some() || scan.layer.is_some() {
                Step::Load
            } else if scan.pass == Pass::Empty {
                Step::Empty
            } else {
                Step::List
            };
            finish(store, Answer::Unusable { step }, out);
            return;
        }
        Event::Resolution { record } => {
            let crate::vocabulary::Resolution { key, inputs, closure, digest, .. } = record;
            let expected = scan.loading.take().expect("resolution load");
            if key != expected || resolutions::key(&inputs) != expected || resolutions::digest(&closure) != digest {
                scan.removing = true;
                out.push(Output::Request(Request::Remove {
                    directory: Directory::Resolutions,
                    name: Removal::Record(expected),
                }));
                return;
            }
            if scan.resolutions.insert(key, ()).is_err() {
                stop_full(store, &env.limits, Limit::Resolutions, out);
                return;
            }
        }
        Event::Unreadable => {
            if scan.layer.is_some() {
                crate::layers::index(scan, None);
            } else {
                scan.loading = None;
            }
        }
        Event::Layer { record } => {
            if !crate::layers::index(scan, Some(record)) {
                stop_full(store, &env.limits, Limit::Layers, out);
                return;
            }
        }
        Event::Want { key } => {
            crate::layers::want(store, key, out);
            return;
        }
        Event::Record { .. }
        | Event::Room
        | Event::Unwritable { .. }
        | Event::Find { .. }
        | Event::Arrived { .. }
        | Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Stage { .. }
        | Event::Seal { .. }
        | Event::Discard { .. } => {
            unreachable!("scan terminal")
        }
    }
    advance(store, env, out);
}
fn advance(store: &mut Store, env: &Env<Limits>, out: &mut Queue<Output>) {
    let scan = &mut store.scan;
    if scan.pass == Pass::Settle {
        settle(store, out);
        return;
    }
    for entry in scan.entries.iter().skip(usize::try_from(scan.at).expect("page cursor")) {
        scan.at = scan.at.saturating_add(1);
        let removal = match entry.shape {
            Shape::Temporary if scan.pass != Pass::Files => Some(Removal::Named(entry.name.clone())),
            Shape::Record(key) if scan.pass == Pass::Resolutions => {
                scan.loading = Some(key);
                out.push(Output::Request(Request::Load { key }));
                return;
            }
            Shape::Record(key) if scan.pass == Pass::Layers => {
                scan.layer = Some((key, entry.name.clone()));
                out.push(Output::Request(Request::LoadLayer { key }));
                return;
            }
            Shape::Record(digest) if scan.pass == Pass::Records => {
                if scan.index.insert(digest, false).is_err() {
                    stop_full(store, &env.limits, Limit::CacheFiles, out);
                    return;
                }
                None
            }
            Shape::File(digest) if scan.pass == Pass::Files => {
                if let Some(paired) = scan.index.get_mut(&digest) {
                    if !*paired {
                        *paired = true;
                        scan.pairs = scan.pairs.saturating_add(1);
                    }
                    if scan.pairs > env.limits.cache_files {
                        stop_full(store, &env.limits, Limit::CacheFiles, out);
                        return;
                    }
                    None
                } else {
                    Some(Removal::File(digest))
                }
            }
            Shape::Stray if scan.pass != Pass::Files => {
                scan.strays = scan.strays.saturating_add(1);
                if scan.first.is_none() {
                    scan.first = Some(Stray { directory: scan.directory(), name: entry.name.clone() });
                }
                None
            }
            Shape::File(_) | Shape::Record(_) | Shape::Temporary | Shape::Fixed | Shape::Stray => None,
        };
        if let Some(name) = removal {
            scan.removing = true;
            out.push(Output::Request(Request::Remove { directory: scan.directory(), name }));
            return;
        }
    }
    scan.entries = Box::new([]);
    if !scan.last {
        out.push(Output::Request(Request::Next { max: env.limits.page }));
        return;
    }
    match scan.pass {
        Pass::Top | Pass::Resolutions | Pass::Layers | Pass::Records => {
            scan.pass = match scan.pass {
                Pass::Top => Pass::Resolutions,
                Pass::Resolutions => Pass::Layers,
                Pass::Layers => Pass::Records,
                Pass::Records => Pass::Files,
                Pass::Files | Pass::Empty | Pass::Settle | Pass::Closing => unreachable!("listing pass"),
            };
            out.push(Output::Request(Request::List { directory: scan.directory(), max: env.limits.page }));
        }
        Pass::Files => {
            scan.pass = Pass::Empty;
            out.push(Output::Request(Request::Empty));
        }
        Pass::Empty | Pass::Settle | Pass::Closing => unreachable!("handled above"),
    }
}
fn settle(store: &mut Store, out: &mut Queue<Output>) {
    let scan = &mut store.scan;
    if let Some(digest) = index::unpaired(&scan.index, scan.after) {
        scan.after = Some(digest);
        scan.removing = true;
        out.push(Output::Request(Request::Remove { directory: Directory::Cache, name: Removal::Record(digest) }));
    } else {
        finish(store, Answer::Ready, out);
    }
}
fn full(limits: &Limits, limit: Limit) -> Answer {
    let most = match limit {
        Limit::CacheFiles => limits.cache_files,
        Limit::Layers => limits.layers,
        Limit::Resolutions => limits.resolutions,
    };
    Answer::Full { limit, most }
}
fn stop_full(store: &mut Store, limits: &Limits, limit: Limit, out: &mut Queue<Output>) {
    store.scan.full = full(limits, limit);
    if store.scan.last {
        finish(store, store.scan.full.clone(), out);
    } else {
        store.scan.pass = Pass::Closing;
        out.push(Output::Request(Request::Close));
    }
}
fn finish(store: &mut Store, answer: Answer, out: &mut Queue<Output>) {
    store.phase = Phase::Ready;
    store.scan.entries = Box::new([]);
    store.usable = answer == Answer::Ready;
    match &answer {
        Answer::Full { limit, most } => store.decline = crate::vocabulary::Decline::Full { limit: *limit, most: *most },
        Answer::Unusable { step } => store.decline = crate::vocabulary::Decline::Unusable { step: *step },
        Answer::Recorded
        | Answer::HasRoom
        | Answer::Found { .. }
        | Answer::Absent
        | Answer::Have { .. }
        | Answer::Free { .. }
        | Answer::Staged { .. }
        | Answer::Sealed { .. }
        | Answer::Discarded { .. }
        | Answer::Declined { .. }
        | Answer::Waiting
        | Answer::Strays { .. }
        | Answer::Cached { .. }
        | Answer::Refused { .. }
        | Answer::Ready
        | Answer::Unopened { .. } => {}
    }
    if answer != Answer::Ready {
        store.scan.index = index::Index::with_capacity(0);
        store.scan.resolutions = Map::with_capacity(0);
        store.scan.layers = Map::with_capacity(0);
    }
    if answer == Answer::Ready
        && let Some(first) = store.scan.first.take()
    {
        out.push(Output::Answer(Answer::Strays { count: u64::from(store.scan.strays), first }));
    }
    out.push(Output::Answer(answer));
}
