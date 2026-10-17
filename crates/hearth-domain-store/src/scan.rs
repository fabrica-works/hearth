//! Paged settlement of startup debris (store.md, 4.4 and 8.2).
use crate::{
    Phase, Store, index,
    limits::Limits,
    vocabulary::{Answer, Directory, Entry, Event, Limit, Output, Removal, Request, Shape, Step, Stray},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{Env, Queue};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pass {
    Top,
    Records,
    Files,
    Settle,
    Closing,
}
#[derive(Debug)]
pub(super) struct Scan {
    pass: Pass,
    pub(super) index: index::Index,
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
    fn directory(&self) -> Directory {
        if self.pass == Pass::Top { Directory::Top } else { Directory::Cache }
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
                finish(store, full(&env.limits), out);
                return;
            }
            if scan.pass == Pass::Settle
                && let Some(digest) = scan.after
            {
                scan.index.remove(&digest);
            }
        }
        Event::Failed => {
            let step = if scan.removing { Step::Remove } else { Step::List };
            finish(store, Answer::Unusable { step }, out);
            return;
        }
        Event::Arrived { .. } | Event::Open | Event::Locked | Event::Busy | Event::Loaded { .. } => {
            unreachable!("scan terminal")
        }
    }
    if scan.pass == Pass::Settle {
        settle(store, out);
        return;
    }
    for entry in scan.entries.iter().skip(usize::try_from(scan.at).expect("page cursor")) {
        scan.at = scan.at.saturating_add(1);
        let removal = match entry.shape {
            Shape::Temporary if scan.pass != Pass::Files => Some(Removal::Named(entry.name.clone())),
            Shape::Record(digest) if scan.pass == Pass::Records => {
                if scan.index.insert(digest, false).is_err() {
                    stop_full(store, &env.limits, out);
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
                        stop_full(store, &env.limits, out);
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
        Pass::Top | Pass::Records => {
            scan.pass = if scan.pass == Pass::Top { Pass::Records } else { Pass::Files };
            out.push(Output::Request(Request::List { directory: Directory::Cache, max: env.limits.page }));
        }
        Pass::Files => {
            scan.pass = Pass::Settle;
            settle(store, out);
        }
        Pass::Settle | Pass::Closing => unreachable!("handled above"),
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
fn full(limits: &Limits) -> Answer {
    Answer::Full { limit: Limit::CacheFiles, most: limits.cache_files }
}
fn stop_full(store: &mut Store, limits: &Limits, out: &mut Queue<Output>) {
    if store.scan.last {
        finish(store, full(limits), out);
    } else {
        store.scan.pass = Pass::Closing;
        out.push(Output::Request(Request::Close));
    }
}
fn finish(store: &mut Store, answer: Answer, out: &mut Queue<Output>) {
    store.phase = Phase::Ready;
    store.scan.entries = Box::new([]);
    store.usable = answer == Answer::Ready;
    if answer != Answer::Ready {
        store.scan.index = index::Index::with_capacity(0);
    }
    out.push(Output::Answer(answer));
}
