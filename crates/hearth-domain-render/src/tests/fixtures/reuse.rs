//! Second-side document stories, driven through the real handoff.
use crate::{
    call::{Call, Side},
    limits::{Limits, SHIPPED},
    revision::{Entry, Mode, Oid},
    vocabulary::{As, Decoded, Output, Request},
};
use skein_lib::{List, Queue, bytes};
// At most five document arrivals, their resumes, and the first side's terminal.
const READ_STEPS: u32 = 32;
pub const NEW_DOCUMENT: &[u8] = b"envs/0.json";
pub const REFUSED_DOCUMENT: &[u8] = b"envs/refused.json";
pub const CHANGED_DOCUMENT: &[u8] = b"envs/changed.json";
const SMALL_DOCUMENT: u64 = 2; // The source is {}.
pub struct Read {
    pub side: Side,
    pub blobs: Box<[(Oid, As)]>,
}
fn entry(path: &[u8]) -> Entry {
    Entry { path: bytes::copy_of(path), oid: Oid(bytes::copy_of(path)), mode: Mode::File, size: SMALL_DOCUMENT }
}
fn handoff(mut side: Side, limits: &Limits) -> Call {
    side.targets = Box::new([]);
    side.commit = Some(Oid(bytes::copy_of(b"before")));
    side.files = true;
    side.ready = true;
    let mut call = Call::diff(bytes::copy_of(b"/repo"), Box::new([]), bytes::copy_of(b"after"), limits);
    call.side = side;
    let mut output = Queue::with_capacity(super::SINGLE_REQUEST);
    for _ in 0..READ_STEPS {
        call.resume(*limits, &mut output);
        if let Some(Output::Request(Request::List { .. })) = output.pop() {
            return call;
        }
    }
    panic!("first side completes")
}
fn read(side: Side, entries: &[Entry], limits: &Limits, large: bool) -> Read {
    let mut call = handoff(side, limits);
    let mut output = Queue::with_capacity(super::SINGLE_REQUEST);
    call.side.listed(*limits, Oid(bytes::copy_of(b"after")), entries, &mut output);
    let mut requested = List::with_capacity(super::OVER_BUDGET_DOCUMENTS);
    for _ in 0..READ_STEPS {
        if let Some(Output::Request(Request::Read { blobs, .. })) = output.pop() {
            for blob in blobs {
                requested.push((blob.oid.clone(), blob.kind)).expect("five documents");
                let value = if large {
                    Decoded::Document(super::list_document())
                } else if blob.oid.0.as_ref() == REFUSED_DOCUMENT {
                    Decoded::Refused {
                        at: super::composition::at(1, 1),
                        fault: crate::errors::Decode::Unexpected,
                        also: None,
                    }
                } else {
                    Decoded::Document(super::decoded::empty_document())
                };
                call.side.decoded(&blob.oid, value);
            }
        }
        if call.side.files {
            return Read { side: call.side, blobs: requested.into_boxed() };
        }
        call.resume(*limits, &mut output);
    }
    panic!("second side reads documents")
}
pub fn documents(refused: bool) -> Read {
    let mut side = Side::new(bytes::copy_of(b"/repo"), Box::new([]), &SHIPPED);
    let mut entries = List::with_capacity(super::ADMITTED_DOCUMENTS);
    entries.push(entry(b"envs/kept.json")).expect("kept");
    if refused {
        entries.push(entry(REFUSED_DOCUMENT)).expect("refused");
    }
    side.documents = crate::revision::Documents::new(entries.as_slice());
    side.documents.set(&Oid(bytes::copy_of(b"envs/kept.json")), super::decoded::empty_document());
    side.entries = entries.as_slice().into();
    entries.push(entry(CHANGED_DOCUMENT)).expect("changed");
    read(side, entries.as_slice(), &SHIPPED, false)
}
pub fn budget() -> Read {
    let side = super::decoded_documents(super::ADMITTED_DOCUMENTS);
    let mut entries = List::with_capacity(super::OVER_BUDGET_DOCUMENTS);
    entries.push(entry(NEW_DOCUMENT)).expect("leading document");
    for entry in &side.entries {
        entries.push(entry.clone()).expect("four retained documents");
    }
    read(side, entries.as_slice(), &SHIPPED, true)
}

pub fn budget_error() -> crate::errors::Error {
    let mut error = super::decode_error(b"envs/d.json", None, super::DECODED_TOO_LARGE, None);
    error.side = Some(crate::diff::Side::To);
    error
}

pub fn blobs(read: &Read) -> Box<[(&[u8], As)]> {
    let mut blobs = List::with_capacity(u32::try_from(read.blobs.len()).expect("read count"));
    for (oid, kind) in &read.blobs {
        blobs.push((oid.0.as_ref(), *kind)).expect("read count");
    }
    blobs.into_boxed()
}
