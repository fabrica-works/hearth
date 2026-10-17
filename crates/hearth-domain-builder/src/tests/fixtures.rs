use crate::{Limits, render, vocabulary::Call};
use alloc::boxed::Box;
use skein_lib::{Env, Time, Wall, bytes};
pub const LIMITS: Limits = Limits { render: render::limits::SHIPPED, store: crate::store::limits::SHIPPED };
pub fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS }
}
pub fn call() -> Call {
    Call::Render {
        resolve: false,
        repository: bytes::copy_of(b"/repo"),
        revision: bytes::copy_of(b"HEAD"),
        targets: Box::new([]),
    }
}
pub fn spec() -> render::parts::Spec {
    render::parts::Spec {
        resolution: None,
        meta: render::parts::meta::Meta {
            target: bytes::copy_of(b"x1-prod"),
            base: bytes::copy_of(b"x1"),
            member: None,
            environment: bytes::copy_of(b"prod"),
            placement: render::parts::meta::Placement {
                kind: render::revision::Runs::Metal,
                firmware: Some(render::revision::Firmware::Efi),
                hardware: Some(bytes::copy_of(b"7th-gen")),
                on: None,
            },
        },
        meta_digest: hearth_lib::encode::Digest([1; 32]),
        commit: render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
        digest: hearth_lib::encode::Digest([2; 32]),
        parts: Box::new([]),
    }
}

pub fn store_record() -> crate::store::vocabulary::StoreRecord {
    crate::store::vocabulary::StoreRecord { layout: 1, made: Wall::EPOCH, range: None, uid: 1000 }
}
pub fn arrival_answer(builder: &mut crate::Builder, last: bool, answer: crate::store::vocabulary::Answer) {
    builder.arrival = Some((
        skein_lib::Token::new(2),
        render::revision::Oid(bytes::copy_of(b"abc")),
        3,
        last,
        hearth_lib::encode::Digest([1; 32]),
    ));
    builder.store_output.push(crate::store::vocabulary::Output::Answer(answer));
}

pub const MIN_UID: u32 = 1000;
pub fn opened_builder() -> crate::Builder {
    use crate::store::vocabulary::Event;
    let mut builder = crate::Builder::new(&LIMITS, MIN_UID);
    let page = Event::Page { entries: Box::new([]), last: true };
    for event in [
        Event::Open,
        Event::Locked,
        Event::Loaded { record: Some(store_record()) },
        page.clone(),
        page.clone(),
        page.clone(),
        page,
        Event::Done,
    ] {
        store_event(&mut builder, event);
    }
    builder
}

pub const FIND_KEY: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    1, 234, 129, 26, 252, 153, 48, 138, 159, 204, 51, 108, 148, 73, 102, 199, 249, 221, 152, 133, 123, 39, 89, 47, 224,
    10, 31, 184, 75, 212, 157, 0,
]);
pub fn store_resolution() -> crate::store::vocabulary::Resolution {
    use crate::store::vocabulary::{Costs, Inputs, Made, Resolution};
    use hearth_lib::encode::Item;
    Resolution {
        key: FIND_KEY,
        digest: hearth_lib::encode::Digest([
            193, 88, 154, 151, 28, 76, 229, 4, 43, 122, 140, 15, 92, 154, 246, 155, 61, 70, 254, 100, 184, 171, 76,
            218, 172, 88, 93, 176, 234, 58, 213, 48,
        ]),
        inputs: Inputs {
            sources: Box::new([Item::Map(0)]),
            request: Box::new([Item::List(1), Item::Bytes(b"hello".as_slice().into())]),
            variant: Box::new([Item::Bytes(b"apt".as_slice().into())]),
            apt: Box::new([Item::Map(0)]),
        },
        closure: Box::new([]),
        sources: Box::new([]),
        made: Made {
            at: Wall::EPOCH,
            apt: Box::new([]),
            mmdebstrap: Box::new([]),
            costs: Costs { fetched: 0, took: skein_lib::Duration::ZERO },
        },
    }
}
pub fn indexed_builder() -> crate::Builder {
    use crate::store::vocabulary::{Entry, Event, Shape};
    let mut builder = opened_builder();
    let page = Event::Page { entries: Box::new([]), last: true };
    for event in [
        Event::Open,
        Event::Locked,
        Event::Loaded { record: Some(store_record()) },
        page.clone(),
        Event::Page {
            entries: Box::new([Entry { name: b"resolution".as_slice().into(), shape: Shape::Record(FIND_KEY) }]),
            last: true,
        },
        Event::Resolution { record: store_resolution() },
        page.clone(),
        page,
        Event::Done,
    ] {
        store_event(&mut builder, event);
    }
    builder
}

fn store_event(builder: &mut crate::Builder, event: crate::store::vocabulary::Event) {
    crate::store::step(
        &mut builder.store,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS.store },
        event,
        &mut builder.store_output,
    );
    for _ in 0..crate::store::MAX_OUT {
        drop(builder.store_output.pop());
    }
}
