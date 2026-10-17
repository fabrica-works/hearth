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
