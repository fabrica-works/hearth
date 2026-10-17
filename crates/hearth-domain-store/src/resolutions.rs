//! Canonical resolution and closure hashes (store.md, 6.4).
use crate::vocabulary::{Closed, Inputs};
use hearth_lib::{encode::Digest, hash::H};
#[must_use]
pub fn key(inputs: &Inputs) -> Digest {
    let mut hash = H::new(b"resolution", 4);
    for value in [&inputs.sources, &inputs.request, &inputs.variant, &inputs.apt] {
        for item in value {
            hash.values().item(item);
        }
    }
    hash.finish()
}
#[must_use]
pub fn digest(closure: &[Closed]) -> Digest {
    let mut hash = H::new(b"closure", 1);
    hash.values().list(u64::try_from(closure.len()).expect("bounded closure"));
    for package in closure {
        hash.values().list(4);
        hash.values().bytes(&package.name);
        hash.values().bytes(&package.architecture);
        hash.values().bytes(&package.version);
        hash.values().digest(&package.sha256);
    }
    hash.finish()
}

/// A find leaves unreadable records absent; startup alone validates their hashes.
pub(super) fn find(
    store: &mut crate::Store,
    event: crate::vocabulary::Event,
    out: &mut skein_lib::Queue<crate::vocabulary::Output>,
) {
    use crate::vocabulary::{Answer, Event, Output, Request};
    let answer = match event {
        Event::Find { key } => {
            if !store.usable {
                Answer::Declined { why: store.decline }
            } else if store.scan.resolutions.contains_key(&key) {
                store.finding = true;
                out.push(Output::Request(Request::Load { key }));
                return;
            } else {
                Answer::Absent
            }
        }
        Event::Resolution { record } => Answer::Found { resolution: record },
        Event::Unreadable | Event::Failed => Answer::Absent,
        Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Page { .. }
        | Event::Done
        | Event::Arrived { .. } => unreachable!("find terminal"),
    };
    store.finding = false;
    out.push(Output::Answer(answer));
}
