//! Meta's canonical record (render.md, 4.4; store.md, 3.1).
use crate::revision::{Firmware, Runs};
use alloc::boxed::Box;
use hearth_lib::{
    encode::{Digest, Encoder},
    hash::H,
};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Meta {
    pub target: Box<[u8]>,
    pub base: Box<[u8]>,
    pub member: Option<Box<[u8]>>,
    pub environment: Box<[u8]>,
    pub placement: Placement,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub kind: Runs,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<Box<[u8]>>,
}
#[must_use]
pub fn digest(meta: &Meta) -> Digest {
    let mut hash = H::new(b"part:meta", 1);
    encode(meta, hash.values());
    hash.finish()
}
pub fn encode(meta: &Meta, record: &mut Encoder) {
    record.map(if meta.member.is_some() { 5 } else { 4 });
    record.key(b"base");
    record.bytes(&meta.base);
    record.key(b"environment");
    record.bytes(&meta.environment);
    if let Some(member) = &meta.member {
        record.key(b"member");
        record.bytes(member);
    }
    record.key(b"placement");
    let p = &meta.placement;
    let count = 1_u64
        .saturating_add(u64::from(p.firmware.is_some()))
        .saturating_add(u64::from(p.hardware.is_some()))
        .saturating_add(u64::from(p.on.is_some()));
    record.map(count);
    if let Some(firmware) = p.firmware {
        record.key(b"firmware");
        record.bytes(match firmware {
            Firmware::Efi => b"efi",
            Firmware::Bios => b"bios",
        });
    }
    if let Some(hardware) = &p.hardware {
        record.key(b"hardware");
        record.bytes(hardware);
    }
    record.key(b"kind");
    record.bytes(match p.kind {
        Runs::Metal => b"metal",
        Runs::Vm => b"vm",
        Runs::Container => b"container",
    });
    if let Some(on) = &p.on {
        record.key(b"on");
        record.bytes(on);
    }
    record.key(b"target");
    record.bytes(&meta.target);
}
