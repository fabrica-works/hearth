//! A diff keeps successful decodes by object id and interpretation (render.md, 8.4).
use super::{Side, files};
use crate::{
    limits::Limits,
    revision::{self, Documents, Entry, Oid},
    vocabulary::{As, Blob, Decoded, Output, Request},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Queue, Token};

#[derive(Debug, Hash)]
pub(super) enum Value {
    Decoded(Decoded),
    Cached(Digest),
}
#[derive(Debug, Hash)]
struct Saved {
    oid: Oid,
    kind: As,
    value: Value,
    bytes: u64,
}
#[derive(Debug, Hash)]
pub(crate) struct Reuse {
    values: Box<[Option<Saved>]>,
    held: u64,
    reading: bool,
}
impl Reuse {
    pub fn take_side(side: &mut Side) -> Self {
        let capacity = u32::try_from(side.entries.len()).expect("listing bound");
        let mut values = List::with_capacity(capacity);
        for index in 0..side.documents.entries().len() {
            let entry = side.documents.entries().get(index).expect("document cursor");
            let oid = entry.oid.clone();
            let kind = if revision::task(&entry.path).is_some() { As::Declaration } else { As::Document };
            if let Some(value) = side.documents.take(index) {
                save(&mut values, oid, kind, Value::Decoded(value), &side.limits);
            }
        }
        side.documents = Documents::new(&[]);
        for entry in &side.entries {
            if let Some(value) = side.templates.remove(&entry.oid) {
                save(
                    &mut values,
                    entry.oid.clone(),
                    As::Template,
                    Value::Decoded(Decoded::Template(value)),
                    &side.limits,
                );
            }
            if let Some(value) = side.links.remove(&entry.oid) {
                save(&mut values, entry.oid.clone(), As::Link, Value::Decoded(Decoded::Link(value)), &side.limits);
            }
            if let Some(value) = side.cached.remove(&entry.oid) {
                save(&mut values, entry.oid.clone(), As::Cached, Value::Cached(value), &side.limits);
            }
        }
        let mut held = 0_u64;
        for value in values.as_slice().iter().flatten() {
            held = held.saturating_add(value.bytes);
        }
        Self { values: values.into_boxed(), held, reading: false }
    }
    pub fn reading(&self) -> bool {
        self.reading
    }
    pub fn listed(&mut self, entries: &[Entry], limits: &Limits) {
        for slot in &mut self.values {
            let Some(saved) = slot else { continue };
            let mut found = false;
            for entry in entries {
                if entry.oid == saved.oid && kind(entry, limits) == Some(saved.kind) {
                    found = true;
                    break;
                }
            }
            if !found {
                self.held = self.held.saturating_sub(saved.bytes);
                *slot = None;
            }
        }
    }
    fn take(&mut self, blob: &Blob) -> Option<Value> {
        for slot in &mut self.values {
            let Some(saved) = slot else { continue };
            if saved.oid == blob.oid && saved.kind == blob.kind {
                let saved = slot.take().expect("saved blob");
                self.held = self.held.saturating_sub(saved.bytes);
                return Some(saved.value);
            }
        }
        None
    }
    pub fn fit(&mut self, remaining: u64) {
        for slot in self.values.iter_mut().rev() {
            if self.held <= remaining {
                break;
            }
            if let Some(saved) = slot
                && saved.bytes != 0
            {
                self.held = self.held.saturating_sub(saved.bytes);
                *slot = None;
            }
        }
    }
}
fn save(values: &mut List<Option<Saved>>, oid: Oid, kind: As, value: Value, limits: &Limits) {
    for saved in values.as_slice().iter().flatten() {
        if saved.oid == oid && saved.kind == kind {
            return;
        }
    }
    let bytes = match &value {
        Value::Decoded(value) => revision::decoded_size(value, limits),
        Value::Cached(_) => 0,
    };
    values.push(Some(Saved { oid, kind, value, bytes })).expect("one saved blob per listed interpretation");
}
fn kind(entry: &Entry, limits: &Limits) -> Option<As> {
    if entry.mode != revision::Mode::Link && entry.mode != revision::Mode::Submodule {
        match revision::class(&entry.path) {
            revision::Class::Fleet | revision::Class::Environment { .. } | revision::Class::Host { .. } => {
                return Some(As::Document);
            }
            revision::Class::Declaration { .. } => return Some(As::Declaration),
            revision::Class::Tree { .. }
            | revision::Class::Template { .. }
            | revision::Class::Script { .. }
            | revision::Class::Readme { .. }
            | revision::Class::Refused { .. }
            | revision::Class::Other => {}
        }
    }
    Some(files::read_kind(entry, limits)?.1)
}
impl Side {
    pub(super) fn start_reads(&mut self, out: &mut Queue<Output>) -> bool {
        if self.reuse.is_some() {
            self.reuse.as_mut().expect("second side").reading = true;
            self.ready = true;
            return self.read_next(out);
        }
        if self.pending.is_empty() {
            self.ready = true;
            return false;
        }
        self.ready = false;
        out.push(Output::Request(Request::Read {
            request: Token::new(2),
            repository: self.repository.clone(),
            commit: self.commit.clone().expect("listed commit"),
            blobs: self.pending.clone(),
        }));
        true
    }
    pub(super) fn read_next(&mut self, out: &mut Queue<Output>) -> bool {
        for _ in 0..self.pending.len() {
            if self.budget.exceeded() {
                break;
            }
            let Some(blob) = self.pending.get(usize::try_from(self.read).expect("read cursor")) else { break };
            let blob = blob.clone();
            match self.reuse.as_mut().expect("second side").take(&blob) {
                Some(Value::Decoded(value)) => self.decoded(&blob.oid, value),
                Some(Value::Cached(digest)) => self.cached(&blob.oid, blob.size, digest),
                None => {
                    self.ready = false;
                    out.push(Output::Request(Request::Read {
                        request: Token::new(2),
                        repository: self.repository.clone(),
                        commit: self.commit.clone().expect("listed commit"),
                        blobs: Box::new([blob]),
                    }));
                    return true;
                }
            }
        }
        self.reuse.as_mut().expect("second side").reading = false;
        self.ready = true;
        false
    }
}
