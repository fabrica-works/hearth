use crate::{Limits, render, vocabulary::Call};
use alloc::boxed::Box;
use skein_lib::{Env, Time, Wall, bytes};
pub const LIMITS: Limits = Limits { render: render::limits::SHIPPED, store: crate::store::limits::SHIPPED };
pub fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS }
}
pub fn call() -> Call {
    Call::Render { repository: bytes::copy_of(b"/repo"), revision: bytes::copy_of(b"HEAD"), targets: Box::new([]) }
}
pub fn spec() -> render::parts::Spec {
    render::parts::Spec {
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
    builder
}
