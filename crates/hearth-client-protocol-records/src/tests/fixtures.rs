use crate::{
    parts::{
        config::{Config, Node, TreeEntry},
        meta::{Firmware, Meta, Placement, PlacementKind},
    },
    records::{Part, PartRecord, Spec},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub fn digest(prefix: [u8; 6]) -> [u8; 32] {
    let mut value = [0; 32];
    for (to, from) in value.iter_mut().zip(prefix) {
        *to = from;
    }
    value
}
pub fn x1_prod() -> Box<Spec> {
    Box::new(Spec {
        meta: Meta {
            target: copy_of(b"x1-prod"),
            base: copy_of(b"x1"),
            member: None,
            environment: copy_of(b"prod"),
            placement: Placement {
                kind: PlacementKind::Metal,
                firmware: Some(Firmware::Efi),
                hardware: Some(copy_of(b"7th-gen")),
                on: None,
            },
        },
        resolution: None,
        meta_digest: digest([0xc4, 0xd3, 0xa0, 0xb1, 0xe2, 0xf5]),
        commit: copy_of(b"1a2b3c4d00000000000000000000000000000000"),
        digest: digest([0x9a, 0x0c, 0x5e, 0x1f, 0x2b, 0x3d]),
        parts: Box::new([Part {
            digest: digest([0xe7, 0xc2, 0xa0, 0xd9, 0x5b, 0x13]),
            record: PartRecord::Config(Config {
                tree: Box::new([TreeEntry {
                    path: copy_of(b"/etc/hostname"),
                    node: Node::File {
                        content_digest: digest([0x3f, 0x1c, 0x2a, 0xb0, 0x7e, 0x44]),
                        mode: 0o644,
                        owner: copy_of(b"root"),
                        group: copy_of(b"root"),
                    },
                }]),
            }),
        }]),
    })
}
pub const X1_PROD_TEXT: &str = concat!(
    "target       x1-prod   base x1   environment prod\n",
    "placement    metal efi 7th-gen\n",
    "spec         9a0c5e1f2b3d\n",
    "config       part e7c2a0d95b13\n",
    "  /etc/hostname            file 0644 root:root 3f1c2ab07e44\n",
);

pub fn with_resolution(mut spec: Box<Spec>, resolution: crate::records::Resolution) -> Box<Spec> {
    spec.resolution = Some(resolution);
    spec
}
pub fn known() -> crate::records::Resolution {
    crate::records::Resolution::Known {
        key: digest([0x0c, 0x7d, 0x9e, 0x41, 0xc0, 0xd2]),
        keys: crate::records::Keys { packages: [0x11; 32], software: [0x22; 32], config: [0x33; 32] },
    }
}
